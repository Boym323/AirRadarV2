import { describe, expect, it } from "vitest";
import { AircraftStore, type Aircraft } from "../src/radar/aircraft-store";
import { toGeoJSON, mapCallsign, mapLabel, trailGeoJSON } from "../src/radar/RadarMap";
import { AIRCRAFT_PATHS, iconForAircraft, validGroundTrack } from "../src/radar/aircraft-icons";

const aircraft = (id = "icao:abc123", at = 1000): Aircraft => ({
  id, lastMessageAt: at, lastPositionAt: at,
  position: { lon: 17.5, lat: 49.2 }, positionOrigin: "adsb", stale: false,
  callsign: "OKTEST", squawk: null, telemetry: { trackDegrees: 92, altitudeBaroFt: 6500 },
});
const generatedAt = (at: number) => new Date(at * 1000).toISOString();
const snapshot = (items: Aircraft[], epoch = "epoch-a", seq = 0, now = 1000) =>
  ({ kind: "snapshot", epoch, seq, generatedAt: generatedAt(now), aircraft: items });
const delta = (upsert: Aircraft[], removed: string[], seq: number, epoch = "epoch-a", now = 1010) =>
  ({ kind: "delta", epoch, seq, generatedAt: generatedAt(now), upsert, removed });

describe("G-05.02 atomický G-03 SSE adapter", () => {
  it("přijme snapshot + delta, aktualizuje identitu a odstraní správný cíl", () => {
    const store = new AircraftStore();
    expect(store.apply(snapshot([aircraft(), aircraft("nonicao:abc123")]))).toBe("applied");
    expect(store.apply(delta([{ ...aircraft(), callsign: "OKNEW" }], ["nonicao:abc123"], 1))).toBe("applied");
    expect(store.snapshot().aircraft).toHaveLength(1);
    expect(store.snapshot().aircraft[0].callsign).toBe("OKNEW");
    expect(store.apply(delta([], ["icao:abc123"], 2))).toBe("applied");
    expect(store.snapshot().aircraft).toHaveLength(0);
    expect(store.snapshot().trails.size).toBe(0);
  });
  it("odmítne přeskočené seq a epoch, další snapshot atomicky obnoví stav", () => {
    const store = new AircraftStore();
    expect(store.apply(snapshot([aircraft()]))).toBe("applied");
    expect(store.apply(delta([aircraft("icao:bbbbbb")], [], 3))).toBe("resync");
    expect(store.snapshot().aircraft).toHaveLength(1);
    expect(store.apply(delta([], [], 1, "epoch-new"))).toBe("resync");
    expect(store.apply(snapshot([aircraft("icao:cccccc")], "epoch-new", 0))).toBe("applied");
    expect(store.snapshot().aircraft.map(a => a.id)).toEqual(["icao:cccccc"]);
  });
  it("nepovolí duplicitní ID, nedovolený rozsah GPS ani část chybných delt", () => {
    const store = new AircraftStore();
    expect(store.apply(snapshot([aircraft(), aircraft()]))).toBe("invalid");
    expect(store.apply(snapshot([aircraft()]))).toBe("applied");
    expect(store.apply(delta([aircraft("icao:bbbbbb"), {...aircraft(), position: {lat: 96, lon: 17}}],[],1))).toBe("invalid");
    expect(store.snapshot().aircraft).toHaveLength(1);
    expect(store.version.seq).toBe(0);
  });
  it("uchovává jen skutečné body, maximálně 64 a 20 minut", () => {
    const store = new AircraftStore();
    store.apply(snapshot([aircraft()], "e", 0));
    for(let i = 1; i <= 90; i++) {
      const item = { ...aircraft("icao:abc123", 1000 + i), position: { lon: 17.5 + i * 0.0001, lat: 49.2 }};
      expect(store.apply(delta([item], [], i, "e", 1000 + i))).toBe("applied");
    }
    const trail = store.snapshot().trails.get("icao:abc123")!;
    expect(trail).toHaveLength(64);
    expect(trail.at(-1)?.at).toBe(1090);
    expect(store.apply(snapshot([aircraft("icao:abc123", 3000)], "e", 91, 3000))).toBe("applied");
    expect(store.snapshot().trails.get("icao:abc123")).toHaveLength(1);
  });
  it("mapová projekce obsahuje jen platné čerstvé cíle a respektuje skutečný heading", () => {
    const points = toGeoJSON([aircraft(), {...aircraft("icao:bbbbbb"), stale: true}], "icao:abc123");
    expect(points.features.map(x => x.id)).toEqual(["icao:abc123"]);
    expect(points.features[0].properties?.heading).toBe(92);
    expect(points.features[0].properties?.selected).toBe(true);
    const unknown = toGeoJSON([{...aircraft(), telemetry: undefined}], null);
    expect(unknown.features[0].properties?.heading).toBeNull();
  });
});

