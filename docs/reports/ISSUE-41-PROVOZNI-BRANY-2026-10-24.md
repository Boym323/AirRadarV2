# Issue #41 — provozní audit a stabilizace DEV LXC

**Datum měření:** 24. 10. 2026, Europe/Prague
**Repozitář:** `Boym323/AirRadarV2`
**SHA auditu:** `43e20d8ce460a5bd3f12503db22ae98c4d02db85`
**DEV LXC:** `lo-dev-airradarv2`, Debian 13, 4 vCPU, 8 GiB RAM, 20 GiB ZFS
**Nasazený SHA:** stejný jako SHA auditu; immutable release `/opt/airradar-v2/releases/43e20d8ce460`

## Výsledek bran

| Bod | Stav | Důkaz |
|---|---|---|
| Brána 1 — ochrana `main` | **TESTED** | Branch Protection API: povinný PR review, aktuální větev, `rust-engine`, `vite-web`, `Česká dokumentace`, zákaz force-push/mazání, ochrana administrátorů a vyřešení konverzací. |
| Brána 2 — služby a deployment | **TESTED** | CI run [#40112006453](https://github.com/Boym323/AirRadarV2/actions/runs/40112006453) má všech 6 jobů PASS; engine, Nginx a runner jsou `active`; `current/VERSION` odpovídá SHA auditu. |
| Brána 3 — DB retence G-04 | **BLOKOVÁNO** | Vzdálený DB LXC `192.168.1.55`, `airradar_v2_dev`, PostgreSQL 17.10, 14 MB, 46 872 řádků, 0 starších než 30 dní. Runtime role nemá `DELETE`; maintenance role, DBA schválení a timer nejsou aktivní. |
| Brána 4 — živý ingest a health | **TESTED** | `/api/v1/health` lokálně i přes DEV web vrací HTTP 200, `status=live`, `stage=G-04`, `live_ingest=true`, `database_connected=true`; API po ingestu obsahovalo 75 cílů. Lokální BEAST `192.168.1.50:30005` odpovídal. |
| Brána 5 — SSE snapshot/delta/reconnect | **TESTED** | Po nasazení snapshot `seq=52` a delty `53–55`, jednotná epocha, plná veřejná DTO, bez `receiverId`; izolovaný harness ověřil reconnect, heartbeat, 503 při 101. klientovi a návrat pod limit. |
| Brána 6 — SSE syntetická kapacita | **TESTED** | 1/10/100 klientů a 25/250/1 000/5 000 cílů; 0 ztracených aktualizací. RSS špička 55 480 KiB při 90 pomalých klientech, růst 4 276 KiB, CPU 15,71 % jednoho jádra při 100×5 000. |
| Brána 7 — mapa a fallback | **TESTED** | CI Chromium MapLibre smoke a fallback při výpadku podkladu PASS v runu #40112006453; fyzický GPU test není tímto důkazem nahrazen. |
| Brána 8 — fyzický FPS | **NEOVĚŘENO** | CI používá headless Chromium/SwiftShader. Chybí skutečný desktop s GPU a telefon; proto není uveden fyzický FPS/p95. |
| Brána 9 — systemd stabilita | **TESTED** | Opraveny pouze V2 DEV jednotky: `ConditionPathExists`, `TimeoutStartSec` pro oneshot služby. `systemd-analyze verify` bez chyb, žádné nové warningy engine/Nginx, služby zůstaly aktivní. |
| Brána 10 — veřejný deploy status | **TESTED** | Runner bridge nyní po úspěšném nasazení atomicky publikuje ověřený `current/VERSION`, nikoli starý stav vypnutého legacy timeru. |
| Brána 11 — dependency/release security | **NEOVĚŘENO** | `npm audit --omit=dev` nehlásí vysoké zranitelnosti; `cargo-audit` není v LXC nainstalován. Licence, SECURITY kontakt, SBOM a supply-chain schválení patří před G-13. |
| Brána 12 — další etapy | **BLOKOVÁNO** | Nejde o součást tohoto auditu; etapy zůstávají v původním etapovém stavu a není proveden produkční deploy. |

## Testy a měření

- Rust: `cargo fmt --check`, `cargo test --locked --workspace` — 67/67 PASS; Clippy s `-D warnings` PASS.
- Web: lint, TypeScript, Vitest 30 testů, import testy 3/3 a Vite build PASS. Lokální LXC má Node 20/npm 9, proto je výsledkem hostitelského běhu pouze podpůrný důkaz; závazný Node 26/npm 11 důkaz je CI.
- SSE izolace: 1/10/100 klientů, 5 000 cílů, limit 100 spojení, 101. klient HTTP 503, žádné tiché ořezání. p95/p99 harnessu 1,76–1,97 s zahrnuje periodický syntetický ingest tick a není čistou síťovou latencí.
- Systém: při běžícím enginu RSS 4,7 MiB, 5 tasků, `MemoryMax=2G`, `LimitNOFILE=4096`, `Restart=on-failure`, účet `airradarv2`; disk 5,3/20 GiB.
- PostgreSQL: pouze read-only dotazy na oddělený host; žádná změna `pg_hba.conf`, firewallu, V1, PP Studio ani jiné databáze.

## Retence G-04

Izolované CI testy retenčního skriptu zůstávají PASS. Skutečná DB má 0 expirovaných řádků, takže nebylo co mazat. Timer `airradarv2-g04-retention.timer` není nalezen a canary zůstává pouze `static`/neaktivní. Bez DBA schválení se žádná maintenance role, timer ani `DELETE` nespouští.

## Závěr

DEV LXC je po nasazení SHA `43e20d8…` provozně stabilní pro aktuální G-03/G-04/G-05 DEV rozsah: živý ingest, DB health, SSE, mapa, rollbackové nasazení a systemd hranice prošly dostupnými testy. Nejde o produkční schválení. Zbývají pouze fyzický GPU/FPS test, DBA schválení skutečné retence a release-security evidence před G-13.
