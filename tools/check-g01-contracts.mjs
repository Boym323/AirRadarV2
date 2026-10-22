#!/usr/bin/env node
// G-01: kontraktová kontrola bez externích služeb a dodatečných závislostí.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
const root=resolve(import.meta.dirname,"..");
const read=p=>readFileSync(resolve(root,p),"utf8");
const data=p=>JSON.parse(read(p));

const matrix=data("contracts/feature-matrix.json");
assert.equal(matrix.features.length,45);
const g02 = new Set(["J01","J02","J03","J04","J05","J11"]);
assert.ok(matrix.features.every(x =>
  x.decision === "KEEP" &&
  (g02.has(x.id) ? x.status === "TESTED" && x.tests.length > 0 : x.status === "NOT_STARTED")
));
const ids=matrix.features.map(x=>x.id);
assert.equal(new Set(ids).size,45);
const modules=read("docs/plan/MODULY-A-ROZSAH.md");
const owners=new Map();
for(const line of modules.split("\n")){
  const m=line.match(/^\| (M\d{2}) — [^|]+\| (G-\d{2}) \|[^|]+\| ([^|]+)\|/);
  if(!m)continue;
  for(const id of m[3].match(/[A-Z]\d{2}/g)||[]){
    assert.ok(!owners.has(id),"Duplicitní vlastník "+id);
    owners.set(id,m[1]+"/"+m[2]);
  }
}
assert.equal(owners.size,45);
assert.deepEqual([...owners.keys()].sort(),[...ids].sort());
const rows=[...read("docs/plan/G-01-MATICE-AKCEPTACE.md")
  .matchAll(/^\| \*\*([A-Z]\d{2})\*\* \| (M\d{2}) \| (G-\d{2}) \| ([^|]+) \| Plán — neověřeno \|$/gm)];
assert.equal(rows.length,45);
assert.equal(new Set(rows.map(x=>x[1])).size,45);
for(const m of rows){
  assert.equal(m[2]+"/"+m[3],owners.get(m[1]));
  assert.ok(m[4].length>=20);
}
const schemas=["aircraft-observation","readsb-json","aircraft-public","sse-v1"]
  .map(x=>data("contracts/"+x+".schema.json"));
for(const s of schemas)assert.equal(s.$schema,"https://json-schema.org/draft/2020-12/schema");
for(const field of ["receiverId","aircraftKey","transport","positionOrigin",
    "receivedAtUtc","messageAtUtc","positionAtUtc","timeBasis","squawk"]){
  assert.ok(schemas[0].required.includes(field));
}
assert.equal(schemas[1].properties.aircraft.maxItems,5000);
assert.equal(schemas[2].properties.receiverId,undefined);
assert.equal(schemas[3].oneOf.length,2);
const e=data("contracts/fixtures/readsb/expected.json");
const source=data("contracts/fixtures/readsb/initial.json");
const repeated=data("contracts/fixtures/readsb/repeated.json");
const changed=data("contracts/fixtures/readsb/changed.json");
assert.equal(e.initial.length,3);
assert.equal(e.initial[0].aircraftKey,"icao:abc123");
assert.equal(e.initial[1].aircraftKey,"nonicao:abc123");
for(const x of e.initial)assert.equal(x.receiverId,"rx-primary");
assert.equal(source.now-source.aircraft[0].seen_pos,e.initial[0].positionAtUtc);
assert.equal(source.now-source.aircraft[0].seen,e.initial[0].messageAtUtc);
assert.equal(repeated.now-repeated.aircraft[0].seen_pos,e.repeatPositionAtUtc);
assert.equal(changed.now-changed.aircraft[0].seen_pos,e.changedPositionAtUtc);
assert.equal(data("contracts/fixtures/readsb/invalid.json").aircraft.length,4);
assert.equal(data("contracts/fixtures/readsb/empty.json").aircraft.length,0);

