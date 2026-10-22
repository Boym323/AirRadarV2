# G-04 — průběžný výsledek: izolovaný začátek PostgreSQL historie

**Datum:** 22. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární ID:** J09 (zatím NOT_STARTED).  
**Výchozí main:** `01786e27dbc25ec66f80f9e4a14170c89e6ffd23`. **Výsledný commit a CI:** k doplnění po ověření na GitHubu.

## Rozsah prvního commitu

- **G-04.04 (příprava):** skutečné soubory reverzibilní SQLx migrace v `database/migrations/`. Jedna tabulka `airradar_v2.flight_samples`, pouze PK `(bucket_at, aircraft_key)`, validace identit, souřadnic, 30s UTC bucketu, telemetrie a typů. Bez partitioning, BRIN, nadbytečných indexů a `dimension_daily`.
- **G-04.04 (testovací infrastruktura):** izolovaný PostgreSQL 18 kontejner v GitHub Actions, SQL test integrity a oprávnění, duplikátů, oddělení ICAO/non-ICAO a vratnosti up/down/up. CI test není ověření sdíleného DB LXC.
- Role, schéma, síť, TLS, DB writer ani retence se na cílovém LXC **neměnily**. V2 a V1 DB zůstávají nedotčené.

## Důkazy a otevřené brány

| Krok | Stav | Co ještě chybí |
|---|---|---|
| G-04.01–.03 | BLOKOVÁNO / NEOVĚŘENO | Read-only audit reálného DB LXC, schválení nových rolí a DB, TLS verify-full a přístupové testy |
| G-04.04 | IMPLEMENTOVÁNO, CI k ověření | Skutečný CI výsledek, pozdější kontrolovaná migrace do samostatné DEV DB |
| G-04.05–.12 | NEZAHÁJENO | SQLx pool, RAM-first sampler, ohraničený writer, dotazy, retence, chaos, restore, měření |

**Důležité:** Migrační `.down.sql` destruktivně smaže data pouze explicitně určené tabulky; nepoužívat jej v databázi s reálnou historií bez zálohy. Test up/down/up je povolen pouze na jednorázovém CI cíli.

### Bezpečnostní omezení

Workflow používá efemérní databázi a testovací přihlašovací údaje. `PGSSLMODE=disable` platí **pouze uvnitř CI kontejneru**, nikoli jako možnost nasazení. Pro oddělené existující PostgreSQL LXC zůstávají povinné `sslmode=verify-full`, SCRAM, oddělené role, pravidla `hostssl`, zálohy mimo LXC a správní schválení.

## Neprovedeno

Žádné měření reálných IOPS/WAL, žádný SQLx pool, živý zápis ani rollback na skutečném serveru. J09 nesmí dostat TESTED/ACCEPTED z pouhého schématu. Není doložena provozní přejímka G-03 do DEV, proto ani tento commit nesmí spustit automatický cutover nebo produkční deploy.

## Následující krok

**G-04.01–.03**: read-only audit samostatného DB LXC a schválení isolation/TLS/role; paralelně **G-04.06** čistý sampler s replay regresí bez SQL přístupu. Potom teprve SQLx pool a bounded DB writer.
