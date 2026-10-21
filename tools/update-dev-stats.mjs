#!/usr/bin/env node
// Metriky vývoje repozitáře bez dalších npm závislostí.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const p = (name) => resolve(root, name);
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const files = execFileSync("git", ["ls-files", "-z"], { cwd: root })
  .toString("utf8").split("\0").filter(Boolean).sort();
const code = files.filter((f) => /\.(rs|ts|tsx|js|mjs|css|sql|py)$/.test(f)
  && !f.endsWith(".d.ts") && !f.includes("/node_modules/"));
const countLines = (f) => {
  const content = readFileSync(p(f), "utf8");
  return content.length ? content.split("\n").length - Number(content.endsWith("\n")) : 0;
};
const sum = (items) => items.reduce((n, f) => n + countLines(f), 0);
const features = JSON.parse(readFileSync(p("contracts/feature-matrix.json"), "utf8")).features;
const keep = features.filter((f) => f.decision === "KEEP");
const removed = features.filter((f) => f.decision === "REMOVE");
if (keep.length !== 46 || removed.length !== 132) throw new Error("Nesouhlasí matice funkcí 46/132.");
const accepted = keep.filter((f) => ["TESTED", "ACCEPTED"].includes(f.status));
if (accepted.some((f) => !f.tests?.length)) throw new Error("Ověřená funkce postrádá důkaz.");
const stages = readFileSync(p("docs/plan/ETAPY-A-STAV.md"), "utf8")
  .split("\n").filter((line) => /^\|\s*G-\d\d\s*\|/.test(line));
