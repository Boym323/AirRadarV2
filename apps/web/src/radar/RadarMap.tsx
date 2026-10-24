import { useEffect, useRef, useState } from "react";
import { Map as LibreMap, NavigationControl, type GeoJSONSource, type StyleSpecification } from "maplibre-gl";
import type { FeatureCollection, Point, MultiLineString } from "geojson";
import "maplibre-gl/dist/maplibre-gl.css";
import type { Aircraft, ViewState } from "./aircraft-store";
import { airportsToGeoJSON, type Airport } from "./airport-catalog";
import { diffAircraftGeoJSON, shouldUseAircraftDiff } from "./aircraft-diff";
import { iconForAircraft, registerAircraftIcons, validGroundTrack } from "./aircraft-icons";
import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";
import { setWorkerUrl } from "maplibre-gl";

setWorkerUrl(workerUrl);

export type MapStyle = "liberty" | "fiord";
const STYLES: Record<MapStyle, string> = {
  liberty: "https://tiles.openfreemap.org/styles/liberty",
  fiord: "https://tiles.openfreemap.org/styles/fiord",
};
// Lokální plátno bez externích tiles zachová radar i při výpadku podkladu.
const FALLBACK_STYLE: StyleSpecification = {
  version: 8,
  sources: {},
  layers: [{ id: "radar-fallback-background", type: "background", paint: { "background-color": "#1a3145" } }],
};
const AIRCRAFT_SOURCE = "radar-aircraft";
const TRAIL_SOURCE = "radar-trail";
const AIRPORT_SOURCE = "radar-airports";
const AIRPORT_POINTS = "radar-airport-points";
const AIRPORT_LABELS = "radar-airport-labels";
const PLANE_LAYER = "radar-aircraft-symbols";
const LABEL_LAYER = "radar-aircraft-labels";
const SELECTION_LAYER = "radar-selected-halo";
const DOT_LAYER = "radar-aircraft-unknown-heading";
const TRAIL_LAYER = "radar-selected-trail";
const emptyAircraft = (): FeatureCollection<Point> => ({ type: "FeatureCollection", features: [] });
const emptyTrail = (): FeatureCollection<MultiLineString> => ({ type: "FeatureCollection", features: [] });

// R04: jedině krátký ověřený callsign. Zlom řádku ani řídicí znaky nesmí
// obcházet MapLibre kolizní pravidla či vytvářet domnělý typ letadla.
export function mapCallsign(raw: string | null): string {
  const value = raw?.trim() ?? "";
  return /^[a-z0-9-]{1,8}$/i.test(value) ? value : "";
}
export function mapLabel(aircraft: Aircraft): { label: string; detailLabel: string } {
  const label = mapCallsign(aircraft.callsign);
  const altitude = aircraft.telemetry?.altitudeBaroFt;
  return {
    label,
    detailLabel: label && typeof altitude === "number" && Number.isFinite(altitude) &&
      altitude >= -2000 && altitude <= 65000
      ? `${label}\n${Math.round(altitude).toLocaleString("cs-CZ")} ft`
      : label,
  };
}


/** Žádný DOM marker — jediný GeoJSON source se stabilními ID. */
export function toGeoJSON(aircraft: Aircraft[], selectedId: string | null): FeatureCollection<Point> {
  return {
    type: "FeatureCollection",
    features: aircraft.filter(a => a.position && !a.stale).map(a => ({
      type: "Feature",
      id: a.id,
      geometry: { type: "Point", coordinates: [a.position!.lon, a.position!.lat] },
      properties: {
        id: a.id,
        ...mapLabel(a),
        heading: validGroundTrack(a.telemetry?.trackDegrees) ? a.telemetry!.trackDegrees : null,
        icon: iconForAircraft(a.emitterCategory, a.telemetry?.trackDegrees),
        selected: a.id === selectedId,
      },
    })),
  };
}
/**
 * R08: pouze skutečné přijímané body z G-03. Při výpadku příjmu >120 s
 * nebo přes antimeridián nezačínat vymýšlený spojovací úsek.
 */
