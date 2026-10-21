# Katalog implementačních modulů AirRadar V2

**Status: schválený cílový návrh, nikoliv existující implementace.** Všech 79 KEEP ID má přesně jeden primární odpovědný modul. Konkrétní modulární členění se může dále zjednodušit na základě měření a českého ADR.

## Co a proč budujeme

V2 se staví od nuly, ne kopírováním `lib/`, `components/`, `app/`, starého databázového schématu ani historických skriptů V1. Dva spustitelné procesy a samostatná DB jsou výchozí architektura; uvedené moduly jsou **logické oblasti, nikoliv nové mikroservisy**.

| Modul | Etapa | Vrstva | Počet KEEP | Rozsah |
|---|---|---|---:|---|
| [M01](#m01) – Lokální příjem a kanonický stav | G-02 | Rust / Tokio | 6 | readsb JSON/BEAST, dekódování Mode-S/CRC, deduplikace a jediný autoritativní RAM state owner |
| [M02](#m02) – Externí zdroje a provenance | G-03 | Rust / Tokio | 3 | ADSBHub a ADSB.lol raw/HTTP, omezené retry/timeout a přednost LOCAL údajů |
| [M03](#m03) – Veřejné API a SSE | G-04 | Rust / Axum | 3 | verzované veřejné DTO, snapshot+delta, sekvence, heartbeaty a bounded fanout |
| [M04](#m04) – Historie a datové úložiště | G-05 | PostgreSQL / SQLx / Rust | 3 | nová DB od nuly, vzorkované zápisy, profily a časové souhrny pro nové lety |
| [M05](#m05) – Mapa a interaktivní provoz | G-06 | Next.js / React / MapLibre | 13 | živá mapa, lokalita provenance, symboly, seznam, detail, trail, kamera, vrstva a panely |
| [M06](#m06) – Letecké katalogy a navigace | G-07 | Rust / SQL | 10 | letiště, eAIP, ATS/NAVAID, prostorová geometrie, ATC a Navigation Integrity |
| [M07](#m07) – Počasí a meteorologické vrstvy | G-08 | Rust providers / web MapLibre | 7 | METAR/TAF, ČHMÚ časová osa, SIGMET a Mode-S počasí s TTL |
| [M08](#m08) – OGN a doplňkové integrace | G-09 | Rust / SQL / izolované providers | 7 | OGN/DDB/SoftRF soukromí, RXW, FlightAware, ADSBDB a Planespotters on-demand |
| [M09](#m09) – Administrace a doručování upozornění | G-10 | Rust / SQL / web | 3 | Web Push/Pushover, transakční claim/retry a bezpečné přihlášení správce |
| [M10](#m10) – Statistiky a provozní telemetrie | G-11 | SQL / Rust / web | 16 | denní agregace, coverage, logbook, zdravotní stav, DB a CPU/RSS metriky, T58 |
| [M11](#m11) – Uživatelská platforma | G-12 | Next.js / PWA / i18n | 6 | responsive UX, PWA, bezpečná offline cache, čeština jako výchozí a angličtina volitelně |
| [M12](#m12) – Zajištění kvality a bezpečnosti | G-13 | GitHub Actions / Rust / TypeScript | 1 | CI, CodeQL/SAST, contract/e2e/load testy, formální přejímka 79 KEEP |
| [M13](#m13) – Nasazení V2 a návrat | G-15 | systemd / samostatný LXC / CI | 1 | izolované releasy, health checks, backup+restore, rollback a schválený cutover |

**Kontrola rozsahu:** 79 / 79 KEEP, bez duplicit a bez vyřazených REMOVE. G-00 je úvodní scaffold, G-01 definuje kontrakty, G-14 ověřuje shadow provoz; žádná z těchto etap nepotřebuje vlastnit samostatnou KEEP funkci.

## Hranice a rozhodnutí každého modulu

### M01 — Lokální příjem a kanonický stav

**Primární etapa:** G-02. **Technologie:** Rust / Tokio. **Závislosti:** G-01.  
**Povinné funkce:** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`.

- **Vytvořit:** readsb JSON/BEAST, dekódování Mode-S/CRC, deduplikace a jediný autoritativní RAM state owner.
- **Záměrně nepřebírat:** vlastní webový polling, ukládání každé zprávy do DB ani paralelní state vlastníci.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M02 — Externí zdroje a provenance

**Primární etapa:** G-03. **Technologie:** Rust / Tokio. **Závislosti:** M01.  
**Povinné funkce:** `E01`, `E02`, `E03`.

- **Vytvořit:** ADSBHub a ADSB.lol raw/HTTP, omezené retry/timeout a přednost LOCAL údajů.
- **Záměrně nepřebírat:** zápisy network-only provozu do lokální historie bez nového produktového rozhodnutí.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M03 — Veřejné API a SSE

**Primární etapa:** G-04. **Technologie:** Rust / Axum. **Závislosti:** M01, M02, G-01.  
**Povinné funkce:** `J06`, `J07`, `J08`.

- **Vytvořit:** verzované veřejné DTO, snapshot+delta, sekvence, heartbeaty a bounded fanout.
- **Záměrně nepřebírat:** nový WebSocket broker, per-client readsb poller či neomezené fronty.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M04 — Historie a datové úložiště

**Primární etapa:** G-05. **Technologie:** PostgreSQL / SQLx / Rust. **Závislosti:** M01, M03.  
**Povinné funkce:** `J09`, `H07`, `H14`.

- **Vytvořit:** nová DB od nuly, vzorkované zápisy, profily a časové souhrny pro nové lety.
- **Záměrně nepřebírat:** migrace staré V1 historie, Prisma RC, kopie monolitického DB modelu či kompletní raw stream.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M05 — Mapa a interaktivní provoz

**Primární etapa:** G-06. **Technologie:** Next.js / React / MapLibre. **Závislosti:** M03; M04 jen pro historické profily.  
**Povinné funkce:** `R01`, `R02`, `R03`, `R04`, `R05`, `R06`, `R07`, `R08`, `R09`, `R10`, `R11`, `R14`, `R15`.

- **Vytvořit:** živá mapa, lokalita provenance, symboly, seznam, detail, trail, kamera, vrstva a panely.
- **Záměrně nepřebírat:** gigantická univerzální React komponenta, kopie starého CSS ani per-entity React polling.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M06 — Letecké katalogy a navigace

**Primární etapa:** G-07. **Technologie:** Rust / SQL. **Závislosti:** M03, M04, M05.  
**Povinné funkce:** `A01`, `A02`, `T08`, `T09`, `T10`, `T13`, `T14`, `R12`, `R17`, `O13`.

- **Vytvořit:** letiště, eAIP, ATS/NAVAID, prostorová geometrie, ATC a Navigation Integrity.
- **Záměrně nepřebírat:** nepodložené frekvence, přebírání historických katalogů bez licence a původu.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M07 — Počasí a meteorologické vrstvy

**Primární etapa:** G-08. **Technologie:** Rust providers / web MapLibre. **Závislosti:** M01, M05, M06.  
**Povinné funkce:** `W01`, `W02`, `W03`, `W06`, `W08`, `W09`, `R18`.

- **Vytvořit:** METAR/TAF, ČHMÚ časová osa, SIGMET a Mode-S počasí s TTL.
- **Záměrně nepřebírat:** dotaz na počasí v každém ADS-B ticku, těžké weather analytiky mimo KEEP.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M08 — OGN a doplňkové integrace

**Primární etapa:** G-09. **Technologie:** Rust / SQL / izolované providers. **Závislosti:** M03, M05.  
**Povinné funkce:** `E04`, `E05`, `E06`, `E07`, `E09`, `E10`, `H03`.

- **Vytvořit:** OGN/DDB/SoftRF soukromí, RXW, FlightAware, ADSBDB a Planespotters on-demand.
- **Záměrně nepřebírat:** snímání identity noTracking, neomezené obohacování každého letadla, nové multi-streamy bez odůvodnění.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M09 — Administrace a doručování upozornění

**Primární etapa:** G-10. **Technologie:** Rust / SQL / web. **Závislosti:** M03, M04.  
**Povinné funkce:** `L11`, `L12`, `O15`.

- **Vytvořit:** Web Push/Pushover, transakční claim/retry a bezpečné přihlášení správce.
- **Záměrně nepřebírat:** nepoužívaný starý rule engine a skutečné rozesílání ze shadow vývoje.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M10 — Statistiky a provozní telemetrie

**Primární etapa:** G-11. **Technologie:** SQL / Rust / web. **Závislosti:** M01 až M09 jen podle jednotlivých sond.  
**Povinné funkce:** `S01`, `S02`, `S03`, `S04`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`, `O09`, `O10`.

- **Vytvořit:** denní agregace, coverage, logbook, zdravotní stav, DB a CPU/RSS metriky, T58.
- **Záměrně nepřebírat:** periodické těžké full-polly, databázový výpočet na každého SSE klienta a neomezená telemetry historie.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M11 — Uživatelská platforma

**Primární etapa:** G-12. **Technologie:** Next.js / PWA / i18n. **Závislosti:** M05, M06.  
**Povinné funkce:** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`.

- **Vytvořit:** responsive UX, PWA, bezpečná offline cache, čeština jako výchozí a angličtina volitelně.
- **Záměrně nepřebírat:** přebarvování místo oprav UX a kopírování globálního CSS V1.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M12 — Zajištění kvality a bezpečnosti

**Primární etapa:** G-13. **Technologie:** GitHub Actions / Rust / TypeScript. **Závislosti:** M01 až M11.  
**Povinné funkce:** `O11`.

- **Vytvořit:** CI, CodeQL/SAST, contract/e2e/load testy, formální přejímka 79 KEEP.
- **Záměrně nepřebírat:** povolování failujících testů, vymyšlené PASS a automatické odemykání deploye.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

### M13 — Nasazení V2 a návrat

**Primární etapa:** G-15. **Technologie:** systemd / samostatný LXC / CI. **Závislosti:** G-14 a zelená G-13.  
**Povinné funkce:** `O12`.

- **Vytvořit:** izolované releasy, health checks, backup+restore, rollback a schválený cutover.
- **Záměrně nepřebírat:** přepnutí V1 proxy nebo výmaz DB bez výslovného souhlasu.
- **Přejímka:** konkrétní testy při deterministickém vstupu, ohraničené zdroje, skutečné profilování relevantních nákladů a evidenční odkazy ke všem souvisejícím KEEP ID.
- **Vlastnictví:** modul má svůj datový kontrakt a testy; příbuzné implementace smějí sdílet knihovnu jen při prokázaném reálném využití, nikoliv pro předčasné abstrakce.

## Převzetí z V1 – přísné rozlišení

- **Převzít pouze požadavky a reference:** názvy vybraných 79 funkcí, bezpečnostní invarianty, kompatibilní datové formáty, ověřené chování, veřejně legální číselníky a anonymizované fixture.
- **Nepřebírat zdrojový strom ani legacy abstrakce:** API controllery, monolitickou React aplikaci, globální CSS, Prisma modely, jednorázové benchmarky, staré dashboardy, nevyžádané endpoints a 99 REMOVE funkcí.
- **Malý izolovaný algoritmus lze cíleně znovu použít až po ověření** licence, parity, bezpečnosti a výkonnostního přínosu; výjimka se zaznamená v českém ADR a testuje se bez přístupu k V1.
- **Nezavádět bez důkazu:** další procesy, broker, Redis, Kafka, Kubernetes, druhou DB, další runtime jazyk a geografické analytické vrstvy mimo KEEP.
- **Zabraň duplikaci:** canonical state existuje pouze v Rust enginu; web odebírá DTO/SSE. DB se zapisuje vzorkovaně a fail-soft. Každý provider má vlastní omezenou cache.
- **Při nových požadavcích:** explicitně přiřaď KEEP/nově schválené ID, vlastníka, spotřebitele, zdroj dat, SLA, retenci, ověřovací test a dopad na výkon. Pokud žádný konzument neexistuje, nic neimplementuj.

## Pravidlo minimalismu

Před vytvořením nové dependency, modulu, tabulky, endpointu, komponenty či background jobu odpověz: *Která aktivní KEEP funkce ji používá? Jaká data a odkud tečou? Jak to změřím? Proč jednodušší řešení nestačí? Jak věc vypnu nebo obnovím?* Bez odpovědi nejde změnu přijmout.

Primární zadání: [ZADANI-AIRRADAR-V2.md](ZADANI-AIRRADAR-V2.md), stav: [ETAPY-A-STAV.md](ETAPY-A-STAV.md), anti-legacy: [CO-NEPREBIRAT-Z-V1.md](CO-NEPREBIRAT-Z-V1.md).
