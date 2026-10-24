# AirRadar V2 — závazné zadání projektu

**Aktuální schválený rozsah:** 45 funkcí uvedených v [aktivní matici](../../contracts/feature-matrix.json).  
**Projektový jazyk:** čeština `cs-CZ` v dokumentaci a výchozím UI, volitelně anglický překlad webu.  
**Nový repozitář:** https://github.com/Boym323/AirRadarV2, referenční V1: https://github.com/Boym323/AirRadar.  
**Stav 23. 10. 2026:** G-00, G-01 10/10, G-02 11/11 a **G-03 10/10 HOTOVO**; **DEV G-03 HTTP/SSE + automatický `main → LXC` deployment ACCEPTED** (Actions [#39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759), nasazené `29ee2ee7`). G-04 je ROZPRACOVÁNO: skutečný DEV audit a izolované provozní testy jsou doložené, ale J09/produkční G-13 zůstávají NEOVĚŘENO.

## 1. Cíl

Postavit novou, jednoduchou a výkonnou aplikaci pro **místní letecký radar** z vlastních readsb JSON a BEAST TCP dat. Nejde o mechanický přepis V1. Projekt má vlastní GitHub repozitář, CI i funkční samostatné vývojové aplikační LXC; v budoucnu se připojí izolovaná V2 databáze na existujícím DB LXC. V1, její historie, síťová konfigurace a služby zůstávají beze změny.

**Definitivní architektura:** Rust/Tokio/Axum je vlastníkem živého RAM stavu, **Vite + React/TypeScript + MapLibre** je statický web, **PostgreSQL + SQLx** je jediná databázová technologie. PostgreSQL běží v existujícím samostatném DB LXC; DEV databáze `airradar_v2_dev` a tabulka `flight_samples` jsou auditně ověřené a po G-04/J09 DEV ACCEPTED je writer aktivován stage-aware DEV runtime. Produkční V2 DB/release zůstávají oddělené. Webový G-00 skeleton už byl převeden z Next.js na Vite. Viz [ADR 0005](../adr/0005-vite-a-postgresql-v2.md), [ADR 0014](../adr/0014-g04-trvaly-dev-runtime-a-stage-aware-autodeploy.md) a [runbook DB](../../database/POSTGRES-LXC.md).

**Primární tok:** `readsb JSON nebo BEAST TCP → Rust parser a normalizace → RAM → HTTP/SSE → MapLibre`. Druhý nezávislý ingest ve webu se nesmí vytvořit. `RAM → bounded sampling queue → samostatná PostgreSQL` nikdy nesmí blokovat živý přenos.

## 2. Produktové moduly

Závazný rozpis devíti logických modulů je v [MODULY-A-ROZSAH.md](MODULY-A-ROZSAH.md). Každá funkce má přidělené ID, jednu primární oblast a vlastní akceptační test. Nové funkcionality se přidávají pouze po výslovném schválení a aktualizaci matice. Návrhy do budoucna jsou mimo aktivní plán v [samostatném archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

Základní produkt tvoří: příjem readsb/BEAST, korektní identita a stav letadel, verzované API se SSE snapshot/delta, úsporná historie, základní mapa s ikonami/seznamem/stopou/follow, letiště a navigační body, vybraná meteorologická data, detekce nouzových squawk, volitelné push kanály, základní statistiky/diagnostika, PWA a lokalizace.

**Schválené UX rozhodnutí (24. 10. 2026):** AirRadar V2 je **radar primárně orientovaný na mapu**. Výchozí pracovní plochou webu je živá MapLibre mapa; seznam letadel, detaily, letiště, meteorologie, notifikace, statistiky a diagnostika mají kontextové/sekundární ovládání. Podrobnosti a otevřené designové volby stanoví [map-first UX/UI specifikace](WEB-UX-UI-SPECIFIKACE.md). Toto rozhodnutí nemění rozsah 45 funkcí ani stav implementace.

**Konkretizace vizuálního zadání (24. 10.):** [ikonografie letadel a popisky](MAPOVE-IKONY-A-POPISKY.md), [úplný UX návrh](WEB-UX-UI-SPECIFIKACE.md) a [audit pokrytí](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md). Typová klasifikace je zatím **specifikace**, nikoli implementované datové pole nebo hotové assety.

## 3. Správnost příjmu a API

- BEAST parser musí správně zvládat escape, rámcování, DF/CRC24, krátké/ztracené rámce a časová razítka.
- Stabilní klíč letadla a jeho provenance z místního readsb, validované polohy, callsign jako měnící se vlastnost a omezené TTL bez fantomových cílů.
- Auto fallback JSON/BEAST se stabilní identitou, řízeným reconnect/backoff a bez nekonečných bufferů.
- REST a SSE jsou verzované, poskytují sanitizovaná veřejná DTO, initial snapshot/delta, heartbeat a kontrolovanou kapacitu klientů. Neprozrazují interní údaje ani soukromou polohu přijímače.
- Web zobrazuje přímo místní data. Nepotřebuje volbu zdroje a nesmí ukazovat fiktivní provoz. Neimplementované služby jsou jasně označené.

## 4. Výkon a databáze

- Nové izolované V2 úložiště začne prázdné, stará historická data se nepřenášejí. **G-04 použije PostgreSQL na existujícím odděleném DB LXC a musí prokázat bezpečné role, existující přístup v LAN bez TLS a bez změn `pg_hba.conf` dle [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md), indexy, retenci a obnovu**; bez přímého uživatele dat nevytvářet tabulku, index ani pozadí worker.
- SQLx migrace, bezpečná oprávnění, bounded zápisová fronta, vzorkování a dávkování, retence podle skutečných aktivních konzumentů.
- Ověřovat `EXPLAIN (ANALYZE, BUFFERS)`, latence P95/P99, CPU/RSS, WAL a velikost dat; nepředstírat porovnání bez baseline.
- Výpadek DB, meteorologického zdroje nebo UI klienta nesmí pozastavit vlastní příjem.
- Rust runtime a statický Vite web mají oddělené provozní hranice; dlouhodobý Node server neběží a diagnostika musí zobrazovat jen skutečné procesy.

## 5. Mapa, počasí a bezpečnost

- Účelné mapové vrstvy a prvky používané živým lokálním radarem: přehledný seznam, ikony/směry, čitelné popisky, skutečné stopy a přibližování.
- Publikované letištní a navigační podklady zobrazovat jen se známým původem a licencí, s jednoznačným rozlišením skutečnosti a neznámé hodnoty.
- ČHMÚ, platná výstraha a dostupná palubní meteorologie s ohraničenými cache/TTL; extrakce BDS dat nesmí přetěžovat parser.
- Kódy squawk 7500, 7600, 7700 se detekují pouze z validních přijatých dat. Skutečné doručování notifikací ve vývoji a shadow režimu musí být výchozím nastavením zakázané.
- Administrativní interní operace vyžadují odpovídající ochranu i bez samostatného administračního UI.

## 6. Testy, dokumentace a instalace

- Výchozí jazyk každého MD, zadání, ADR, reportu a viditelného UI je čeština. Klíče API, technické názvy a stabilní identifikátory zůstávají anglické.
- Povinné GitHub CI: Rust fmt/Clippy/testy a Vite/React lint/typecheck/test/build, kontrola českých MD a vazeb na aktivní rozsah.
- Syntetický replay, kontraktové testy, UI E2E a odpovídající výkonnostní testy se tvoří dle reálných etap. Bez doložených výsledků funkce nemůže mít status `ACCEPTED`.
- V2 má vlastní aplikační LXC, porty a účet. Samostatná databáze bude v existujícím PostgreSQL LXC s izolovaným přístupem, zálohami a rollbackem. Žádné produkční přesměrování či změna V1 bez výslovného souhlasu.

## 7. Etapy a navázání nového vlákna

Realizace probíhá po jednotlivých českých zadáních [G-00 až G-13](ETAPY-A-STAV.md) a [140 konkrétních krocích G-XX.YY](KROKOVY-IMPLEMENTACNI-PLAN.md). Při každé etapě se aktualizuje `docs/reports/`, `CHANGELOG.md`, strojová matice, generovaný Markdown a měření. Ve vlákně začni čtením `AGENTS.md` a [ZAČNI ZDE](ZACNI-ZDE-NOVE-VLAKNO.md), nikoliv kopírováním starého projektu.

## Technické limity a navazující průzkum

Výchozí datový model, retence, návrhové parametry SSE, readsb/BEAST, mapy, Playwright a spojení LXC jsou v [šesti technických kontraktech](TECHNICKE-KONTRAKTY-A-LIMITY.md) a v [ADR 0006](../adr/0006-kontrakty-dat-streamu-mapy-a-lxc.md). Zůstává nutné jejich správnost ověřit v G-01 až G-11; samostatné DB LXC se touto rešerší neměnilo.

## RAM-first a minimalizace opotřebení SSD

[ADR 0007](../adr/0007-ram-first-deduplikace-a-usporny-zapis.md) a [detailní RAM-first návrh](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md) nahrazují pevnou 15s historii: Rust drží jediný hot state/trail/cache; SQLx ukládá jen významně novou validní polohu po adaptivní minimálně 60s politice (výjimka velkých manévrů), dávkově a přes bounded buffer. Nebude přidáváno diskové cache, sekundární indexy, partitions ani další DB služby bez měřeného opodstatnění. Úspora se musí doložit replay, CPU/RSS, SQL rows/s, WAL a IOPS. Databázová dostupnost nesmí ovlivnit živý radar.
