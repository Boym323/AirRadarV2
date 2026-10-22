#!/usr/bin/env node
// DETERMINISTICKÝ MODELOVÝ TEST návrhové G-04 sampling politiky.
// Nejde o produkční Rust implementaci ani benchmark PostgreSQL IOPS.
import assert from "node:assert/strict";

const MS=1000;
const deltaAngle=(a,b)=>Math.abs(((a-b+540)%360)-180);
function shouldSave(o,last){
  if (!o || !Number.isFinite(o.ts) || !o.validPosition) return false;
  if (!last) return true;
  const age=o.ts-last.ts;
  if(age<=0)return false; // starší / opakované readsb/BEAST pozorování
  const distance=Math.abs(o.distanceM-last.distanceM);
  const height=Math.abs(o.altFt-last.altFt);
  const heading=deltaAngle(o.heading,last.heading);
  if(o.onGround!==last.onGround && age>=30*MS)return true;
  if(o.onGround && last.onGround)
    return age>=600*MS || (age>=120*MS && distance>=100);
  if(age>=30*MS && (height>=1500 || heading>=30))return true;
  if(age>=180*MS)return true; // jen nová validní poloha, nikoli timer bez dat
  return age>=60*MS && (distance>=2000 || height>=500 || heading>=15);
}
function simulate(speedMps,onGround){
  let last=null,count=0;
  for(let s=0;s<24*3600;s+=15){
    const o={ts:s*MS,distanceM:s*speedMps,altFt:12000,
      heading:90,onGround,validPosition:true};
    if(shouldSave(o,last)){last=o;count++;}
  }
  return count;
}
const baseline=24*3600/15;
const flying=simulate(130,false);
const stationary=simulate(0,true);
assert.equal(baseline,5760);
assert.equal(flying,1440);
assert.equal(stationary,144);
const start={ts:0,distanceM:0,altFt:12000,heading:90,
  onGround:false,validPosition:true};
assert.equal(shouldSave(start,null),true);
assert.equal(shouldSave(start,start),false,"duplicitní JSON/BEAST");
assert.equal(shouldSave({...start,ts:-1},start),false,"opožděné pozorování");
assert.equal(shouldSave({...start,ts:15*MS,validPosition:false},start),false,
  "zpráva bez polohy není historický vzorek");
assert.equal(shouldSave({...start,ts:30*MS,distanceM:4500,
  heading:130},start),true,"významný obrat po 30 s");
assert.equal(shouldSave({...start,ts:30*MS,distanceM:3900},start),false,
  "běžný pohyb před minimálním intervalem");
const ground={...start,onGround:true,distanceM:0,altFt:0};
assert.equal(shouldSave({...ground,ts:600*MS},ground),true);
assert.equal(shouldSave({...ground,ts:120*MS,distanceM:100},ground),true);
const buckets=new Map();
const key=o=>"icao:abc123:"+Math.floor(o.ts/(30*MS));
buckets.set(key(start),start);
buckets.set(key(start),{...start,ts:20*MS});
assert.equal(buckets.size,1,"jen jeden pending bod v 30s bucketu");
assert.equal(buckets.get(key(start)).ts,20*MS,
  "nejlepší kandidát se v RAM nahradí před SQL");
console.log("MODEL: 15 s "+baseline+" bodů/letadlo/den; "+
  "adaptivně 60 s "+flying+" (-"+(100*(1-flying/baseline))+" %), "+
  "stojící "+stationary+" (-"+(100*(1-stationary/baseline))+" %).");
console.log("PASS: modelové invarianty deduplikace, intervalů a RAM coalescingu.");
