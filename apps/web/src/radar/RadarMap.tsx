import { useEffect, useRef, useState } from "react";
import { Map as LibreMap, NavigationControl, supported, type GeoJSONSource } from "maplibre-gl";
import type { FeatureCollection, Point, LineString } from "geojson";
import "maplibre-gl/dist/maplibre-gl.css";
import type { Aircraft, ViewState } from "./aircraft-store";
import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";
import { setWorkerUrl } from "maplibre-gl";

setWorkerUrl(workerUrl);

export type MapStyle = "liberty" | "fiord";
const STYLES: Record<MapStyle, string> = {
  liberty: "https://tiles.openfreemap.org/styles/liberty",
  fiord: "https://tiles.openfreemap.org/styles/fiord",
};
const AIRCRAFT_SOURCE = "radar-aircraft";
const TRAIL_SOURCE = "radar-trail";
const PLANE_LAYER = "radar-aircraft-symbols";
const DOT_LAYER = "radar-aircraft-unknown-heading";
const TRAIL_LAYER = "radar-selected-trail";
const emptyAircraft = (): FeatureCollection<Point> => ({ type: "FeatureCollection", features: [] });
const emptyTrail = (): FeatureCollection<LineString> => ({ type: "FeatureCollection", features: [] });

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
        label: a.callsign || "",
        heading: a.telemetry?.trackDegrees ?? null,
        selected: a.id === selectedId,
      },
    })),
  };
}
function trailGeoJSON(frame: ViewState, selectedId: string | null): FeatureCollection<LineString> {
  const trail = selectedId ? frame.trails.get(selectedId) : undefined;
  if (!trail || trail.length < 2) return emptyTrail();
  return {
    type: "FeatureCollection",
    features: [{
      type: "Feature",
      properties: { id: selectedId },
      geometry: { type: "LineString", coordinates: trail.map(p => [p.lon, p.lat]) },
    }],
  };
}
function addPlaneImage(map: LibreMap) {
  if (map.hasImage("radar-plane")) return;
  const canvas = document.createElement("canvas");
  canvas.width = 64;
  canvas.height = 64;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.lineJoin = "round";
  ctx.fillStyle = "#f9fcff";
  ctx.strokeStyle = "#17314d";
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(32, 4);
  ctx.lineTo(39, 25);
  ctx.lineTo(57, 38);
  ctx.lineTo(56, 45);
  ctx.lineTo(37, 39);
  ctx.lineTo(36, 52);
  ctx.lineTo(44, 58);
  ctx.lineTo(43, 62);
  ctx.lineTo(32, 58);
  ctx.lineTo(21, 62);
  ctx.lineTo(20, 58);
  ctx.lineTo(28, 52);
  ctx.lineTo(27, 39);
  ctx.lineTo(8, 45);
  ctx.lineTo(7, 38);
  ctx.lineTo(25, 25);
  ctx.closePath();
  ctx.stroke();
  ctx.fill();
  map.addImage("radar-plane", ctx.getImageData(0, 0, 64, 64), { pixelRatio: 2 });
}
function addRadarLayers(map: LibreMap) {
  if (!map.isStyleLoaded()) return;
  addPlaneImage(map);
  if (!map.getSource(AIRCRAFT_SOURCE))
    map.addSource(AIRCRAFT_SOURCE, { type: "geojson", data: emptyAircraft(), promoteId: "id" });
  if (!map.getSource(TRAIL_SOURCE))
    map.addSource(TRAIL_SOURCE, { type: "geojson", data: emptyTrail() });
  if (!map.getLayer(TRAIL_LAYER))
    map.addLayer({
      id: TRAIL_LAYER, type: "line", source: TRAIL_SOURCE,
      paint: { "line-color": "#f5bc58", "line-width": 3, "line-opacity": .9 },
    });
  if (!map.getLayer(PLANE_LAYER))
    map.addLayer({
      id: PLANE_LAYER, type: "symbol", source: AIRCRAFT_SOURCE,
      filter: ["==", ["typeof", ["get", "heading"]], "number"],
      layout: {
        "icon-image": "radar-plane",
        "icon-size": ["case", ["get", "selected"], 1.3, .95],
        "icon-rotate": ["get", "heading"],
        "icon-rotation-alignment": "map",
        "icon-allow-overlap": true,
        "text-field": ["get", "label"],
        "text-offset": [0, 1.6],
        "text-size": 11,
        "text-optional": true,
      },
      paint: { "text-color": "#ffffff", "text-halo-color": "#142133", "text-halo-width": 2 },
    });
  if (!map.getLayer(DOT_LAYER))
    map.addLayer({
      id: DOT_LAYER, type: "circle", source: AIRCRAFT_SOURCE,
      filter: ["!=", ["typeof", ["get", "heading"]], "number"],
      paint: {
        "circle-radius": ["case", ["get", "selected"], 9, 6],
        "circle-color": "#73cbea",
        "circle-stroke-width": 2,
        "circle-stroke-color": "#142133",
      },
    });
}
type Props = {
  frame: ViewState;
  selectedId: string | null;
  follow: boolean;
  style: MapStyle;
  onSelect: (id: string) => void;
  onStopFollow: () => void;
};
export function RadarMap({ frame, selectedId, follow, style, onSelect, onStopFollow }: Props) {
  const container = useRef<HTMLDivElement>(null);
  const currentStyle = useRef(style);
  const map = useRef<LibreMap | null>(null);
  const props = useRef({ frame, selectedId, follow, onSelect, onStopFollow });
  props.current = { frame, selectedId, follow, onSelect, onStopFollow };
  const [error, setError] = useState<string | null>(null);
  // Jedna instance na mount; SSE state ani selection ji nesmí remountovat.
  useEffect(() => {
    if (!container.current) return;
    if (!supported()) {
      setError("Prohlížeč nepodporuje WebGL. Seznam letadel zůstává dostupný.");
      return;
    }
    const instance = new LibreMap({
      container: container.current,
      style: STYLES[currentStyle.current],
      center: [17.67, 49.23],
      zoom: 7.7,
      attributionControl: { compact: true },
    });
    map.current = instance;
    instance.addControl(new NavigationControl({ showCompass: true }), "top-right");
    const refresh = () => {
      addRadarLayers(instance);
      const current = props.current;
      (instance.getSource(AIRCRAFT_SOURCE) as GeoJSONSource | undefined)
        ?.setData(toGeoJSON(current.frame.aircraft, current.selectedId));
      (instance.getSource(TRAIL_SOURCE) as GeoJSONSource | undefined)
        ?.setData(trailGeoJSON(current.frame, current.selectedId));
    };
    instance.on("style.load", refresh);
    const click = (event: { features?: Array<{ properties: Record<string, unknown> | null }> }) => {
      const id = event.features?.[0]?.properties?.id;
      if (typeof id === "string") props.current.onSelect(id);
    };
    instance.on("click", PLANE_LAYER, click);
    instance.on("click", DOT_LAYER, click);
    instance.on("dragstart", () => props.current.onStopFollow());
    const tick = setInterval(() => {
      if (!instance.isStyleLoaded()) return;
      refresh();
      const selected = props.current.frame.aircraft.find(a => a.id === props.current.selectedId);
      if (props.current.follow && selected?.position) {
        instance.easeTo({ center: [selected.position.lon, selected.position.lat], duration: 700 });
      }
    }, 1000);
    instance.on("error", () => {
      // Styl/dlaždice může být krátkodobě offline; letadla jsou i v seznamu.
      setError("Mapový podklad může být nedostupný; živý seznam funguje dál.");
    });
    return () => {
      clearInterval(tick);
      instance.remove();
      map.current = null;
    };
  }, []);
  useEffect(() => {
    if (map.current && currentStyle.current !== style) {
      currentStyle.current = style;
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
