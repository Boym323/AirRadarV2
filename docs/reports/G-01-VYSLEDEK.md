# G-01 — Implementace kontraktů a golden fixtures

**Datum:** 22. 10. 2026. **Stav:** **ROZPRACOVÁNO** (část G-01 hotova; etapa jako celek zatím není převzata). **Pracovní základ:** commit `a437c820`; závěrečný SHA a aktuální CI se uvádí podle `main`.

## Skutečně vytvořené soubory

- `apps/engine/src/contract.rs` — Rust typy a validace `aircraftKey`, `receiverId`, časů polohy a původu, policy `persistable`, jednoduchý test plausibility, SSE snapshot/delta seq helper.
- `apps/web/lib/contract.ts` a `apps/web/tests/contract.test.ts` — TypeScript typy a stejné golden případy, test JSON/BEAST identity a SSE seq.
- `contracts/fixtures/observations.json` — 7 syntetických validních/nevalidních pozorování sdílených mezi Rust/TS.
- `contracts/fixtures/beast-frames.json` + `tools/check-g01-contracts.mjs` — kontrola rámce BEAST, escaping `0x1a`, truncation, známého DF17 a CRC24. **Není to skutečný runtime BEAST parser**.
- `contracts/aircraft.schema.json`, `contracts/stream.schema.json`, `contracts/fixtures/stream.json` — cílové G-01 DTO a snapshot/delta golden případy.
- `contracts/openapi.yaml` — přidané cílové komponenty AircraftV1/SnapshotV1/DeltaV1; **G-00 skutečné endpointy zůstávají nezměněné a pravdivě `NOT_IMPLEMENTED`**.
- `contracts/g01-acceptance.json` — konkrétní plánované PASS/FAIL scénáře pro každou z 45 aktivních funkcí bez změny jejich `NOT_STARTED` statusu.
- `contracts/G-01-ROZHODNUTI.md` — závazné policy P0-01/02/03 včetně omezení BEAST-only a resync.

## Co je prokazatelně otestováno

- Společné syntetické JSON fixtures v Rust `cargo test` a TypeScript `vitest`, kontrola squawk, chybějící polohy, nesprávného času a ne-ICAO identity.
- Negativní plausibility konfliktu poloh, epoch/seq gap, srovnání identity přes oba transporty.
- CI dokumentační kontrola golden BEAST DF17/CRC24/escape a SSE pořadí; kontrola matice 45 funkcí a jejich modulových vlastníků.
- Zatím **žádný živý readsb, skutečný BEAST decoder, DB, runtime SSE, MapLibre, Playwright E2E ani produkční deploy**.

## Nedokončené podmínky převzetí G-01

1. Formální parity aktualizovaných Rust/TS/OpenAPI typů ještě vyžadují externí schema validátor a golden test pro kompletní payload; dnešní testy pokrývají jen zvolená klíčová pole.
2. BEAST/Mode-S **short/long, další DF/AP parity, CPR even/odd a CRC negativní fixture** je třeba rozšířit; současný syntetický test pokrývá jeden DF17 long referenční rámec a escaping.
3. **Atomické spojení registrace subscriberu a snapshotu** musí být reálně dokázané až s G-03 stream producerem, nikoli pouhým typovým testem `seq`.
4. Kontrakty potřebují úplný test boundary (NaN, zlomky epoch, extra pole, negativní squawk, validní nulová GPS). Přejímací scénáře 45 funkcí mají nyní plán, ne PASS.
5. Aktualizovat status G-01 na HOTOVO **až když budou skutečně splněny všechny kroky G-01.01–G-01.10**, nikoli na základě tohoto průběžného reportu.

## Bezpečnost

V1, produkční přijímač, PostgreSQL LXC, historii ani live push jsme neměnili. Žádná další externí data ani vyřazené funkce nebyly přidány. Nové API schéma nedává oprávnění k deployi.

## CI

Doložená úspěšná kontrola první Rust/TS vrstvy: [GitHub Actions 39746139094](https://github.com/Boym323/AirRadarV2/actions/runs/39746139094). Následující commity s fixture/acceptance je nutno ověřit na jejich přesném SHA; nejsou tím předem označené PASS.
