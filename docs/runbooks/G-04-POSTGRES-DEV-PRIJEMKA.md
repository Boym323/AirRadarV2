# G-04 — postup bezpečné instalace PostgreSQL historie do DEV

**Stav:** připravený runbook; neověřen na existujícím sdíleném DB LXC. **Týká se pouze AirRadar V2.** Před operacemi musí správce potvrdit oddělení V2 od databází V1 a dalších aplikací. Neprovádí se žádná automatická instalace ani migrace po merge do `main`.

## 1. Čistě read-only kontrola G-04.01–.02

S přihlašovacím účtem určeným správcem, `PGSSLMODE=verify-full`, důvěryhodnou CA a správným DNS jménem ověřit:

```sql
SELECT current_database(), current_user, version();
SELECT ssl, version, cipher FROM pg_stat_ssl WHERE pid = pg_backend_pid();
SELECT current_setting('server_version'),
       current_setting('max_connections'),
       current_setting('data_directory');
SELECT pg_size_pretty(pg_database_size(current_database()));
```

Kontrola existujícího backup/PITR, volného prostoru, HA a správného `pg_hba_file_rules` patří správci a **nesmí** obsahovat zbytečné výpisy jiných databází. Nevytvářet účet ani DB bez schválení. Ověřit negativní handshake s neplatnou CA/hostname a že V2 runtime role nemůže přistoupit do jiných databází.

## 2. DBA připraví novou izolovanou DB

Název testovací DB je `airradar_v2_dev`, role pro migrace a runtime musí být odlišné, bez `SUPERUSER`/`CREATEDB`/`CREATEROLE` a runtime bez `CREATE`, `UPDATE`, `DELETE` a `TRUNCATE`. DDL spustit výlučně pod migrační rolí po záloze. Spuštění neprovádí GitHub Actions proti DB LXC.

Schéma z `database/migrations/20261022210900_g04_initial.up.sql` má jediný primární index a žádné partitions. Migrační `.down.sql` **maže data** a nesmí sloužit jako běžný rollback historie.

## 3. Připojení Rustu je výhradně opt-in

`--g04-dev-history` zapne DEV historický writer a G-03 HTTP/SSE. Bez tohoto přepínače se SQL pool nevytváří. Konfigurační tajemství jsou výhradně v root-owned systémové konfiguraci, nikoli v Gitu, klientském Vite nebo v logu:

```text
AIRRADAR_V2_DEV_DB_HOST=postgres-dev.priklad.internal
AIRRADAR_V2_DEV_DB_PORT=5432
AIRRADAR_V2_DEV_DB_USER=airradar_v2_dev_app
AIRRADAR_V2_DEV_DB_PASSWORD=<vložit pouze do chráněného lokálního env souboru>
AIRRADAR_V2_DEV_CA_CERT=/etc/airradar-v2/pg-ca.pem
```

Jméno a heslo jsou samostatné hodnoty (není potřeba URL escapování ani `DATABASE_URL`). V produkčním repozitáři nesmějí být skutečné údaje. Použít správně zabezpečený systemd `EnvironmentFile` (s přístupem jen pro oprávněný účet), případně systemd credentials; nikdy nezadávat tajemství do argumentů procesu, veřejných logů nebo do chatu. Přihlášení na PostgreSQL serveru vyžaduje `SCRAM-SHA-256` přes konkrétní `hostssl` pravidlo v `pg_hba.conf`. Heslo autentizuje, zatímco TLS ověřuje a šifruje transport.

Engine používá pevně databázi `airradar_v2_dev`, TLS `verify-full` a CA certifikát serveru. Pool má max. 2 spojení a krátký timeout. Nejprve otestovat pouze v izolovaném DEV prostředí; výpadek SQL nesmí ovlivnit příjem a SSE. Ověřit `pg_stat_ssl` na spojení runtime role, nezapisovat reálné adresy či hesla do reportu.

## 4. Retence, indexy a obnova

Výchozí retence detailu je 30 dní. Skript `database/maintenance/g04_retention_once.sql` odmítne nesprávnou DB a maže nanejvýš 1 000 starých poloh v jedné krátké transakci; **nemá automatický timer**. Další index nebo partitioning přidat jen po reprodukovatelném `EXPLAIN (ANALYZE, BUFFERS)`, vyhodnocení WAL/CPU a schválení.

Denní `pg_dump -Fc` pouze V2 DB má být šifrovaně uložen mimo DB LXC; návrhová retence 14 denních + 8 týdenních a měsíční nezávislá obnova. Cíle RPO 24 h a RTO 2 h nejsou ověřené. PostgreSQL fyzické zálohy/PITR společného clusteru musí doložit DBA; neměnit globální WAL parametry.

## 5. Důkazní brána před G-04 HOTOVO

- ověřené G-04.01–.03 na reálném DB LXC a explicitně schválená migrační práva;
- samostatný DB test na nové DEV DB: migrace, negativní TLS/DDL test, odpojení a zotavení SQLx bez výpadku G-03;
- reálný identický replay pro 15s/30s/60s profily, indexy a partitions, SQL p95/p99, RAM/CPU, WAL/IOPS a sémantická kvalita historie;
- skutečně obnovená záloha v nezávislém DEV cíli, potvrzený rollback a písemný GO.

**Zákaz:** změnit V1, starou historii, DB nastavení jiných služeb či automaticky přesměrovat aplikaci na SQL.
