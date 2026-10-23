import { useEffect, useMemo, useState } from "react";
import { RadarMap, type MapStyle } from "./radar/RadarMap";
import { useRadarStream } from "./radar/use-radar-stream";
import type { Aircraft } from "./radar/aircraft-store";

function displayName(a: Aircraft): string {
  return a.callsign?.trim() || a.id;
}
function formatAltitude(a: Aircraft): string {
  const altitude = a.telemetry?.altitudeBaroFt;
  return typeof altitude === "number" ? `${altitude.toLocaleString("cs-CZ")} ft` : "—";
}
export default function App() {
  const { frame, status } = useRadarStream();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "position">("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [follow, setFollow] = useState(false);
  const [style, setStyle] = useState<MapStyle>("liberty");
  const selected = frame.aircraft.find(a => a.id === selectedId) ?? null;
  // Nezachovávat výběr zaniklého cíle a při jeho případném návratu
  // automaticky neskákat na jiný/stale objekt se stejnou identitou.
  useEffect(() => {
    if (status === "live" && selectedId && frame.epoch && !selected) {
      setSelectedId(null);
      setFollow(false);
    }
  }, [frame.epoch, selected, selectedId, status]);
  const shown = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase("cs-CZ");
    return frame.aircraft.filter(a =>
      (filter === "all" || (!!a.position && !a.stale)) &&
      (!needle || (a.callsign ?? "").toLocaleLowerCase("cs-CZ").includes(needle) ||
        a.id.toLowerCase().includes(needle))
    ).sort((a, b) => displayName(a).localeCompare(displayName(b), "cs"));
  }, [frame.aircraft, filter, query]);
  const select = (id: string) => { setSelectedId(id); setFollow(false); };
  const connectionMessage = status === "live" ? "Živý provoz" :
    status === "connecting" ? "Připojování k radaru" :
    status === "unavailable" ? "Provoz je dočasně nedostupný" :
    "Spojení přerušeno — probíhá obnova";
  return (
    <main className="radar-page">
      <header className="radar-header">
        <div>
          <p className="eyebrow">AIRRADAR V2 · G-05 / VÝVOJ</p>
          <h1>Živý letecký provoz</h1>
        </div>
        <div className={`connection connection-${status}`} role="status" aria-live="polite">
          <span className="status-dot" /> {connectionMessage}
        </div>
      </header>
      <section className="radar-grid" aria-label="Živý radar">
        <div className="map-panel">
          <div className="toolbar">
            <span>{frame.aircraft.length.toLocaleString("cs-CZ")} letadel</span>
            <label>
              Podklad
              <select aria-label="Mapový podklad" value={style} onChange={e => setStyle(e.target.value as MapStyle)}>
                <option value="liberty">Světlý</option>
                <option value="fiord">Tmavý</option>
              </select>
            </label>
          </div>
          <RadarMap frame={frame} selectedId={selected?.id ?? null}
            follow={follow} style={style} onSelect={select} onStopFollow={() => setFollow(false)} />
          <div className="map-footer">
            <span>Lokální ADS-B · polohy pouze z potvrzených pozorování</span>
            <span>© OpenStreetMap contributors · OpenFreeMap</span>
          </div>
        </div>
        <aside className="traffic-panel" aria-label="Seznam letadel">
          <h2>Letadla v dosahu</h2>
          <label className="search-label" htmlFor="traffic-query">Hledat volací znak nebo ICAO</label>
          <input id="traffic-query" type="search" placeholder="Např. OK123 nebo icao:…" value={query} onChange={e => setQuery(e.target.value)} />
          <label className="filter-label" htmlFor="traffic-filter">Zobrazit</label>
          <select id="traffic-filter" value={filter} onChange={e => setFilter(e.target.value as "all" | "position")}>
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
          {shown.length === 0 && <p className="empty-message">{status === "live" ? "Žádná odpovídající letadla." : "Není připojen živý provoz. Čekáme na data radaru."}</p>}
          {selected && (
            <section className="selected-panel" aria-label="Vybrané letadlo">
              <div className="selected-heading">
                <h3>{displayName(selected)}</h3>
                <button type="button" onClick={() => { setSelectedId(null); setFollow(false); }} aria-label="Zrušit výběr">✕</button>
              </div>
              <p>{selected.id} · {formatAltitude(selected)}</p>
              <p>{selected.position && !selected.stale ? "Poloha dostupná" : "Bez čerstvé polohy"}</p>
              <button type="button" disabled={!selected.position || selected.stale}
                onClick={() => setFollow(previous => !previous)}>
                {follow ? "Ukončit sledování" : "Sledovat na mapě"}
              </button>
            </section>
          )}
        </aside>
      </section>
      <footer>Vývojová verze G-05 · Mapový podklad vyžaduje internet. AirRadar V1 zůstává samostatný.</footer>
    </main>
  );
}
