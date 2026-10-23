# G-04 — postup bezpečné instalace PostgreSQL historie do DEV

**Stav:** připravený runbook; read-only audit DEV je doložen, canary není schválen. **Týká se pouze AirRadar V2.** Před operacemi musí správce potvrdit oddělení V2 od databází V1 a dalších aplikací. Neprovádí se žádná automatická instalace ani migrace po merge do `main`.

## 1. Čistě read-only kontrola G-04.01–.02

S již existujícím účtem z důvěryhodné LAN nejprve bez úprav PostgreSQL ověřit, zda **stávající pravidla** dovolují `PGSSLMODE=disable`:

```sql
SELECT current_database(), current_user, version();
SELECT ssl, version, cipher FROM pg_stat_ssl WHERE pid = pg_backend_pid();
SELECT current_setting('server_version'),
       current_setting('max_connections'),
       current_setting('data_directory');
SELECT pg_size_pretty(pg_database_size(current_database()));
```

Pokud spojení uspěje, ověřit `pg_stat_ssl.ssl=false`, skutečný účet/DB a odmítnutí chybného hesla. Kontrola volného místa/WAL, CPU/RAM, HA a existujících `pg_hba_file_rules` patří správci; historie V2 nemá povinné zálohy dle ADR 0010, **bez změny pravidel**. Pokud spojení selže (např. žádné odpovídající pravidlo), označit přístup jako BLOKOVÁNO a nic neotevírat. Jméno a heslo nešifrují data; vzdálenou či veřejnou síť nepovolovat.

## 2. DBA připraví novou izolovanou DB

Název testovací DB je `airradar_v2_dev`, role pro migrace a runtime musí být odlišné, bez `SUPERUSER`/`CREATEDB`/`CREATEROLE` a runtime bez `CREATE`, `UPDATE`, `DELETE` a `TRUNCATE`. DDL spustit výlučně pod migrační rolí po záloze. Spuštění neprovádí GitHub Actions proti DB LXC.

Schéma z `database/migrations/20261022210900_g04_initial.up.sql` má jediný primární index a žádné partitions. Migrační `.down.sql` je záměrně **zakázaná operace**; rollback canary znamená jen zastavení writeru bez zásahu do G-03. Při úplné ztrátě V2 DB může historie nenávratně zmizet a nové schéma lze vytvořit ze schválených migrací dle ADR 0010.

## 3. Připojení Rustu je výhradně opt-in

`--g04-dev-history` zapne DEV historický writer a G-03 HTTP/SSE. Bez tohoto přepínače se SQL pool nevytváří. Konfigurační tajemství jsou výhradně v root-owned systémové konfiguraci, nikoli v Gitu, klientském Vite nebo v logu:

```text
AIRRADAR_V2_DEV_DB_HOST=postgres-dev.priklad.internal
AIRRADAR_V2_DEV_DB_PORT=5432
AIRRADAR_V2_DEV_DB_USER=airradar_v2_dev_app
AIRRADAR_V2_DEV_DB_PASSWORD=<vložit pouze do chráněného lokálního env souboru>
```

Jméno a heslo jsou samostatné hodnoty (není potřeba URL escapování ani `DATABASE_URL`). V produkčním repozitáři nesmějí být skutečné údaje. Použít správně zabezpečený systemd `EnvironmentFile` (s přístupem jen pro oprávněný účet), případně systemd credentials; nikdy nezadávat tajemství do argumentů procesu, veřejných logů nebo do chatu. Autentizaci řídí **existující konfigurace PostgreSQL**; žádné nové `hostnossl` ani změna `pg_hba.conf` nejsou součástí G-04. Pro připojení bez TLS stačí již platné a oprávněné pravidlo typu `host`, pokud skutečně pokrývá uživatele, databázi a zdrojovou IP. SCRAM je doporučený mechanismus pro heslo, ale jeho dostupnost je třeba ověřit, ne předpokládat. Spojení **není šifrováno**. Viz [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md).

Engine používá pevně databázi `airradar_v2_dev`, `sslmode=disable`, bez CA/certifikátu; pool má max. 2 spojení a krátký timeout. Na sdíleném DB serveru **nevypínat TLS globálně**, mohou ho potřebovat jiné aplikace. Nejprve otestovat pouze v izolovaném DEV prostředí; výpadek SQL nesmí ovlivnit příjem a SSE. Ověřit `pg_stat_ssl` na spojení runtime role, nezapisovat reálné adresy či hesla do reportu.

