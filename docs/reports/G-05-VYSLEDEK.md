# G-05 — průběžný výsledek implementace živé mapy

**Datum:** 23. 10. 2026 · **Stav:** ROZPRACOVÁNO, první 4/11 kroků ověřené (G-05.01/.02/.05/.06) · [PR #22](https://github.com/Boym323/AirRadarV2/pull/22) (draft). Před merge byla implementace izolovaná od `main`; první celek je určen pro DEV náhled, ne produkční release.

## Implementované části

- **G-05.01 — MapLibre klient:** MapLibre GL JS 6.15.0 připnutý v `package.json` a reprodukovatelném `package-lock.json`. Vite worker přes `?worker&url`; jeden React mount = jedna MapLibre instance, WebGL fallback na seznam. Default OpenFreeMap Liberty, bez přístupového klíče, s atribucí. 
- **G-05.02 — G-03 SSE adapter:** `apps/web/src/radar/aircraft-store.ts` a `use-radar-stream.ts`; atomické snapshot/delta, `epoch/seq` kontrola a vynucení nového snapshotu při gapu/reconnectu. 5 000 max. letadel, validace identity/souřadnic a maximálně jeden React publikovaný update za sekundu. Žádné přímé DB nebo další ADS-B ingest.
- **G-05.03/.04 — mapová reprezentace:** jediný GeoJSON aircraft source, symbolová ikona letadla pouze při známém reálném `trackDegrees`, jinak bod; popisek podle skutečného callsign. Stabilní feature ID. Nepoužívají se jednotlivé DOM markery.
- **G-05.05/.06 — seznam a výběr:** jeden store pro mapu i vyhledatelný seznam, filtr podle dostupné čerstvé polohy, výběr letadla a vysloveně neprodukční prázdný stav.
- **G-05.07/.08/.09 — navazující základ:** browser-only přijaté body stopy omezené na 64 bodů/20 minut bez extrapolace, volitelný follow s vypnutím při pan a podklady Liberty/Fiord. Pro finální TESTED je nutná prohlížečová interakční regrese.
- **Jednotkové testy:** `apps/web/tests/g05-radar.test.ts` testuje atomické delty, duplicitní cíle, gaps, epochy, odstranění, validitu souřadnic, retenci a heading. `apps/web/tests/app.test.tsx` ověřuje české UI i stav bez live dat.

## Co zůstává

- **G-05.10** letištní zdroj je závislý na G-06; žádný falešný katalog nebyl přidán.
- **G-05.11** Playwright se skutečným browser/WebGL, referenční výkon/FPS pro 25/100/500/2000 a setData vs updateData se musí změřit v samostatném prostředí; čísla nejsou vydávána za ověřená.
- Ověřené části **R01 a R06 mají stav `TESTED`**: [hlavní CI #40024971130](https://github.com/Boym323/AirRadarV2/actions/runs/40024971130) PASS včetně Chromium reálného map canvasu nad deterministickým SSE a offline podkladem. Další mapové funkce mají stav IN_PROGRESS, R12 NOT_STARTED; etapa se neuzavírá.

## Hranice bezpečnosti

- G-03 zůstává jedinou autoritou v Rust RAM a web konzumuje jen `/api/v1/stream`.
- Pouze veřejná/sanitizovaná DTO; žádný nový runtime server, DB či externí datový zdroj letadel.
- Podklady OpenFreeMap potřebují internet a jejich dostupnost nesmí blokovat seznam letadel.
- Žádný zásah do AirRadar V1, PP Studio, PostgreSQL či provozního G-03.

## Důkazy

**První důkaz:** CI #40024971130 PASS: Rust, Vite ESLint/TypeScript/Vitest/build, dokumentace a skutečný Chromium canvas se SSE snapshot/delta, odstraněním cíle, výběrem, style switch a bez remountu. Skutečný GPU/FPS benchmark, mobilní browser a celkovou G-05.11 akceptaci označujeme **NEOVĚŘENO**.
