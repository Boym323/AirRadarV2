import { useEffect, useRef, useState } from "react";
import { AircraftStore, type ViewState } from "./aircraft-store";

export type ConnectionStatus = "connecting" | "live" | "disconnected" | "unavailable";
export function useRadarStream() {
  const store = useRef(new AircraftStore());
  const [frame, setFrame] = useState<ViewState>(() => store.current.snapshot());
  const [status, setStatus] = useState<ConnectionStatus>("connecting");
  useEffect(() => {
    let disposed = false;
    let source: EventSource | null = null;
    let restartTimer: ReturnType<typeof setTimeout> | undefined;
    let retries = 0;
    function restart() {
      source?.close();
      source = null;
      if (disposed) return;
      store.current.reset();
      setFrame(store.current.snapshot());
      setStatus("disconnected");
      clearTimeout(restartTimer);
      // Bounded resync: nový EventSource dostane vždy serverový snapshot.
      restartTimer = setTimeout(connect, Math.min(1000 * 2 ** Math.min(retries++, 4), 15000));
    }
    function message(event: MessageEvent<string>) {
      if (disposed) return;
      let payload: unknown;
      try { payload = JSON.parse(event.data); } catch { restart(); return; }
      const applied = store.current.apply(payload);
      if (applied !== "applied") { restart(); return; }
      setStatus("live");
    }
    function connect() {
      if (disposed) return;
      setStatus("connecting");
      source = new EventSource("/api/v1/stream");
      source.addEventListener("snapshot", (event) => { retries = 0; message(event as MessageEvent<string>); });
      source.addEventListener("delta", (event) => message(event as MessageEvent<string>));
      source.addEventListener("unavailable", () => {
        store.current.reset();
        setFrame(store.current.snapshot());
        setStatus("unavailable");
        source?.close();
        source = null;
        restartTimer = setTimeout(connect, 5000);
      });
      source.onerror = () => {
        if (disposed) return;
        // EventSource automaticky znovu navazuje. Všechny delty bez nové
        // atomické snapshot epochy musí být odmítnuty.
        restart();
      };
    }
    // Oddělená max. 1 Hz publikace UI; SSE nedělá React setState za každý event.
    const ticker = setInterval(() => { if (!disposed) setFrame(store.current.snapshot()); }, 1000);
    connect();
    return () => {
      disposed = true;
      source?.close();
      clearTimeout(restartTimer);
      clearInterval(ticker);
    };
  }, []);
  return { frame, status };
}
