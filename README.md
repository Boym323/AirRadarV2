# AirRadar V2

Nová generace leteckého radaru, budovaná **od nuly**. Tento repozitář je zcela oddělený od [AirRadar V1](https://github.com/Boym323/AirRadar). Původní aplikace, produkční LXC a databáze se nemění.

**Aktuální stav 23. 10. 2026:** G-00, G-01 (10/10), G-02 (11/11) a **G-03 (10/10) jsou HOTOVO**. G-03 živě běží na izolovaném DEV LXC: HTTP/SSE snapshot/delta/15s heartbeat/reconnect, read-only BEAST a automatický `main → CI → immutable release → runner → LXC` deployment. Poslední provozně ověřená verze před uzavřením: [`29ee2ee7`](https://github.com/Boym323/AirRadarV2/commit/29ee2ee75feab9279123b77d50895d648c5419d7), [GitHub Actions #39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759) **6/6 PASS**. **G-04 je ROZPRACOVÁNO** (izolovaná SQL migrace; skutečná V2 DB a writer zatím nejsou aktivovány). Produkční V1 nedotčena.

<!-- AIRRADAR_STATS_START -->
## Statistiky vývoje (automaticky aktualizované)

![Stav CI](https://github.com/Boym323/AirRadarV2/actions/workflows/g02-dev-candidate.yml/badge.svg?branch=main)

Aktualizace statistik: **denně ve 03:17 (Europe/Prague)** a ručně na vyžádání.

| Ukazatel | Stav |
|---|---:|
| Řádky zdrojového kódu, včetně prázdných | **2 651** |
| Rust | 1 703 |
| TypeScript / JavaScript / CSS | 842 |
| Soubory zdrojového kódu | 25 |
| Testovací soubory (ne počet testů) | 3 |
| Dokumenty Markdown | 66 |
| Všechny sledované soubory | 127 |
| Dokončené etapy | **2/14** (14,3 %) |
| Ověřené KEEP funkce | **0/45** (0,0 %) |

![Historie řádků zdrojového kódu](docs/metrics/loc.svg)

Měřený commit: [f07fdb35](https://github.com/Boym323/AirRadarV2/commit/f07fdb35d64ab601e9675dc48b33aa4942b440bc) · [Historie](docs/metrics/history.json) · [Metodika](docs/metrics/README.md) · [Výsledky CI](https://github.com/Boym323/AirRadarV2/actions/workflows/g02-dev-candidate.yml).

> Jde o vývojové statistiky repozitáře, nikoli o výkon živého radaru. Funkce se počítají jen ze strojové matice s důkazem testu. Úspěšná CI sama o sobě nedokazuje dokončení funkcí.
<!-- AIRRADAR_STATS_END -->

> **Poznámka k metrikám:** automatický blok zachycuje **commit uvedený v položce „Měřený commit“**. Dokud se liší od aktuálního `main`, jeho čísla jsou historický snapshot, nikoli aktuální provozní stav. K uzavření G-03 dne 23. 10. platí **4/14 etap HOTOVO** a **9/45 funkcí TESTED**, viz [stav etap](docs/plan/ETAPY-A-STAV.md) a [aktivní matice](docs/plan/MATICE-FUNKCI.md). Denní workflow blok průběžně přepočítává; tuto poznámku ponech i po přepočtu.

## Kontinuální DEV nasazování z main

**Ověřeno na commitu `29ee2ee7`, run #39906245759 (SUCCESS):** registrovaný repo-scoped runner v DEV LXC přebírá pouze SHA-ověřené prerelease
z úspěšného GitHub CI `main` a kontroluje Rust/web s rollbackem. Post-deploy
healthcheck probíhá přímo na runneru, bez čekání na veřejný G-02 report. Viz
[provozní runbook](docs/runbooks/DEV-MAIN-AUTODEPLOY.md). Starý polling timer je
vypnutý; G-02 zůstává chráněno sdíleným lockem a jeho přejímky jsou oddělené.

## Česká zadání a vývoj

| Soubor | Účel |
|---|---|
| [Závazné zadání](docs/plan/ZADANI-AIRRADAR-V2.md) | Architektura místního radaru a provozní invarianty |
| **[Technologický průzkum](docs/plan/TECHNOLOGICKY-PRUZKUM.md)** | Ověřené zdroje a odpovědi na výběr jazyků, webu, DB, streamu a LXC |
| [Cílová architektura](docs/plan/ARCHITEKTURA-V2.md) | Doporučený Rust engine + lehký statický web a DB brána |
| [Technologická rozhodnutí](docs/plan/TECHNOLOGICKA-ROZHODNUTI.md) | Co zachovat, co doporučit změnit a co ještě vyžaduje benchmark |
| [Benchmarkový plán](docs/plan/BENCHMARKY-TECHNOLOGII.md) | Reprodukovatelná měření B01–B07; výsledky zatím nemáme |
| [Aktivní matice 45 funkcí](docs/plan/MATICE-FUNKCI.md) | Jediný zdroj požadavků k implementaci |
| [Katalog modulů](docs/plan/MODULY-A-ROZSAH.md) | Vlastnictví funkcí v 9 modulech |
| [Podrobné popisy](docs/plan/PODROBNY-KATALOG-FUNKCI.md) | Co každá aktivní funkce skutečně dělá |
| [Etapy a stav](docs/plan/ETAPY-A-STAV.md) | 14 navazujících etap G-00 až G-13 |
| **[140 konkrétních implementačních kroků](docs/plan/KROKOVY-IMPLEMENTACNI-PLAN.md)** | Detailní návaznosti, testovací brány a odkazy na G-01 až G-13 |
| [Začít v novém vlákně](docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md) | Kontext projektu a práce s Markdown dokumenty |
| [Pravidla pro Codex](AGENTS.md) | Čeština, izolace, testy a evidence |
| [CHANGELOG](CHANGELOG.md) | Změny a důkazy v historii projektu |
| [Archiv budoucích rozšíření](docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md) | Oddělená evidence případných nových funkcí |

## Aktuální architektura a navržené změny

- **Rust/Tokio/Axum:** readsb JSON a BEAST TCP, vlastní RAM letadel, bezpečné REST API a SSE.
- **Vite/React:** nasazený statický webový základ; **MapLibre, seznam, stopa a follow jsou teprve plán G-05**.
- **PostgreSQL/SQLx:** samostatná V2 databáze na existujícím odděleném DB LXC; historie V1 se nepřenáší.
- **Jedno místní pokrytí:** uživatel pracuje vždy s daty místního přijímače, bez přepínače datových zdrojů.
- **45 schválených funkcí:** 9 KEEP funkcí má důkaz `TESTED` (6 v G-02 a `J06/J07/J08` v G-03); etapová DEV přejímka G-03 je `ACCEPTED`, nikoliv plošná produktová akceptace všech 45 funkcí.

**Zvoleno a implementováno:** Vite + React/TypeScript pro webový základ a Rust HTTP/SSE nad RAM. **PostgreSQL/SQLx** je cílová technologie; G-04 už obsahuje izolovanou SQL migraci v repozitáři, avšak reálná V2 DB na odděleném LXC ani runtime writer nejsou aktivovány. Viz [ADR 0005](docs/adr/0005-vite-a-postgresql-v2.md).

**DEV engine běží s `--g03-live-api` a skutečným read-only BEAST zdrojem**, vrací `stage=G-03`, `status=live` při aktuálních pozorováních a vysílá SSE. Výchozí samostatné `cargo run` bez tohoto přepínače nadále záměrně spouští historický G-00 bootstrap `NOT_IMPLEMENTED`. Databázový writer G-04 a produkční vydání nejsou aktivní.

## Lokální vývoj

Vyžaduje Rust stable, **Node.js 26.12.0 a npm 11.20.0**. Soubor `.node-version` sjednocuje lokální vývoj a GitHub CI. Nepoužívej produkční checkout AirRadar V1.

~~~bash
cargo run --locked -p airradarv2-engine
# http://127.0.0.1:4101/api/v1/health

cd apps/web
npm ci
npm run dev
# http://127.0.0.1:4100
~~~

## Kontroly

~~~bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
node tools/check-docs.mjs
cd apps/web
npm ci
npm run lint
npm run typecheck
npm test
npm run build
~~~

Pro vývoj je schváleno **45 aktivních funkcí**; 9 z nich má důkaz `TESTED`. G-03 je skutečně nasazeno jen v **DEV**, nikoli do produkční V1. Závislosti jsou připnuté v `Cargo.lock` a `apps/web/package-lock.json`. [CI běh 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202) ověřil na commitu `99df0456` Rust, Vite a HTTP smoke testy; podrobný [report](docs/reports/G-00-BOOTSTRAP.md). Pozdější změny dokumentace a českého úvodu se ověřují samostatným CI.

**Není nakonfigurováno žádné produkční nasazení V2.** Izolované DEV LXC je funkční a automaticky aktualizované; V1 se nemění.

## PostgreSQL a statický provoz

PostgreSQL v existujícím separátním LXC potřebuje vlastní databázi/role a bezpečně omezený síťový přístup: [postup](database/POSTGRES-LXC.md). Žádná DB nebyla v této etapě změněna. Vite produkuje `apps/web/dist/`, který bude v produkci poskytovat reverse proxy bez Node serveru; `npm run preview` je pouze kontrola buildu.

## Konkretizace technologií před implementací (22. 10. 2026)

[**Přesné technické kontrakty pro 6 oblastí**](docs/plan/TECHNICKE-KONTRAKTY-A-LIMITY.md) sjednocují PostgreSQL schéma/indexy/retenci/zálohy, readsb/BEAST a deduplikaci, SSE a limity paměti, výkonnost MapLibre, Playwright testy a bezpečné spojení LXC. Český [ADR 0006](docs/adr/0006-kontrakty-dat-streamu-mapy-a-lxc.md) je výchozím rozhodnutím. Parametry jsou **návrhem k měření, nikoli implementovaným produkčním řešením**.

## Optimalizace CPU, RAM a diskových zápisů — 22. 10. 2026

**Nově zvolená politika:** [deduplikace už v Rust RAM](docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md), adaptivní uložení běžných pohybujících se letadel **nejdříve po 60 s**, výjimka významný manévr nejdříve po 30 s, 30s UTC SQL deduplikační bucket, nejvýše 128 řádků v batch či 5s flush po uzavření bucketu. Živá mapa a 1Hz SSE čtou RAM bez DB dotazu na každý event.

[Revidovaný fyzický model PostgreSQL](database/POSTGRES-DATOVY-MODEL.md) nezařazuje další indexy ani partitioning bez reálného měření. [ADR 0007](docs/adr/0007-ram-first-deduplikace-a-usporny-zapis.md) vysvětluje změnu proti pevné 15s historii. **Jde o implementační návrh, nikoli již dosaženou úsporu diskových IOPS.**

## Architektonický audit před implementací

[**Audit 22. 10. 2026 — uzavřená rozhodnutí a zbývající rizika**](docs/audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md) potvrzuje stabilní stack Rust/Vite/PostgreSQL, ale dokumentuje otevřené kontrakty časových značek, integrity pozorování, SSE atomického snapshotu, logbooku, notifikačních oprávnění a skutečné DB infrastruktury. **G-01 lze zahájit; jeho převzetí vyžaduje tři blokující kontraktové testy.** Zelené G-00 CI není potvrzení implementace 45 funkcí.

## Historie: dokončení G-01 — datové kontrakty

Dne 22. 10. 2026 byla dokončena **G-01**: [report](docs/reports/G-01-VYSLEDEK.md), [CI 39748325104](https://github.com/Boym323/AirRadarV2/actions/runs/39748325104) (Rust, web i nezávislý OpenAPI/JSON Schema validátor PASS). Následuje G-02 ingest. Kontrakty jsou hotové, **živý radar, PostgreSQL připojení ani produkční release nejsou**.

## Historie: samostatný DEV LXC — první ruční deploy (22. 10. 2026)

Po G-01 je ověřen samostatný Debian 13.7 LXC (1 vCPU, 8 GiB RAM, 20 GiB disk). [První nasazení krok za krokem](docs/runbooks/DEV-LXC-PRVNI-NASAZENI.md) používá **ruční GitHub Actions artefakt**, SHA-256 a oddělený systemd + Nginx na privátní IPv4. **Žádný auto-deploy ani spojení s V1, DB či readsb.**

## Historie: první instalovaný G-00/G-01 DEV základ

Správce dne 22. 10. 2026 doložil první úspěšné **ruční DEV LXC nasazení** commitu `227a8619b590d2c536761ffefc75cd0a5633936c`. Integrita balíčku OK, systemd Rust/Nginx active, health `bootstrap`, API letadel `NOT_IMPLEMENTED`; [přejímací report](docs/reports/DEV-LXC-PRVNI-NASAZENI-2026-10-22.md). Jde výhradně o G-00/G-01 základ, ne živý radar ani produkční release.

## Historie: první testovaná implementace G-02

[Průběžný report](docs/reports/G-02-VYSLEDEK.md) a [CI 39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657): lokální JSON/BEAST, DF17/CPR, RAM a bounded supervisor, offline syntetický replay; **G-02 je ROZPRACOVÁNO**, zbývají metadata, širší dekódování a výkonnostní prověrky. Záměrně **není nasazeno do DEV LXC**, G-00/G-01 verze zůstává funkční.

## Historie: rozšířená G-02 implementace (22. 10. 2026)

[Historický mezistav — CI #39769323592](https://github.com/Boym323/AirRadarV2/actions/runs/39769323592): 26 Rust testů, 3/3 joby PASS. DF17/CPR, letová telemetrie readsb/TC19, jeden RAM stav, JSON+BEAST replay, 5 000 cílů syntetický 1×/2×/4× benchmark. V matici je šest M01 funkcí `TESTED`. **G-02 = 10/11 kroků, zbývá povinná G-02.11 přejímka přímo na DEV LXC.** G-02 stále není nasazena do DEV a veřejný G-00/G-01 web není živý radar. [Report](docs/reports/G-02-VYSLEDEK.md).
