#!/usr/bin/env node
// G-01: deterministické fixture a validační smoke; NENÍ to implementace G-02 parseru.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (p) => JSON.parse(readFileSync(resolve(root, p), "utf8"));
const beast = read("contracts/fixtures/beast-frames.json");
const stream = read("contracts/fixtures/stream.json");
const sample = read("contracts/fixtures/observations.json");
const aircraftSchema = read("contracts/aircraft.schema.json");
const streamSchema = read("contracts/stream.schema.json");
assert.equal(sample.cases.length, 7);
assert.equal(aircraftSchema.properties.aircraftKey.type, "string");
assert.equal(streamSchema.oneOf.length, 2);

function parseOneBeast(hex) {
  if (!/^(?:[0-9a-f]{2})+$/i.test(hex)) return null;
  const bytes = Buffer.from(hex, "hex");
  if (bytes[0] !== 0x1a) return null;
  const expectedLength = ({ 0x31: 2, 0x32: 7, 0x33: 14 })[bytes[1]];
  if (expectedLength === undefined) return null;
  const frame = [];
  for (let i = 2; i < bytes.length; i++) {
    if (bytes[i] === 0x1a) {
      if (++i >= bytes.length || bytes[i] !== 0x1a) return null;
      frame.push(0x1a);
    } else frame.push(bytes[i]);
  }
  if (frame.length !== 6 + 1 + expectedLength) return null;
  return { payload: Buffer.from(frame.slice(7)), timestamp: Buffer.from(frame.slice(0, 6)) };
}

// DF17 s přímou CRC paritou; jiné DF/AP parity si zaslouží vlastní golden testy G-02.
function modeSLongCrc24(payload) {
  let crc = 0;
  for (const byte of payload) {
    for (let bit = 7; bit >= 0; bit--) {
      const feedback = ((byte >> bit) & 1) ^ ((crc >> 23) & 1);
      crc = ((crc << 1) & 0xffffff) ^ (feedback ? 0xfff409 : 0);
    }
  }
  return crc;
}
for (const fixture of beast.cases) {
  const decoded = parseOneBeast(fixture.hex);
  assert.equal(decoded !== null, fixture.valid, fixture.name);
  if (!fixture.valid) continue;
  assert.equal(decoded.payload.toString("hex"), fixture.expectedMessageHex, fixture.name);
  assert.equal((decoded.payload[0] >> 3) & 31, 17, "golden DF17");
  assert.equal(modeSLongCrc24(decoded.payload), 0, "ověřená DF17 CRC");
}
assert.equal(parseOneBeast(beast.cases[1].hex).timestamp.includes(0x1a), true);

let lastSeq;
let epoch;
for (const event of stream.cases.map((c) => c.payload)) {
  if (event.kind === "snapshot") {
    assert.equal(Array.isArray(event.aircraft), true);
    epoch = event.epoch;
    lastSeq = event.seq;
  } else {
    assert.equal(event.epoch, epoch);
    assert.equal(event.seq, lastSeq + 1);
    assert.equal(new Set(event.removed).size, event.removed.length);
    lastSeq = event.seq;
  }
}
console.log("PASS G-01: JSON golden, BEAST escape/length/DF17 CRC, SSE snapshot/delta sequence.");