## 4. Retence, indexy a obnova

Výchozí retence detailu je 30 dní. Skript `database/maintenance/g04_retention_once.sql` odmítne nesprávnou DB a maže nanejvýš 1 000 starých poloh v jedné krátké transakci; **nemá automatický timer**. Další index nebo partitioning přidat jen po reprodukovatelném `EXPLAIN (ANALYZE, BUFFERS)`, vyhodnocení WAL/CPU a schválení.

**ADR 0010:** historie V2 je postradatelná a vlastník výslovně nevyžaduje denní zálohy, PITR ani provozní RPO/RTO. V případě havárie může nenávratně zmizet celá historie; jiné databáze a jejich zálohování zůstávají nedotčené. Přesto je nutné před canary ověřit volné místo, WAL/CPU/RAM a ochranu sdíleného clusteru.

Připravený `tools/backup_g04_dev.sh` zůstává **volitelná ruční pomůcka**, nikoliv povinný backup timer. Dřívější izolovaná zkouška exportu/obnovy G-04.11 zůstává důkazem testu, ale historie V2 může při havárii nenávratně zmizet (ADR 0010).

## 5. Důkazní brána před G-04 HOTOVO

- ověřené G-04.01–.03 na reálném DB LXC bez změn `pg_hba.conf` či firewallu; existující síťový přístup a oprávnění DB musí být prokázána; pokud nejsou, G-04 zůstává BLOKOVÁNO;
- samostatný DB test na nové DEV DB: migrace až po schválení, kontrola `pg_stat_ssl.ssl=false`, odmítnutí špatného hesla, kontrola skutečného síťového dosahu, negativní DDL test a zotavení SQLx bez výpadku G-03;
- reálný identický replay pro 15s/30s/60s profily, indexy a partitions, SQL p95/p99, RAM/CPU, WAL/IOPS a sémantická kvalita historie;
- izolovaný pg_dump/pg_restore test G-04.11 již doložen. Pravidelné zálohy/RPO/RTO historie V2 nejsou bránou; zůstávají kapacitní limity, potvrzený nedestruktivní rollback a písemné GO.

## 6. Read-only audit kapacity sdíleného DB LXC

Skript `database/maintenance/g04_readonly_audit.sql` ověří pouze aktuální DB, WAL
počítadla a bezpečné provozní nastavení. Volné místo filesystemu DB LXC, CPU a RAM
hostitele musí správce doložit samostatným read-only výstupem (`df -P`, `free -b`,
`nproc`/`/proc/stat`) se SHA artefaktu, časem a identitou DB LXC. SQL účet tyto údaje
nesmí nahrazovat velikostí databáze. Kapacita WAL musí obsahovat použití, volné místo
WAL filesystemu a upozornění na `archive_mode=off`; globální WAL ani `pg_hba.conf`
se nemění.

Podle [ADR 0010](../adr/0010-g04-postradatelna-historie-bez-zaloh.md) se pro historii V2 pravidelné zálohy a provozní RPO/RTO nepožadují; žádný timer se nezavádí. Volný disk/WAL a CPU/RAM sdíleného PostgreSQL LXC zůstávají nepřekročitelnou provozní bránou.

## 7. Kontrolní seznam prvního časově omezeného canary

Canary se smí spustit až po samostatném GO a pouze s `AIRRADAR_V2_HISTORY_TRACE=1` v chráněném lokálním prostředí. Unit používá samostatný artefakt v `/opt/airradar-v2/canary/releases/<ověřený SHA>/`, nikdy `/opt/airradar-v2/current`, má `MemoryMax=2G`, `RuntimeMaxSec=1875` a `Restart=no`. Prvních 1 800 s je minimální důkazní běh; zbývajících 75 s pokrývá uzavření posledního 30s bucketu (nejpozději bucket + 35 s) a vyprázdnění fronty. Evaluator i tak vyžaduje minimálně 1 800 s skutečných časových důkazů a poslední `pending=0`/`inflight=0`. Výchozí akceptační limit je **nejvýše 1 % kandidátních historických vzorků** ztracených součtem `dropped_capacity`, `dropped_stale` a `dropped_writer`; jmenovatel je poslední Rust `eligible`, protože drop counter je jeho podmnožina. Předchozí hodnoty 18,1 % a 59,0 % z 2×/4× testu tento limit nesplňují.

