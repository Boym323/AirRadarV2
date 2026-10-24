import test from "node:test";
import assert from "node:assert/strict";
import { catalogFromCsv, csvRows, SOURCE_BLOB, SOURCE_COMMIT } from "./import-airports.mjs";

const header = "ident,type,name,latitude_deg,longitude_deg,iso_country,municipality,icao_code";
const row = (id, type, name, lat, lon, country, municipality = "", icao = "") =>
  [id,type,name,lat,lon,country,municipality,icao].map(v =>
    /[,\n"]/.test(String(v)) ? '"' + String(v).replaceAll('"','""') + '"' : String(v)).join(",");
const sample = [header,
  row("LKTB","medium_airport","Brno–Tuřany",49.1513,16.6944,"CZ","Brno","LKTB"),
  row("LZIB","large_airport","Bratislava",48.1702,17.2127,"SK","Bratislava","LZIB"),
  row("LOWW","large_airport",'Vienna, "Schwechat"',48.1103,16.5697,"AT","Vienna","LOWW"),
  row("XXXX","closed","Uzavřeno",49,18,"CZ"),
  row("AAAA","small_airport","Mimo region",36,21,"GR"),
].join("\r\n");

test("G-06.01: katalog má přesný zdroj a veřejnou licenci", () => {
  const catalog = catalogFromCsv(sample);
  assert.equal(catalog.source.license, "The Unlicense / public domain");
  assert.equal(catalog.source.commit, SOURCE_COMMIT);
  assert.equal(catalog.source.blob, SOURCE_BLOB);
  assert.deepEqual(catalog.airports.map(a=>a.id),["LKTB","LOWW","LZIB"]);
  assert.equal(catalog.airports[0].icao, "LKTB");
  assert.equal(catalog.airports[1].name,'Vienna, "Schwechat"');
  assert.equal(catalog.airports[2].country,"SK");
});

test("G-06.03: CSV uvozovky a CRLF se parsují konzistentně", () => {
  const parsed=[...csvRows('a,b\r\n"abc,def","a""b"\r\n')];
  assert.deepEqual(parsed,[["a","b"],["abc,def",'a"b']]);
  assert.throws(()=>[...csvRows('a,b\n"bad,quote')],/uvozovky/);
});

test("G-06.03: odmítne chybnou hlavičku, duplicitu a souřadnice", () => {
  assert.throws(()=>catalogFromCsv("icao,name\na,b"),/hlavičk/);
  assert.throws(()=>catalogFromCsv([header, ...Array(3).fill(row("LKTB","small_airport","Stejné",49,16,"CZ"))].join("\n")),/Duplicitn/);
  assert.throws(()=>catalogFromCsv([header,
    row("LKTB","medium_airport","Letiště",95,16,"CZ"),
    row("LZIB","medium_airport","Letiště",48,17,"SK"),
    row("LOWW","medium_airport","Letiště",48,16,"AT")].join("\n")),/Neplatn/);
});
