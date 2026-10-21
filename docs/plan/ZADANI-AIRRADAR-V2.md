# AirRadar V2 — závazné zadání greenfield projektu

**Schválená volba produktu:** 21. 10. 2026; **46 KEEP / 132 REMOVE** podle [matice](../../contracts/feature-matrix.json).  
**Repo:** https://github.com/Boym323/AirRadarV2; referenční V1: https://github.com/Boym323/AirRadar.  
**Jazyk zadání, reportů a UI:** čeština, `cs-CZ`. Volitelná angličtina UI U04 zůstává v rozsahu.  
**Stav kódu:** pouze G-00 scaffold s neprodukčními API; aktuální moduly ještě nejsou implementované.

## 1. Cíl a inženýrské hranice

Od nuly postavit úsporný a bezpečný **místní letecký radar** s použitím Rust + Tokio + Axum (jediný live state owner), Next.js + React + MapLibre (web) a nové PostgreSQL + SQLx (vzorkovaná historie). Nepřevádět zdrojový strom ani databázi V1. Celé V2 poběží v novém odděleném LXC, nikoli na produkčním V1.

Rozsah je **46 výslovně zachovaných funkcí**: každý bod v [MODULY-A-ROZSAH.md](MODULY-A-ROZSAH.md) a jednotlivé G-XX etapy. **132 REMOVE neimplementovat**, a to ani jako tajně přidaný „helper“, pokud by obcházel přímý produktový zákaz. J03 ve V2 je nativní Rust parser bez Node akcelerační vrstvy.

## 2. Skutečné datové toky

```text
lokální readsb JSON / BEAST TCP ──→ Rust normalizace a validace
                               └──→ jediný RAM stav letadel
                                         ├──→ Rust REST + SSE full/delta ──→ web / MapLibre
                                         └──→ bounded sampling ──→ PostgreSQL V2
ČHMÚ radar / SIGMET ──→ samostatná cache ──→ pouze odpovídající UI/API
Mode-S BDS počasí  ──→ oddělené dekódování ──→ W08/W09
squawk 7500/7600/7700 ──→ bezpečný detektor L06 ──→ opt-in L11/L12
```

- **Bez externího ADS-B**: E01/E02/E03 a celý G-03 jsou pryč. R02 přepínač LOCAL/EXTENDED zůstává, ale bez externího zdroje musí transparentně ukazovat nedostupnost EXTENDED; žádná fiktivní network letadla.
- **Bez OGN/FLARM a obohacení**: E04–E10, H03 a G-09 mimo rozsah. Nesmí se v runtime objevit nepotřebný internetový lookup každého letadla.
- **Bez ATC a ATS analytiky**: R17/T08/T09/T13/T14 a další původní REMOVE se neobnovují. A01/A02/T10 katalog letišť/frekvencí a NAVAID zůstává jako lehký datový produkt; O13 automatická synchronizace je pryč.
- **Bez historických profilů letadel, 7/30denních přehledů a coverage analýzy**: H07/H14/S02/S03/S04 jsou mimo rozsah. J09 dál ukládá potřebné nové vzorky pro S06/S09.
- **Bez METAR/TAF** W01/W02; W03 ČHMÚ radar, W06 SIGMET a W08/W09 Mode-S meteorologie zůstávají samostatnými moduly. R18 univerzální weather+airspace overlay není potřeba.
- **Bez systémového admin portálu a release produktu** O09–O13/O15; nezbytné testy, autentizace chráněných operací, zálohy a dokumentovaný bezpečný deployment jsou stále povinné inženýrské brány, nikoli důvod obnovovat jejich produktové ID.

## 3. Živý příjem a veřejná rozhraní

- Přijímat readsb JSON a lokální BEAST TCP pouze read-only, validovat DF/CRC/čas, icao, polohu a kvalitu.
- Jeden kanonický RAM state; continuity/TTL/stale guard; přepnutí BEAST/JSON bez duplicit.
- Rust REST veřejná projekce s absolutně sanitizovanými DTO, žádné privátní souřadnice přijímače ani credentials.
- SSE V1 snapshoty a SSE V2 initial+delta, heartbeat, per-client bounded queue, backpressure, sequence/reconnect, 503 při nedostatku kapacity.
- MapLibre mapa, ikony a popisky, seznam a filtry, live trail/follow, letiště a mapové podklady. **Nevytvářet** vyřazené R05/R07/R10/R11/R15/R17/R18.

