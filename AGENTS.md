# AGENTS.md — závazná pravidla pro AirRadar V2

**Veškerá nová zadání, Markdown dokumentace, reporty, ADR a výchozí UI musí být v češtině (`cs-CZ`).** Technické identifikátory API, SQL a kódu zůstávají běžně anglicky.

## Před každou změnou

1. Ověř aktuální `main`, pracovní strom, `AGENTS.md`, [začátek pro nové vlákno](docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md), [hlavní zadání](docs/plan/ZADANI-AIRRADAR-V2.md), [technologická rozhodnutí](docs/plan/TECHNOLOGICKA-ROZHODNUTI.md) a relevantní jeden úkol v `docs/ukoly/`. Před migrací frontend/DB prostuduj také [průzkum](docs/plan/TECHNOLOGICKY-PRUZKUM.md) a [benchmarkový plán](docs/plan/BENCHMARKY-TECHNOLOGII.md).
2. `contracts/feature-matrix.json` je **pouze seznam aktivních 45 funkcí**. Každá má vlastníka v [MODULY-A-ROZSAH.md](docs/plan/MODULY-A-ROZSAH.md). Rozšíření nemíchej do tohoto seznamu ani do kódu; jejich jediným záznamem je [oddělený archiv](docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md), který se otevírá pouze na výslovnou žádost.
3. Při přidávání souboru, modulu, tabulky, závislosti, datového zdroje nebo UI prvku uveď konkrétní aktivní ID, konzumenta a test. Nemá-li přínos nebo přidává zbytečné komplikace, neimplementuj jej.
4. Každou etapu uzavírej českým `docs/reports/G-XX-VYSLEDEK.md`, aktualizovaným `docs/plan/ETAPY-A-STAV.md`, `CHANGELOG.md` a testovacími důkazy v aktivní matici.
5. Protržení hranic architektury či datových modelů vyžaduje české ADR.

## Kritické technické hranice

- `Boym323/AirRadar`, jeho LXC, historie, PostgreSQL, porty, CI a konfigurace jsou **nedotknutelné**.
- V2 má vlastní aplikační LXC a **PostgreSQL v již existujícím separátním databázovém LXC**. Nové V2 databáze, role a SQLx migrace patří až do G-04; není povoleno měnit jinou DB, používat SQLite ani migrovat historii V1.
- Živá letadla mají jediného vlastníka — Rust RAM; web přijímá pouze verzovaná HTTP/SSE DTO. Fronty a retry jsou ohraničené.
- Radar přijímá místní readsb/BEAST. Web nikdy nepotřebuje volit zdroj dat.
- DB, počasí ani notifikační doručení nesmějí blokovat živý příjem. Ve vývojových/replay/shadow testech nikdy neposílej reálné upozornění.
- Ochrana soukromí veřejných DTO, bezpečná práce s tajemstvími, přístupová ochrana mutací, zálohy a rollback jsou povinné i bez zvláštní produktové funkce.
- CI nyní provádí Rust fmt/Clippy/test, **Vite/React** lint/typecheck/test/build a `node tools/check-docs.mjs`. CI používá čisté npm ci, Vite build a kontrolu statického webu. Další kontrakt/E2E/load testy zaváděj podle etapy.
- Nevyžaduj ani nevymýšlej hesla a interní adresy; nevytvářej produkční deploy bez výslovného schválení.

## Průběžná evidence

Všechna čísla skutečného výkonu uváděj s metodikou a přesným SHA. Stav `TESTED`/`ACCEPTED` se udělí jen při doloženém testu. Po každé změně matice regeneruj český Markdown a spusť `node tools/check-docs.mjs`. Chybějící důkazy označ `NEOVĚŘENO`, nikoli `HOTOVO`.
