# G-04 — průběžný výsledek: izolovaný začátek PostgreSQL historie

**Datum:** 22. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární ID:** J09 (IN_PROGRESS, bez přejímky).  
**Výchozí main:** `01786e27dbc25ec66f80f9e4a14170c89e6ffd23`. **Ověřený SHA:** `141cdc8de03c4929d697011596e9b7f149aea073`. **Izolovaná migrace:** [CI #39856030008](https://github.com/Boym323/AirRadarV2/actions/runs/39856030008) **PASS**.

## Rozsah prvního commitu

- **G-04.04 (příprava):** skutečné soubory reverzibilní SQLx migrace v `database/migrations/`. Jedna tabulka `airradar_v2.flight_samples`, pouze PK `(bucket_at, aircraft_key)`, validace identit, souřadnic, 30s UTC bucketu, telemetrie a typů. Bez partitioning, BRIN, nadbytečných indexů a `dimension_daily`.
- **G-04.04 (testovací infrastruktura):** izolovaný PostgreSQL 18 kontejner v GitHub Actions (**PASS pro SQL migraci**), SQL test integrity a oprávnění, duplikátů, oddělení ICAO/non-ICAO a vratnosti up/down/up. CI test není ověření sdíleného DB LXC.
- Role, schéma, síť, TLS, DB writer ani retence se na cílovém LXC **neměnily**. V2 a V1 DB zůstávají nedotčené.

## Důkazy a otevřené brány

| Krok | Stav | Co ještě chybí |
|---|---|---|
| G-04.01–.03 | BLOKOVÁNO / NEOVĚŘENO | Read-only audit reálného DB LXC, schválení nových rolí a DB, TLS verify-full a přístupové testy |
| G-04.04 | SQL migrace + izolovaný CI PASS | Pozdější kontrolovaná migrace do samostatné DEV DB, práva a TLS na cílovém LXC |
| G-04.05–.12 | NEZAHÁJENO | SQLx pool, RAM-first sampler, ohraničený writer, dotazy, retence, chaos, restore, měření |

**Důležité:** Migrační `.down.sql` destruktivně smaže data pouze explicitně určené tabulky; nepoužívat jej v databázi s reálnou historií bez zálohy. Test up/down/up je povolen pouze na jednorázovém CI cíli.

### Bezpečnostní omezení

Workflow používá efemérní databázi a testovací přihlašovací údaje. `PGSSLMODE=disable` platí **pouze uvnitř CI kontejneru**, nikoli jako možnost nasazení. Pro oddělené existující PostgreSQL LXC zůstávají povinné `sslmode=verify-full`, SCRAM, oddělené role, pravidla `hostssl`, zálohy mimo LXC a správní schválení.

## Zjištění při přejímce společné CI

Původní regresní test topologie předpokládal přesně tři workflow; čtvrté izolované DB CI proto v běhu #39856029962 vyvolalo FAIL dokumentačního jobu, přestože databázový test prošel. Regrese nyní povoluje čtvrté workflow pouze s explicitním testem, že jde o GitHub-hosted DB-only proces bez deploy/release příkazů. Společná CI opraveného commitu vyžaduje ověření.

## Neprovedeno

Žádné měření reálných IOPS/WAL, žádný SQLx pool, živý zápis ani rollback na skutečném serveru. J09 nesmí dostat TESTED/ACCEPTED z pouhého schématu. Není doložena provozní přejímka G-03 do DEV, proto ani tento commit nesmí spustit automatický cutover nebo produkční deploy.

## Následující krok

**G-04.01–.03**: read-only audit samostatného DB LXC a schválení isolation/TLS/role; paralelně **G-04.06** čistý sampler s replay regresí bez SQL přístupu. Potom teprve SQLx pool a bounded DB writer.

## Pokračování G-04 v izolovaném PR #21 (23. 10. 2026)

**Větev:** `codex/g04-history-20261023`, [PR #21](https://github.com/Boym323/AirRadarV2/pull/21). Kód v PR není provozní přejímka ani release. Přesné konečné SHA a CI doplnit po testech.

- **G-04.05**: SQLx PostgreSQL pool (max. 2 připojení) s explicitním `--g04-dev-history`, povinným TLS `verify-full` a samostatnou DEV DB. Žádné automatické DB připojení v běžném G-03.
- **G-04.06**: Rust RAM-first sampler nad úspěšně sloučenou čerstvou polohou, odlišuje BEAST/JSON identitu, drží 30s bucket, prahy 30/60/120/180/600 s a odmítá zastaralé či nedůvěryhodné body. Unit testy samplingu/duplicit/manévrů/země.
- **G-04.07**: bounded pending map (2048), maximálně 128 v transakci, 5s uzavírací delay, expirace nejpozději 30s od konce bucketu, jedno inflight batch a omezené SQL timeouty. Potvrzené watermarks až po úspěšném SQL; výpadek DB znamená čítač ztrát, ne neomezený backfill.
- **G-04.08**: časově omezené SQLx čtení jedné identity s maximálně 1000 výsledky; nevytváří se další tabulka ani veřejný endpoint, který by duplikoval RAM.
- **G-04.09/.11**: ručně spouštěná retenční SQL (30 dní, max 1000 řádků, guard DB name), samostatný plán obnovení a izolovaný CI restore test.
- **G-04.01–.03/.10/.12**: skutečné DBA/TLS/backup údaje, síťový výpadek, výkon/replay a obnova na reálném DB LXC **NEOVĚŘENO**. Není dovoleno označit G-04/J09 za HOTOVO/ACCEPTED.

### Testovací brána
Výchozí SQL migrace na PostgreSQL 18 prošla [CI #39856030008](https://github.com/Boym323/AirRadarV2/actions/runs/39856030008). Nové Rust/SQLx, retence a zálohování musí projít novým CI na finálním SHA; do té doby jsou TESTY NEOVĚŘENO. Měřená úspora SSD ani latence na DEV LXC dosud nejsou k dispozici. Žádný produkční ani DEV DB LXC nebyl změněn.
