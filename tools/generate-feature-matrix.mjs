#!/usr/bin/env node
// Generátor české matice výhradně z aktivních požadavků.
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, "..");
const data = JSON.parse(readFileSync(resolve(root, "contracts/feature-matrix.json"), "utf8"));
if (!Array.isArray(data.features) || !data.features.length || data.features.length !== data.counts.KEEP || new Set(data.features.map(f => f.id)).size !== data.features.length) throw Error("Neplatná aktivní matice.");
for (const f of data.features) {
  if (f.decision !== "KEEP" || !["NOT_STARTED","IN_PROGRESS","TESTED","ACCEPTED"].includes(f.status)) throw Error("Neaktivní funkce ve zdrojové matici: " + f.id);
  if (["TESTED","ACCEPTED"].includes(f.status) && !f.tests?.length) throw Error("Funkce bez testu: " + f.id);
}
const groups = {J:"Místní příjem",R:"Mapa",A:"Letiště",T:"Navigace",W:"Počasí",L:"Upozornění",S:"Statistiky",O:"Provoz",U:"Uživatelské prostředí"};
const rows = ["# Aktivní matice funkcí AirRadar V2","","Tento soubor se generuje ze `contracts/feature-matrix.json` a obsahuje výhradně aktuálně schválené funkce.","","**Rozsah: " + data.features.length + " aktivních funkcí.** Budoucí rozšíření eviduje [oddělený archiv](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).","","| ID | Oblast | Funkce | Stav | Testovací důkazy |","|---|---|---|---|---|"];
for (const f of data.features) rows.push("| `"+f.id+"` | "+(groups[f.id[0]]||"Ostatní")+" | "+f.name.replaceAll("|","\\|")+" | `"+f.status+"` | "+(f.tests?.length?f.tests.join(", "):"—")+" |");
rows.push("","## Aktualizace","","- Jednotlivá KEEP ID musí mít právě jeden vlastní modul.","- Stav `TESTED` nebo `ACCEPTED` musí mít doklad v poli `tests`.","- Používej `node tools/generate-feature-matrix.mjs` a kontrolu `--check`.","");
const output = rows.join("\n");
const dest = resolve(root, "docs/plan/MATICE-FUNKCI.md");
if (process.argv.includes("--check")) {
  if (readFileSync(dest,"utf8") !== output) throw Error("Generovaná matice neodpovídá JSON.");
  console.log("Matice v pořádku: " + data.features.length);
} else {
  writeFileSync(dest,output);
  console.log("Česká matice aktualizována.");
}
