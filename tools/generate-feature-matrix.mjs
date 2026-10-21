#!/usr/bin/env node
// Vytváření čitelného českého přehledu požadavků z JSON.
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const jsonPath = resolve(root, "contracts/feature-matrix.json");
const outputPath = resolve(root, "docs/plan/MATICE-FUNKCI.md");
const categories = {
  J: "Živý příjem a stav", R: "Mapa a vizualizace", H: "Detail letadla",
  A: "Letiště", T: "ATC, ATS a navigace", W: "Počasí",
  L: "Upozornění", E: "Datové zdroje a integrace",
  S: "Statistiky", O: "Diagnostika a provoz", U: "Uživatelské rozhraní",
};
function render(d) {
  const rows = [
    "# Matice funkcí AirRadar V2",
    "",
    "**Výchozí jazyk: čeština.** Tento soubor se generuje ze `contracts/feature-matrix.json`; neupravuj jej ručně. Použij `node tools/generate-feature-matrix.mjs`.",
    "",
    "**Rozsah:** " + d.counts.KEEP + " požadovaných funkcí (KEEP) a " + d.counts.REMOVE + " vyřazených funkcí (REMOVE). Označení `NOT_STARTED` znamená, že funkce dosud není implementovaná.",
    "",
    "## Požadované funkce",
    "",
    "| ID | Oblast | Funkce | Stav | Důkazy |",
    "|---|---|---|---|---|",
  ];
  for (const x of d.features.filter(x => x.decision === "KEEP")) {
    rows.push("| `" + x.id + "` | " + (categories[x.id[0]] || "Další oblast") + " | " + x.name.replaceAll("|", "\\|") + " | `" + x.status + "` | " + (x.tests?.length ? x.tests.join(", ") : "—") + " |");
  }
  rows.push("", "## Funkce mimo rozsah", "", "Tyto funkce mají v JSON stav `EXCLUDED` a nebudou do V2 automaticky přenášeny.", "", "| ID | Funkce |", "|---|---|");
  for (const x of d.features.filter(x => x.decision === "REMOVE")) {
    rows.push("| `" + x.id + "` | " + x.name.replaceAll("|", "\\|") + " |");
  }
  rows.push("", "## Údržba", "", "- Aktualizuj skutečně ověřený stav v `contracts/feature-matrix.json`.", "- Pro `TESTED` nebo `ACCEPTED` je nutný důkaz v poli `tests`.", "- Spusť `node tools/generate-feature-matrix.mjs` a v CI ověř `node tools/generate-feature-matrix.mjs --check`.", "", "Stav vývoje najdeš v [plánu etap](ETAPY-A-STAV.md).", "");
  return rows.join("\n");
}
const matrix = JSON.parse(readFileSync(jsonPath, "utf8"));
const keep = matrix.features.filter(x => x.decision === "KEEP");
const removed = matrix.features.filter(x => x.decision === "REMOVE");
const ids = new Set(matrix.features.map(x => x.id));
if (keep.length !== 79 || removed.length !== 99 || ids.size !== matrix.features.length ||
    matrix.counts.KEEP !== keep.length || matrix.counts.REMOVE !== removed.length) {
  throw new Error("Matice musí obsahovat 79 KEEP, 99 REMOVE a jedinečná ID.");
}
for (const x of matrix.features) {
  if (!["NOT_STARTED","IN_PROGRESS","TESTED","ACCEPTED","EXCLUDED"].includes(x.status)) throw new Error("Neznámý stav: " + x.id);
  if (x.decision === "REMOVE" && x.status !== "EXCLUDED") throw new Error("Chybné vyřazení: " + x.id);
  if (x.decision === "KEEP" && ["TESTED","ACCEPTED"].includes(x.status) && !x.tests?.length) throw new Error("Chybí důkaz: " + x.id);
}
const output = render(matrix);
if (process.argv.includes("--check")) {
  if (readFileSync(outputPath, "utf8") !== output) throw new Error("Markdown matice neodpovídá JSON; spusť generátor.");
  console.log("Matice 79 KEEP / 99 REMOVE souhlasí.");
} else {
  writeFileSync(outputPath, output);
  console.log("Česká matice funkcí vytvořena.");
}
