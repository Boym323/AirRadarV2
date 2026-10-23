# G-04 — výsledek auditu a průběžná přejímka PostgreSQL historie

**Aktualizace:** 23. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární KEEP:** J09 (`IN_PROGRESS`).

## Aktuální rozhodnutí — ADR 0011: ručně dozorovaný canary (23. 10. 2026)

Vlastník zvolil **jednorázový ručně dozorovaný DEV canary bez automatického monitoringu DB LXC** a bez vytváření monitorovacího SSH účtu. Podle [ADR 0011](../adr/0011-g04-rucne-hlidany-canary.md) se v systemd G-04 jednotce **spouští přímo SHA-pinned Rust binárka**, ne `monitor_g04_canary.sh`. `CPUQuota=50%`, `MemoryMax=2G`, `RuntimeMaxSec=1875` a `Restart=no` zůstávají. Historicky připravený forced-command SSH monitor je pro tento test pouze **neaktivní podklad**, není součástí provozní brány.

**Přijaté riziko:** sdílené PostgreSQL LXC (dříve 1 vCPU, 1 GiB RAM, cca 3,1 GiB disk free) **nebude automaticky hlídané**. Správce musí být přítomen, ručně ověřit baseline, dohlížet nejpozději po 5 minutách a při DB free pod 2 GiB / SQL chybách / dlouhém přetížení / degradaci G-03 či PP Studio / nemožnosti dohledu **okamžitě zastavit jen G-04**. Samotné procesní limity aplikace nechrání sdílenou DB před zaplněním. Stávající evaluator je následnou přejímkou a musí mít skutečné ruční důkazy host metrics, SSE a DB readback.

**Není schválen start ani trvalé zapnutí.** Nyní je nutná CI na aktuálním PR HEAD, ověření skutečné nainstalované jednotky na DEV (mohla zůstat starší monitorovaná verze), lokální secure-env a přítomnosti správce. PR #21 není mergován; G-04/J09 zůstávají IN_PROGRESS. Předchozí NO-GO z důvodu chybějícího SSH monitorovacího účtu bylo pro tento jednorázový test **nahrazeno**, nikoli potichu označeno jako PASS.

## Historické opravy P1 (dřívější záznam)



Evaluator nyní vyžaduje úplné monotónní countery včetně `connection`, časové snapshoty
v nejméně čtyřech bodech po dobu nejméně 1 800 s, skutečný interval `started_at`/`finished_at`, vyvážení Rust
counterů a readback kladného počtu nových řádků. Ztráta se počítá jako
`drops / eligible`, nikoli `drops / (eligible + drops)`. Trvalý růst fronty se hledá
v časové řadě, nikoli pouze porovnáním posledních tří hodnot. Přidány jsou regresní
testy pro chybějící spojení, rostoucí frontu, DB výpadek, SQL timeout, špatné čítače,
nesprávný jmenovatel a falešné `duration_seconds`.

Canary unit je SHA-bound mimo `/opt/airradar-v2/current`, má `MemoryMax=2G`,
`RuntimeMaxSec=1875` (1800 s důkazního běhu + 35 s uzavření 30s bucketu a rezerva
pro drain) a `Restart=no`; G-03 unit, port 4101 a automatický deployment
se nemění. Úmyslný běh canary, rollback v LXC a read-only audit DB LXC nebyly tímto
commitem spuštěny; provozní důkazy proto zůstávají NEOVĚŘENO.
**Větev/PR:** `codex/g04-history-20261023`, draft PR #21. **Implementační SHA:** `fe0ac66fdebea25d21a2d211cd74cab922dd7d05`. **Předchozí benchmark binárky:** `36afdda5e752cc320496f44672a94e9434eea9eb`. **Měřicí harnessy/report:** `ebc20307c8962593c3895908f4b54e2f63aeaa8a` / `4e1ff8b7396f78377ce3f429c5fe663e8fd1df67`.

## DEV preflight 23. 10. 2026 — canary nespouštěn

Preflight proběhl přímo na DEV LXC `lo-dev-airradarv2` (x86-64, Debian 13) bez startu,
restartu nebo enable G-04. Pracovní větev byla před kontrolou fast-forwardována na PR tip
`d1c54d1807bb093c0c63f6535908a3abc988004a`; canary artefakt byl záměrně sestaven z přesného
implementačního SHA `fe0ac66fdebea25d21a2d211cd74cab922dd7d05` v izolovaném worktree.

### Artefakt a instalace

