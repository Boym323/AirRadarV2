import { emitterCategory, type EmitterCategory } from "../../lib/g01-contract";

/** G-05.02/.07: jediný verzovaný SSE model nad kontraktem G-03.
 *  Žádná databáze, druhý ingest nebo fiktivní polohy. */
export type Position = { lat: number; lon: number };
export type Aircraft = {
  id: string;
  lastMessageAt: number;
  lastPositionAt: number | null;
  position: Position | null;
  positionOrigin: string;
  stale: boolean;
  callsign: string | null;
  squawk: string | null;
  /** Volitelné a až po ověření G-03: ADS-B emitter category; nikdy se neodvozuje na klientu. */
  emitterCategory?: EmitterCategory;
  telemetry?: {
    altitudeBaroFt?: number;
    groundSpeedKt?: number;
    trackDegrees?: number;
    verticalRateFpm?: number;
    onGround?: boolean;
  };
};
export type TrailPoint = Position & { at: number };
export type ViewState = {
  aircraft: Aircraft[];
  trails: ReadonlyMap<string, TrailPoint[]>;
  epoch: string | null;
  seq: number | null;
};
export type ApplyResult = "applied" | "invalid" | "resync";
const MAX_AIRCRAFT = 5000;
const TRAIL_TTL_SECONDS = 1200;
const MAX_TRAIL_POINTS = 64;
const AIRCRAFT_ID = /^(icao|nonicao):[0-9a-f]{6}$/;
const isObject = (x: unknown): x is Record<string, unknown> =>
  typeof x === "object" && x !== null && !Array.isArray(x);
const finite = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const validPosition = (p: unknown): p is Position =>
  isObject(p) && finite(p.lat) && finite(p.lon) &&
  Math.abs(p.lat) <= 90 && Math.abs(p.lon) <= 180;
