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
for (const e of errors) console.error(e);
if (errors.length) process.exit(1);
console.log("České dokumenty a jejich požadovaná struktura souhlasí.");
