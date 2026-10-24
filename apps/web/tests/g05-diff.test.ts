import { describe, expect, it } from "vitest";
import type { Feature, FeatureCollection, Point } from "geojson";
import { diffAircraftGeoJSON, shouldUseAircraftDiff } from "../src/radar/aircraft-diff";

const aircraft = (id:string, lon:number, selected=false):Feature<Point> => ({
  type:"Feature", id,
  geometry:{type:"Point", coordinates:[lon,49.1]},
  properties:{id, label:id, heading:90,selected,icon:"generic-fixedwing"},
});
const fc = (...features:Feature<Point>[]):FeatureCollection<Point> =>
  ({type:"FeatureCollection",features});

describe("G-05.11 — bezpečný inkrementální GeoJSON worker update",()=>{
  it("rozliší přidání, odstranění, změnu geometrie a výběrového popisku",()=>{
    const before=fc(aircraft("a",16),aircraft("b",17),aircraft("gone",18));
    const after=fc(aircraft("a",16.1),aircraft("b",17,true),aircraft("new",19));
    const change=diffAircraftGeoJSON(before,after);
    expect(change?.changes).toBe(4);
    expect(change?.diff.remove).toEqual(["gone"]);
    expect(change?.diff.add?.map(f=>f.id)).toEqual(["new"]);
    expect(change?.diff.update?.map(f=>f.id)).toEqual(["a","b"]);
    expect(change?.diff.update?.[0].newGeometry).toEqual(after.features[0].geometry);
    expect(change?.diff.update?.[1].newGeometry).toBeUndefined();
    expect(change?.diff.update?.[1].removeAllProperties).toBe(true);
    expect(change?.diff.update?.[1].addOrUpdateProperties).toContainEqual({key:"selected",value:true});
  });
  it("pro identické pohledy nemění worker; zduplikovaná/neplatná id mají setData fallback",()=>{
    const value=fc(aircraft("abc",17),aircraft("def",18));
    expect(diffAircraftGeoJSON(value,fc(...value.features))?.changes).toBe(0);
    expect(diffAircraftGeoJSON(fc(aircraft("a",17),aircraft("a",18)),value)).toBeNull();
    expect(diffAircraftGeoJSON(value,fc(aircraft("a",17),aircraft("a",18)))).toBeNull();
  });
  it("udrží jednoduché setData při změně epochy, malém počtu a hromadném snapshotu",()=>{
    expect(shouldUseAircraftDiff(2000,2000,200,true)).toBe(true);
    expect(shouldUseAircraftDiff(500,500,90,true)).toBe(true);
    expect(shouldUseAircraftDiff(100,100,10,true)).toBe(false);
    expect(shouldUseAircraftDiff(500,500,400,true)).toBe(false);
    expect(shouldUseAircraftDiff(2000,2000,200,false)).toBe(false);
    expect(shouldUseAircraftDiff(2000,2000,0,true)).toBe(false);
  });
  it("pro 2 000 identit mění pouze 10 % letadel bez ztráty dalších ID",()=>{
    const before=fc(...Array.from({length:2000},(_,i)=>aircraft(String(i),16+i/1000)));
    const after=fc(...before.features.map((row,i)=>
      i%10===0 ? aircraft(String(i),16+i/1000+.0001) : row));
    const patch=diffAircraftGeoJSON(before,after);
    expect(patch?.changes).toBe(200);
    expect(patch?.diff.add).toBeUndefined();
    expect(patch?.diff.remove).toBeUndefined();
    expect(patch?.diff.update).toHaveLength(200);
    expect(shouldUseAircraftDiff(2000,2000,patch!.changes,true)).toBe(true);
  });
});