function validateAircraft(input: unknown): input is Aircraft {
  if (!isObject(input) || typeof input.id !== "string" || !AIRCRAFT_ID.test(input.id) ||
    !finite(input.lastMessageAt) || input.lastMessageAt < 0 ||
    (input.lastPositionAt !== null && (!finite(input.lastPositionAt) || input.lastPositionAt < 0)) ||
    (input.position !== null && !validPosition(input.position)) ||
    typeof input.positionOrigin !== "string" ||
    typeof input.stale !== "boolean" ||
    (input.callsign !== null && (typeof input.callsign !== "string" || input.callsign.length > 8)) ||
    (input.squawk !== null && (typeof input.squawk !== "string" || !/^[0-7]{4}$/.test(input.squawk))) ||
    (input.emitterCategory !== undefined && emitterCategory(input.emitterCategory) === undefined)) {
    return false;
  }
  if (input.position !== null && input.lastPositionAt === null) return false;
  if (input.telemetry !== undefined &&
    (!isObject(input.telemetry) ||
      ["altitudeBaroFt", "groundSpeedKt", "trackDegrees", "verticalRateFpm"].some(
        key => key in (input.telemetry as Record<string, unknown>) &&
          !finite((input.telemetry as Record<string, unknown>)[key])
      ))) return false;
  return true;
}
function envelope(input: unknown): input is Record<string, unknown> & {
  epoch: string; seq: number; generatedAt: string;
} {
  return isObject(input) && typeof input.epoch === "string" && input.epoch.length > 0 &&
    Number.isSafeInteger(input.seq) && Number(input.seq) >= 0 &&
    typeof input.generatedAt === "string";
}
/** Store mutuje pouze při atomickém přijetí celého platného snapshotu/delty. */
export class AircraftStore {
  private entries = new Map<string, Aircraft>();
  private paths = new Map<string, TrailPoint[]>();
  private currentEpoch: string | null = null;
  private currentSeq: number | null = null;
  get version() { return { epoch: this.currentEpoch, seq: this.currentSeq }; }
  snapshot(): ViewState {
    return {
      aircraft: Array.from(this.entries.values()),
      trails: new Map(Array.from(this.paths.entries(), ([id, points]) => [id, [...points]])),
      epoch: this.currentEpoch,
      seq: this.currentSeq,
    };
  }
  reset() {
    this.entries.clear();
    this.paths.clear();
    this.currentEpoch = null;
    this.currentSeq = null;
  }
  private remember(target: Map<string, TrailPoint[]>, aircraft: Aircraft, now: number) {
    if (!aircraft.position || aircraft.lastPositionAt === null || aircraft.stale) return;
    if (aircraft.lastPositionAt > now + 2 || now - aircraft.lastPositionAt > TRAIL_TTL_SECONDS) return;
    const old = target.get(aircraft.id) ?? [];
    const cleaned = old.filter(point => now - point.at <= TRAIL_TTL_SECONDS);
    const latest = cleaned.at(-1);
    if (!latest || (aircraft.lastPositionAt > latest.at &&
      (aircraft.position.lat !== latest.lat || aircraft.position.lon !== latest.lon))) {
      cleaned.push({ ...aircraft.position, at: aircraft.lastPositionAt });
    }
    target.set(aircraft.id, cleaned.slice(-MAX_TRAIL_POINTS));
  }
  apply(input: unknown): ApplyResult {
    if (!envelope(input)) return "invalid";
    const now = Date.parse(input.generatedAt) / 1000;
    if (!Number.isFinite(now)) return "invalid";
    if (input.kind === "snapshot") {
      if (!Array.isArray(input.aircraft) || input.aircraft.length > MAX_AIRCRAFT ||
        !input.aircraft.every(validateAircraft)) return "invalid";
      const ids = input.aircraft.map((v: Aircraft) => v.id);
      if (new Set(ids).size !== ids.length) return "invalid";
      if (input.epoch === this.currentEpoch && this.currentSeq !== null &&
        input.seq < this.currentSeq) return "resync";
      const next = new Map<string, Aircraft>();
      const paths = input.epoch === this.currentEpoch ? new Map(this.paths) : new Map<string, TrailPoint[]>();
      for (const aircraft of input.aircraft as Aircraft[]) {
        next.set(aircraft.id, aircraft);
        this.remember(paths, aircraft, now);
      }
      for (const id of paths.keys()) if (!next.has(id)) paths.delete(id);
      this.entries = next;
      this.paths = paths;
      this.currentEpoch = input.epoch;
      this.currentSeq = input.seq;
      return "applied";
    }
    if (input.kind !== "delta" || !Array.isArray(input.upsert) ||
      !Array.isArray(input.removed) || input.upsert.length > MAX_AIRCRAFT ||
      !input.upsert.every(validateAircraft) ||
      !input.removed.every((id: unknown) => typeof id === "string" && AIRCRAFT_ID.test(id))) return "invalid";
    if (this.currentEpoch !== input.epoch || this.currentSeq === null ||
      input.seq !== this.currentSeq + 1) return "resync";
    const ids = (input.upsert as Aircraft[]).map(a => a.id);
    if (new Set(ids).size !== ids.length ||
      new Set(input.removed as string[]).size !== input.removed.length ||
      ids.some(id => (input.removed as string[]).includes(id))) return "invalid";
    const next = new Map(this.entries);
    const paths = new Map(this.paths);
    for (const id of input.removed as string[]) { next.delete(id); paths.delete(id); }
    for (const aircraft of input.upsert as Aircraft[]) {
      next.set(aircraft.id, aircraft);
      this.remember(paths, aircraft, now);
    }
    if (next.size > MAX_AIRCRAFT) return "invalid";
    for (const [id, points] of paths) {
      const recent = points.filter(point => now - point.at <= TRAIL_TTL_SECONDS);
      if (recent.length) paths.set(id, recent); else paths.delete(id);
    }
    this.entries = next;
    this.paths = paths;
    this.currentSeq = input.seq;
    return "applied";
  }
}
