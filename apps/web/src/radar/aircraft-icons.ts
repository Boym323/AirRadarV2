/**
 * G-05.03 / R03: Původní vektorové siluety shora, společná 64×64 mřížka.
 * Žádné externí ikony ani heuristiky podle callsignu/rychlosti/výšky.
 * Kategorie je pouze volitelné metadata budoucího validovaného veřejného DTO.
 */
import type { Map as LibreMap } from "maplibre-gl";

export const AIRCRAFT_PATHS = {
  "generic-fixedwing": "M 32 4 L 38 23 L 57 37 L 56 44 L 37 39 L 36 52 L 44 58 L 43 62 L 32 57 L 21 62 L 20 58 L 28 52 L 27 39 L 8 44 L 7 37 L 26 23 Z",
  "large-fixedwing": "M 32 3 Q 36 4 38 19 L 41 29 L 60 41 L 59 48 L 39 43 L 37 53 L 46 59 L 45 63 L 32 58 L 19 63 L 18 59 L 27 53 L 25 43 L 5 48 L 4 41 L 23 29 L 26 19 Q 28 4 32 3 Z",
  "light-fixedwing": "M 32 5 L 36 26 L 60 35 L 59 41 L 36 37 L 35 52 L 42 57 L 41 61 L 32 57 L 23 61 L 22 57 L 29 52 L 28 37 L 5 41 L 4 35 L 28 26 Z",
  "high-performance": "M 32 4 L 38 22 L 58 45 L 54 49 L 37 40 L 37 53 L 44 59 L 43 62 L 32 58 L 21 62 L 20 59 L 27 53 L 27 40 L 10 49 L 6 45 L 26 22 Z",
  "rotorcraft": "M 30 5 Q 32 2 34 5 L 35 23 L 59 25 L 59 31 L 36 32 L 36 46 L 44 51 L 43 55 L 35 53 L 35 61 L 29 61 L 29 53 L 21 55 L 20 51 L 28 46 L 28 32 L 5 31 L 5 25 L 29 23 Z",
  "glider": "M 32 6 L 35 27 L 62 31 L 62 36 L 35 34 L 34 54 L 43 58 L 42 62 L 32 58 L 22 62 L 21 58 L 30 54 L 29 34 L 2 36 L 2 31 L 29 27 Z",
  "ultralight": "M 32 9 L 37 29 L 53 33 L 52 40 L 37 38 L 36 52 L 42 57 L 41 60 L 32 56 L 23 60 L 22 57 L 28 52 L 27 38 L 12 40 L 11 33 L 27 29 Z",
  "lighter-than-air": "M 32 4 C 14 4 12 18 16 34 Q 20 47 29 51 L 29 58 L 35 58 L 35 51 Q 44 47 48 34 C 52 18 50 4 32 4 Z M 28 58 L 36 58 L 36 63 L 28 63 Z",
  "uav": "M 32 16 L 35 25 L 50 16 L 54 21 L 39 33 L 54 44 L 50 49 L 35 40 L 34 55 L 30 55 L 29 40 L 14 49 L 10 44 L 25 33 L 10 21 L 14 16 L 29 25 Z",
  "ground-unit": "M 17 11 L 47 11 L 53 23 L 53 49 L 47 55 L 17 55 L 11 49 L 11 23 Z M 17 25 L 47 25 L 47 43 L 17 43 Z",
} as const;

export type AircraftIcon = keyof typeof AIRCRAFT_PATHS;
const CATEGORY_ICON: Record<string, AircraftIcon> = {
  A1: "light-fixedwing", A2: "light-fixedwing",
  A3: "large-fixedwing", A4: "large-fixedwing", A5: "large-fixedwing",
  A6: "high-performance", A7: "rotorcraft",
  B1: "glider", B2: "lighter-than-air", B4: "ultralight",
  B6: "uav", C 1: "ground-unit", C 2: "ground-unit",
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