if (stages.length !== 16) throw new Error("Chybí jedna z etap G-00 až G-15.");
const stagesDone = stages.filter((line) => /\bHOTOVO\b/.test(line.split("|")[3] || "")).length;
const snap = {
  sha: git("rev-parse", "HEAD"),
  at: git("show", "-s", "--format=%cI", "HEAD"),
  sourceLines: sum(code),
  rustLines: sum(code.filter((f) => f.endsWith(".rs"))),
  webLines: sum(code.filter((f) => /\.(ts|tsx|js|mjs|css)$/.test(f))),
  sourceFiles: code.length,
  testFiles: code.filter((f) => /(^|\/)(__tests__|tests?)\//.test(f) || /\.(test|spec)\./.test(f)).length,
  markdownFiles: files.filter((f) => f.endsWith(".md")).length,
  trackedFiles: files.length,
  stagesDone,
  stagesTotal: stages.length,
  featuresVerified: accepted.length,
  featuresTotal: keep.length
};

mkdirSync(p("docs/metrics"), { recursive: true });
const historyPath = p("docs/metrics/history.json");
const history = existsSync(historyPath)
  ? JSON.parse(readFileSync(historyPath, "utf8"))
  : { schemaVersion: 1, snapshots: [] };
if (history.schemaVersion !== 1 || !Array.isArray(history.snapshots)) throw new Error("Chybná historie.");
// Nové měření se ukládá pouze při změně sledovaných metrik.
// Pomocné commity generované automatizací tak nevytvářejí falešný trend.
const measuredKeys = [
  "sourceLines", "rustLines", "webLines", "sourceFiles", "testFiles",
  "markdownFiles", "trackedFiles", "stagesDone", "stagesTotal",
  "featuresVerified", "featuresTotal"
];
const previous = history.snapshots.at(-1);
const changed = !previous || measuredKeys.some((key) => snap[key] !== previous[key]);
if (changed && !history.snapshots.some((s) => s.sha === snap.sha)) {
  history.snapshots.push(snap);
}
const measured = history.snapshots.at(-1);
writeFileSync(historyPath, JSON.stringify(history, null, 2) + "\n");

const f = (n) => n.toLocaleString("cs-CZ");
const ratio = (n, total) => (100 * n / total).toFixed(1).replace(".", ",") + " %";
const table = [
  "## Statistiky vývoje (automaticky aktualizované)",
  "",
  "![Stav CI](https://github.com/Boym323/AirRadarV2/actions/workflows/ci.yml/badge.svg?branch=main)",
  "",
  "Aktualizace statistik: **denně ve 03:17 (Europe/Prague)** a ručně na vyžádání.",
  "",
  "| Ukazatel | Stav |",
  "|---|---:|",
  "| Řádky zdrojového kódu, včetně prázdných | **" + f(measured.sourceLines) + "** |",
  "| Rust | " + f(measured.rustLines) + " |",
  "| TypeScript / JavaScript / CSS | " + f(measured.webLines) + " |",
  "| Soubory zdrojového kódu | " + f(measured.sourceFiles) + " |",
  "| Testovací soubory (ne počet testů) | " + f(measured.testFiles) + " |",
  "| Dokumenty Markdown | " + f(measured.markdownFiles) + " |",
  "| Všechny sledované soubory | " + f(measured.trackedFiles) + " |",
  "| Dokončené etapy | **" + measured.stagesDone + "/" + measured.stagesTotal + "** (" + ratio(measured.stagesDone, measured.stagesTotal) + ") |",
  "| Ověřené KEEP funkce | **" + measured.featuresVerified + "/" + measured.featuresTotal + "** (" + ratio(measured.featuresVerified, measured.featuresTotal) + ") |",
  "",
  "![Historie řádků zdrojového kódu](docs/metrics/loc.svg)",
  "",
  "Měřený commit: [" + measured.sha.slice(0, 8) + "](https://github.com/Boym323/AirRadarV2/commit/" + measured.sha + ")"
    + " · [Historie](docs/metrics/history.json) · [Metodika](docs/metrics/README.md)"
    + " · [Výsledky CI](https://github.com/Boym323/AirRadarV2/actions/workflows/ci.yml).",
  "",
  "> Jde o vývojové statistiky repozitáře, nikoli o výkon živého radaru. "
    + "Funkce se počítají jen ze strojové matice s důkazem testu. "
    + "Úspěšná CI sama o sobě nedokazuje dokončení funkcí.",
].join("\n");
const file = p("README.md");
const start = "<!-- AIRRADAR_STATS_START -->";
const end = "<!-- AIRRADAR_STATS_END -->";
const original = readFileSync(file, "utf8");
const a = original.indexOf(start), b = original.indexOf(end);
if (a < 0 || b <= a) throw new Error("README neobsahuje blok statistik.");
writeFileSync(file, original.slice(0, a) + start + "\n" + table + "\n" + end
  + original.slice(b + end.length));

const values = history.snapshots;
const max = Math.max(1, ...values.map((x) => x.sourceLines));
const limit = Math.ceil(max * 1.1);
const xy = (x, i) => {
  const horizontal = values.length === 1 ? 462 : 66 + i * 792 / (values.length - 1);
  return [horizontal.toFixed(1), (185 - 142 * x.sourceLines / limit).toFixed(1)];
};
const polyline = values.map((x, i) => xy(x, i).join(",")).join(" ");
const points = values.length === 1
  ? '<circle cx="' + xy(values[0], 0)[0] + '" cy="' + xy(values[0], 0)[1] + '" r="5" fill="#60a5fa"/>'
  : "";
const svg = [
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 880 230" role="img" aria-label="Historie řádků kódu AirRadar V2">',
  '<rect width="880" height="230" rx="12" fill="#0f172a"/>',
  '<text x="24" y="27" fill="#f8fafc" font-family="Arial,sans-serif" font-size="17" font-weight="bold">Vývoj počtu řádků zdrojového kódu</text>',
  '<path d="M66 43V185H858" fill="none" stroke="#64748b"/>',
  '<path d="M66 114H858" fill="none" stroke="#334155" stroke-dasharray="4 4"/>',
  '<text x="53" y="49" text-anchor="end" fill="#cbd5e1" font-size="12">' + limit + '</text>',
  '<text x="53" y="189" text-anchor="end" fill="#cbd5e1" font-size="12">0</text>',
  '<text x="66" y="213" fill="#cbd5e1" font-size="12">' + values[0].at.slice(0, 10) + '</text>',
  '<text x="858" y="213" fill="#cbd5e1" font-size="12" text-anchor="end">' + values.at(-1).at.slice(0, 10) + '</text>',
  '<polyline points="' + polyline + '" fill="none" stroke="#60a5fa" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>',
  points,
  '<text x="844" y="27" text-anchor="end" fill="#93c5fd" font-size="14">' + measured.sourceLines + ' řádků</text>',
  '</svg>',
].join("\n") + "\n";
writeFileSync(p("docs/metrics/loc.svg"), svg);
console.log("Aktuální metriky:", JSON.stringify(snap));
console.log("Počet historických vzorků:", values.length);
