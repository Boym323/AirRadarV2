# Databáze AirRadar V2 — návrh a první SQLx migrace

**Stav G-04.04:** SQLx migrační soubory jsou připravené v [migrations](migrations/), jejich izolovaný PostgreSQL 18 [CI test #39856030008](https://github.com/Boym323/AirRadarV2/actions/runs/39856030008) je PASS. Provozní databáze ani DEV DB na existujícím LXC nebyla vytvořena a migrace tam nebyla spuštěna.

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

**Návrh před G-04:** [SQL tabulky, indexy, denní UTC partitions, sampling, retence a zálohy](POSTGRES-DATOVY-MODEL.md). **Ověření bezpečnosti LXC:** [TLS/firewall/role/backup](../docs/runbooks/POSTGRES-LXC-SECURITY-BACKUP.md). Žádné SQL zatím nebylo aplikováno.

**Optimalizace CPU a SSD:** [RAM-first deduplikace a adaptivní historie](../docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md), [ADR 0007](../docs/adr/0007-ram-first-deduplikace-a-usporny-zapis.md). Obě dokumentace nahrazují starý plán fixního 15s INSERT. `node tools/check-eco-sampling.mjs` ověřuje pouze deterministický matematický model, ne skutečnou PostgreSQL zátěž.


## G-04 implementace (PR #21)

SQLx runtime a RAM sampler jsou izolovaně implementované. Jednorázová retence: [g04_retention_once.sql](maintenance/g04_retention_once.sql). Bezpečný [postup G-04](../docs/runbooks/G-04-POSTGRES-DEV-PRIJEMKA.md) vyžaduje schválenou samostatnou DEV databázi, TLS a test restore. Tento repozitář nenastavuje databázové LXC automaticky.
