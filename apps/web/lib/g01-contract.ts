// G-01: stejný datový kontrakt jako Rust, bez živého přijímače nebo databáze.
export type Position = { lat: number; lon: number };
export type FlightData = { altitudeBaroFt?: number; groundSpeedKt?: number; trackDegrees?: number; verticalRateFpm?: number; onGround?: boolean };
export type TimeBasis = "derived_readsb" | "receiver_utc" | "server_arrival" | "unknown";
export type PositionOrigin = "adsb" | "mlat" | "tisb" | "rebroadcast" | "estimated" | "unknown";
export type Observation = {
  receiverId: string;
  transport: "readsb_json" | "beast_tcp";
  aircraftKey: string;
  receivedAtUtc: number;
  messageAtUtc: number;
  timeBasis: TimeBasis;
  positionOrigin: PositionOrigin;
  positionAtUtc: number | null;
  position: Position | null;
  callsign: string | null;
  squawk: string | null;
  telemetry?: FlightData;
};
export type PublicAircraft = {
  id: string;
  lastMessageAt: number;
  lastPositionAt: number | null;
  position: Position | null;
  positionOrigin: PositionOrigin;
  stale: boolean;
  callsign: string | null;
  squawk: string | null;
  telemetry?: FlightData;
};
export type Snapshot = {
  kind: "snapshot"; epoch: string; seq: number; generatedAt: string;
  aircraft: PublicAircraft[];
};
export type Delta = {
  kind: "delta"; epoch: string; seq: number; generatedAt: string;
  upsert: PublicAircraft[]; removed: string[];
};
export type StreamEvent = Snapshot | Delta;
export type Replica = { epoch: string | null; seq: number | null; aircraft: Map<string, PublicAircraft> };

const isObject = (x: unknown): x is Record<string, unknown> =>
  typeof x === "object" && x !== null && !Array.isArray(x);
