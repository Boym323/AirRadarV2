#!/usr/bin/env node
// G-06.01/.02/.03 → G-05.10 (R12): statický veřejný datový podklad.
// Stahuje se JEDNOU při buildu, nikoli z browseru/radaru za běhu.
// Zdroj OurAirports, The Unlicense (public domain), připnutý git commit i blob.
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const SOURCE_COMMIT = "3b10d0b967cc04c62eb6411939e4e9936a88e701";
export const SOURCE_BLOB = "1efc0826092c91e47add98ad1f0fa700bcd8b805";
const SOURCE_URL = `https://raw.githubusercontent.com/davidmegginson/ourairports-data/${SOURCE_COMMIT}/airports.csv`;
const COUNTRIES = new Set(["CZ", "SK", "AT", "PL", "HU"]);
const TYPES = new Set(["large_airport", "medium_airport", "small_airport"]);
const scriptDir = dirname(fileURLToPath(import.meta.url));
const outputFile = resolve(scriptDir, "../public/data/airports-ourairports.json");

export function* csvRows(text) {
  let row = [], field = "", quoted = false, started = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (quoted) {
      if (ch === '"' && text[i + 1] === '"') { field += '"'; i++; }
      else if (ch === '"') { quoted = false; }
      else { field += ch; }
    } else if (ch === '"' && !started) {
      quoted = true;
      started = true;
    } else if (ch === ',') {
      row.push(field);
      field = ""; started = false;
    } else if (ch === '\n' || ch === '\r') {
      if (ch === '\r' && text[i + 1] === '\n') i++;
      row.push(field);
      if (row.length > 1 || row[0]) yield row;
      row = []; field = ""; started = false;
    } else {
      field += ch;
      started = true;
    }
  }
  if (quoted) throw new Error("Nedokončené CSV uvozovky");
  if (row.length || field) { row.push(field); yield row; }
}

export function catalogFromCsv(csv) {
  const iterator = csvRows(csv);
  const header = iterator.next().value;
  if (!header) throw new Error("CSV nemá hlavičku");
  const keys = ["ident", "type", "name", "latitude_deg", "longitude_deg", "iso_country", "municipality", "icao_code"];
  const positions = Object.fromEntries(keys.map(key => [key, header.indexOf(key)]));
  if (Object.values(positions).some(pos => pos < 0)) throw new Error("Nekompatibilní hlavička OurAirports");
  const airports = [], ids = new Set();
  for (const row of iterator) {
    if (row.length !== header.length) throw new Error("Poškozený počet CSV polí");
    const item = key => row[positions[key]];
    const country = item("iso_country");
    const type = item("type");
    if (!COUNTRIES.has(country) || !TYPES.has(type)) continue;
    const id = item("ident");
    const name = item("name").trim();
    const latitude = Number(item("latitude_deg"));
    const longitude = Number(item("longitude_deg"));
    if (!/^[A-Z0-9-]{2,16}$/.test(id) ||
        !name || name.length > 128 ||
        !item("latitude_deg") || !item("longitude_deg") ||
        !Number.isFinite(latitude) || !Number.isFinite(longitude) ||
        Math.abs(latitude) > 90 || Math.abs(longitude) > 180) {
      throw new Error(`Neplatné letiště: ${id}`);
    }
    if (ids.has(id)) throw new Error(`Duplicitní letiště: ${id}`);
    ids.add(id);
    const icao = item("icao_code");
    const municipality = item("municipality").trim().slice(0, 80);
    airports.push({
      id, ...( /^[A-Z]{4}$/.test(icao) ? { icao } : {}),
      name, type, lat: latitude, lon: longitude, country,
      ...(municipality ? { municipality } : {}),
    });
    if (airports.length > 4000) throw new Error("Zdroji letišť chybí kapacitní limit");
  }
  if (airports.length < 3) throw new Error("Dataset je prázdný nebo příliš malý");
  airports.sort((a, b) => a.id.localeCompare(b.id, "en"));
  return {
    version: 1, source: {
      name: "OurAirports", license: "The Unlicense / public domain",
      repository: "davidmegginson/ourairports-data",
      commit: SOURCE_COMMIT, blob: SOURCE_BLOB,
    },
    countries: [...COUNTRIES],
    airports,
  };
}

async function main() {
  const local = process.env.AIRRADAR_V2_AIRPORT_CSV;
  let bytes;
  if (local) bytes = await readFile(resolve(local));
  else {
    let lastError;
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        const response = await fetch(SOURCE_URL, { signal: AbortSignal.timeout(30000) });
        if (!response.ok) throw new Error(`OurAirports HTTP ${response.status}`);
        bytes = Buffer.from(await response.arrayBuffer());
        if (bytes.length > 20_000_000) throw new Error("CSV přesáhlo 20 MB");
        break;
      } catch (error) { lastError = error; }
    }
    if (!bytes) throw lastError;
  }
  const gitBlob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
  if (gitBlob !== SOURCE_BLOB) throw new Error(`Nepovolená verze OurAirports: ${gitBlob}`);
  const catalog = catalogFromCsv(bytes.toString("utf8"));
  await mkdir(dirname(outputFile), { recursive: true });
  await writeFile(outputFile, JSON.stringify(catalog));
  console.log(`G-06 import: ${catalog.airports.length} validních letišť pro ${catalog.countries.join(", ")}; blob ${gitBlob}`);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
