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
  const [layersOpen, setLayersOpen] = useState(false);
  const [airports, setAirports] = useState<Airport[]>([]);
  const [showAirports, setShowAirports] = useState(true);
  const [selectedAirportId, setSelectedAirportId] = useState<string | null>(null);
  useEffect(() => {
    const controller = new AbortController();
    if (typeof fetch !== "function") return () => controller.abort();
    loadAirportCatalog(controller.signal)
      .then(setAirports)
      .catch(() => { /* Katalog je volitelný: ostatní mapa se vykreslí i bez něj. */ });
    return () => controller.abort();
  }, []);
  const listToggleRef = useRef<HTMLButtonElement>(null);
  const layersToggleRef = useRef<HTMLButtonElement>(null);
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
    setLayersOpen(false);
  };
  const selectAirport = (id: string) => {
    if (!airports.some(a => a.id === id)) return;
    setSelectedAirportId(id);
    setSelectedId(null);
    setFollow(false);
    setPanel("airport");
    setLayersOpen(false);
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
    setLayersOpen(false);
    if (panel === "traffic") closePanel();
    else setPanel("traffic");
  };
  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if (event.key !== "/" || event.altKey || event.ctrlKey || event.metaKey) return;
      const el = event.target;
      if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement) return;
      const search = document.querySelector<HTMLInputElement>(".quick-search input");
      if (search) {
        event.preventDefault();
        search.focus();
      } else {
        setLayersOpen(false);
        setPanel("traffic");
        window.requestAnimationFrame(() => document.getElementById("traffic-query")?.focus());
      }
    };
    document.addEventListener("keydown", handleShortcut);
    return () => document.removeEventListener("keydown", handleShortcut);
  }, []);
  const connectionMessage = status === "live" ? "Živý provoz" :
    status === "connecting" ? "Připojování k radaru" :
    status === "unavailable" ? "Provoz je dočasně nedostupný" :
    "Spojení přerušeno — probíhá obnova";
  const hasCurrentPosition = !!selected?.position && !selected.stale;

  return (
    <main className="radar-page">
      <header className="radar-header">
        <div className="brand" aria-label="AirRadar V2 – živý letecký radar">
          <span className="brand-mark" aria-hidden="true">
            <svg viewBox="0 0 32 32" fill="currentColor"><path d="M15.5 2c-1.3 0-2.1 1.1-2.1 2.9v8.5L3.2 18.5a2.1 2.1 0 0 0-1.2 1.9V23l11.4-3.7v6.3l-3.1 2.4v2l5.2-1.4 5.2 1.4v-2l-3.1-2.4v-6.3L29 23v-2.6a2.1 2.1 0 0 0-1.2-1.9L17.6 13.4V4.9c0-1.8-.8-2.9-2.1-2.9Z"/></svg>
          </span>
          <div className="brand-copy">
            <div className="brand-title"><h1>AirRadar</h1><span className="version-tag">V2</span></div>
            <span className="brand-subtitle">Živý letecký radar</span>
          </div>
        </div>
        <nav className="header-actions" aria-label="Hlavní ovládání radaru">
          <label className="quick-search">
            <svg className="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" aria-hidden="true"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/></svg>
            <input type="search" aria-label="Hledat letadlo" placeholder="Hledat letadlo nebo ICAO"
              autoComplete="off" value={query} onChange={event => {
                setQuery(event.target.value);
                setLayersOpen(false);
                setPanel("traffic");
              }} onFocus={() => { setLayersOpen(false); setPanel("traffic"); }} />
            <span className="quick-search-shortcut" aria-hidden="true">/</span>
          </label>
          <div className="flight-summary">
            <span className="radar-count" aria-label="Počet zachycených letadel">{aircraftCount(frame.aircraft.length)}</span>
            <div className={"connection connection-" + status} role="status" aria-live="polite">
              <span className="status-dot" aria-hidden="true" />
              <span className="connection-label">{connectionMessage}</span>
            </div>
          </div>
          <span className="header-divider" aria-hidden="true" />
          <button ref={listToggleRef} type="button" className="header-button"
            aria-label="Letadla" aria-controls="traffic-panel" aria-expanded={panel === "traffic"}
            onClick={toggleTraffic}>
            <svg className="button-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" aria-hidden="true">
              <path d="M4 7h16M4 12h16M4 17h16"/><circle cx="7" cy="7" r="1" fill="currentColor"/><circle cx="7" cy="12" r="1" fill="currentColor"/>
            </svg>
            <span className="button-label">Letadla</span>
          </button>
          <button ref={layersToggleRef} type="button" className="header-button layers-trigger"
            aria-label="Vrstvy" aria-controls="layers-panel" aria-expanded={layersOpen}
            onClick={() => { setLayersOpen(value => !value); }}>
            <svg className="button-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" aria-hidden="true">
              <path d="m12 3 9 5-9 5-9-5 9-5ZM3 12l9 5 9-5M3 16l9 5 9-5"/>
            </svg>
            <span className="button-label">Vrstvy</span>
          </button>
        </nav>
      </header>

      <section className="radar-stage" aria-label="Živý radar">
        <div className="map-surface">
          <RadarMap frame={frame} selectedId={selected?.id ?? null}
            airports={airports} selectedAirportId={selectedAirport?.id ?? null}
            showAirports={showAirports} onSelectAirport={selectAirport}
            follow={follow} style={style} onSelect={select} onStopFollow={() => setFollow(false)} />
        </div>
        {layersOpen && (
          <aside id="layers-panel" className="layers-panel" aria-label="Nastavení mapy">
            <div className="panel-heading">
              <div><span className="panel-eyebrow">ZOBRAZENÍ</span><h2>Mapa a vrstvy</h2></div>
              <button className="close-button" type="button" aria-label="Zavřít vrstvy"
                onClick={() => { setLayersOpen(false); layersToggleRef.current?.focus(); }}>✕</button>
            </div>
            <label className="layer-field" htmlFor="radar-map-style">Mapový podklad</label>
            <select id="radar-map-style" aria-label="Mapový podklad" value={style}
              onChange={event => setStyle(event.target.value as MapStyle)}>
              <option value="fiord">Atlas · výchozí</option>
              <option value="dark">Noční · vysoký kontrast</option>
              <option value="liberty">Světlý</option>
            </select>
            <label className="layer-row">
              <span className="layer-copy"><strong>Letiště</strong>
                <small>Veřejný katalog OurAirports</small></span>
              <input type="checkbox" aria-label="Letiště" checked={showAirports}
                disabled={!airports.length} onChange={event => setShowAirports(event.target.checked)} />
            </label>
          </aside>
        )}

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
    </main>
  );
}
