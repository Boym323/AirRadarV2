# G-04 — výsledek auditu a průběžná přejímka PostgreSQL historie

**Aktualizace:** 23. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární KEEP:** J09 (`IN_PROGRESS`).
**Větev/PR:** `codex/g04-history-20261023`, draft PR #21. **Auditní SHA:** `4659d63822d842a69fb0e141b9dcf1c5480e381e`.

## Audit G-04.01–G-04.12

| Krok | Stav | Důkaz / zbývá |
|---|---|---|
| G-04.01 | ČÁSTEČNĚ | Read-only audit PASS: PostgreSQL 17.10, DB 7.4 MiB, `max_connections=100`, `wal_level=replica`, `fsync/full_page_writes/synchronous_commit=on`; cluster backup politika a kapacita sdíleného LXC nejsou doložené. |
| G-04.02 | TESTED pro DEV | Připojení přes `PgSslMode::Disable`, `pg_stat_ssl.ssl=false`, bez změny `pg_hba.conf`/firewallu; důvěryhodnost LAN je provozní riziko. |
| G-04.03 | TESTED pro přístup | DB owner `airradar_v2_dev_migrator`, schema/table owner `postgres`; obě role jsou bez SUPERUSER/CREATEDB/CREATEROLE, runtime má pouze USAGE + SELECT/INSERT. Přístup do jiných DB nebyl zkoušen. |
| G-04.04 | TESTED v CI/DEV kompatibilitě | Jedna tabulka, 30s bucket, PK deduplikace, opakované `up` a odmítnutý destruktivní `down`; migrace na skutečné DB nebyla spuštěna. |
| G-04.05 | TESTED v kódu | SQLx pool max. 2, acquire 2 s, SQL timeout 5 s, opt-in `--g04-dev-history`; reconnect/chaos provozní test chybí. |
| G-04.06 | TESTED v unit testech | RAM-first sampler, 30/60/120/180/600 s pravidla, 30s bucket a validace polohy. |
| G-04.07 | TESTED v unit testech/kódu | Pending + inflight nejvýše 2048, dávka nejvýše 128, coalescing, ACK po COMMIT, retry 3 pokusy s backoffem; reálný výpadek DB chybí. |
| G-04.08 | TESTED v SQLx testu | Časové čtení jedné identity, limit 1000, bez query v živé RAM/SSE cestě. |
| G-04.09 | ČÁSTEČNĚ | Retenční SQL je omezené na `airradar_v2_dev` a 1000 řádků; izolovaný benchmark potvrdil PK-only plán, ale 30denní retenční profil produkční DB chybí. |
| G-04.10 | TESTED izolovaně | Lokální PostgreSQL na `127.0.0.1:55432` byl zastaven/spuštěn; JSON/RAM/SSE zůstaly živé a po obnově se zapsal nový vzorek. Sdílený LXC nebyl dotčen. |
| G-04.11 | TESTED izolovaně | `pg_dump -Fc` + `pg_restore` do samostatné lokální `airradar_v2_restore_ci`: schema, 1 řádek, 8 constraintů a 1 index PASS. RPO/RTO níže platí pouze pro tento malý izolovaný dump. |
| G-04.12 | ČÁSTEČNĚ TESTED | Izolovaný 1×/2×/4× replay 5 000/10 000/20 000 řádků PASS; p95/p99, WAL, `pg_stat_io`, client CPU/RSS změřeny. Není to SQLx writer benchmark ani měření serverového CPU sdílené DB. |

## Skutečné DEV ověření

Read-only/transaction test z aplikačního hostitele (`192.168.1.126 → 192.168.1.55:5432`) dne 23. 10. 2026:

- `current_database() = airradar_v2_dev`, `current_user = airradar_v2_dev_app`, `pg_stat_ssl.ssl = false`;
- `airradar_v2.flight_samples` existuje;
- runtime má `USAGE` schématu a `SELECT, INSERT`, ale nemá `CREATE, UPDATE, DELETE, TRUNCATE`;
- SQLx integrační test deduplikace a čtení prošel proti této DB; zápis byl idempotentní přes PK `(bucket_at, aircraft_key)` a zanechal jeden testovací vzorek `icao:fed123`;
- transakční zápis/čtení vzorku proběhl a byl vrácen `ROLLBACK`, takže test nezanechal fixture;
- špatné heslo a DDL runtime účtem byly odmítnuty.

Heslo ani interní tajemství nejsou součástí reportu, Git historie ani CI logu. Testovací vzorek byl vložen pouze do již připravené tabulky; migrace, role ani nastavení sdíleného DB LXC nebyly tímto testem měněny. Runtime účet nemá `DELETE`, proto se vzorek neodstraňoval destruktivním obcházením oprávnění.

## Izolovaný G-04.10–G-04.12 důkaz

