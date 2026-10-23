# AGENTS.md — závazná pravidla pro AirRadar V2

**Veškerá nová zadání, Markdown dokumentace, reporty, ADR a výchozí UI musí být v češtině (`cs-CZ`).** Technické identifikátory API, SQL a kódu zůstávají běžně anglicky.

## Před každou změnou

1. Ověř aktuální `main`, pracovní strom, `AGENTS.md`, [začátek pro nové vlákno](docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md), [hlavní zadání](docs/plan/ZADANI-AIRRADAR-V2.md), [technologická rozhodnutí](docs/plan/TECHNOLOGICKA-ROZHODNUTI.md) [krokový plán](docs/plan/KROKOVY-IMPLEMENTACNI-PLAN.md) a relevantní jeden úkol v `docs/ukoly/`. Před migrací frontend/DB prostuduj také [průzkum](docs/plan/TECHNOLOGICKY-PRUZKUM.md) a [benchmarkový plán](docs/plan/BENCHMARKY-TECHNOLOGII.md).
2. `contracts/feature-matrix.json` je **pouze seznam aktivních 45 funkcí**. Každá má vlastníka v [MODULY-A-ROZSAH.md](docs/plan/MODULY-A-ROZSAH.md). Rozšíření nemíchej do tohoto seznamu ani do kódu; jejich jediným záznamem je [oddělený archiv](docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md), který se otevírá pouze na výslovnou žádost.
3. Při přidávání souboru, modulu, tabulky, závislosti, datového zdroje nebo UI prvku uveď konkrétní krok `G-XX.YY`, aktivní ID, konzumenta a test. Checkbox `[x]` je pravdivý pouze s důkazem implementace. Nemá-li přínos nebo přidává zbytečné komplikace, neimplementuj jej.
4. Každou etapu uzavírej českým `docs/reports/G-XX-VYSLEDEK.md`, aktualizovaným `docs/plan/ETAPY-A-STAV.md`, `CHANGELOG.md` a testovacími důkazy v aktivní matici.
5. Protržení hranic architektury či datových modelů vyžaduje české ADR.

## Kritické technické hranice

- `Boym323/AirRadar`, jeho LXC, historie, PostgreSQL, porty, CI a konfigurace jsou **nedotknutelné**.
- V2 má vlastní aplikační LXC a **PostgreSQL v již existujícím separátním databázovém LXC**. Nové V2 databáze, role a SQLx migrace patří až do G-04; není povoleno měnit jinou DB, používat SQLite ani migrovat historii V1.
- Živá letadla mají jediného vlastníka — Rust RAM; web přijímá pouze verzovaná HTTP/SSE DTO. Fronty a retry jsou ohraničené.
- Radar přijímá místní readsb/BEAST. Web nikdy nepotřebuje volit zdroj dat.
- DB, počasí ani notifikační doručení nesmějí blokovat živý příjem. Ve vývojových/replay/shadow testech nikdy neposílej reálné upozornění.
- Ochrana soukromí veřejných DTO, bezpečná práce s tajemstvími, přístupová ochrana mutací a bezpečný rollback jsou povinné i bez zvláštní produktové funkce. Zálohy se řídí hodnotou dat: **výhradně postradatelná historie AirRadar V2 G-04 má výjimku [ADR 0010](docs/adr/0010-g04-postradatelna-historie-bez-zaloh.md)**. Zálohování PP Studio, V1 a dalších databází se nemění.
- CI nyní provádí Rust fmt/Clippy/test, **Vite/React** lint/typecheck/test/build a `node tools/check-docs.mjs`. CI používá čisté npm ci, Vite build a kontrolu statického webu. Další kontrakt/E2E/load testy zaváděj podle etapy.
- Nevyžaduj ani nevymýšlej hesla a interní adresy; nevytvářej produkční deploy bez výslovného schválení.

## Průběžná evidence

Všechna čísla skutečného výkonu uváděj s metodikou a přesným SHA. Stav `TESTED`/`ACCEPTED` se udělí jen při doloženém testu. Po každé změně matice regeneruj český Markdown a spusť `node tools/check-docs.mjs`. Chybějící důkazy označ `NEOVĚŘENO`, nikoli `HOTOVO`.

## Technické kontrakty

Při každé práci na G-01 až G-11 zkontroluj [závazný rozcestník](docs/plan/TECHNICKE-KONTRAKTY-A-LIMITY.md) a příslušnou z šesti technických specifikací. Výchozí sampling, retenci, SSE limity, MapLibre render a PostgreSQL bezpečnostní režim/backup neměň bez konkrétního měření a českého ADR. Pro G-04 se uplatní výslovná výjimka [ADR 0009](docs/adr/0009-g04-postgresql-lan-bez-tls.md): důvěryhodná LAN bez TLS a heslové ověření podle stávajícího přístupu serveru, **bez nového `hostnossl` a bez úprav `pg_hba.conf`/firewallu**. Pokud existující autorizace spojení neumožní, G-04 je BLOKOVÁNO. Tyto návrhové hodnoty ještě nejsou naměřené ani implementované.

## Úspornost provozu jako závazné pravidlo

Před G-01/G-02/G-04/G-09 přečti [RAM-first deduplikaci a adaptivní sampler](docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md), [revidovaný DB model](database/POSTGRES-DATOVY-MODEL.md) a [ADR 0007](docs/adr/0007-ram-first-deduplikace-a-usporny-zapis.md). Neimplementuj pevný 15s INSERT, per-aircraft SQL query, nekonečné SQL fronty, cache server navíc ani druhý SQL index/partition bez měření. Testovací model `node tools/check-eco-sampling.mjs` není důkaz reálných IOPS; uživatelskou kvalitu historie a bezpečnou obnovu musí doložit G-04.

## Identita původu lokálního přijímače

Při G-01/G-02 dodrž [ADR 0008](docs/adr/0008-identita-puvodu-lokalniho-prijmu.md): `receiverId` není `transport` a ani jedno není `aircraftKey`. Dvě cesty JSON/BEAST téhož readsb tvoří **jednu RF stanici**; nezdvojovat letadla, statistiky ani SQL vzorky. Tento minimální kontrakt **nepovoluje** žádné další datové kanály či UI; případné návrhy jsou izolovány výhradně v jediném archivu budoucích rozšíření.

## Architektonické blokace před přejímkou G-01

Před G-01 si přečti [audit 22. 10. 2026](docs/audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md). Uzavři P0-01 až P0-03 přímo v G-01: skutečný čas/původ polohy (`readsb`/BEAST), konfliktní identitu a atomický, verzovaný SSE snapshot+delta. Ostatní P1 body patří před přejímku cílových G-XX etap. Audit neznamená schválení dalších zdrojů nebo funkcí a není náhradou za testy.
