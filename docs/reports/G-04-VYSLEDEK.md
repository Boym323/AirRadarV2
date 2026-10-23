# G-04 — výsledek auditu a průběžná přejímka PostgreSQL historie

**Aktualizace:** 23. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární KEEP:** J09 (`IN_PROGRESS`).
**Větev/PR:** `codex/g04-history-20261023`, draft PR #21. **Implementační SHA:** `0daeb2f755bff81ce518a0415942a7e6b82579b0`. **Předchozí benchmark binárky:** `36afdda5e752cc320496f44672a94e9434eea9eb`. **Měřicí harnessy/report:** `ebc20307c8962593c3895908f4b54e2f63aeaa8a` / `4e1ff8b7396f78377ce3f429c5fe663e8fd1df67`.

## Audit G-04.01–G-04.12

| Krok | Stav | Důkaz / zbývá |
|---|---|---|
| G-04.01 | ČÁSTEČNĚ | Read-only audit PASS: PostgreSQL 17.10, DB 7 558 kB, `max_connections=100`, `shared_buffers=128MB`, `wal_level=replica`, `fsync/full_page_writes/synchronous_commit=on`; `archive_mode=off`, `archive_command=(disabled)`. Externí `pg_dump -Fc` je uložen mimo DB LXC, ale filesystemové volné místo, host CPU/RAM, WAL disk a provozní RPO/RTO sdíleného LXC nejsou dostupné runtime účtem. |
| G-04.02 | TESTED pro DEV | Připojení přes `PgSslMode::Disable`, `pg_stat_ssl.ssl=false`, bez změny `pg_hba.conf`/firewallu; důvěryhodnost LAN je provozní riziko. |
| G-04.03 | TESTED pro přístup | DB owner `airradar_v2_dev_migrator`, schema/table owner `postgres`; obě aplikační role jsou bez SUPERUSER/CREATEDB/CREATEROLE, runtime má pouze USAGE + SELECT/INSERT. Vlastnictví `postgres` neblokuje runtime, ale další DDL musí provést migrátor nebo vlastník s explicitním právem; vlastníci nebyli měněni. |
| G-04.04 | TESTED v CI/DEV kompatibilitě | Jedna tabulka, 30s bucket, PK deduplikace, opakované `up` a odmítnutý destruktivní `down`; migrace na skutečné DB nebyla spuštěna. |
| G-04.05 | TESTED v kódu | SQLx pool max. 2, acquire 2 s, SQL timeout 5 s, opt-in `--g04-dev-history`; interní diagnostika stavu připojení a timeoutů je opt-in přes `AIRRADAR_V2_HISTORY_TRACE`. |
| G-04.06 | TESTED v unit/replay testech | RAM-first sampler, 30/60/120/180/600 s pravidla, 30s bucket, validace polohy a stejná rekonstruovaná trasa pro srovnání 15/30/60 s; skutečný RF replay nebyl k dispozici. |
| G-04.07 | TESTED v unit testech/kódu | Pending + inflight nejvýše 2048, dávka nejvýše 128, coalescing, ACK po COMMIT, retry 3 pokusy s backoffem; interní snapshot obsahuje pending/inflight/commit/drop counters. |
| G-04.08 | TESTED v SQLx testu | Časové čtení jedné identity, limit 1000, bez query v živé RAM/SSE cestě. |
| G-04.09 | TESTED izolovaně | 2 501 starých řádků v izolované DB se odstranilo dávkami 1 000/1 000/501, nový řádek zůstal, PK index zůstal jediným indexem; 360 448 B relation / 8 099 507 B DB. Sdílená DEV retence nebyla spuštěna. |
| G-04.10 | TESTED izolovaně | Lokální PostgreSQL na `127.0.0.1:55432` byl zastaven/spuštěn; JSON/RAM/G-03 SSE zůstaly živé a po obnově se zapsal nový vzorek. Sdílený LXC nebyl dotčen. |
| G-04.11 | TESTED izolovaně | `pg_dump -Fc` + `pg_restore` do samostatné lokální `airradar_v2_restore_ci`: schema, 1 řádek, 8 constraintů a 1 index PASS. RPO/RTO níže platí pouze pro tento malý izolovaný dump. |
| G-04.12 | ČÁSTEČNĚ TESTED | Skutečný Rust→RAM sampler→bounded fronta→SQLx→PostgreSQL writer replay PASS; 1×/2×/4× hodnoty níže včetně ztrát bounded fronty, RSS, CPU ticků, WAL a SSE. Serverové CPU sdílené DB a skutečný DEV LXC zůstávají NEOVĚŘENO. Připraven je 30min evaluator; canary dosud neběžel. |

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
- **Skutečný SQLx writer:** harness `tools/benchmark_g04_writer.sh` z SHA `ebc20307c8962593c3895908f4b54e2f63aeaa8a` spustil binárku implementačního SHA `36afdda`, `--g04-dev-history`, lokální JSON source, G-03 SSE a PostgreSQL 17.11 na loopbacku. Každý profil začal čistým `TRUNCATE`, resetem WAL statistik a novou fixture; čekalo se na potvrzený počet řádků, nikoli na timeout. `accepted` je počet potvrzených SQL řádků, `dropped_by_bound` je záměrný limit RAM pending 2048.