describe("G-05.03 — veřejný kategoriový kontrakt", () => {

  it("G-05 atomický SSE store odmítne podvrženou kategorii", () => {
    const store = new AircraftStore();
    expect(store.apply(snapshot([{...aircraft(), emitterCategory:"A7"}]))).toBe("applied");
    expect(store.snapshot().aircraft[0].emitterCategory).toBe("A7");
    const invalid = {...aircraft(), emitterCategory:"C0"} as unknown as Aircraft;
    expect(store.apply(delta([invalid], [], 1))).toBe("invalid");
    expect(store.version.seq).toBe(0);
    expect(store.snapshot().aircraft[0].emitterCategory).toBe("A7");
    expect(store.apply(delta([{...aircraft(), emitterCategory:"B1"}], [], 1))).toBe("applied");
    expect(store.snapshot().aircraft[0].emitterCategory).toBe("B1");
    expect(toGeoJSON(store.snapshot().aircraft, null).features[0].properties?.icon).toBe("glider");
  });
  it("2 000 stabilních cílů používá jediný GeoJSON source bez DOM markerů", () => {
    const targets = Array.from({length: 2000}, (_, i) => ({
      ...aircraft(`icao:${i.toString(16).padStart(6, "0")}`),
      position: {lat: 48 + (i % 60) * .01, lon: 14 + (i % 90) * .01},
      emitterCategory: (i % 2 === 0 ? "A7" : "B1") as "A7" | "B1",
    }));
    const points = toGeoJSON(targets, targets[0].id).features;
    expect(points).toHaveLength(2000);
    expect(new Set(points.map(f => f.id)).size).toBe(2000);
    expect(points[0].properties?.icon).toBe("rotorcraft");
    expect(points[1].properties?.icon).toBe("glider");
  });

});

describe("G-05.03 / R03 — vlastní směrové symboly", () => {
  it("má originální vektorovou siluetu pro každou podporovanou kategorii", () => {
    expect(Object.keys(AIRCRAFT_PATHS)).toHaveLength(10);
    for (const shape of Object.values(AIRCRAFT_PATHS)) {
      expect(shape).toMatch(/^M/);
      expect(shape).toMatch(/Z$/);
      expect(shape.length).toBeGreaterThan(35);
    }
    for (const [categories, expected] of [
      [["A1", "A2"], "light-fixedwing"],
      [["A3", "A4", "A5"], "large-fixedwing"],
      [["A6"], "high-performance"],
      [["A7"], "rotorcraft"],
      [["B1"], "glider"],
      [["B2"], "lighter-than-air"],
      [["B4"], "ultralight"],
      [["B6"], "uav"],
      [["C1", "C2"], "ground-unit"],
    ] as const) {
      for (const category of categories) expect(iconForAircraft(category, 90)).toBe(expected);
    }
  });
  it("nevymýšlí kategorii, neznámý směr ani neplatný track", () => {
    for (const missing of [undefined, null, "A0", "B0", "C0", "B3", "A9", "military", 123]) {
      expect(iconForAircraft(missing, 0)).toBe("generic-fixedwing");
    }
    for (const invalid of [undefined, null, NaN, Infinity, -1, 360, "90"]) {
      expect(validGroundTrack(invalid)).toBe(false);
      expect(iconForAircraft("A7", invalid)).toBeNull();
    }
    for (const correct of [0, 90, 180, 270, 359.9]) {
      expect(validGroundTrack(correct)).toBe(true);
    }
  });
  it("GeoJSON používá důvěryhodný track, fallback, nadmořskou výšku a stabilní id", () => {
    const rows: Aircraft[] = [
      { ...aircraft(), emitterCategory: "A7" },
      { ...aircraft("icao:bbbbbb"), emitterCategory: "B1", telemetry: {trackDegrees: 0}},
      { ...aircraft("icao:cccccc"), telemetry: {altitudeBaroFt: 6100}},
      { ...aircraft("icao:dddddd"), callsign: null, emitterCategory: "A3" },
    ];
    const output = toGeoJSON(rows, "icao:abc123");
    expect(output.features.map(x => x.id)).toEqual(rows.map(x => x.id));
    expect(output.features[0].properties).toMatchObject({
      icon: "rotorcraft", heading: 92, label: "OKTEST",
      selected: true,
    });
    expect(String(output.features[0].properties?.detailLabel).replace(/\s+/g, " ")).toBe("OKTEST 6 500 ft");
    expect(output.features[1].properties?.icon).toBe("glider");
    expect(output.features[2].properties?.icon).toBeNull();
    expect(output.features[3].properties?.icon).toBe("large-fixedwing");
    expect(output.features[3].properties?.detailLabel).toBe("");
  });
});

