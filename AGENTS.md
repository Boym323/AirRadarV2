# Pravidla vývoje AirRadar V2 pro Codex a další agenty

**Výchozí jazyk projektu: čeština (cs-CZ).** Nejde o přepis AirRadar V1; V2 vzniká na zelené louce v samostatném repozitáři `Boym323/AirRadarV2`.

## Povinné pořadí práce

1. Před změnami přečti [hlavní české zadání](docs/plan/ZADANI-AIRRADAR-V2.md), [stav etap](docs/plan/ETAPY-A-STAV.md), [pravidla dokumentace](docs/plan/PRAVIDLA-DOKUMENTACE.md) a pouze dokumentaci související s právě řešenou etapou.
2. Pro aktuální etapu vytvoř nebo uprav **Markdown soubor** `docs/ukoly/G-XX-NAZEV.md` s rozsahem, závislostmi, scénáři ověření a výsledkem. Nespojuj několik etap do jednoho rozsáhlého neověřitelného zásahu.
3. Při implementaci průběžně aktualizuj `docs/plan/ETAPY-A-STAV.md` a na závěr dolož skutečné testy, CI run, commit SHA, měření a blokéry v `docs/reports/G-XX-VYSLEDEK.md`. **Nikdy neoznač za hotové něco, co nebylo ověřeno.**
4. Pokud změna trvale ovlivní architekturu, jazyk, bezpečnost, databázový model nebo provoz, zapiš důvod a alternativy jako české ADR v `docs/adr/`.
5. Matice požadovaných funkcí je v `contracts/feature-matrix.json`; čitelný přehled je v `docs/plan/MATICE-FUNKCI.md`. Stav funkce aktualizuj až po skutečném testu a přidej odkaz na důkaz; Markdown matice musí zůstat synchronizovaná.

## Závazná jazyková pravidla

- **Veškeré nové a upravované zadání, dokumentace, ADR, návody, reporty, plán práce, pull-request popisy a uživatelská vysvětlení piš česky.**
- **Všechny nové viditelné texty aplikace a výchozí nastavení jazyka piš česky.** HTML má `lang="cs"`; časová a číselná prezentace podle `cs-CZ`.
- Komentáře vysvětlující obchodní logiku a věcné popisy testů piš přednostně česky. Zachovej anglické identifikátory kódu, názvy technologií, HTTP cesty, JSON klíče, API/OpenAPI `operationId`, SQL názvy, standardizované stavové kódy a příkazy.
- Funkce **U04** stále požaduje budoucí volitelnou češtinu **i angličtinu v aplikaci**. Výchozí lokalizace je `cs-CZ`; angličtina je až sekundární překlad s vlastními testy. Technické a projektové dokumenty i poté zůstávají česky.
- Vlastní Git commit zprávy udržuj stručné a přehledné; český popis je upřednostněn. Nepropisuj překlady do strojových kontraktů bez kompatibilitního testu.

## Kritické provozní a bezpečnostní hranice

- AirRadar V1 (`Boym323/AirRadar`) včetně jeho LXC, databáze, služeb, CI/CD, portů a historie **neměň ani nemaž**.
- V2 bude mít samostatný nový LXC a databázi; podmínky jsou v [návodu LXC](docs/runbooks/LXC-SETUP.md). Nesmí se použít V1 credentials, adresář služby ani deploy runner.
- **Jediným vlastníkem živého stavu letadel bude Rust engine.** Next.js nesmí implementovat nezávislý ingest ani zápisy historie.
- Kritická cesta zůstává `readsb/BEAST → Rust RAM → SSE/HTTP → web`. Výpadek PostgreSQL, počasí nebo metadat nesmí zastavit živá letadla.
- V2 začne bez staré letové historie; nové vzorkované záznamy se budou ukládat až v etapě G-05. Nezasahuj do historie V1.
- Cílový rozsah je **79 zachovat (KEEP)** a **99 nezařazovat (REMOVE)**, nikoli automatické kopírování starých modulů. Žádná z 79 funkcí nesmí být prohlášena za implementovanou bez ověření.
- Soukromí OGN, rozlišení LOCAL/EXTENDED, identita ICAO, časové známky, ochrana veřejných DTO a autentizace administrace jsou povinné.
- Etapa G-00 je **pouze funkční základ aplikace**, nikoliv živý radar. API vrací `NOT_IMPLEMENTED`; nezaměňuj jej za provozní data.
- Žádný produkční deploy, změna DNS/reverse proxy, konfigurace V1, skutečné odesílání notifikací ani práce s tajemstvími bez výslovného samostatného povolení.
- Volba jazyků: Rust/Tokio/Axum backend, TypeScript/Next.js/MapLibre web, PostgreSQL/SQLx data; další technologie pouze s naměřeným přínosem a českým ADR.

## Povinné kontroly

Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`; web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; [kontrola dokumentace](tools/check-docs.mjs) a později integrační/Playwright/performance testy dle etapy.

**Report používej v češtině se stavem HOTOVO / ČÁSTEČNĚ / BLOKOVÁNO**, skutečným SHA a odkazy na důkazy. Pokud výsledky nejsou k dispozici, napiš „neověřeno“, nevymýšlej měření.

