# ADR 0003 — Jednotný Node.js 26 v projektu AirRadar V2

**Stav:** navrženo, do úspěšného CI. **Datum:** 2026-10-21.

## Kontext
CI a dokumentace stále používaly Node.js 22. V říjnu 2026 je dostupný Node.js 26.12.0, nové záplaty Next.js 16.4 a současné verze vývojových nástrojů.

## Rozhodnutí
Soubor `.node-version` určuje Node.js 26.12.0 ve vývoji i GitHub Actions. `apps/web/package.json` deklaruje Node 26/npm 11. Přesně zachycené transitive dependencies jsou v `package-lock.json`, CI používá `npm ci`. Rust stable a Cargo.lock bez zásahu, žádné nasazení.

## Alternativy
- Node 24 LTS je konzervativnější, ale nesplní požadavek projektu na Node 26.
- Plovoucí `node-version: 26` je automatické, ale nepředvídatelné pro reprodukci chyb.
- Změna runtime produkčního LXC není v této fázi schválena.

## Následky
Před přijetím požadujeme ověřený lockfile, lint, typy, testy, build a regresní API smoke. Aktualizace Node 26 na novou patch verzi opět projde CI.