export function trailGeoJSON(frame: ViewState, selectedId: string | null): FeatureCollection<MultiLineString> {
  const trail = selectedId ? frame.trails.get(selectedId) : undefined;
  if (!trail || trail.length < 2) return emptyTrail();
  const segments: number[][][] = [];
  let current: number[][] = [];
  let previous: (typeof trail)[number] | undefined;
  for (const point of trail) {
    if (!Number.isFinite(point.lat) || !Number.isFinite(point.lon) ||
        !Number.isFinite(point.at) || Math.abs(point.lat) > 90 ||
        Math.abs(point.lon) > 180) continue;
    if (previous && (point.at <= previous.at || point.at - previous.at > 120 ||
      Math.abs(point.lon - previous.lon) > 180)) {
      if (current.length >= 2) segments.push(current);
      current = [];
    }
    current.push([point.lon, point.lat]);
    previous = point;
  }
  if (current.length >= 2) segments.push(current);
  if (!segments.length) return emptyTrail();
  return {
    type: "FeatureCollection",
    features: [{
      type: "Feature",
      properties: { id: selectedId },
      geometry: { type: "MultiLineString", coordinates: segments },
    }],
  };
}
function addRadarLayers(map: LibreMap) {
  if (!map.isStyleLoaded()) return;
  registerAircraftIcons(map);
  if (!map.getSource(AIRCRAFT_SOURCE))
    map.addSource(AIRCRAFT_SOURCE, { type: "geojson", data: emptyAircraft(), promoteId: "id" });
  if (!map.getSource(TRAIL_SOURCE))
    map.addSource(TRAIL_SOURCE, { type: "geojson", data: emptyTrail() });
  if (!map.getLayer(TRAIL_LAYER))
    map.addLayer({
      id: TRAIL_LAYER, type: "line", source: TRAIL_SOURCE,
      paint: { "line-color": "#f5bc58", "line-width": 3, "line-opacity": .9 },
    });
  if (!map.getSource(AIRPORT_SOURCE))
    map.addSource(AIRPORT_SOURCE, { type: "geojson", data: emptyAircraft(), promoteId:"id" });
  if (!map.getLayer(AIRPORT_POINTS))
    map.addLayer({
      id:AIRPORT_POINTS,type:"circle",source:AIRPORT_SOURCE,
      paint:{
        "circle-radius":["case",["get","selected"],7,4],
        "circle-color":["case",["get","selected"],"#ffe2a1","#efa948"],
        "circle-stroke-width":2,
        "circle-stroke-color":"#13283d",
      },
    });
  if (!map.getLayer(AIRPORT_LABELS))
    map.addLayer({
      id:AIRPORT_LABELS,type:"symbol",source:AIRPORT_SOURCE,
      layout:{
        "text-field":["step",["zoom"],"",9,["get","label"],12,["get","name"]],
        "text-size":11,
        "text-offset":[0,1.4],
        "text-anchor":"top",
        "text-allow-overlap":false,
        "text-padding":3,
      },
      paint:{
        "text-color":"#ffe3a8",
        "text-halo-color":"#162b3a",
        "text-halo-width":2,
      },
    });
  if (!map.getLayer(SELECTION_LAYER))
    map.addLayer({
      id: SELECTION_LAYER, type: "circle", source: AIRCRAFT_SOURCE,
      filter: ["==", ["get", "selected"], true],
      paint: {
        "circle-radius": 17,
        "circle-color": "#082438",
        "circle-opacity": .35,
        "circle-stroke-width": 2.5,
        "circle-stroke-color": "#77cef4",
      },
    });
  if (!map.getLayer(PLANE_LAYER))
    map.addLayer({
      id: PLANE_LAYER, type: "symbol", source: AIRCRAFT_SOURCE,
      filter: ["==", ["typeof", ["get", "heading"]], "number"],
      layout: {
        "icon-image": ["concat", "radar-", ["get", "icon"]],
        "icon-size": ["case", ["get", "selected"], 1, .75],
        "icon-rotate": ["get", "heading"],
        "icon-rotation-alignment": "map",
        "icon-allow-overlap": true,
        "icon-ignore-placement": true,
        "symbol-sort-key": ["case", ["get", "selected"], 2, 1],
      },
    });
  if (!map.getLayer(DOT_LAYER))
    map.addLayer({
      id: DOT_LAYER, type: "circle", source: AIRCRAFT_SOURCE,
      filter: ["!=", ["typeof", ["get", "heading"]], "number"],
      paint: {
        "circle-radius": ["case", ["get", "selected"], 6, 5],
        "circle-color": "#73cbea",
        "circle-stroke-width": 2,
        "circle-stroke-color": "#142133",
      },
    });
  if (!map.getLayer(LABEL_LAYER))
    map.addLayer({
      id: LABEL_LAYER, type: "symbol", source: AIRCRAFT_SOURCE,
      layout: {
        "text-field": ["case", ["get", "selected"], ["get", "detailLabel"],
          ["step", ["zoom"], "", 8, ["get", "label"], 11, ["get", "detailLabel"]]],
        "text-size": ["case", ["get", "selected"], 13, 11],
        "text-offset": [0, 1.4],
        "text-anchor": "top",
        "text-max-width": 12,
        "text-padding": 2,
        "text-allow-overlap": ["get", "selected"],
        "text-ignore-placement": false,
        "symbol-sort-key": ["case", ["get", "selected"], 0, 1],
      },
      paint: {
        "text-color": "#f9fcff",
        "text-halo-color": "#112536",
        "text-halo-width": 2,
      },
    });
}
export function followTargetChanged(
  previous: { id: string; lon: number; lat: number } | null,
  next: { id: string; lon: number; lat: number },
): boolean {
  return previous === null || previous.id !== next.id ||
    previous.lon !== next.lon || previous.lat !== next.lat;
}

