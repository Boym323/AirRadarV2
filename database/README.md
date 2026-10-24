# Databáze AirRadar V2 — návrh a první SQLx migrace

**Stav G-04:** SQLx migrační soubory jsou připravené v [migrations](migrations/), izolované PostgreSQL CI je PASS. DEV DB `airradar_v2_dev` na existujícím LXC je read-only auditně dostupná a schema/tabulka odpovídá migraci; migrace nebyla tímto agentem proti sdílenému LXC spuštěna.

V G-04 vznikne vlastní PostgreSQL `airradar_v2` **v již existujícím samostatném databázovém LXC** s odděleným účtem a SQLx migracemi. Nová databáze začne **bez historických letů z V1**. Nikdy ji nenastavuj na produkční `DATABASE_URL` původního AirRadaru.

## Zásady

- Autoritativní živá letadla zůstávají v Rust RAM; PostgreSQL přijímá pouze vzorkované historie a provozní referenční údaje.
- Datové modely tvořit podle konkrétních dotazů a přístupových vzorců, ne kopírováním Prisma schématu V1.
- Potřebné vzorkované validní polohy ukládat přes ohraničenou frontu a dávkové transakce; samostatnou entitu letových instancí nevytvářet bez konkrétního schváleného konzumenta.
- Retenci a indexy stanovit po měření: velikost dat, CPU, WAL, rychlost INSERT, četnost SELECT a p95/p99.
- Začít s jednoduchým SQL a SQLx migracemi, oddělit integritu a přístupová práva.
- **Žádný automatický přesun či mazání dat V1**. Pro G-04 použít jen DEV databázi se samostatným účtem.
- Prázdná historie po spuštění V2 není chyba; UI musí zobrazovat pravdivé „zatím nejsou data“.

Podrobné požadavky: [hlavní zadání](../docs/plan/ZADANI-AIRRADAR-V2.md) a etapa [G-04](../docs/plan/ETAPY-A-STAV.md).

Bezpečné připojení a role: [POSTGRES-LXC.md](POSTGRES-LXC.md). Skutečný DB LXC nebyl touto změnou upraven.

**Návrh a audit:** [SQL tabulky, indexy, sampling, retence a zálohy](POSTGRES-DATOVY-MODEL.md). **Ověření bezpečnosti LXC:** [LAN/firewall/role/backup](../docs/runbooks/POSTGRES-LXC-SECURITY-BACKUP.md). Žádné DDL SQL nebylo tímto agentem aplikováno na sdílený LXC.

**Optimalizace CPU a SSD:** [RAM-first deduplikace a adaptivní historie](../docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md), [ADR 0007](../docs/adr/0007-ram-first-deduplikace-a-usporny-zapis.md). Obě dokumentace nahrazují starý plán fixního 15s INSERT. `node tools/check-eco-sampling.mjs` ověřuje pouze deterministický matematický model, ne skutečnou PostgreSQL zátěž.


## Pravidelná retence G-04.09 — připraveno, nikoli zapnuto

Dřívější ruční skript s dávkou 1 000 řádků doplňuje [DBA-managed 30denní údržba](../docs/runbooks/G-04-RETENCE-30-DNI.md) s explicitním GO, 20×1 000 omezením a separátní rolí. Systemd timer se instaluje **jen na DB LXC správcem**, nikoli na DEV app LXC a ne po merge. [ADR 0017](../docs/adr/0017-g04-dba-retence.md).

## G-04 implementace (PR #21)

SQLx runtime a RAM sampler jsou izolovaně implementované. Jednorázová retence: [g04_retention_once.sql](maintenance/g04_retention_once.sql). Bezpečný [postup G-04](../docs/runbooks/G-04-POSTGRES-DEV-PRIJEMKA.md) vyžaduje samostatnou DEV databázi a bezpečnou kapacitu sdíleného LXC; izolovaný export/restore G-04.11 již proběhl. Pravidelné zálohy historie V2 nejsou požadovány ([ADR 0010](../docs/adr/0010-g04-postradatelna-historie-bez-zaloh.md)). Tento repozitář nenastavuje databázové LXC automaticky.

**Připojení G-04 v LAN:** podle [ADR 0009](../docs/adr/0009-g04-postgresql-lan-bez-tls.md) používá SQLx `sslmode=disable` a přihlášení jménem a heslem z chráněné konfigurace. Použijí se výhradně existující pravidla PostgreSQL a firewallu; nové `hostnossl` se nepřidává. Nešifrovaný přenos je vědomou výjimkou pouze pro důvěryhodnou místní síť.

**Přístup bez změny `pg_hba.conf`:** G-04 použije pouze stávající PostgreSQL pravidla a privátní spojení v LAN. Pravidlo `host` může pokrývat i TCP bez TLS; nové `hostnossl` není vyžadováno ani vytvářeno. Pokud přihlášení neprojde, nesmí jej kód obejít ani měnit pravidla; viz [ADR 0009](../docs/adr/0009-g04-postgresql-lan-bez-tls.md).
