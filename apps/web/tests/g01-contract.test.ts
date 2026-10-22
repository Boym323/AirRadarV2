import { describe, expect, it } from "vitest";
import initial from "../../../contracts/fixtures/readsb/initial.json";
import repeated from "../../../contracts/fixtures/readsb/repeated.json";
import changed from "../../../contracts/fixtures/readsb/changed.json";
import invalid from "../../../contracts/fixtures/readsb/invalid.json";
import empty from "../../../contracts/fixtures/readsb/empty.json";
import expected from "../../../contracts/fixtures/readsb/expected.json";
import golden from "../../../contracts/fixtures/sse/sequence.json";
import {
  aircraftKey, applyEvent, isSqlEligible, normalizeReadsb, parsePublicAircraft,
  parseStreamEvent, type Replica
} from "../lib/g01-contract";

describe("G-01: shodný Rust/TypeScript datový kontrakt", () => {
  it("normalizuje syntetický readsb a uchová oddělenou non-ICAO identitu", () => {
    const obs = normalizeReadsb(initial, "rx-primary", 1792627200);
    expect(obs).toEqual(expected.initial);
    expect(aircraftKey("ABC123")).toBe("icao:abc123");
    expect(aircraftKey("~ABC123")).toBe("nonicao:abc123");
    expect(isSqlEligible(obs[0])).toBe(true);
    expect(isSqlEligible(obs[1])).toBe(false);
    expect(isSqlEligible(obs[2])).toBe(false);
    const second = normalizeReadsb(repeated, "rx-primary", 1792627201);
    expect(second[0].positionAtUtc).toBe(obs[0].positionAtUtc);
    const next = normalizeReadsb(changed, "rx-primary", 1792627205);
    expect(next[0].positionAtUtc).toBe(expected.changedPositionAtUtc);
  });
  it("nepovolí neplatné polohy, squawk a rozhozené hodiny", () => {
    expect(normalizeReadsb(invalid, "rx-primary", 1792627200)).toEqual([]);
    expect(normalizeReadsb(empty, "rx-primary", 1792627210)).toEqual([]);
    expect(() => normalizeReadsb(initial, "bad-station", 1792627200)).toThrow();
    expect(() => normalizeReadsb(initial, "rx-primary", 1792627400)).toThrow();
  });
  it("SSE snapshot → delta → removed, odmítne gap a restart bez snapshotu", () => {
    let r: Replica = { epoch: null, seq: null, aircraft: new Map() };
    r = applyEvent(r, parseStreamEvent(golden.snapshot));
    expect(r.aircraft.size).toBe(1);
    r = applyEvent(r, parseStreamEvent(golden.delta));
    expect(r.aircraft.get("icao:abc123")?.squawk).toBe("7700");
    r = applyEvent(r, parseStreamEvent(golden.removed));
    expect(r.aircraft.size).toBe(0);
    expect(() => applyEvent(r, parseStreamEvent(golden.gap))).toThrow();
    r = applyEvent(r, parseStreamEvent(golden.restart));
    expect(r.seq).toBe(0);
  });
  it("bezpečně odmítne interní receiverId a neplatné veřejné DTO", () => {
    const good = golden.snapshot.aircraft[0];
    expect(parsePublicAircraft(good).id).toBe("icao:abc123");
    expect(() => parsePublicAircraft({ ...good, receiverId: "rx-primary" })).toThrow();
    expect(() => parsePublicAircraft({ ...good, position: { lat: 190, lon: 14 } })).toThrow();
    expect(() => parseStreamEvent({ ...golden.snapshot, aircraft: [good, good] })).toThrow();
    expect(() => parseStreamEvent({ ...golden.delta, seq: -1 })).toThrow();
  });
});