const beast=data("contracts/fixtures/beast/cases.json");
const bytes=s=>(s.match(/[a-fA-F0-9]{2}/g)||[]).map(x=>parseInt(x,16));
function decode(s){
  const raw=bytes(s),kind=raw[1],n={49:2,50:7,51:14}[kind];
  assert.equal(raw[0],26);
  assert.ok(n!==undefined);
  const body=[];
  for(let i=2;i<raw.length;i++){
    if(raw[i]===26)assert.equal(raw[++i],26);
    body.push(raw[i]);
  }
  assert.equal(body.length,n+7);
  return {kind,ticks:body.slice(0,6).reduce((a,b)=>a*256+b,0),
    rssi:body[6],payload:body.slice(7)};
}
function crc24(input){
  let r=0;
  for(const byte of [...input,0,0,0]){
    for(let b=7;b>=0;b--){
      const top=(r&0x800000)!==0;
      r=((r<<1)&0xffffff)|((byte>>b)&1);
      if(top)r^=0xfff409;
    }
  }
  return r;
}
assert.equal(crc24(bytes(beast.knownCrc.data)),parseInt(beast.knownCrc.parity,16));
const cpr=new Map();
for(const item of beast.valid){
  const f=decode(item.hex);
  assert.equal(f.kind,parseInt(item.type,16));
  assert.equal(f.ticks,item.receiverTicks);
  assert.equal(f.rssi,item.rssi);
  assert.deepEqual(f.payload,bytes(item.modeS));
  if(f.kind===51){
    assert.equal(f.payload[0]>>3,17);
    assert.equal(crc24(f.payload.slice(0,11)),f.payload.slice(11).reduce((a,b)=>a*256+b,0));
    const me=BigInt("0x"+item.modeS.slice(8,22));
    const odd=Number((me>>34n)&1n);
    assert.equal(item.cpr,odd?"odd":"even");
    cpr.set(item.name,{lat:Number((me>>17n)&0x1ffffn),lon:Number(me&0x1ffffn)});
  }
}
const even=cpr.get(beast.cprPair.even),odd=cpr.get(beast.cprPair.odd);
assert.ok(even&&odd);
const mod=(a,b)=>((a%b)+b)%b;
const j=Math.floor((59*even.lat-60*odd.lat)/131072+0.5);
const lat=360/60*(mod(j,60)+even.lat/131072);
const nl=Math.floor(2*Math.PI/Math.acos(1-(1-Math.cos(Math.PI/30))/
  Math.cos(lat*Math.PI/180)**2));
const m=Math.floor((even.lon*(nl-1)-odd.lon*nl)/131072+0.5);
const lon=360/nl*(mod(m,nl)+even.lon/131072);
assert.ok(Math.abs(lat-beast.cprPair.decodeReference.lat)<beast.cprPair.decodeReference.toleranceDegrees);
assert.ok(Math.abs(lon-beast.cprPair.decodeReference.lon)<beast.cprPair.decodeReference.toleranceDegrees);
for(const item of beast.invalid){
  if(item.expected==="bad_crc"){
    const f=decode(item.hex);
    assert.notEqual(crc24(f.payload.slice(0,11)),f.payload.slice(11).reduce((a,b)=>a*256+b,0));
  }else assert.throws(()=>decode(item.hex));
}
const sse=data("contracts/fixtures/sse/sequence.json");
assert.equal(sse.snapshot.seq+1,sse.delta.seq);
assert.equal(sse.delta.seq+1,sse.removed.seq);
assert.notEqual(sse.removed.seq+1,sse.gap.seq);
assert.notEqual(sse.snapshot.epoch,sse.restart.epoch);
assert.equal(sse.snapshot.aircraft[0].receiverId,undefined);
assert.equal(sse.oversizedMode,"reject_then_reconnect");
const api=read("contracts/openapi.yaml");
for(const word of ["AircraftPublicV1","SnapshotV1","DeltaV1","ErrorV1",
  "x-g01-planned-contract:","NOT_IMPLEMENTED","SNAPSHOT_TOO_LARGE"]){
  assert.ok(api.includes(word));
}
assert.ok(!api.includes("receiverId:"));
console.log("PASS G-01: 45 KEEP / 9 vlastníků, schema, JSON časy, BEAST/CRC/CPR, SSE/gap, OpenAPI.");
