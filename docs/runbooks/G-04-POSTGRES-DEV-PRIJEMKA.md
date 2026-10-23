# G-04 — postup bezpečné instalace PostgreSQL historie do DEV

**Stav:** připravený runbook; neověřen na existujícím sdíleném DB LXC. **Týká se pouze AirRadar V2.** Před operacemi musí správce potvrdit oddělení V2 od databází V1 a dalších aplikací. Neprovádí se žádná automatická instalace ani migrace po merge do `main`.

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

Pokud spojení uspěje, ověřit `pg_stat_ssl.ssl=false`, skutečný účet/DB a odmítnutí chybného hesla. Kontrola backup/PITR, prostoru, HA a existujících `pg_hba_file_rules` patří správci, **bez změny pravidel**. Pokud spojení selže (např. žádné odpovídající pravidlo), označit přístup jako BLOKOVÁNO a nic neotevírat. Jméno a heslo nešifrují data; vzdálenou či veřejnou síť nepovolovat.

## 2. DBA připraví novou izolovanou DB

Název testovací DB je `airradar_v2_dev`, role pro migrace a runtime musí být odlišné, bez `SUPERUSER`/`CREATEDB`/`CREATEROLE` a runtime bez `CREATE`, `UPDATE`, `DELETE` a `TRUNCATE`. DDL spustit výlučně pod migrační rolí po záloze. Spuštění neprovádí GitHub Actions proti DB LXC.

Schéma z `database/migrations/20261022210900_g04_initial.up.sql` má jediný primární index a žádné partitions. Migrační `.down.sql` je záměrně **zakázaná operace**; rollback historie se provádí pouze obnovou do nové izolované databáze z `pg_dump`.

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

Denní `pg_dump -Fc` pouze V2 DB má být šifrovaně uložen mimo DB LXC; návrhová retence 14 denních + 8 týdenních a měsíční nezávislá obnova. Cíle RPO 24 h a RTO 2 h nejsou ověřené. PostgreSQL fyzické zálohy/PITR společného clusteru musí doložit DBA; neměnit globální WAL parametry.

## 5. Důkazní brána před G-04 HOTOVO

- ověřené G-04.01–.03 na reálném DB LXC bez změn `pg_hba.conf` či firewallu; existující síťový přístup a oprávnění DB musí být prokázána; pokud nejsou, G-04 zůstává BLOKOVÁNO;
- samostatný DB test na nové DEV DB: migrace až po schválení, kontrola `pg_stat_ssl.ssl=false`, odmítnutí špatného hesla, kontrola skutečného síťového dosahu, negativní DDL test a zotavení SQLx bez výpadku G-03;
- reálný identický replay pro 15s/30s/60s profily, indexy a partitions, SQL p95/p99, RAM/CPU, WAL/IOPS a sémantická kvalita historie;
- skutečně obnovená záloha v nezávislém DEV cíli, potvrzený rollback a písemný GO.

**Zákaz:** změnit V1, starou historii, DB nastavení jiných služeb či automaticky přesměrovat aplikaci na SQL.