- Příkaz: `cargo build --release --locked -p airradarv2-engine`.
- Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1`, `stable-x86_64-unknown-linux-gnu`.
- Výsledný SHA-256: `1dd11387635eeb16040b69f33e09a6108d1fecf2491dde56699cc9b5971ba21b`.
- Artefakt: `/opt/airradar-v2/canary/releases/fe0ac66fdebea25d21a2d211cd74cab922dd7d05/airradarv2-engine`,
  `root:root`, `0755`, ELF x86-64; `ldd` nehlásí chybějící knihovnu. Vazba commit/toolchain/build/hash
  je v `BUILD-METADATA.txt` vedle binárky. `/opt/airradar-v2/current` ani G-03 release nebyly změněny.

### Aktuální G-03 baseline

`airradarv2-engine.service` je `active/enabled`, PID `8924`, `--g03-live-api`,
`AIRRADAR_V2_BEAST_ADDR=192.168.1.50:30005`; RSS procesu bylo `5 116 KiB`,
`MemoryCurrent=4 317 184 B`. Health `127.0.0.1:4101/api/v1/health` vrátil
`status=live`, `stage=G-03`, `live_ingest=true`, `database_connected=false`.
Aircraft endpoint vrátil HTTP 200 a 29 087 B, `/api/v1/stream` vrátil živý snapshot
a proces měl navázané TCP spojení na readsb `192.168.1.50:30005`.

### Canary unit a credentials

Unit `/etc/systemd/system/airradarv2-g04-canary.service` je načtená jako `static`,
`inactive/dead`, bez listeneru na `127.0.0.1:4114`; není povolená pro boot ani deployment.
`systemd-analyze verify` pro G-04 již nemá chybu vlastní jednotky; vypisuje pouze starší
nesouvisející varování G-03 `ConditionPathIsExecutable` a G-02 oneshot `RuntimeMaxSec`.
Ověřeno: SHA-bound `ExecStart`, samostatný `EnvironmentFile`, `Restart=no`, `MemoryMax=2G`,
`RuntimeMaxSec=1875`, oddělený working directory a `--g04-dev-history`.

Na DEV je `/etc/airradar-v2/g04-canary.env`, `root:root`, `0600`, s adresou DB
`192.168.1.55:5432`, uživatelem `airradar_v2_dev_app` a BEAST adresou. Neprázdnost hesla
byla ověřena bez jeho čtení do výstupu; samotná hodnota zůstává neveřejná. Runtime credentials
jsou tedy připravené, jednotka se však stále nesmí spustit bez monitorovacího přístupu.

### DB, monitoring a návrat

`pg_isready` pro `192.168.1.55:5432` potvrdil pouze `accepting connections`; autentizovaný
SQL/capacitní audit zatím neproběhl, protože v env chybí schválený SSH cíl pro read-only metriky
DB LXC. Volné místo DB filesystemu, WAL, CPU/RAM hostitele, SQL připojení a fronty proto zůstávají
**NEOVĚŘENO**. Nový monitor tuto situaci správně ukončí fail-closed před startem child procesu.

`tools/rollback_g04_canary.sh` prošel `bash -n` a statickou kontrolou a byl spuštěn s G-04
ve stavu `inactive`; skončil `0`, kontrola G-03 zůstala `active` a data se neměnila.
ADR 0010 potvrzuje, že ztráta historie V2 je přijatá; neodstraňuje však povinné kapacitní,
bezpečnostní, monitoringové a rollback brány. Readsb konfigurace zůstává beze změny a G-04
je nakonfigurován na stejný read-only BEAST server přes vlastní TCP spojení až po případném GO.

### Výsledek preflightu

| Kontrola | Výsledek |
|---|---|
| SHA-pinned binárka, architektura, knihovny, metadata | PASS |
| Credentials root:root `0600` s neprázdným heslem | PASS, hodnota nebyla vypsána |
| Unit validní, static/inactive, oddělená od G-03 | PASS s nesouvisejícími verify varováními |
| G-03 HTTP/SSE a BEAST baseline | PASS |
| PostgreSQL dostupnost | ČÁSTEČNĚ — TCP/`pg_isready` PASS, SQL/capacita NEOVĚŘENO |
| Vynucovaný monitoring CPU/RAM/WAL/místa/front | PASS implementace/test; DB LXC přístup NEOVĚŘEN |
| Rollback a ochrana historie | PASS staticky i proti neaktivní G-04 |
| PR #21 CI | PASS: run `39956195962` a `39956196089`, všechny 4 kontroly PASS |

**Závěr: NO-GO. Výsledek není `READY FOR USER GO — CANARY NOT STARTED`, protože chybí schválený
read-only přístup k aktuálním metrikám sdíleného PostgreSQL LXC a jeho kapacitní audit.
Canary nebyl spuštěn. G-04/J09 zůstává `ROZPRACOVÁNO` / `IN_PROGRESS`; nic není `HOTOVO` ani `ACCEPTED`.**

## Příprava fail-closed monitoringu — 23. 10. 2026

Do PR byla doplněna implementace `tools/monitor_g04_canary.sh` a její funkční regresní test
`tools/test_monitor_g04_canary.sh`. Monitor je součástí stejného SHA-pinned release jako binárka,
vyžaduje `root:root 0600` env soubor a schválený `AIRRADAR_G04_DB_MONITOR_SSH_TARGET`; bez něj
končí před spuštěním G-04. SQL dotaz čte pouze `pg_stat_wal.wal_bytes` bez `pg_ls_waldir()`,
hostitelský dohled přes omezený forced-command SSH helper používá `loadavg`, `free -b` a `df -Pk`. Limity jsou 2 GiB volného místa, 10 MiB/s růst WAL, 80 %
load/cpu na počet CPU a 90 % použité RAM; limity procesu G-04 jsou `CPUQuota=50%` a `MemoryMax=2G`.
Hodnoty jsou výchozí provozní brána a musí být před canary potvrzeny skutečným DBA auditním výstupem.
Monitor byl do release adresáře na DEV nainstalován jako `root:root`, `0755`; SHA-256 zdroje i
instalované kopie je `89f0e0d134303b86db5fd7ebc43cac27ea0798722f3636ca2c9d2a44a269a0dc`.

Provedené důkazy bez spuštění canary:

- env soubor existuje jako `root:root`, `0600`; neprázdnost hesla byla ověřena bez jeho čtení do výstupu;
- funkční test prošel PASS scénářem a ověřil fail-closed při SSH výpadku/timeoutu, stale diagnostice,
  degradaci G-03, nízkém disku, SQL chybě a nebezpečném WAL růstu; child G-04 byl v každém scénáři ukončen;
- chybějící monitorovací přístup vedl k očekávanému exit `78` před startem child procesu;
- rollback skript byl spuštěn při neaktivní G-04 a skončil `0`; G-03 zůstal `active`;
- `bash -n`, funkční monitor test a boundary test prošly; žádná změna V1, PP Studio, `pg_hba.conf`, firewallu ani globální PostgreSQL konfigurace nebyla provedena.

Aktuální stav zůstává **NO-GO**: v chráněném DEV env není schválený SSH cíl/read-only přístup
k metrikám sdíleného PostgreSQL LXC. Canary nebyl spuštěn. Po doplnění přístupu je nutné znovu
provést preflight a teprve po výslovném GO instalovat monitor i binárku do stejného release.

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

## Backup — výjimka pro postradatelnou historii V2

Vlastník schválil [ADR 0010](../adr/0010-g04-postradatelna-historie-bez-zaloh.md): stará historie letadel AirRadar V2 je postradatelná. Denní zálohy, PITR a provozní RPO/RTO nejsou vyžadovány; při havárii může zmizet celá historie. Dřívější izolované pg_dump/pg_restore důkazy G-04.11 zůstávají platné, nejde však o provozní backup politiku. Zálohy ostatních databází a sdílený PostgreSQL nejsou dotčeny.

Přípravný read-only dump `pg_dump -Fc` z 23. 10. 2026 byl uložen mimo DB LXC do lokálního chráněného artefaktu (mode 0600, adresář 0700), velikost 4 165 B, SHA-256 `8419d06ab60736b47f6d54e0d7571e8d4beaf0c4e07d8d8d8d5e632bf37d51c8`. `pg_restore --list` prošel. Obnova proběhla pouze do nezávislé izolované databáze; to dokládá obnovitelnost tohoto snapshotu, nikoli denní plán, dostupnost kopií ani provozní RPO/RTO. Připravený `tools/backup_g04_dev.sh` vyžaduje chráněný `PGPASSFILE`, pevně kontroluje databázi `airradar_v2_dev` a nemění serverovou konfiguraci. Pro historii V2 se žádný pravidelný backup timer nezavádí (ADR 0010).

Z runtime SQL účtu nebylo možné bezpečně zjistit volné místo filesystému DB LXC, CPU/RAM hostitele ani WAL disk. Tyto tři hodnoty jsou provozní blokace pro canary; nesmí se nahrazovat velikostí databáze 7 558 kB.

### Vlastnictví objektů a migrace

DB vlastní `airradar_v2_dev_migrator`, ale schéma `airradar_v2` a tabulku `flight_samples` vlastní `postgres`. Runtime role smí pouze `USAGE`, `SELECT` a `INSERT`; migrace proto nesmí běžet pod `airradar_v2_dev_app`. Každou budoucí změnu musí předem prověřit a provést DBA pod vlastníkem objektu nebo schváleným migračním účtem s dočasným, přesně omezeným DDL právem. V tomto PR se vlastníci ani granty nemění.

### Připravený canary a hranice automatického deploymentu

`deploy/systemd/airradarv2-g04-canary.service` je pouze neinstalovaná šablona: používá loopback port `4114`, chráněný `/etc/airradar-v2/g04-canary.env` mode 0600 a `--g04-dev-history`; nikdy se sama nepovoluje. G-03 zůstává v existující jednotce na portu `4101` s `--g03-live-api`. `tools/check-g04-deploy-boundary.mjs` kontroluje, že main auto-deploy ani G-03 instalační skripty neobsahují přepnutí na G-04. Merge do `main` tedy podle aktuální topologie nemá tento canary unit instalovat ani spouštět.

Předpokladem 30minutového běhu je samostatný schválený readsb/BEAST vstup nebo bezpečně multiplexovaný feed; canary se nesmí spustit pouze přidáním druhého konzumenta produkčního receiveru. `tools/evaluate_g04_canary.py` vyžaduje diagnostický log, minimálně 1 800 s, SQL readback, G-03 SSE heartbeat, CPU/RAM a WAL měření. Vrací FAIL při ztrátě nad 1 %, timeoutu, trvalém růstu fronty, nedostupném spojení nebo chybějícím provozním důkazu. Návrat provádí výhradně `tools/rollback_g04_canary.sh`: zastaví opt-in unit, ověří aktivní G-03 a nemaže historii.

## Co je implementováno

PR obsahuje `history.rs` (RAM sampler), `history_store.rs` (SQLx pool, batch INSERT, query a omezený retry), napojení do jediného ingest actoru, retenční SQL a izolované CI/restore testy. G-02 RAM/readsb/BEAST a G-03 HTTP/SSE zůstávají opt-in a databáze není aktivní v běžném režimu.

Destruktivní `.down.sql` migrace je fail-closed; rollback canary pouze zastaví opt-in writer a ponechá G-03 v provozu. Při ztrátě V2 DB lze po schválení DBA obnovit jen prázdné schéma; stará historie může být neobnovitelná (ADR 0010). `up.sql` je idempotentní vůči existující tabulce, ale před ostrým spuštěním musí migrační účet ověřit kompatibilitu sloupců a omezení.

## Neuzavřené brány

**Zálohy postradatelné V2 historie nejsou blokací** dle ADR 0010. Chybí read-only důkaz volného místa/WAL/CPU/RAM sdíleného DB LXC, schválený budoucí migrační postup, bezpečný samostatný feed a skutečný 30minutový canary. RF replay před canary zůstává neověřen. Ztráty 18,1 % při 2× a 59,0 % při 4× jsou mimo navrženou canary toleranci; writer proto nesmí být aktivován bez časově omezeného scénáře s limitem ztrát. G-04 není HOTOVO a J09 není TESTED/ACCEPTED.

## Brána prvního DEV canary — zatím NO-GO

Před samostatným GO musí být doloženo:

1. schválený vlastník schématu/tabulky nebo potvrzený migrační postup pro objekty vlastněné `postgres`;
2. potvrzené přijetí úplné ztráty historie V2 dle ADR 0010; provozní backup/RPO/RTO se nepožadují. Namísto nich doložit volnou kapacitu sdílené DB (disk, WAL, CPU/RAM);
3. canary dataset s minimálně 100 letadly a časovým replayem, `dropped_capacity + dropped_stale + dropped_writer <= 1 %` všech kandidátních historických vzorků;
4. žádné `sql_timeouts`, `connection=down` nebo růst `pending/inflight` na konci canary;
5. CPU/RAM aplikace, SQL latency p95/p99, WAL a SSE heartbeat bez regresního překročení schválených limitů;
6. připravený rollback: zastavit pouze opt-in writer, vrátit proces na `--g03-live-api`, ověřit BEAST/RAM/SSE a ponechat databázová data nedestruktivně.

Aktuální 2× a 4× ztráty tento limit nesplňují. Doporučení je **NO-GO**, dokud nebude canary záměrně omezen na provozní rychlost s doloženou ztrátou nejvýše 1 % nebo nebude vlastníkem schválena jiná hodnota.

### Bezpečný postup další aktivace

1. Správce potvrdí postup budoucích migrací a bezpečnou kapacitu sdíleného DB LXC. Pro historii V2 nejsou vyžadovány zálohy/RPO/RTO (ADR 0010). V1, `pg_hba.conf` a firewall se nemění.
2. Na nové/ověřené DEV DB proběhne pouze kompatibilitní migrace a negativní test práv.
3. V izolaci se změří restart/timeout/reconnect, restore a benchmarky; živý příjem/SSE musí pokračovat při SQL degradaci.
4. Teprve po písemném schválení se spustí `--g04-dev-history`; návrat je zastavení tohoto opt-in procesu a návrat na `--g03-live-api`.