## 4. PostgreSQL a úsporné statistiky

- Nová oddělená DB pro V2, žádná stará letová historie, žádný import V1 dat ani sdílené credentials.
- Query-first minimální tabulky pro vzorkované lety/pozice J09, agregace S06/S09, případné nezbytné delivery stavy a omezené provozní metriky O06/O07.
- SQLx migrace, bounded async queue, batch INSERT/UPSERT, retence, time-range indexy, žádný raw frame stream na disk.
- `EXPLAIN (ANALYZE, BUFFERS)` v DEV, výkonnostní profily pro latenci dotazů, WAL a I/O. DB offline nesmí zastavit živý radar.
- Statistické API pouze S01/S06/S09; chybějící typ/aerolinku bez E10 označovat jako neznámé, žádné domýšlené hodnoty.

## 5. Počasí, nouzové kódy a diagnostika

- ČHMÚ radar W03 s časovou osou, platný SIGMET/AirSIGMET W06, Mode-S/BDS 4,4 W08 a profil/kvalita měřeného počasí W09. Každý provider má bounded TTL/cache, volání nikdy neleží na ingest hot path.
- Squawk 7500/7600/7700 z lokálních validních dat je jedinou explicitně schválenou detekcí L06. Zjistit stav, přechod a duplicitu; L11/L12 jsou volitelné transporty. **Ve výchozím stavu a shadow režimu zakázat skutečné odesílání**.
- O01–O08: jednoduché stavové sondy jen pro aktivní subsystémy, diagnostické SSE, ochrana interních údajů, omezená historie výkonu.
- Rust CPU/RSS/fronty měřit odlišně od Node.js event-loop/GC.

## 6. Web a lokalizace

Český výchozí web a volitelná angličtina U04, mobil/tablet/desktop U01, PWA U02, offline poslední známý stav U03 s označením zastarání, globální hledání U05 a zkratky U08. Žádné kopírování `globals.css` či monolitické mapové komponenty V1.

## 7. Testy, CI, provoz a LXC

- Povinné Rust fmt/Clippy/tests, web lint/typecheck/Vitest/Playwright, kvalitní česká dokumentace v MD a strojová feature evidence.
- Testy validních a chybně formátovaných paketů, 1×/2×/4× replay, 1/10/100 SSE klientů, DB restart, žádný memory leak, bezpečné Web Push/Pushover failure cases.
- CI/testy se musí spouštět i při vyřazení O11 jako produktu. Změny dokumentace a vývoje patří do `CHANGELOG.md`, nikoliv do obnovené O14 aplikace.
- Izolovaný nový LXC a V2 DB; až po ověření lze schválit bezpečný read-only shadow bez vlivu na V1.
- G-15 smí připravit a po zvláštním schválení použít ruční release a rollback. **Nesmí automaticky měnit doménu, proxy, produkční V1, historická data ani tajemství**.

## 8. Etapy a výstupy

[Plán etap a stav](ETAPY-A-STAV.md) a samostatné `docs/ukoly/G-XX-*.md` jsou závazné implementační jednotky. **G-03 a G-09 jsou zrušené**; ostatní etapy jsou rozdělené podle devíti modulů. Každá dodává: změny, konkrétní KEEP ID, český report, automatické testy a skutečný výkonový důkaz, pokud výkon mění; nedokončené stavem ČÁSTEČNĚ/BLOKOVÁNO.

**Vývojový princip:** stav `ACCEPTED` nedostane žádná funkce bez testu a důkazu. Počty KEEP/REMOVE a denní statistiky repo jsou od 21. 10. 2026 46/132; historické metriky před změnou se nemají retrospektivně přepisovat.

## 9. Pokyn pro nový Codex běh

> Nejdříve přečti `AGENTS.md`, `docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md`, `docs/plan/ZADANI-AIRRADAR-V2.md` a `docs/plan/MODULY-A-ROZSAH.md`. Platí **46 KEEP / 132 REMOVE**, navrácena L06, G-03/G-09 zrušeny. Pracuj pouze na jedné následující etapě, zachovej Rust/Next.js/PostgreSQL hranice. Po implementaci proveď testy a aktualizuj český report, JSON/MD matici, stav etap a changelog. Bez změny V1 a bez produkčního deploye.