| Profil | vstup | accepted | dropped_by_bound | wall | batch | CPU ticks | RSS | WAL | SSE bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1× | 1 250 | 1 250 | 0 | 34 255 ms | 10 | 43 | 20 428 KiB | 319 392 B | 513 987 |
| 2× | 2 500 | 2 048 | 452 | 40 437 ms | 16 | 100 | 30 124 KiB | 513 462 B | 1 027 189 |
| 4× | 5 000 | 2 048 | 2 952 | 40 407 ms | 16 | 215 | 48 120 KiB | 537 155 B | 2 013 589 |

CPU jsou součet Linux `/proc/<pid>/stat` user+system ticků procesu; nejsou to procenta serveru. SSE bytes jsou současný lokální stream klienta, nikoli plný browser benchmark. Výsledky potvrzují neblokující živý stream a ohraničenou ztrátu při saturaci, nikoli bezeztrátové uložení všech 5 000 vstupů.

### Časově rozložený replay a férovost

Harness `tools/benchmark_g04_writer_long.sh` na SHA `0daeb2f755bff81ce518a0415942a7e6b82579b0` poslal 16 atomických readsb snapshotů po 10 s, celkem 64 letadel. Měřená binárka byla `36afdda5e752cc320496f44672a94e9434eea9eb`; pozdější změny v `0daeb2f` přidaly diagnostiku a regresní test, nikoli změnu sampleru/writeru. Každý snapshot měl nové časové pozorování a pohyb; po doběhnutí byly v izolované PostgreSQL ověřeny `rows=64`, `aircraft=64`, `min_rows_per_aircraft=3`, `max_rows_per_aircraft=3`, `avg_rows_per_aircraft=3,00`. Žádné letadlo nebylo zvýhodněno ani ztraceno; běh nepřekročil pending limit 2048.

### Interní diagnostika

Při `AIRRADAR_V2_HISTORY_TRACE=1` Rust periodicky (15 s) loguje pouze číselný snapshot bez hosta, uživatele a hesla: `pending`, `inflight`, `eligible`, `coalesced`, `committed`, `dropped_capacity`, `dropped_stale`, `dropped_writer`, `sql_attempts`, `committed_batches`, `failed_batches`, `sql_timeouts` a stav `unknown/up/down`. SQL timeout je rozlišen od obecné chyby dotazu; po třech pokusech se inflight dávka zahodí přes `ACK=false`.
- **Backup/restore skutečné DEV DB:** read-only `pg_dump -Fc` databáze `airradar_v2_dev` (pouze schéma `airradar_v2`) vytvořil 4 165 B za 0,09 s; SHA-256 dumpu `8419d06ab60736b47f6d54e0d7571e8d4beaf0c4e07d8d8d8d5e632bf37d51c`. Dump byl obnoven pouze do nezávislé lokální databáze `airradar_v2_restore_ci`: `pg_restore` 0,03 s, schema `airradar_v2.flight_samples`, 1 řádek, 8 constraintů, 1 PK index, bez chyb. RPO tohoto snapshotu je 0 řádků a naměřené RTO 0,03 s; obojí platí pouze pro tento malý dataset, nikoli jako provozní SLA. Předchozí lokální syntetický dump měl 3 850 B/0,124 s/0,060 s.
- **Benchmark metodika:** PostgreSQL 17.11, stejný SQL model, jeden klient, dávky 128 řádků, PK-only, `fsync=on`, lokální loopback, před každým během `TRUNCATE` a reset WAL statistik. Výsledky:

| Profil | Řádky / dávky | wall | dávkové p95/p99 | WAL | client CPU user/sys | max RSS |
|---:|---:|---:|---:|---:|---:|---:|
| 1× | 5 000 / 40 | 67 ms | 0,827 / 1,552 ms | 1 340 182 B | 0,01 / 0,00 s | 7 688 KiB |
| 2× | 10 000 / 79 | 93 ms | 0,760 / 1,127 ms | 2 532 939 B | 0,01 / 0,00 s | 7 768 KiB |
| 4× | 20 000 / 157 | 154 ms | 0,936 / 1,336 ms | 5 052 683 B | 0,01 / 0,01 s | 7 820 KiB |

`pg_stat_io` delta pro relation writes/extends/fsyncs/hits byla postupně `0/90/0/+31163`, `0/176/0/+61518`, `0/348/0/+121948`. Jde o izolované lokální měření SQL dávkového modelu, nikoli důkaz výkonu sdíleného DB LXC. Kvalita rekonstruované trasy byla porovnána skriptem `tools/benchmark_g04_sampling.py`:

| Profil | SQL řádky | max. prostorová chyba | body v manévru | payload |
|---:|---:|---:|---:|---:|
| 15 s | 41 | 58,51 m | 5 | 3 521 B |
| 30 s | 21 | 232,83 m | 3 | 1 794 B |
| 60 s | 11 | 895,87 m | 2 | 925 B |
| adaptivní | 12 | 232,83 m | 3 | 1 015 B |

Stejná deterministická 10min trasa obsahovala 60s obrat a stoupání; jde o reprodukovatelný kvalitativní replay skriptu `ebc20307c8962593c3895908f4b54e2f63aeaa8a`, ne o skutečný RF záznam. Agresivnější 60s profil proto nebyl plošně povolen.

## Backup a RPO/RTO

Read-only audit skutečné DEV DB zjistil `archive_mode=off` a `archive_command=(disabled)`. Runtime účet nemá přístup ke cluster filesystemu ani k externí backup politice; existence denních dumpů, WAL/PITR a retenčních kopií tedy zůstává **NEZNÁMÁ**, nikoli PASS. Dříve naměřené dump/restore časy jsou pouze test izolovaného malého datasetu a nesmí se vydávat za provozní RPO/RTO.

Přípravný read-only dump `pg_dump -Fc` z 23. 10. 2026 byl uložen mimo DB LXC do lokálního chráněného artefaktu (mode 0600, adresář 0700), velikost 4 165 B, SHA-256 `8419d06ab60736b47f6d54e0d7571e8d4beaf0c4e07d8d8d8d5e632bf37d51c8`. `pg_restore --list` prošel. Obnova proběhla pouze do nezávislé izolované databáze; to dokládá obnovitelnost tohoto snapshotu, nikoli denní plán, dostupnost kopií ani provozní RPO/RTO. Připravený `tools/backup_g04_dev.sh` vyžaduje chráněný `PGPASSFILE`, pevně kontroluje databázi `airradar_v2_dev` a nemění serverovou konfiguraci. Denní timer smí být zaveden až po schválení umístění, šifrování, retence 14 denních + 8 týdenních kopií a nezávislého měsíčního restore.

Z runtime SQL účtu nebylo možné bezpečně zjistit volné místo filesystému DB LXC, CPU/RAM hostitele ani WAL disk. Tyto tři hodnoty jsou provozní blokace pro canary; nesmí se nahrazovat velikostí databáze 7 558 kB.

### Vlastnictví objektů a migrace

DB vlastní `airradar_v2_dev_migrator`, ale schéma `airradar_v2` a tabulku `flight_samples` vlastní `postgres`. Runtime role smí pouze `USAGE`, `SELECT` a `INSERT`; migrace proto nesmí běžet pod `airradar_v2_dev_app`. Každou budoucí změnu musí předem prověřit a provést DBA pod vlastníkem objektu nebo schváleným migračním účtem s dočasným, přesně omezeným DDL právem. V tomto PR se vlastníci ani granty nemění.