type Props = {
  frame: ViewState;
  airports: Airport[];
  selectedAirportId: string | null;
  showAirports: boolean;
  onSelectAirport: (id:string)=>void;
  selectedId: string | null;
  follow: boolean;
  style: MapStyle;
  onSelect: (id: string) => void;
  onStopFollow: () => void;
};
export function RadarMap({ frame, airports, selectedAirportId, showAirports, onSelectAirport, selectedId, follow, style, onSelect, onStopFollow }: Props) {
  const container = useRef<HTMLDivElement>(null);
  const currentStyle = useRef(style);
  const fallbackActive = useRef(false);
  const styleEverLoaded = useRef(false);
  const map = useRef<LibreMap | null>(null);
  const lastFollowTarget = useRef<{ id: string; lon: number; lat: number } | null>(null);
  const props = useRef({ frame, airports, selectedAirportId, showAirports, onSelectAirport, selectedId, follow, onSelect, onStopFollow });
  props.current = { frame, airports, selectedAirportId, showAirports, onSelectAirport, selectedId, follow, onSelect, onStopFollow };
  const [error, setError] = useState<string | null>(null);
  // Jedna instance na mount; SSE state ani selection ji nesmí remountovat.
  useEffect(() => {
    if (!container.current) return;
    let instance: LibreMap;
    try {
      instance = new LibreMap({
        container: container.current,
        style: STYLES[currentStyle.current],
        center: [17.67, 49.23],
        zoom: 7.7,
        attributionControl: { compact: true },
      });
    } catch {
      setError("Mapu nelze inicializovat (WebGL). Seznam letadel zůstává dostupný.");
      return;
    }
    map.current = instance;
    // G-05.11: instrumentace pouze pro místní CI benchmark, bez veřejného API.
    const benchMode = window.location.hostname === "127.0.0.1" &&
      new URLSearchParams(window.location.search).has("g05-benchmark");
    if (benchMode) {
      (window as Window & { __g05BenchMap?: LibreMap }).__g05BenchMap = instance;
    }
    instance.addControl(new NavigationControl({ showCompass: true }), "top-right");
    let lastVersion: string | null = null;
    let previousAircraft: FeatureCollection<Point> | null = null;
    let previousEpoch: string | null = null;
    let revision = 0;
    const refresh = () => {
      const current = props.current;
      const key = JSON.stringify([current.frame.epoch, current.frame.seq, current.selectedId]);
      // G-03 bez delty posílá každou sekundu nový React snapshot se stejným seq.
      // Nepřekreslovat stejné polohy a štítky opakovaně.
      if (lastVersion === key) return;
      addRadarLayers(instance);
      const source = instance.getSource(AIRCRAFT_SOURCE) as GeoJSONSource | undefined;
      const trail = instance.getSource(TRAIL_SOURCE) as GeoJSONSource | undefined;
      if (!source || !trail) return;
      const next = toGeoJSON(current.frame.aircraft, current.selectedId);
      const patch = previousAircraft ? diffAircraftGeoJSON(previousAircraft, next) : null;
      const sameEpoch = previousAircraft !== null && current.frame.epoch === previousEpoch;
      const generation = ++revision;
      if (patch?.changes === 0 && sameEpoch) {
        // Stable data, žádný worker rebuild. Stopa se přesto aktualizuje při změně seq.
      } else if (patch && previousAircraft && shouldUseAircraftDiff(
        previousAircraft.features.length, next.features.length, patch.changes, sameEpoch,
      )) {
        // Ověřeno #40095503907: u 2 000 cílů s 10 % změn výrazně menší p95.
        void source.updateData(patch.diff).catch(() => {
          if (revision === generation && instance.getSource(AIRCRAFT_SOURCE) === source) {
            void source.setData(next);
          }
        });
      } else {
        void source.setData(next);
      }
      void trail.setData(trailGeoJSON(current.frame, current.selectedId));
      previousAircraft = next;
      previousEpoch = current.frame.epoch;
      lastVersion = key;
    };
    const refreshAirportLayer = () => {
      if (!instance.isStyleLoaded()) return;
      const current=props.current;
      (instance.getSource(AIRPORT_SOURCE) as GeoJSONSource | undefined)
        ?.setData(airportsToGeoJSON(current.airports,current.selectedAirportId));
      for (const id of [AIRPORT_POINTS,AIRPORT_LABELS]) {
        if (instance.getLayer(id)) instance.setLayoutProperty(id,"visibility",current.showAirports?"visible":"none");
      }
    };
    instance.on("style.load", () => {
      // Úspěšné obnovení podkladu ruší staré varování, nikoli samotný radar.
      styleEverLoaded.current = true;
      if (!fallbackActive.current) setError(null);
      // Změna stylu zahodí registry map sources; nikdy neposlat diff do nové mapy.
      previousAircraft = null;
      previousEpoch = null;
      lastVersion = null;
      revision++;
      refresh();
      refreshAirportLayer();
    });
    const click = (event: { features?: Array<{ properties: Record<string, unknown> | null }> }) => {
      const id = event.features?.[0]?.properties?.id;
      if (typeof id === "string") props.current.onSelect(id);
    };
    instance.on("click", PLANE_LAYER, click);
    instance.on("click", DOT_LAYER, click);
    instance.on("click", AIRPORT_POINTS, event => {
      if (instance.queryRenderedFeatures(event.point,{layers:[PLANE_LAYER,DOT_LAYER]}).length) return;
      const id=event.features?.[0]?.properties?.id;
      if (typeof id==="string") props.current.onSelectAirport(id);
    });
    // G-05.08 / R09: automatické easeTo nesmí bojovat s ruční navigací.
    // zoomstart/movestart nejsou vhodné: vyvolává je také naše easeTo.
    const stopFollow = () => props.current.onStopFollow();
    instance.on("dragstart", stopFollow);
    instance.on("rotatestart", stopFollow);
    instance.on("pitchstart", stopFollow);
    const canvasContainer = instance.getCanvasContainer();
    const manualKeyboardMove = (event: KeyboardEvent) => {
      if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "+", "=", "-"].includes(event.key)) {
        stopFollow();
      }
    };
    canvasContainer.addEventListener("wheel", stopFollow, { passive: true });
    canvasContainer.addEventListener("touchmove", stopFollow, { passive: true });
    canvasContainer.addEventListener("keydown", manualKeyboardMove);
    const tick = setInterval(() => {
      if (!instance.isStyleLoaded()) return;
      if (!benchMode) refresh();
      const selected = props.current.frame.aircraft.find(a => a.id === props.current.selectedId);
      if (props.current.follow && selected?.position && !selected.stale) {
        const nextTarget = { id: selected.id, lon: selected.position.lon, lat: selected.position.lat };
        if (followTargetChanged(lastFollowTarget.current, nextTarget)) {
          const reduceMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
          instance.easeTo({ center: [nextTarget.lon, nextTarget.lat], duration: reduceMotion ? 0 : 700 });
          lastFollowTarget.current = nextTarget;
        }
      }
    }, 1000);
    instance.on("error", () => {
      // Při chybě stylu přepni pouze jednou na lokální plátno bez tiles.
      // Chyba jednotlivé dlaždice po načtení stylu fallback neaktivuje.
      if (!styleEverLoaded.current && !fallbackActive.current) {
        fallbackActive.current = true;
        instance.setStyle(FALLBACK_STYLE, { diff: false });
      }
      setError(fallbackActive.current
        ? "Mapový podklad není dostupný; živá letadla zůstávají zobrazena."
        : "Některé dlaždice mapového podkladu nejsou dostupné; živá letadla fungují dál.");
    });
    instance.on("idle", () => {
      // Po úspěšném dokončení style/tile loadingu odstranit zastaralou chybu.
      if (!fallbackActive.current && instance.isStyleLoaded() && instance.areTilesLoaded()) setError(null);
    });
    return () => {
      clearInterval(tick);
      canvasContainer.removeEventListener("wheel", stopFollow);
      canvasContainer.removeEventListener("touchmove", stopFollow);
      canvasContainer.removeEventListener("keydown", manualKeyboardMove);
      if (benchMode) delete (window as Window & { __g05BenchMap?: LibreMap }).__g05BenchMap;
      revision++;
      instance.remove();
      map.current = null;
    };
  }, []);
  useEffect(() => {
    // Nový výběr či opětovné zapnutí follow musí udělat úvodní přesun kamery.
    lastFollowTarget.current = null;
  }, [selectedId, follow]);
  useEffect(() => {
    const instance=map.current;
    if (!instance?.isStyleLoaded()) return;
    (instance.getSource(AIRPORT_SOURCE) as GeoJSONSource | undefined)
      ?.setData(airportsToGeoJSON(airports,selectedAirportId));
    for (const id of [AIRPORT_POINTS,AIRPORT_LABELS]) {
      if (instance.getLayer(id)) instance.setLayoutProperty(id,"visibility",showAirports?"visible":"none");
    }
  },[airports,selectedAirportId,showAirports]);
  useEffect(() => {
    if (map.current && currentStyle.current !== style) {
      currentStyle.current = style;
      fallbackActive.current = false;
      styleEverLoaded.current = false;
      setError(null);
      map.current.setStyle(STYLES[style]);
    }
  }, [style]);
  return (
    <div className="map-wrap">
      <div ref={container} className="map-canvas" role="region" aria-label="Mapa živého provozu" />
      {error && <p className="map-error" role="status">{error}</p>}
    </div>
  );
}
