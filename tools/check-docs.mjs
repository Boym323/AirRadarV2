#!/usr/bin/env node
// Ověření povinné dokumentace a výchozí české lokalizace.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const required = [
  "README.md","AGENTS.md","docs/plan/ZADANI-AIRRADAR-V2.md",
  "docs/plan/ETAPY-A-STAV.md","docs/plan/PRAVIDLA-DOKUMENTACE.md",
  "docs/plan/GREENFIELD-BRIEF.md","docs/plan/MATICE-FUNKCI.md",
  "docs/plan/PODROBNY-KATALOG-FUNKCI.md",
  "docs/ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md",
  "docs/sablony/ZADANI-ETAPY.md","docs/sablony/REPORT-ETAPY.md",
  "docs/reports/G-00-BOOTSTRAP.md",
  "docs/adr/0001-greenfield-boundaries.md",
  "docs/adr/0002-cestina-jako-vychozi-jazyk.md",
  "docs/runbooks/LXC-SETUP.md",
];
const docs = Object.fromEntries(required.map(p => [p, readFileSync(resolve(root,p),"utf8")]));
const errors = [];
if (!docs["AGENTS.md"].includes("Výchozí jazyk projektu: čeština")) errors.push("AGENTS.md nevyžaduje češtinu.");
if (!docs["README.md"].includes("Dokumentace a zadání — vše česky")) errors.push("README nemá český rozcestník.");
if (!docs["docs/plan/GREENFIELD-BRIEF.md"].includes("ZADANI-AIRRADAR-V2.md")) errors.push("Staré zadání neodkazuje na české.");
if (!docs["docs/plan/ETAPY-A-STAV.md"].includes("G-15")) errors.push("Chybí plán etap.");
const layout = readFileSync(resolve(root,"apps/web/app/layout.tsx"),"utf8");
const page = readFileSync(resolve(root,"apps/web/app/page.tsx"),"utf8");
if (!layout.includes('lang="cs"')) errors.push("Web nemá HTML lang=cs.");
if (!page.includes("Není připojen živý provoz")) errors.push("Úvodní web není česky.");
try {
  execFileSync(process.execPath,[resolve(root,"tools/generate-feature-matrix.mjs"),"--check"],{stdio:"inherit"});
} catch {
  errors.push("Vygenerovaná matice se liší od JSON.");
}

// Kontrola, že katalog vysvětluje právě všechny funkce v JSON, bez ztracených ID.
// Popisy samy jsou produktovou interpretací, nikoli potvrzením implementace.
const katalog = docs["docs/plan/PODROBNY-KATALOG-FUNKCI.md"];
const boundary = "## Část II — nyní vyřazené funkce";
const ending = "## Důležité závislosti a otevřené otázky";
if (!katalog.includes(boundary) || !katalog.includes(ending)) {
  errors.push("V katalogu chybí hranice sekcí KEEP/REMOVE.");
} else {
  const [keptPart, removedRest] = katalog.split(boundary);
  const removedPart = removedRest.split(ending)[0];
  const rawMatrix = JSON.parse(readFileSync(resolve(root, "contracts/feature-matrix.json"), "utf8"));
  const extract = text => {
    const rows = [...text.matchAll(/^\| \*\*([A-Z]\d{2})\*\* \| (.*?) \| (.*?) \|$/gm)];
    for (const row of rows) {
      if (row[3].trim().length < 45) errors.push("Příliš stručný popis: " + row[1]);
    }
    return rows.map(row => row[1]);
  };
  const actualKeep = extract(keptPart);
  const actualRemove = extract(removedPart);
  const expectedKeep = rawMatrix.features.filter(f => f.decision === "KEEP").map(f => f.id);
  const expectedRemove = rawMatrix.features.filter(f => f.decision === "REMOVE").map(f => f.id);
  if (JSON.stringify([...actualKeep].sort()) !== JSON.stringify([...expectedKeep].sort())) errors.push("Sekce KEEP v katalogu se liší od matice.");
  if (JSON.stringify([...actualRemove].sort()) !== JSON.stringify([...expectedRemove].sort())) errors.push("Sekce REMOVE v katalogu se liší od matice.");
  if (new Set([...actualKeep, ...actualRemove]).size !== rawMatrix.features.length) errors.push("V katalogu jsou chybějící nebo duplicitní ID.");
}

// Kontrola jediného odpovědného modulu pro každé ze 46 schválených ID.
const moduly = readFileSync(resolve(root, "docs/plan/MODULY-A-ROZSAH.md"), "utf8");
const maticeModulu = JSON.parse(readFileSync(resolve(root, "contracts/feature-matrix.json"), "utf8"));
const moduloveRadky = [...moduly.matchAll(/^- \*\*Primární vlastněná KEEP ID \(\d+\):\*\* ([^\n]+)$/gm)];
if (moduloveRadky.length !== 9) errors.push("Očekáváno devět definovaných modulů.");
const prirazenaId = moduloveRadky.flatMap(m => [...m[1].matchAll(/[A-Z]\d{2}/g)].map(x => x[0]));
const povinnaId = maticeModulu.features.filter(f => f.decision === "KEEP").map(f => f.id);
if (prirazenaId.length !== povinnaId.length ||
    new Set(prirazenaId).size !== povinnaId.length ||
    JSON.stringify([...prirazenaId].sort()) !== JSON.stringify([...povinnaId].sort())) {
  errors.push("Moduly neobsahují právě všech 46 KEEP ID bez duplicit.");
}
const zadaniEtap = [
  "G-02-LOKALNI-INGEST","G-03-EXTERNI-ZDROJE-ZRUSENO","G-04-API-A-SSE",
  "G-05-POSTGRESQL-HISTORIE","G-06-ZIVA-MAPA","G-07-LETISTE-NAVAID",
  "G-08-VYBRANE-POCASI","G-09-OGN-INTEGRACE-ZRUSENO",
  "G-10-NOUZOVE-KODY-A-NOTIFIKACE","G-11-STATISTIKY-DIAGNOSTIKA",
  "G-12-PWA-LOKALIZACE","G-13-ZAVERECNE-TESTOVANI",
  "G-14-ODDELENY-SHADOW","G-15-BEZPECNY-RELEASE"
];
for (const zadani of zadaniEtap) {
  const cesta = "docs/ukoly/" + zadani + ".md";
  let obsah = "";
  try { obsah = readFileSync(resolve(root, cesta), "utf8"); }
  catch { errors.push("Chybí české zadání: " + cesta); continue; }
  if (!obsah.startsWith("# " + zadani.slice(0, 4))) {
    errors.push("Nesprávná hlavička zadání " + cesta);
  }
}
for (const kod of ["G-03", "G-09"]) {
  if (!readFileSync(resolve(root, "docs/plan/ETAPY-A-STAV.md"), "utf8").includes("| " + kod + " |")) {
    errors.push("Chybí evidovaná zrušená etapa " + kod);
  }
}
for (const e of errors) console.error(e);
if (errors.length) process.exit(1);
console.log("České dokumenty a jejich požadovaná struktura souhlasí.");
