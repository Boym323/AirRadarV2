# CHANGELOG — AirRadar V2

Sledujeme **skutečné změny kódu, dokumentace a provozních kontraktů**; všechny zápisy jsou česky. Jednotlivé implementační etapy mají vlastní reporty a přesné odkazy na CI.

## [Nepublikováno]

### Přidáno
- Detailní český krokový implementační plán `docs/plan/KROKOVY-IMPLEMENTACNI-PLAN.md`: **140 konkrétních kroků** v G-01 až G-13 s návaznostmi, definicí výstupu a odpovídajícími testy.
- Všech 13 etapových MD zadání rozšířeno o zaškrtávací kroky `G-XX.YY`, bezpečnostní hranice a pravidla evidence PASS/FAIL; žádná z 45 funkcí není touto dokumentací označena jako hotová.
- Automatická kontrola úplnosti a jednoznačnosti kroku, evidence vlastnictví 45 ID a rozcestník pro nová vlákna.

### Změněno
- Webový základ G-00 převeden na **Vite + React + TypeScript** se statickým `dist/`, českým UI a HTTP smoke.
- Odstraněn Next.js runtime a závislosti; CI používá uzamčené `npm ci`.
- PostgreSQL definitivně zvolena na existujícím separátním DB LXC; nové V2 připojení a migrace až G-04.
- Žádný produkční deploy ani změna LXC/DB nebyly provedeny.

### Přidáno
- Důkladná česká rešerše technologií a architektury: `TECHNOLOGICKY-PRUZKUM.md`, `ARCHITEKTURA-V2.md`, `BENCHMARKY-TECHNOLOGII.md` a `TECHNOLOGICKA-ROZHODNUTI.md`.
- Rozhodovací brány a reprodukovatelné benchmarky Rust/Go/Node, Next.js/Vite, SQLite/PostgreSQL, BEAST/JSON, SSE a MapLibre.
- Architektonické doporučení Vite/React místo produkčního Next.js serveru a podmíněný kandidát SQLite WAL; skutečný výběr DB a výkonové výsledky zůstávají neověřené.

### Změněno
- Vývojová platforma modernizována na Node.js 26.12.0/npm 11, Next.js 16.4.2, React 19.3.1, TypeScript 6 (poslední podporovaná řada), ESLint 10 a Vitest 5. Regenerace lockfilu a CI neprovádí nasazení.
- Produktový plán je nyní **místní radar s 45 aktivními funkcemi**. Rozšíření jsou přesunuta do jediného odděleného souboru pro budoucí posouzení; aktivní zdrojová matice neobsahuje nevybrané požadavky.
- Aktualizováno rozdělení devíti logických modulů, navazující české implementační úkoly a souvislé číslování etap G-00 až G-13.
- Zesílena kontrola synchronizace matice, české dokumentace a vývojových statistik bez pevně zakódovaného historického rozsahu.
- Základní přijímací API ani UI implementace se tímto rozhodnutím nestaly živým radarem.

### Ověření
- Modernizace runtime ověřena [CI 39673418390](https://github.com/Boym323/AirRadarV2/actions/runs/39673418390), commit `d59f98d4`: Node 26, web lint/types/test/build/smoke, Rust testy a česká dokumentace prošly.
- Stav tohoto dokumentačního čištění musí potvrdit CI na jeho finálním SHA. Žádné produkční nasazení se neprovádí.

## 2026-10-21 — Založení nezávislého AirRadar V2

### Přidáno
- Samostatný GitHub repozitář a počáteční Rust/Tokio/Axum HTTP scaffold s neimplementovaným seznamem letadel.
- Next.js vývojová stránka s češtinou jako výchozím jazykem.
- Připnuté závislosti v Cargo/npm lockfile, dokumentace, OpenAPI, počáteční matice požadavků a CI bez deploye.
- Automatizované statistiky vývoje zdrojového kódu s historickými snapshoty, které se zpětně nepřepisují.

### Ověřeno
- [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202): Rust + web unit/build a HTTP smoke při počátečním ověření.
- [GitHub Actions 39662365349](https://github.com/Boym323/AirRadarV2/actions/runs/39662365349): Rust, web a česká dokumentace.
- Živá data, databáze, LXC a produkční deployment tímto nebyly zřízeny.

## Pravidlo zápisu

Každá skutečná implementační změna vyžaduje položku v části Nepublikováno, český etapový report a zdrojové důkazy. Historická označení verzí vzniknou teprve po skutečném vydání. Tento soubor není náhradou za automatizované testy.