Evaluator vyžaduje v každém diagnostickém snapshotu všechny countery, `connection=up`
a `observed_at`; musí existovat nejméně čtyři časově seřazené snapshoty pokrývající
nejméně 1 800 sekund a kumulativní countery nesmějí klesat. JSON metrics
musí mít skutečné `started_at`/`finished_at` s rozdílem nejméně 1 800 sekund, nikoli
jen ručně nastavené `duration_seconds`. PASS dále vyžaduje vyvážení
`committed + drops + pending + inflight = eligible`, skutečný DB readback s kladným
počtem nových řádků, nejméně dva G-03 SSE heartbeat důkazy, CPU/RAM/WAL a volné místo
DB LXC. Chybějící hodnota je FAIL.

- [ ] Správce písemně potvrdil DB ownera `airradar_v2_dev_migrator`, skutečné vlastníky schématu/tabulky (`postgres`) a postup DDL/migrací bez změny ownerů.
- [ ] Potvrzeno rozhodnutí [ADR 0010](../adr/0010-g04-postradatelna-historie-bez-zaloh.md): akceptujeme úplnou ztrátu historie V2. Nevyžaduje se provozní záloha/RPO/RTO; otestovaná je bezpečná degradace G-04 bez vlivu na G-03 a kapacita sdíleného LXC.
- [ ] Před startem je uložen baseline G-03: proces, SHA, BEAST/RAM/SSE, CPU/RSS, DB row count a rollback příkaz.
- [ ] Canary má samostatný read-only JSON/BEAST feed; druhé BEAST TCP připojení je povoleno jen po ověření podpory receiveru a nulového dopadu na G-03. Jinak je canary BLOKOVÁNO.
- [ ] Canary běží pouze časově omezeně, s monitoringem `pending`, `inflight`, `committed`, všech drop counters, SQL timeoutů a `connection=up`.
- [ ] Na konci platí `pending=0`, `inflight=0`, `connection=up`, žádné SQL timeouty a ztráta nejvýše 1 %; jinak NO-GO.
- [ ] Rollback znamená pouze zastavení opt-in writeru, návrat na `--g03-live-api`, kontrolu BEAST/RAM/SSE a nedestruktivní ponechání DB dat.

Připravená jednotka `deploy/systemd/airradarv2-g04-canary.service` není součástí běžného G-03 installeru a nesmí být před GO instalována. Port `4114` je pouze interní loopback; G-03 `4101` se nemění. `tools/evaluate_g04_canary.py` je rozhodovací brána (exit 0 = PASS, exit 1 = FAIL) a `tools/rollback_g04_canary.sh` neobsahuje mazání DB dat. Požadované provozní důkazy: DB readback s novými reálnými ICAO, poslední `pending=0`/`inflight=0`, nulové SQL timeouty, `connection=up`, ztráta kandidátů nejvýše 1 %, stabilní G-03 heartbeat a známé CPU/RAM/WAL/volné místo DB LXC.

### Automatický fail-closed dohled

`tools/monitor_g04_canary.sh` je wrapper SHA-pinned release. Každých 15 s provádí pouze `SELECT` přes existující runtime přístup, read-only příkaz přes schválený SSH cíl DB LXC (`AIRRADAR_G04_DB_MONITOR_SSH_TARGET`) a health kontrolu G-03. Sleduje volné místo dat/WAL filesystemu (minimum 2 GiB), kumulativní růst WAL, load/CPU, RAM a G-04 diagnostiku. Při chybě aktuální metriky, SQL timeoutu, `connection != up`, degradaci G-03 nebo překročení limitu ukončí pouze podřízený G-04 proces; G-03 nereštartuje. Unit má navíc `CPUQuota=50%`, `MemoryMax=2G`, `Restart=no`.

Před instalací jednotky musí být monitorovací skript a binárka společně vloženy do stejného immutable release adresáře a ověřeny proti SHA; chybějící skript je blokován `ConditionPathExists`. Cílový env soubor zůstává `root:root`, `0600`; hodnoty tajemství se nekopírují do reportů. Chybějící SSH cíl nebo jeho výpadek je záměrně NO-GO/fail-closed.

**Zákaz:** změnit V1, starou historii, DB nastavení jiných služeb či automaticky přesměrovat aplikaci na SQL.