const number = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const age = (x: unknown): x is number => number(x) && x >= 0;
const keyRegex = /^(icao|nonicao):[0-9a-f]{6}$/;
const observedOrigin = (x: unknown): PositionOrigin => {
  switch (x) {
    case "adsb_icao":
    case "adsb_icao_nt":
    case "adsb_other": return "adsb";
    case "mlat": return "mlat";
    case "tisb_icao":
    case "tisb_other":
    case "tisb_trackfile": return "tisb";
    case "adsr_icao":
    case "adsr_other": return "rebroadcast";
    default: return "unknown";
  }
};
export function aircraftKey(x: unknown): string | null {
  if (typeof x !== "string") return null;
  const marked = x.startsWith("~");
  const hex = marked ? x.slice(1) : x;
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) return null;
  return (marked ? "nonicao:" : "icao:") + hex.toLowerCase();
}
function goodPosition(lat: unknown, lon: unknown): lat is number {
  return number(lat) && number(lon) && lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180;
}
export function normalizeReadsb(input: unknown, receiverId: string, receivedAt: number): Observation[] {
  if (!isObject(input) || !age(input.now) || !number(receivedAt) ||
    Math.abs(input.now - receivedAt) > 10 || !Number.isSafeInteger(input.messages) ||
    (input.messages as number) < 0 || !Array.isArray(input.aircraft) ||
    input.aircraft.length > 5000 || receiverId !== "rx-primary") {
    throw new Error("Chybný readsb snapshot nebo nedůvěryhodný receiver");
  }
  const now = input.now;
  const output: Observation[] = [];
  for (const row of input.aircraft) {
    if (!isObject(row)) continue;
    const id = aircraftKey(row.hex);
    if (id === null || !age(row.seen) || row.seen > 60) continue;
    if (row.squawk !== undefined && (typeof row.squawk !== "string" ||
      !/^[0-7]{4}$/.test(row.squawk))) continue;
    if ((row.lat === undefined) !== (row.lon === undefined)) continue;
    if (row.lat !== undefined && !goodPosition(row.lat, row.lon)) continue;
    if (row.seen_pos !== undefined && !age(row.seen_pos)) continue;
    const position = row.lat !== undefined && age(row.seen_pos) &&
      row.seen_pos <= 15 && row.seen_pos <= now && goodPosition(row.lat, row.lon)
      ? { lat: row.lat, lon: row.lon as number } : null;
    const callsign = typeof row.flight === "string" ? row.flight.trim() : "";
    output.push({
      receiverId, transport: "readsb_json", aircraftKey: id, receivedAtUtc: now,
      messageAtUtc: now - row.seen,
      timeBasis: position ? "derived_readsb" : "unknown",
      positionOrigin: observedOrigin(row.type),
      positionAtUtc: position ? now - (row.seen_pos as number) : null,
      position,
      callsign: callsign.length > 0 && callsign.length <= 8 ? callsign : null,
      squawk: typeof row.squawk === "string" ? row.squawk : null
    });
  }
  return output;
}
export function isSqlEligible(o: Observation): boolean {
  return o.position !== null && o.positionAtUtc !== null &&
    o.positionOrigin === "adsb" &&
    (o.timeBasis === "derived_readsb" || o.timeBasis === "receiver_utc");
}
export function parsePublicAircraft(data: unknown): PublicAircraft {
  if (!isObject(data) ||
    Object.keys(data).some(k => ![
      "id", "lastMessageAt", "lastPositionAt", "position", "positionOrigin",
      "stale", "callsign", "squawk", "telemetry"
    ].includes(k)) ||
    typeof data.id !== "string" || !keyRegex.test(data.id) ||
    !age(data.lastMessageAt) ||
    (data.lastPositionAt !== null && !age(data.lastPositionAt)) ||
    (data.position !== null &&
      (!isObject(data.position) || Object.keys(data.position).some(k => !["lat", "lon"].includes(k)) ||
        !goodPosition(data.position.lat, data.position.lon))) ||
    (data.position === null) !== (data.lastPositionAt === null) ||
    !["adsb", "mlat", "tisb", "rebroadcast", "estimated", "unknown"].includes(String(data.positionOrigin)) ||
    typeof data.stale !== "boolean" ||
    (data.callsign !== null && (typeof data.callsign !== "string" || data.callsign.length > 8)) ||
    (data.squawk !== null && (typeof data.squawk !== "string" || !/^[0-7]{4}$/.test(data.squawk))) ||
    (data.telemetry !== undefined && (!isObject(data.telemetry) ||
      Object.entries(data.telemetry).some(([k,v]) => {
        if (k === "onGround") return typeof v !== "boolean";
        if (!["altitudeBaroFt","groundSpeedKt","trackDegrees","verticalRateFpm"].includes(k) || !number(v)) return true;
        if (k === "altitudeBaroFt") return !Number.isInteger(v) || v < -2000 || v > 65000;
        if (k === "verticalRateFpm") return !Number.isInteger(v) || v < -15000 || v > 15000;
        return v < 0 || v >= (k === "trackDegrees" ? 360 : 1500.00001);
      })))) {
    throw new Error("Neplatné nebo neveřejné Aircraft DTO");
  }
  return data as PublicAircraft;
}
export function parseStreamEvent(raw: unknown): StreamEvent {
  if (!isObject(raw) || typeof raw.epoch !== "string" || !raw.epoch ||
    !Number.isSafeInteger(raw.seq) || (raw.seq as number) < 0 ||
    typeof raw.generatedAt !== "string" || !/^\d{4}-\d{2}-\d{2}T/.test(raw.generatedAt) ||
    !number(Date.parse(raw.generatedAt))) {
    throw new Error("Chybné SSE záhlaví");
  }
  if (raw.kind === "snapshot" && Array.isArray(raw.aircraft)) {
    if (Object.keys(raw).some(k => !["kind", "epoch", "seq", "generatedAt", "aircraft"].includes(k))) {
      throw new Error("Nedovolená položka SSE");
    }
    const aircraft = raw.aircraft.map(parsePublicAircraft);
    if (new Set(aircraft.map(a => a.id)).size !== aircraft.length) throw new Error("Duplicitní id");
    return { kind: "snapshot", epoch: raw.epoch, seq: raw.seq as number,
      generatedAt: raw.generatedAt, aircraft };
  }
  if (raw.kind === "delta" && Array.isArray(raw.upsert) && Array.isArray(raw.removed)) {
    if (Object.keys(raw).some(k => !["kind", "epoch", "seq", "generatedAt", "upsert", "removed"].includes(k))) {
      throw new Error("Nedovolená položka SSE");
    }
    const upsert = raw.upsert.map(parsePublicAircraft);
    const removed = raw.removed;
    if (!removed.every((s: unknown) => typeof s === "string" && keyRegex.test(s)) ||
      new Set(removed).size !== removed.length || new Set(upsert.map(a => a.id)).size !== upsert.length) {
      throw new Error("Neplatné položky delty");
    }
    return { kind: "delta", epoch: raw.epoch, seq: raw.seq as number,
      generatedAt: raw.generatedAt, upsert, removed: removed as string[] };
  }
  throw new Error("Neplatná SSE událost");
}
export function applyEvent(replica: Replica, event: StreamEvent): Replica {
  if (event.kind === "snapshot") {
    return { epoch: event.epoch, seq: event.seq,
      aircraft: new Map(event.aircraft.map(a => [a.id, a])) };
  }
  if (event.epoch !== replica.epoch || replica.seq === null || event.seq !== replica.seq + 1) {
    throw new Error("SSE gap/restart: vyžadován nový snapshot");
  }
  const aircraft = new Map(replica.aircraft);
  for (const a of event.upsert) aircraft.set(a.id, a);
  for (const id of event.removed) aircraft.delete(id);
  return { epoch: replica.epoch, seq: event.seq, aircraft };
}
