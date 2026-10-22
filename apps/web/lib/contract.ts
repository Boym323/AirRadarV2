// G-01 kontrakt: nepovoluje živý ingest ani implementaci G-03 SSE.
export type Observation = {
  receiverId: string;
  transport: "readsb_json" | "beast_tcp";
  aircraftKey: string;
  receivedAtMs: number;
  positionAtMs: number | null;
  timeBasis: "derived_readsb" | "receiver_utc" | "server_arrival" | "unknown";
  positionOrigin: "adsb" | "mlat" | "tisb" | "estimated" | "unknown";
  position: { lat: number; lon: number } | null;
  callsign: string | null;
  squawk: string | null;
};

export function validObservation(o: Observation): boolean {
  const p = o.position;
  const fieldsValid = /^[a-zA-Z0-9_-]{1,64}$/.test(o.receiverId)
    && /^(icao|nonicao):[0-9a-f]{6}$/.test(o.aircraftKey)
    && Number.isSafeInteger(o.receivedAtMs) && o.receivedAtMs >= 0
    && (o.callsign === null || (o.callsign.length <= 16 && ![...o.callsign].some((ch) => ch.charCodeAt(0) < 32 || ch.charCodeAt(0) === 127)))
    && (o.squawk === null || /^[0-7]{4}$/.test(o.squawk));
  if (!fieldsValid) return false;
  if (p === null) return o.positionAtMs === null;
  return o.positionAtMs !== null
    && Number.isSafeInteger(o.positionAtMs) && o.positionAtMs >= 0
    && o.positionAtMs <= o.receivedAtMs + 10_000
    && o.timeBasis !== "unknown"
    && Number.isFinite(p.lat) && Number.isFinite(p.lon)
    && p.lat >= -90 && p.lat <= 90 && p.lon >= -180 && p.lon <= 180;
}

export function persistable(o: Observation): boolean {
  return validObservation(o) && o.position !== null
    && (o.timeBasis === "derived_readsb" || o.timeBasis === "receiver_utc")
    && o.positionOrigin === "adsb";
}

export type PublicAircraft = {
  aircraftKey: string;
  position: { lat: number; lon: number } | null;
  positionAtMs: number | null;
  stale: boolean;
};

export type StreamPayload =
  | { kind: "snapshot"; epoch: string; seq: number; aircraft: PublicAircraft[] }
  | { kind: "delta"; epoch: string; seq: number; upsert: PublicAircraft[]; removed: string[] };

export const acceptsDelta = (epoch: string, lastSeq: number, nextEpoch: string, seq: number) =>
  epoch === nextEpoch && Number.isSafeInteger(lastSeq) && Number.isSafeInteger(seq)
  && lastSeq >= 0 && seq === lastSeq + 1;