Lokální PostgreSQL 17.11 na `127.0.0.1:55432` byl inicializován jako samostatný cluster, použita stejná migrace a testovací runtime role. Tento cluster byl jediný, který byl během výpadku nebo zátěže zastaven.

- **Výpadek/reconnect:** při zastavení clusteru zůstal JSON ingest, Rust RAM a G-03 API aktivní; `aircraft` vracel živý stav, zatímco databáze byla nedostupná. Po restartu a novém validním vzorku se počet řádků zvýšil z `20001` na `20002`. Retry má 3 pokusy s prodlevami 100/250 ms a neomezený backlog nevzniká.
- **5000 cílů:** kompaktní fixture 5000 letadel prošla přes `--g04-dev-history`; `/api/v1/aircraft` HTTP 200, payload 994062 B. Plnější fixture s callsign/telemetrií správně vrátila 503 `CAPACITY_EXCEEDED`, což potvrzuje bezpečný limit payloadu, nikoli zablokování ingestu.
- **Backup/restore skutečné DEV DB:** read-only `pg_dump -Fc` databáze `airradar_v2_dev` (pouze schéma `airradar_v2`) vytvořil 4 165 B za 0,09 s; SHA-256 dumpu `8419d06ab60736b47f6d54e0d7571e8d4beaf0c4e07d8d8d8d5e632bf37d51c`. Dump byl obnoven pouze do nezávislé lokální databáze `airradar_v2_restore_ci`: `pg_restore` 0,03 s, schema `airradar_v2.flight_samples`, 1 řádek, 8 constraintů, 1 PK index, bez chyb. RPO tohoto snapshotu je 0 řádků a naměřené RTO 0,03 s; obojí platí pouze pro tento malý dataset, nikoli jako provozní SLA. Předchozí lokální syntetický dump měl 3 850 B/0,124 s/0,060 s.
- **Benchmark metodika:** PostgreSQL 17.11, stejný SQL model, jeden klient, dávky 128 řádků, PK-only, `fsync=on`, lokální loopback, před každým během `TRUNCATE` a reset WAL statistik. Výsledky:

| Profil | Řádky / dávky | wall | dávkové p95/p99 | WAL | client CPU user/sys | max RSS |
|---:|---:|---:|---:|---:|---:|---:|
| 1× | 5 000 / 40 | 67 ms | 0,827 / 1,552 ms | 1 340 182 B | 0,01 / 0,00 s | 7 688 KiB |
| 2× | 10 000 / 79 | 93 ms | 0,760 / 1,127 ms | 2 532 939 B | 0,01 / 0,00 s | 7 768 KiB |
| 4× | 20 000 / 157 | 154 ms | 0,936 / 1,336 ms | 5 052 683 B | 0,01 / 0,01 s | 7 820 KiB |

`pg_stat_io` delta pro relation writes/extends/fsyncs/hits byla postupně `0/90/0/+31163`, `0/176/0/+61518`, `0/348/0/+121948`. Jde o izolované lokální měření SQL dávkového modelu, nikoli důkaz výkonu sdíleného DB LXC nebo celého SQLx writeru. Kvalita adaptivního vzorkování je doložena unit/replay testy; samostatné srovnání tras 15/30/60 s stále chybí.

## Co je implementováno

PR obsahuje `history.rs` (RAM sampler), `history_store.rs` (SQLx pool, batch INSERT, query a omezený retry), napojení do jediného ingest actoru, retenční SQL a izolované CI/restore testy. G-02 RAM/readsb/BEAST a G-03 HTTP/SSE zůstávají opt-in a databáze není aktivní v běžném režimu.

Destruktivní `.down.sql` migrace je nyní fail-closed; rollback historie se provádí pouze `pg_dump`/`pg_restore` do nové izolované databáze. `up.sql` je idempotentní vůči existující tabulce, ale před ostrým spuštěním musí migrační účet ověřit kompatibilitu sloupců a omezení.

## Neuzavřené brány

Stále chybí ověřená cluster backup politika sdíleného LXC, 30denní retenční profil, srovnání 15/30/60 s, měření serverového CPU a benchmark skutečného SQLx writeru proti izolovanému PostgreSQL. Proto **G-04 není HOTOVO a J09 není TESTED/ACCEPTED**. PR #21 nesmí aktivovat writer v DEV bez samostatného GO, návratu na běžný G-03 režim a ověřeného restore.

### Bezpečný postup další aktivace

1. Správce potvrdí migrátor, backup/RPO/RTO a izolaci databáze bez změny V1, `pg_hba.conf` nebo firewallu.
2. Na nové/ověřené DEV DB proběhne pouze kompatibilitní migrace a negativní test práv.
3. V izolaci se změří restart/timeout/reconnect, restore a benchmarky; živý příjem/SSE musí pokračovat při SQL degradaci.
4. Teprve po písemném schválení se spustí `--g04-dev-history`; návrat je zastavení tohoto opt-in procesu a návrat na `--g03-live-api`.