### Připravený canary a hranice automatického deploymentu

`deploy/systemd/airradarv2-g04-canary.service` je pouze neinstalovaná šablona: používá loopback port `4114`, chráněný `/etc/airradar-v2/g04-canary.env` mode 0600 a `--g04-dev-history`; nikdy se sama nepovoluje. G-03 zůstává v existující jednotce na portu `4101` s `--g03-live-api`. `tools/check-g04-deploy-boundary.mjs` kontroluje, že main auto-deploy ani G-03 instalační skripty neobsahují přepnutí na G-04. Merge do `main` tedy podle aktuální topologie nemá tento canary unit instalovat ani spouštět.

Předpokladem 30minutového běhu je samostatný schválený readsb/BEAST vstup nebo bezpečně multiplexovaný feed; canary se nesmí spustit pouze přidáním druhého konzumenta produkčního receiveru. `tools/evaluate_g04_canary.py` vyžaduje diagnostický log, minimálně 1 800 s, SQL readback, G-03 SSE heartbeat, CPU/RAM a WAL měření. Vrací FAIL při ztrátě nad 1 %, timeoutu, trvalém růstu fronty, nedostupném spojení nebo chybějícím provozním důkazu. Návrat provádí výhradně `tools/rollback_g04_canary.sh`: zastaví opt-in unit, ověří aktivní G-03 a nemaže historii.

## Co je implementováno

PR obsahuje `history.rs` (RAM sampler), `history_store.rs` (SQLx pool, batch INSERT, query a omezený retry), napojení do jediného ingest actoru, retenční SQL a izolované CI/restore testy. G-02 RAM/readsb/BEAST a G-03 HTTP/SSE zůstávají opt-in a databáze není aktivní v běžném režimu.

Destruktivní `.down.sql` migrace je nyní fail-closed; rollback historie se provádí pouze `pg_dump`/`pg_restore` do nové izolované databáze. `up.sql` je idempotentní vůči existující tabulce, ale před ostrým spuštěním musí migrační účet ověřit kompatibilitu sloupců a omezení.

## Neuzavřené brány

Stále chybí ověřená cluster backup politika sdíleného LXC, provozní RPO/RTO, skutečný RF replay pro sampling a měření serverového CPU. Ztráty 18,1 % při 2× a 59,0 % při 4× jsou mimo navrženou canary toleranci; writer proto nesmí být aktivován bez časově omezeného scénáře s limitem ztrát. G-04 není HOTOVO a J09 není TESTED/ACCEPTED.

## Brána prvního DEV canary — zatím NO-GO

Před samostatným GO musí být doloženo:

1. schválený vlastník schématu/tabulky nebo potvrzený migrační postup pro objekty vlastněné `postgres`;
2. potvrzené externí zálohy, RPO/RTO a postup obnovy; aktuální `archive_mode=off` samo o sobě není záloha;
3. canary dataset s minimálně 100 letadly a časovým replayem, `dropped_capacity + dropped_stale + dropped_writer <= 1 %` všech kandidátních historických vzorků;
4. žádné `sql_timeouts`, `connection=down` nebo růst `pending/inflight` na konci canary;
5. CPU/RAM aplikace, SQL latency p95/p99, WAL a SSE heartbeat bez regresního překročení schválených limitů;
6. připravený rollback: zastavit pouze opt-in writer, vrátit proces na `--g03-live-api`, ověřit BEAST/RAM/SSE a ponechat databázová data nedestruktivně.

Aktuální 2× a 4× ztráty tento limit nesplňují. Doporučení je **NO-GO**, dokud nebude canary záměrně omezen na provozní rychlost s doloženou ztrátou nejvýše 1 % nebo nebude vlastníkem schválena jiná hodnota.

### Bezpečný postup další aktivace

1. Správce potvrdí migrátor, backup/RPO/RTO a izolaci databáze bez změny V1, `pg_hba.conf` nebo firewallu.
2. Na nové/ověřené DEV DB proběhne pouze kompatibilitní migrace a negativní test práv.
3. V izolaci se změří restart/timeout/reconnect, restore a benchmarky; živý příjem/SSE musí pokračovat při SQL degradaci.
4. Teprve po písemném schválení se spustí `--g04-dev-history`; návrat je zastavení tohoto opt-in procesu a návrat na `--g03-live-api`.
