/**
 * G-05.03 / R03: Původní vektorové siluety shora, společná 64×64 mřížka.
 * Žádné externí ikony ani heuristiky podle callsignu/rychlosti/výšky.
 * Kategorie je pouze volitelné metadata budoucího validovaného veřejného DTO.
 */
import type { Map as LibreMap } from "maplibre-gl";

export const AIRCRAFT_PATHS = {
  "generic-fixedwing": "M32 4 L38 23 L57 37 L56 44 L37 39 L36 52 L44 58 L43 62 L32 57 L21 62 L20 58 L28 52 L27 39 L8 44 L7 37 L26 23 Z",
  "large-fixedwing": "M32 3 Q36 4 38 19 L41 29 L60 41 L59 48 L39 43 L37 53 L46 59 L45 63 L32 58 L19 63 L18 59 L27 53 L25 43 L5 48 L4 41 L23 29 L26 19 Q28 4 32 3 Z",
  "light-fixedwing": "M32 5 L36 26 L60 35 L59 41 L36 37 L35 52 L42 57 L41 61 L32 57 L23 61 L22 57 L29 52 L28 37 L5 41 L4 35 L28 26 Z",
  "high-performance": "M32 4 L38 22 L58 45 L54 49 L37 40 L37 53 L44 59 L43 62 L32 58 L21 62 L20 59 L27 53 L27 40 L10 49 L6 45 L26 22 Z",
  "rotorcraft": "M30 5 Q32 2 34 5 L35 23 L59 25 L59 31 L36 32 L36 46 L44 51 L43 55 L35 53 L35 61 L29 61 L29 53 L21 55 L20 51 L28 46 L28 32 L5 31 L5 25 L29 23 Z",
  "glider": "M32 6 L35 27 L62 31 L62 36 L35 34 L34 54 L43 58 L42 62 L32 58 L22 62 L21 58 L30 54 L29 34 L2 36 L2 31 L29 27 Z",
  "ultralight": "M32 9 L37 29 L53 33 L52 40 L37 38 L36 52 L42 57 L41 60 L32 56 L23 60 L22 57 L28 52 L27 38 L12 40 L11 33 L27 29 Z",
  "lighter-than-air": "M32 4 C14 4 12 18 16 34 Q20 47 29 51 L29 58 L35 58 L35 51 Q44 47 48 34 C52 18 50 4 32 4 Z M28 58 L36 58 L36 63 L28 63 Z",
  "uav": "M32 16 L35 25 L50 16 L54 21 L39 33 L54 44 L50 49 L35 40 L34 55 L30 55 L29 40 L14 49 L10 44 L25 33 L10 21 L14 16 L29 25 Z",
  "ground-unit": "M17 11 L47 11 L53 23 L53 49 L47 55 L17 55 L11 49 L11 23 Z M17 25 L47 25 L47 43 L17 43 Z",
} as const;

export type AircraftIcon = keyof typeof AIRCRAFT_PATHS;
const CATEGORY_ICON: Record<string, AircraftIcon> = {
  A1: "light-fixedwing", A2: "light-fixedwing",
  A3: "large-fixedwing", A4: "large-fixedwing", A5: "large-fixedwing",
  A6: "high-performance", A7: "rotorcraft",
  B1: "glider", B2: "lighter-than-air", B4: "ultralight",
  B6: "uav", C1: "ground-unit", C2: "ground-unit",
};

/** Záměrně bez odhadu; neznámý směr = MapLibre kruh, ne otočené letadlo. */
export function validGroundTrack(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0 && value < 360;
}

export function iconForAircraft(category: unknown, track: unknown): AircraftIcon | null {
  if (!validGroundTrack(track)) return null;
  if (typeof category !== "string") return "generic-fixedwing";
  return CATEGORY_ICON[category] ?? "generic-fixedwing";
}

/** Kreslit originální SVG path segmenty jednou do canvas image registru MapLibre. */
export function registerAircraftIcons(map: Pick<LibreMap, "hasImage" | "addImage">): void {
  for (const [name, path] of Object.entries(AIRCRAFT_PATHS)) {
    const id = `radar-${name}`;
    if (map.hasImage(id)) continue;
    const canvas = document.createElement("canvas");
    canvas.width = 64;
    canvas.height = 64;
    const context = canvas.getContext("2d");
    if (!context) continue;
    const silhouette = new Path2D(path);
    context.lineJoin = "round";
    context.lineCap = "round";
    context.fillStyle = "#f9fcff";
    context.strokeStyle = "#17314d";
    context.lineWidth = 3;
    context.stroke(silhouette);
    context.fill(silhouette);
    map.addImage(id, context.getImageData(0, 0, 64, 64), { pixelRatio: 2 });
  }
}
