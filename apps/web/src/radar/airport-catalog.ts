/** G-06.02/.07 → G-05.10 (R12): lokální build-time OurAirports katalog.
 * Žádná potřeba nového serveru, dalšího SQL nebo živého API přijímače. */
import type { FeatureCollection, Point } from "geojson";

export const AIRPORT_DATA_PATH = "/data/airports-ourairports.json";
export const AIRPORT_SOURCE_REVISION = "3b10d0b967cc04c62eb6411939e4e9936a88e701";
export const MAX_AIRPORTS = 4000;
export type Airport = {
  id: string;
  icao?: string;
  name: string;
  type: "large_airport" | "medium_airport" | "small_airport";
  lat: number;
  lon: number;
  country: "CZ" | "SK" | "AT" | "PL" | "HU";
  municipality?: string;
};
const valid = (x: unknown): x is Record<string,unknown> => !!x && typeof x==="object" && !Array.isArray(x);
const validPosition = (lat: unknown, lon: unknown): lat is number =>
  typeof lat==="number" && typeof lon==="number" && Number.isFinite(lat) &&
  Number.isFinite(lon) && Math.abs(lat)<=90 && Math.abs(lon)<=180;
export function parseAirportCatalog(data: unknown): Airport[] {
  if (!valid(data) || data.version!==1 || !valid(data.source) ||
    data.source.commit!==AIRPORT_SOURCE_REVISION ||
    !Array.isArray(data.airports) || data.airports.length<3 ||
    data.airports.length>MAX_AIRPORTS) throw new Error("Neplatný/verzově neznámý letištní katalog");
  const seen=new Set<string>();
  const output: Airport[]=[];
  for (const item of data.airports) {
    if (!valid(item) || typeof item.id!=="string" ||
      !/^[A-Z0-9-]{2,16}$/.test(item.id) ||
      seen.has(item.id) ||
      typeof item.name!=="string" || item.name.trim().length<1 || item.name.length>128 ||
      !validPosition(item.lat,item.lon) ||
      !["CZ","SK","AT","PL","HU"].includes(String(item.country)) ||
      !["large_airport","medium_airport","small_airport"].includes(String(item.type)) ||
      (item.icao!==undefined && (typeof item.icao!=="string" || !/^[A-Z]{4}$/.test(item.icao))) ||
      (item.municipality!==undefined &&
        (typeof item.municipality!=="string" || item.municipality.length>80))) {
      throw new Error("Poškozený záznam letiště");
    }
    seen.add(item.id);
    output.push(item as Airport);
  }
  return output;
}
export async function loadAirportCatalog(signal: AbortSignal): Promise<Airport[]> {
  const response=await fetch(AIRPORT_DATA_PATH,{signal,cache:"force-cache"});
  if (!response.ok) throw new Error("Katalog letišť není dostupný");
  return parseAirportCatalog(await response.json());
}

export function airportsToGeoJSON(airports: Airport[], selectedId: string|null): FeatureCollection<Point> {
  return {
    type:"FeatureCollection",
    features: airports.map(a=>({
      type:"Feature", id:a.id,
      geometry:{type:"Point",coordinates:[a.lon,a.lat]},
      properties:{
        id:a.id,
        code:a.icao || a.id,
        label:a.icao || a.id,
        name:a.name,
        selected:a.id===selectedId,
        type:a.type,
      },
    })),
  };
}
