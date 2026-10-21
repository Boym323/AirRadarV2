# Databáze AirRadar V2 — připravovaný návrh

**Stav G-00: databáze nebyla vytvořena, nebyla provedena žádná migrace.**

V G-04 vznikne vlastní PostgreSQL `airradar_v2` **v již existujícím samostatném databázovém LXC** s odděleným účtem a SQLx migracemi. Nová databáze začne **bez historických letů z V1**. Nikdy ji nenastavuj na produkční `DATABASE_URL` původního AirRadaru.

## Zásady

- Autoritativní živá letadla zůstávají v Rust RAM; PostgreSQL přijímá pouze vzorkované historie a provozní referenční údaje.
- Datové modely tvořit podle konkrétních dotazů a přístupových vzorců, ne kopírováním Prisma schématu V1.
- Vzorkované pozice a letové instance zapisovat přes ohraničenou frontu a dávkové transakce.
- Retenci a indexy stanovit po měření: velikost dat, CPU, WAL, rychlost INSERT, četnost SELECT a p95/p99.
- Začít s jednoduchým SQL a SQLx migracemi, oddělit integritu a přístupová práva.
- **Žádný automatický přesun či mazání dat V1**. Pro G-04 použít jen DEV databázi se samostatným účtem.
- Prázdná historie po spuštění V2 není chyba; UI musí zobrazovat pravdivé „zatím nejsou data“.

Podrobné požadavky: [hlavní zadání](../docs/plan/ZADANI-AIRRADAR-V2.md) a etapa [G-04](../docs/plan/ETAPY-A-STAV.md).

Bezpečné připojení a role: [POSTGRES-LXC.md](POSTGRES-LXC.md). Skutečný DB LXC nebyl touto změnou upraven.
