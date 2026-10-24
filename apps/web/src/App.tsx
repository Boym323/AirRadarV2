import { useEffect, useMemo, useRef, useState } from "react";
import { RadarMap, type MapStyle } from "./radar/RadarMap";
import { useRadarStream } from "./radar/use-radar-stream";
import type { Aircraft } from "./radar/aircraft-store";
import { loadAirportCatalog, type Airport } from "./radar/airport-catalog";

type Panel = "none" | "traffic" | "detail" | "airport";

function displayName(a: Aircraft): string {
  return a.callsign?.trim() || a.id;
}
function formatAltitude(a: Aircraft): string {
  const altitude = a.telemetry?.altitudeBaroFt;
  return typeof altitude === "number" ? altitude.toLocaleString("cs-CZ") + " ft" : "—";
}
function formatNumber(value: number | undefined, unit: string): string {
  return typeof value === "number" ? value.toLocaleString("cs-CZ") + " " + unit : "—";
}
function aircraftCount(count: number): string {
  const word = count === 1 ? "letadlo" : count >= 2 && count <= 4 ? "letadla" : "letadel";
  return count.toLocaleString("cs-CZ") + " " + word;
}

export default function App() {
  const { frame, status } = useRadarStream();
  const [panel, setPanel] = useState<Panel>("none");
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "position">("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [follow, setFollow] = useState(false);
  const [style, setStyle] = useState<MapStyle>("fiord");
  const [airports, setAirports] = useState<Airport[]>([]);
  const [airportsError, setAirportsError] = useState(false);
  const [showAirports, setShowAirports] = useState(true);
  const [selectedAirportId, setSelectedAirportId] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    if (typeof fetch !== "function") return () => controller.abort();
    loadAirportCatalog(controller.signal)
      .then(setAirports)
      .catch(() => { if (!controller.signal.aborted) setAirportsError(true); });
    return () => controller.abort();
  }, []);
  const listToggleRef = useRef<HTMLButtonElement>(null);
  const selected = frame.aircraft.find(a => a.id === selectedId) ?? null;
  const selectedAirport = airports.find(a => a.id === selectedAirportId) ?? null;

  // Reconnect nesmí obnovit výběr identity, která z nového snapshotu zmizela.
  useEffect(() => {
    if (status === "live" && selectedId && frame.epoch && !selected) {
      setSelectedId(null);
      setFollow(false);
      setPanel(previous => previous === "detail" ? "none" : previous);
    }
  }, [frame.epoch, selected, selectedId, status]);
  // Zastaralá nebo neznámá poloha nesmí zůstat cílem automatické kamery.
  useEffect(() => {
    if (follow && (!selected?.position || selected.stale || status !== "live")) {
      setFollow(false);
    }
  }, [follow, selected, status]);

  const shown = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase("cs-CZ");
    return frame.aircraft.filter(a =>
      (filter === "all" || (!!a.position && !a.stale)) &&
      (!needle || (a.callsign ?? "").toLocaleLowerCase("cs-CZ").includes(needle) ||
        a.id.toLowerCase().includes(needle))
    ).sort((a, b) => displayName(a).localeCompare(displayName(b), "cs"));
  }, [frame.aircraft, filter, query]);

  const select = (id: string) => {
    setSelectedId(id);
    setSelectedAirportId(null);
    setFollow(false);
    setPanel("detail");
  };
  const selectAirport = (id: string) => {
    if (!airports.some(a => a.id === id)) return;
    setSelectedAirportId(id);
    setSelectedId(null);
    setFollow(false);
    setPanel("airport");
  };
  const closePanel = () => {
    if (panel === "airport") setSelectedAirportId(null);
    if (panel === "detail") {
      setSelectedId(null);
      setFollow(false);
    }
    setPanel("none");
    listToggleRef.current?.focus();
  };
  const toggleTraffic = () => {
    if (panel === "traffic") closePanel();
    else setPanel("traffic");
  };
  const connectionMessage = status === "live" ? "Živý provoz" :
    status === "connecting" ? "Připojování k radaru" :
    status === "unavailable" ? "Provoz je dočasně nedostupný" :
    "Spojení přerušeno — probíhá obnova";
  const hasCurrentPosition = !!selected?.position && !selected.stale;

  return (
    <main className="radar-page">
      <header className="radar-header">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true"><svg viewBox="0 0 32 32" role="img"><path d="M3 14.1 13.8 11l4-7.2 2 .8-1.6 7.1 8.4 2.6c1 .3 1.6 1 1.6 1.7s-.6 1.4-1.6 1.7l-8.4 2.5 1.6 7.2-2 .8-4-7.3L3 18.8l-1.2-1.6L3 15.6l7.1-1.4L3 14.1Z" /></svg></span>
          <div>
            <h1>AirRadar V2</h1>
            <span className="brand-subtitle">Místní letecký radar</span>
          </div>
        </div>
        <div className="header-actions">
          <span className="radar-count" aria-label="Počet zachycených letadel">{aircraftCount(frame.aircraft.length)}</span>
          <div className={"connection connection-" + status} role="status" aria-live="polite">
            <span className="status-dot" aria-hidden="true" />
            {connectionMessage}
          </div>
          <button ref={listToggleRef} type="button" className="header-button"
            aria-controls="traffic-panel" aria-expanded={panel === "traffic"}
            onClick={toggleTraffic}>
            {panel === "traffic" ? "Skrýt letadla" : "Letadla"}
          </button>
          <div className="header-tools">
            <label className="style-control">
              <input type="checkbox" checked={showAirports}
                disabled={!airports.length} onChange={event => setShowAirports(event.target.checked)} />
              Letiště
            </label>
            <label className="style-control">
              <span>Podklad</span>
              <select aria-label="Mapový podklad" value={style} onChange={event => setStyle(event.target.value as MapStyle)}>
                <option value="fiord">Tmavý</option>
                <option value="liberty">Světlý</option>
              </select>
            </label>
          </div>
        </div>
      </header>

      <section className="radar-stage" aria-label="Živý radar">
        <div className="map-surface">
          <RadarMap frame={frame} selectedId={selected?.id ?? null}
            airports={airports} selectedAirportId={selectedAirport?.id ?? null}
            showAirports={showAirports} onSelectAirport={selectAirport}
            follow={follow} style={style} onSelect={select} onStopFollow={() => setFollow(false)} />
        </div>
        <div className="map-disclosure">Lokální ADS-B · jen potvrzené polohy</div>

        {panel === "traffic" && (
          <aside id="traffic-panel" className="radar-panel traffic-panel" aria-label="Seznam letadel">
            <div className="panel-heading">
              <h2>Letadla v dosahu</h2>
              <button type="button" className="close-button" onClick={closePanel} aria-label="Zavřít seznam">✕</button>
            </div>
            <label className="field-label" htmlFor="traffic-query">Hledat volací znak nebo ICAO</label>
            <input id="traffic-query" type="search" autoComplete="off"
              placeholder="Např. OK123 nebo icao:…" value={query}
              onChange={event => setQuery(event.target.value)} />
            <label className="field-label" htmlFor="traffic-filter">Zobrazit</label>
            <select id="traffic-filter" value={filter}
              onChange={event => setFilter(event.target.value as "all" | "position")}>
              <option value="all">Všechna letadla</option>
              <option value="position">S aktuální polohou</option>
            </select>
            <ul className="traffic-list">
              {shown.slice(0, 300).map(a => (
                <li key={a.id}>
                  <button type="button" className={a.id === selectedId ? "traffic-button selected" : "traffic-button"}
                    onClick={() => select(a.id)} aria-pressed={a.id === selectedId}>
                    <span className="traffic-name">{displayName(a)}</span>
                    <span className="traffic-meta">{formatAltitude(a)} · {a.position && !a.stale ? "poloha" : "bez čerstvé polohy"}</span>
                  </button>
                </li>
              ))}
            </ul>
            {shown.length > 300 && <p className="hint">Zobrazeno prvních 300 letadel. Upřesni vyhledávání.</p>}
            {shown.length === 0 && <p className="empty-message">
              {status === "live" ? "Žádná odpovídající letadla." : "Není připojen živý provoz. Čekáme na data radaru."}
            </p>}
          </aside>
        )}

        {panel === "airport" && selectedAirport && (
          <aside className="radar-panel detail-panel" aria-label="Vybrané letiště">
            <div className="panel-heading">
              <div><span className="panel-eyebrow">Letiště · OurAirports</span>
                <h2>{selectedAirport.name}</h2></div>
              <button type="button" className="close-button"
                aria-label="Zavřít detail letiště" onClick={closePanel}>✕</button>
            </div>
            <p className="aircraft-id">{selectedAirport.icao || selectedAirport.id}</p>
            <dl className="telemetry-grid">
              <div><dt>Země</dt><dd>{selectedAirport.country}</dd></div>
              <div><dt>Obec</dt><dd>{selectedAirport.municipality || "—"}</dd></div>
              <div><dt>Souřadnice</dt>
                <dd>{selectedAirport.lat.toFixed(4)}°, {selectedAirport.lon.toFixed(4)}°</dd></div>
            </dl>
            <p className="detail-hint">Referenční veřejný katalog OurAirports. Pro letovou navigaci použij aktuální eAIP, nikoli tento mapový náhled.</p>
          </aside>
        )}

        {panel === "detail" && selected && (
          <aside className="radar-panel detail-panel" aria-label="Vybrané letadlo">
            <div className="panel-heading">
              <div>
                <span className="panel-eyebrow">Vybrané letadlo</span>
                <h2>{displayName(selected)}</h2>
              </div>
              <button type="button" className="close-button" aria-label="Zavřít detail letadla" onClick={closePanel}>✕</button>
            </div>
            <p className="aircraft-id">{selected.id}</p>
            <p className={hasCurrentPosition ? "position-state fresh" : "position-state stale"}>
              {hasCurrentPosition ? "Aktuální poloha dostupná" : "Bez čerstvé polohy"}
            </p>
            <dl className="telemetry-grid">
              <div><dt>Výška</dt><dd>{formatAltitude(selected)}</dd></div>
              <div><dt>Rychlost vůči zemi</dt><dd>{formatNumber(selected.telemetry?.groundSpeedKt, "kt")}</dd></div>
              <div><dt>Směr letu</dt><dd>{formatNumber(selected.telemetry?.trackDegrees, "°")}</dd></div>
              <div><dt>Vertikální rychlost</dt><dd>{formatNumber(selected.telemetry?.verticalRateFpm, "ft/min")}</dd></div>
            </dl>
            <button type="button" className="follow-button" aria-pressed={follow}
              disabled={!hasCurrentPosition || status !== "live"}
              onClick={() => setFollow(previous => !previous)}>
              {follow ? "Ukončit sledování" : "Sledovat na mapě"}
            </button>
            <p className="detail-hint">Stopa zobrazuje pouze skutečné přijaté polohy za posledních 20 minut.</p>
          </aside>
        )}
      </section>
      <footer className="radar-footer">
        AirRadar V2 · Lokální ADS-B · Letiště: OurAirports (public domain){airportsError ? " — katalog dočasně nedostupný" : ""}
      </footer>
    </main>
  );
}
