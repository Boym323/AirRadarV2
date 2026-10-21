# AirRadar V2

Nová generace leteckého radaru, budovaná **od nuly**. Tento repozitář je zcela oddělený od [AirRadar V1](https://github.com/Boym323/AirRadar). Původní aplikace, produkční LXC a databáze se nemění.

**Stav:** dokončen a ověřen základ G-00; **nejde ještě o živý radar**. G-01 a další etapy zatím nejsou realizovány.

<!-- AIRRADAR_STATS_START -->
## Statistiky vývoje (automaticky aktualizované)

![Stav CI](https://github.com/Boym323/AirRadarV2/actions/workflows/ci.yml/badge.svg?branch=main)

Aktualizace statistik: **denně ve 03:17 (Europe/Prague)** a ručně na vyžádání.

| Ukazatel | Stav |
|---|---:|
| Řádky zdrojového kódu, včetně prázdných | **496** |
| Rust | 149 |
| TypeScript / JavaScript / CSS | 347 |
| Soubory zdrojového kódu | 12 |
| Testovací soubory (ne počet testů) | 1 |
| Dokumenty Markdown | 21 |
| Všechny sledované soubory | 48 |
| Dokončené etapy | **1/16** (6,3 %) |
| Ověřené KEEP funkce | **0/45** (0,0 %) |

![Historie řádků zdrojového kódu](docs/metrics/loc.svg)

Měřený commit: [8b3c6d17](https://github.com/Boym323/AirRadarV2/commit/8b3c6d17726ce3898fe220dfbfc421c04b4bec44) · [Historie](docs/metrics/history.json) · [Metodika](docs/metrics/README.md) · [Výsledky CI](https://github.com/Boym323/AirRadarV2/actions/workflows/ci.yml).

> Jde o vývojové statistiky repozitáře, nikoli o výkon živého radaru. Funkce se počítají jen ze strojové matice s důkazem testu. Úspěšná CI sama o sobě nedokazuje dokončení funkcí.
<!-- AIRRADAR_STATS_END -->

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
| [Začít v novém vlákně](docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md) | Kontext projektu a práce s Markdown dokumenty |
| [Pravidla pro Codex](AGENTS.md) | Čeština, izolace, testy a evidence |
| [CHANGELOG](CHANGELOG.md) | Změny a důkazy v historii projektu |
| [Archiv budoucích rozšíření](docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md) | Oddělená evidence případných nových funkcí |

## Aktuální architektura a navržené změny

- **Rust/Tokio/Axum:** readsb JSON a BEAST TCP, vlastní RAM letadel, bezpečné REST API a SSE.
- **Vite/React/MapLibre:** lehká česká mapa, seznam letadel, popisky, stopa a follow.
- **PostgreSQL/SQLx:** samostatná V2 databáze na existujícím odděleném DB LXC; historie V1 se nepřenáší.
- **Jedno místní pokrytí:** uživatel pracuje vždy s daty místního přijímače, bez přepínače datových zdrojů.
- **45 schválených funkcí:** žádná z nich zatím není potvrzena jako implementovaná. G-00 je pouze základ projektu.

**Zvoleno a implementováno:** Vite + React/TypeScript pro webový skeleton. **PostgreSQL/SQLx** na existujícím samostatném DB LXC jako definitivní databáze; V2 DB ani migrace ještě nevznikly. Viz [ADR 0005](docs/adr/0005-vite-a-postgresql-v2.md).

API nyní vrací zkušební `NOT_IMPLEMENTED`. Nejsou zatím připojena živá ADS-B data, databáze ani veřejný release.

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

Pro vývoj je nyní schváleno **45 aktivních funkcí**. Žádný produkční modul ještě není implementován. Závislosti jsou připnuté v `Cargo.lock` a `apps/web/package-lock.json`. [CI běh 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202) ověřil na commitu `99df0456` Rust, Vite a HTTP smoke testy; podrobný [report](docs/reports/G-00-BOOTSTRAP.md). Pozdější změny dokumentace a českého úvodu se ověřují samostatným CI.

**Není nakonfigurováno žádné produkční nasazení.** Samostatný LXC zajistí vlastník projektu; V1 zůstává dostupná pro srovnávání.

## PostgreSQL a statický provoz

PostgreSQL v existujícím separátním LXC potřebuje vlastní databázi/role a bezpečně omezený síťový přístup: [postup](database/POSTGRES-LXC.md). Žádná DB nebyla v této etapě změněna. Vite produkuje `apps/web/dist/`, který bude v produkci poskytovat reverse proxy bez Node serveru; `npm run preview` je pouze kontrola buildu.
