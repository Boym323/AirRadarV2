import { describe, expect, it } from "vitest";
import { AIRPORT_SOURCE_REVISION, parseAirportCatalog, airportsToGeoJSON } from "../src/radar/airport-catalog";
const airport=(id:string,lon:number)=>({
  id,name:"Testovací letiště " + id,type:"small_airport",
  lat:49.23,lon,country:"CZ",municipality:"Test"});

const fixture=(rows:unknown[])=>({
  version:1,source:{name:"OurAirports",commit:AIRPORT_SOURCE_REVISION},
  airports:rows
});
describe("G-05.10/R12 ← G-06: atomický statický letištní katalog",()=>{
  it("validuje zdroj a připíná přesné souřadnice bez odhadů",()=>{
    const rows=[airport("CZ-TEST1",17.67),airport("CZ-TEST2",17.69),airport("CZ-TEST3",17.71)];
    const list=parseAirportCatalog(fixture(rows));
    expect(list).toHaveLength(3);
    const geo=airportsToGeoJSON(list,"CZ-TEST1");
    expect(geo.features.map(f=>f.id)).toEqual(rows.map(r=>r.id));
    expect(geo.features[0].geometry.coordinates).toEqual([17.67,49.23]);
    expect(geo.features[0].properties?.selected).toBe(true);
    expect(geo.features[1].properties?.selected).toBe(false);
  });
  it("nepřijme duplicitní, nevalidní ani příliš velký katalog",()=>{
    const rows=[airport("CZ-TEST1",17.67),airport("CZ-TEST2",17.69),airport("CZ-TEST3",17.71)];
    expect(()=>parseAirportCatalog(fixture([rows[0],rows[0],rows[2]]))).toThrow();
    expect(()=>parseAirportCatalog(fixture([rows[0],{...rows[1],lat:91},rows[2]]))).toThrow();
    expect(()=>parseAirportCatalog(fixture([rows[0],{...rows[1],name:""},rows[2]]))).toThrow();
    expect(()=>parseAirportCatalog(fixture(rows.slice(0,2)))).toThrow();
    expect(()=>parseAirportCatalog({...fixture(rows),source:{commit:"unverified"}})).toThrow();
    expect(()=>parseAirportCatalog(fixture(Array(4001).fill(rows[0])))).toThrow();
  });
});
