# G-01 — Závěrečný report: kontrakty a deterministická data

**Datum:** 22. 10. 2026. **Stav:** **HOTOVO — 10/10 kroků G-01.01–G-01.10**.  
**Výchozí kontrolovaný commit:** `a437c820`. **Výsledný ověřený commit:** `14f591c461cbb6b1af6eca8d71327e86479bdc84`.  
**Ověření:** [GitHub Actions #39748325104](https://github.com/Boym323/AirRadarV2/actions/runs/39748325104), **3/3 PASS**: Rust engine, Vite/TS web, česká dokumentace a nezávislý validátor OpenAPI 3.1 / JSON Schema 2020-12.

## Doložená implementace

| Krok | Výstup a skutečný důkaz | Výsledek |
|---|---|---|
| G-01.01 | 45 KEEP ID, 9 unikátních vlastníků, kontrola `tools/check-g01-contracts.mjs` | PASS |
| G-01.02 | Doménové typy/validation: `apps/engine/src/contract.rs`, `apps/web/lib/g01-contract.ts`, JSON Schema | PASS |
| G-01.03 | Identita, původ ADS-B/MLAT/TIS-B, `receiverId`, časová sémantika, validace squawk/poloh | PASS |
| G-01.04 | `contracts/fixtures/readsb/{initial,repeated,changed,invalid,empty,expected}.json` | PASS |
| G-01.05 | `contracts/fixtures/beast/cases.json`: escaped 0x1A, 31/32/33, DF17/CRC24 a CPR even/odd, poškozené rámce | PASS |
| G-01.06 | Testované stavové scénáře identity, výpadku, duplicit, neplatných poloh a konfliktů v Rust/TS | PASS |
| G-01.07 | `contracts/openapi.yaml`, čtyři Draft 2020-12 schémata a `tools/validate-g01-schemas.py` — skutečná OpenAPI 3.1 validace, kontrola YAML duplicit, pozitivní/negativní fixtures | PASS |
| G-01.08 | `contracts/fixtures/sse/sequence.json`; snapshot/delta/upsert/removed, gap/epoch restart a kontrakt resync | PASS |
| G-01.09 | `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `npm run lint/typecheck/test/build`, Node fixture smoke a nezávislý schema validátor | PASS |
| G-01.10 | `docs/plan/G-01-MATICE-AKCEPTACE.md` a `contracts/g01-acceptance.json`: plánovaný PASS/FAIL scénář pro všech 45 produktových ID, stavy `NOT_STARTED` zachovány | PASS |

## Přijaté doménové hranice

- **P0-01:** Čas zprávy `now-seen` se nikdy nevydává za čas polohy `now-seen_pos`. BEAST receiver ticks bez ověřeného UTC mapování nejsou legitimní SQL position timestamp. Původ polohy a časová základna jsou povinné součásti důvěryhodného DTO.
- **P0-02:** ICAO a ne-ICAO identity jsou odlišné; `receiverId` není `transport`; JSON/BEAST z téhož readsb neznamenají dva přijímače. Zjevně nesmyslný skok není destruktivní update. Budoucí externí integrace nejsou schválené.
- **P0-03:** Formát SSE používá `epoch/seq`, snapshot a delta s `removed` a pravidlem pro vynucený resync. **G-03 musí skutečně zajistit atomickou registraci odběratele a počáteční snapshot**, jinak může vzniknout závod mezi snapshotem a deltou; tato runtime část do G-01 nepatří.

## Důležité omezení — co G-01 záměrně neimplementovala

- **Živý příjem readsb/BEAST**, TCP streaming parser a autoritativní RAM radar: **G-02**, ne dnešní etapový kontrakt.
- **Skutečný Axum REST/SSE provoz se snapshot/delta:** **G-03**. Současný G-00 endpoint pravdivě vrací `NOT_IMPLEMENTED`.
- **PostgreSQL + SQLx schema, adaptivní vzorkování, backups:** **G-04**. Existující DB LXC nebyl kontaktován ani upraven.
- **MapLibre, počasí, notifikace, Playwright browser E2E a produkční release:** až následné etapy. Žádné produkční upozornění, registrace nebo mapový live rendering nebyly spouštěny.
- 45 aktivních ID bylo pouze **kontraktově připraveno**. Stavy v `contracts/feature-matrix.json` zůstávají `NOT_STARTED`.

## Rizika před G-02 a G-03

Překládat toto kontraktové ověření do skutečného runtime musí přísně zachovat jednoho vlastníka RAM, správné CRC/parity podle DF, CPR časovou platnost, memory bounds a load/backpressure testy. Formální schema validace neprokazuje fyzikální výkon, robustnost binárního TCP parseru ani atomický multi-client SSE producer. G-02/G-03 potřebují samostatné implementační testy a reporty.

**Závěr:** G-01 předána do G-02. V1, jeho produkce, PostgreSQL a LXC jsou beze změny.