describe("G-05.04 / R04 — popisky a priorita vybraného cíle", () => {
  it("odstraňuje podvržené řídicí znaky a nikdy nevymýšlí neznámé údaje", () => {
    expect(mapCallsign(" OK-123 ")).toBe("OK-123");
    for (const value of ["A\\nB", "<script>", "P R A H A", "abcdefgh9", "☃"]) {
      expect(mapCallsign(value)).toBe("");
    }
    expect(mapLabel({...aircraft(), callsign:null, telemetry:undefined})).toEqual({
      label:"", detailLabel:"",
    });
    expect(mapLabel({...aircraft(), telemetry:{trackDegrees:90}})).toEqual({
      label:"OKTEST", detailLabel:"OKTEST",
    });
    expect(mapLabel({...aircraft(), telemetry:{altitudeBaroFt: Infinity}}).detailLabel).toBe("OKTEST");
    expect(mapLabel({...aircraft(), telemetry:{altitudeBaroFt: 100000}}).detailLabel).toBe("OKTEST");
  });
  it("GeoJSON zaručuje krátký text, omezenou výšku a selected prioritu", () => {
    const items=[aircraft(), {...aircraft("icao:bbbbbb"), callsign:"FAKE\\nAB"}];
    const fc=toGeoJSON(items,"icao:abc123");
    expect(fc.features[0].properties?.selected).toBe(true);
    expect(fc.features[0].properties?.detailLabel?.includes("ft")).toBe(true);
    expect(fc.features[1].properties?.label).toBe("");
    expect(fc.features[1].properties?.detailLabel).toBe("");
  });
});

describe("G-05.07 / R08 — věrná skutečná stopa bez interpolace výpadků", () => {
  const trail=(coords: [number,number,number][]) => ({
    aircraft: [aircraft()], epoch: "e", seq: 3,
    trails: new Map([["icao:abc123",coords.map(([lon,lat,at])=>({lon,lat,at}))]]),
  });
  it("prázdná/samotná stopa se nevykreslí", () => {
    expect(trailGeoJSON(trail([]),"icao:abc123").features).toHaveLength(0);
    expect(trailGeoJSON(trail([[17,49,1000]]),"icao:abc123").features).toHaveLength(0);
    expect(trailGeoJSON(trail([[17,49,1000],[18,49,1010]]),null).features).toHaveLength(0);
  });
  it("objeví se spojnice jen ze skutečných následných bodů a přeruší se při výpadku", () => {
    const frame=trail([
      [17,49,1000],[17.1,49.1,1010],[17.2,49.2,1400],
      [17.3,49.3,1410],[17.4,49.4,1420]
    ]);
    const geom=trailGeoJSON(frame,"icao:abc123").features[0].geometry;
    expect(geom.type).toBe("MultiLineString");
    expect(geom.coordinates).toEqual([
      [[17,49],[17.1,49.1]],
      [[17.2,49.2],[17.3,49.3],[17.4,49.4]],
    ]);
  });
  it("nespojuje letadla skrz celý svět při přechodu přes ±180°", () => {
    const geometry=trailGeoJSON(trail([
      [179.9,15,1000],[-179.9,15.1,1010],[-179.8,15.2,1020]
    ]),"icao:abc123").features[0].geometry;
    expect(geometry.coordinates).toEqual([[[-179.9,15.1],[-179.8,15.2]]]);
  });
  it("při odstranění cíle SSE odebere i jeho celou stopu", () => {
    const store=new AircraftStore();
    expect(store.apply(snapshot([aircraft()]))).toBe("applied");
    expect(store.apply(delta([{
      ...aircraft("icao:abc123",1010),position:{lon:17.7,lat:49.2}
    }],[],1,"epoch-a",1010))).toBe("applied");
    expect(trailGeoJSON(store.snapshot(),"icao:abc123").features).toHaveLength(1);
    expect(store.apply(delta([],["icao:abc123"],2))).toBe("applied");
    expect(store.snapshot().trails.size).toBe(0);
    expect(trailGeoJSON(store.snapshot(),"icao:abc123").features).toHaveLength(0);
  });
});
