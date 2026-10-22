# G-01 — Výsledek kontraktů a deterministických dat

**Datum:** 22. 10. 2026. **Stav:** **ROZPRACOVÁNO — 9 z 10 dílčích kroků je doložených**.  
**Baseline:** `a437c820`. **Úspěšný ověřený commit:** `06e7fdf58b34f2cc9f7eec275fe749deb2caf25e`.  
**CI:** [GitHub Actions #39747410591](https://github.com/Boym323/AirRadarV2/actions/runs/39747410591) — **PASS rust-engine, vite-web a česká dokumentace**.

## Co skutečně existuje

- `apps/engine/src/contract.rs` — Rust kontrakty, normalizace readsb JSON, validační limity a nepřítomná pole, oddělení ICAO/non-ICAO a JSON/BEAST; kontrola časů, kvality původu pozice, deduplikace a plausibility. Dále BEAST frame envelope/CRC a model klientské SSE repliky.
- `apps/web/lib/g01-contract.ts` a `apps/web/tests/g01-contract.test.ts` — TypeScript typy, readsb normalizace a testy SSE snapshot/delta/restart, bezpečné veřejné DTO.
- `contracts/fixtures/readsb/{initial,repeated,changed,invalid,empty,expected}.json` — deterministické vstupy i očekávané výstupy s ne-ICAO cílem, chybějícími polohami a časovým posunem.
- `contracts/fixtures/beast/cases.json` — BEAST typy 31/32/33, escaped `0x1a`, syntetické DF17/CRC24, CPR even/odd, chybné či zkrácené rámce a omezení samotného BEAST timestampu.
- `contracts/fixtures/sse/sequence.json` — snapshot, upsert delta, odstranění, chybějící sekvence a nový epoch.
- `contracts/{readsb-json,aircraft-observation,aircraft-public,sse-v1}.schema.json` — deklarované JSON Schema pro vývojové kontrakty.
- `contracts/openapi.yaml` — připravené budoucí DTO a odpovědi, aniž by se současné **G-00 stub endpointy** vydávaly za hotový ingest.
- `docs/plan/G-01-MATICE-AKCEPTACE.md` a `contracts/g01-acceptance.json` — všech **45 aktivních funkcí** má vlastníka a plánovaný důkaz; jejich skutečný status zůstává `NOT_STARTED`.
- `contracts/G-01-ROZHODNUTI.md` — explicitní P0 rozhodnutí o původu, čase polohy, konfliktu identit a pravidlech SSE.

## Skutečně ověřené kontroly

- Rust `cargo fmt --all -- --check`, Clippy `-D warnings`, unit testy včetně golden readsb/BEAST, parity a SSE repliky.
- TypeScript lint, typecheck, Vitest a Vite build; G-00 web dál pravdivě uvádí, že nepřijímá živá data.
- Node `tools/check-g01-contracts.mjs` kontroluje 45 funkcí, 9 vlastníků, JSON Schema strukturu, readsb časovou sémantiku, BEAST/CRC/CPR, SSE posloupnost a plánované OpenAPI komponenty.
- Kontrola archivovaných požadavků a celkem 140 etapových kroků; žádná produktová funkce nebyla označena za implementovanou.

## Zbývající blokace úplného převzetí

**G-01.07 — formální validátor OpenAPI/JSON Schema.** Současný kontrolní skript testuje jejich důležitou strukturu a vazby, **nejde ale o plnohodnotnou nezávislou validaci OpenAPI 3.1 ani úplnou JSON Schema validaci všech fixture**. Před označením G-01 jako HOTOVO je nutné použít deklarovaný validátor se skutečnými pozitivními i negativními payloady a jeho PASS doložit na přesném commit SHA.

Implementace skutečného TCP streaming BEAST parseru, serverového SSE producenta se zárukou atomické hranice a SQLx writeru jsou až **G-02/G-03/G-04**; jejich absence není falešně počítána mezi G-01 PASS. Po připojení runtime je nutné testovat jeho soulad s nynějšími kontrakty, včetně backpressure a restartu.

## Bezpečnost a rozsah

Žádný zásah do produkčního AirRadar V1, readsb přijímače, PostgreSQL LXC ani V2 deploymentu. Nejsou implementovány externí feedy, další stanice, notifikační doručování ani odložené funkce. Web stále obsahuje zkušební G-00 endpointy a nemá živý letový provoz.

**Závěr:** plnohodnotné kontraktové základy a golden testy jsou v main, CI PASS; etapa je **9/10, ROZPRACOVÁNO** do dokončení formální validace G-01.07.
