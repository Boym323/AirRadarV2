# AirRadar V2

Nová generace leteckého radaru, budovaná **od nuly**. Tento repozitář je zcela oddělený od [AirRadar V1](https://github.com/Boym323/AirRadar). Původní aplikace, produkční LXC a databáze se nemění.

**Stav:** dokončen a ověřen základ G-00; **nejde ještě o živý radar**. G-01 a další etapy zatím nejsou realizovány.

## Dokumentace a zadání — vše česky

| Dokument | Účel |
|---|---|
| **[Hlavní zadání AirRadar V2](docs/plan/ZADANI-AIRRADAR-V2.md)** | Úplné zadání, cíle, architektura, databáze, provoz a kritéria úspěchu |
| [Plán etap a jejich stav](docs/plan/ETAPY-A-STAV.md) | Pořadí G-00 až G-15, závislosti a stav realizace |
| [Matice všech funkcí](docs/plan/MATICE-FUNKCI.md) | 79 požadovaných funkcí a 99 vyřazených |
| **[Podrobný katalog 178 funkcí](docs/plan/PODROBNY-KATALOG-FUNKCI.md)** | Vysvětlení, co každá funkce dělá; určeno k rozhodnutí KEEP/REMOVE |
| [Pravidla práce s dokumentací](docs/plan/PRAVIDLA-DOKUMENTACE.md) | Jazyk, systém zadání, reportů a ADR |
| [Zadání G-01](docs/ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) | Další samostatná implementační etapa |
| [Výsledky G-00](docs/reports/G-00-BOOTSTRAP.md) | Ověřené testy a hranice aktuální implementace |
| [Příprava nového LXC](docs/runbooks/LXC-SETUP.md) | Oddělené vývojové a budoucí provozní prostředí |
| [Pokyny pro Codex](AGENTS.md) | Závazné postupy, čeština a bezpečnost |

**Veškerá nová projektová dokumentace, zadání a výchozí uživatelské rozhraní jsou česky.** Identifikátory API, jazyky programování a názvy proměnných zůstávají standardní. Volitelná angličtina uživatelského rozhraní bude později součástí požadavku U04.

## Navržená architektura

- **Rust / Tokio / Axum:** samostatný engine pro budoucí ADS-B/BEAST příjem, jednotný stav letadel a SSE.
- **TypeScript / Next.js / React / MapLibre:** mapa a klientská část (živá mapa začíná až v G-06).
- **PostgreSQL + SQLx:** nová izolovaná databáze; ukládaná historie začne od nuly v G-05.
- **79 KEEP / 99 REMOVE:** plánovaný funkční rozsah, nikoliv počet aktuálně dokončených funkcí.

G-00 obsahuje pouze zkušební Rust API `/api/v1/health`, neimplementovaný výpis letadel a SSE se stavem `NOT_IMPLEMENTED`. Web je informační úvodní stránka. **Neexistuje zatím skutečný přijímač, živá mapa, aktivní databáze ani notifikace.**

## Lokální vývoj

Vyžaduje Rust stable, Node.js 22.18+ a npm 10+. Nepoužívej produkční checkout AirRadar V1.

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

Závislosti jsou připnuté v `Cargo.lock` a `apps/web/package-lock.json`. [CI běh 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202) ověřil na commitu `99df0456` Rust, Next.js a HTTP smoke testy; podrobný [report](docs/reports/G-00-BOOTSTRAP.md). Pozdější změny dokumentace a českého úvodu se ověřují samostatným CI.

**Není nakonfigurováno žádné produkční nasazení.** Samostatný LXC zajistí vlastník projektu; V1 zůstává dostupná pro srovnávání.

