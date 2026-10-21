#!/usr/bin/env node
// Česká dokumentace, hranice aktivního rozsahu a jediný archiv budoucích funkcí.
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const read = p => readFileSync(resolve(root, p), "utf8");
const fail = message => { throw new Error(message); };

const required = [
  "README.md", "AGENTS.md", "CHANGELOG.md",
  "docs/plan/ZADANI-AIRRADAR-V2.md",
  "docs/plan/MODULY-A-ROZSAH.md",
  "docs/plan/MATICE-FUNKCI.md",
  "docs/plan/PODROBNY-KATALOG-FUNKCI.md",
  "docs/plan/ETAPY-A-STAV.md",
  "docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md",
  "docs/plan/CO-NEPREBIRAT-Z-V1.md",
  "docs/plan/PRAVIDLA-DOKUMENTACE.md",
  "docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md",
  "docs/ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md",
  "docs/reports/G-00-BOOTSTRAP.md",
  "docs/runbooks/LXC-SETUP.md",
  "docs/sablony/REPORT-ETAPY.md",
];
for (const file of required) {
  if (read(file).length < 50) fail("Chybí povinná česká dokumentace: " + file);
}
if (!read("AGENTS.md").includes("češtině")) fail("Chybí české pravidlo AGENTS.");
if (!read("apps/web/app/layout.tsx").includes('lang="cs"')) fail("Web nemá lang=cs.");
if (!read("apps/web/app/page.tsx").includes("Není připojen živý provoz")) fail("Úvodní stránka není česky.");
execFileSync(process.execPath, [resolve(root,"tools/generate-feature-matrix.mjs"),"--check"], {stdio:"inherit"});

const activeData = JSON.parse(read("contracts/feature-matrix.json"));
const active = activeData.features;
if (active.length !== activeData.counts.KEEP || !active.length ||
    new Set(active.map(f=>f.id)).size !== active.length ||
    active.some(f=>f.decision!=="KEEP")) fail("Aktivní matice obsahuje chybu nebo jinou než KEEP funkci.");

const archivePath = "docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md";
const archive = read(archivePath);
const deferred = [...archive.matchAll(/^\| \*\*([A-Z]\d{2})\*\* \| ([^|]+) \| ([^|]+) \|$/gm)].map(m=>m[1]);
if (deferred.length !== new Set(deferred).size || !deferred.length) fail("Archiv musí obsahovat jednotlivé jedinečné funkce.");
if (deferred.some(id => active.some(f=>f.id===id))) fail("Aktivní ID se nachází v archivu; funkci je nutné přesunout.");
if (!archive.includes("Celkem: " + deferred.length)) fail("Počet funkcí v záhlaví archivu neodpovídá položkám.");
if (active.length + deferred.length !== 178) fail("Při přesunu z původního inventáře se ztratilo některé ID.");

const descriptions = read("docs/plan/PODROBNY-KATALOG-FUNKCI.md");
const described = [...descriptions.matchAll(/^\| \*\*([A-Z]\d{2})\*\* \|/gm)].map(x=>x[1]);
if (JSON.stringify([...described].sort()) !== JSON.stringify(active.map(f=>f.id).sort())) {
  fail("Aktivní slovní katalog není synchronizovaný s maticí.");
}

const modules = read("docs/plan/MODULY-A-ROZSAH.md");
const assignments = [...modules.matchAll(/^\*\*Primární vlastněná KEEP ID \(\d+\):\*\* ([^\n]+)$/gm)];
if (assignments.length !== 9) fail("Chybí některý z devíti modulů.");
const moduleIds = assignments.flatMap(m=>[...m[1].matchAll(/`([A-Z]\d{2})`/g)].map(x=>x[1]));
if (moduleIds.length !== active.length || new Set(moduleIds).size !== active.length ||
    JSON.stringify([...moduleIds].sort()) !== JSON.stringify(active.map(f=>f.id).sort())) {
  fail("Moduly nepokrývají všechna aktivní ID právě jednou.");
}

const taskFiles = readdirSync(resolve(root,"docs/ukoly")).filter(p=>p.endsWith(".md") && !p.startsWith("G-00-"));
const taskStages = taskFiles.map(p=>p.match(/^G-(\d{2})-/)?.[1]);
if (taskStages.length !== 13 || taskStages.some(v=>v===undefined) ||
    JSON.stringify([...taskStages].sort()) !== JSON.stringify(Array.from({length:13},(_,i)=>String(i+1).padStart(2,"0")))) {
  fail("Etapy G-01 až G-13 nemají přesně jedno české zadání.");
}
const stages = [...read("docs/plan/ETAPY-A-STAV.md").matchAll(/^\| G-(\d{2}) \|/gm)].map(m=>m[1]);
if (stages.length !== 14 || JSON.stringify(stages) !== JSON.stringify(Array.from({length:14},(_,i)=>String(i).padStart(2,"0")))) {
  fail("Plán etap není souvislý G-00 až G-13.");
}

// Brána proti opětovnému přenesení odložených ID do aktivních dokumentů a kódu.
const gitFiles=execFileSync("git",["ls-files","-z"],{cwd:root}).toString().split("\0").filter(Boolean);
const deferredPattern=new RegExp("\\b(?:"+deferred.join("|")+")\\b");
const excluded = new Set([archivePath]);
for(const path of gitFiles){
  if(excluded.has(path) || /(?:-lock\.json|Cargo\.lock)$/.test(path) ||
     !/\.(md|json|yaml|yml|mjs|ts|tsx|rs)$/.test(path)) continue;
  if(deferredPattern.test(read(path))) fail("Odložená funkce se znovu objevila mimo archiv: "+path);
}
console.log("Česká dokumentace a rozsah ověřeny: "+active.length+" aktivních, "+deferred.length+" odložených; 9 modulů, 14 etap.");
