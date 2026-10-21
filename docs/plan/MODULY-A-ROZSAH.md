# Katalog implementačních modulů AirRadar V2

**Aktuálně schválený rozsah:** **46 KEEP a 132 REMOVE** podle [strojové matice](../../contracts/feature-matrix.json). Níže uvedené moduly jsou pouze **logické části aplikace**, nikoli devět nezávislých procesů. Jde o zadání budoucí práce, ne seznam hotových implementací.

Přesné rozdělení 46 funkčních ID je jedinečné: každá má **jednoho primárního vlastníka**.

| Modul | Etapa | KEEP ID | Co bude implementovat |
|---|---|---|---|
| **M01** — Příjem readsb/BEAST a stav letadel | G-02 | `J01`, `J02`, `J03`, `J04`, `J05`, `J11` | Místní readsb JSON/BEAST, nativní CRC24/Mode-S, fallback, identita, bounded RAM. |
| **M02** — Veřejné API a SSE | G-04 | `J06`, `J07`, `J08` | Bezpečné HTTP DTO, snapshoty a delta SSE s omezenými frontami. |
| **M03** — Úsporná letová historie | G-05 | `J09` | Nová izolovaná DB, vzorkování, dávkový zápis a retence bez migrace V1. |
| **M04** — Živá mapa a základní ovládání | G-06 | `R01`, `R02`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14` | Mapa, symboly, popisky, seznam, stopa, follow, letiště a podklady. Bez detailního panelu, pokročilých barev a HUD. |
| **M05** — Letiště a NAVAID/FIX | G-07 | `A01`, `A02`, `T10` | Lehký katalog letišť včetně drah/frekvencí a navigačních bodů, bez ATS tratí, ATC sektorů či automatického importního systému. |
| **M06** — Vybrané meteorologické informace | G-08 | `W03`, `W06`, `W08`, `W09` | ČHMÚ radar, SIGMET/AirSIGMET, Mode-S BDS 4,4 a kvalita pozorování, bez METAR/TAF. |
| **M07** — Nouzové squawk a doručení upozornění | G-10 | `L06`, `L11`, `L12` | Detekce 7500/7600/7700 a volitelné Web Push/Pushover, bez obecného rule enginu. |
| **M08** — Lehké statistiky a provozní zdraví | G-11 | `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08` | Čítače, typy/aerolinky, logbook a diagnostiky včetně SSE; bez 7/30 dashboardů, coverage analýz a T58 produktu. |
| **M09** — Responzivní web, PWA a lokalizace | G-12 | `U01`, `U02`, `U03`, `U04`, `U05`, `U08` | Desktop/mobil, PWA, poctivě označená offline cache, čeština/angličtina, hledání a zkratky. |

## Detail vlastnictví a omezení

### M01 — Příjem readsb/BEAST a stav letadel

- **Etapa:** G-02; **technologie:** Rust/Tokio.
- **Primární vlastněná KEEP ID (6):** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`.
- **Výstup:** Místní readsb JSON/BEAST, nativní CRC24/Mode-S, fallback, identita, bounded RAM.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M02 — Veřejné API a SSE

- **Etapa:** G-04; **technologie:** Rust/Axum.
- **Primární vlastněná KEEP ID (3):** `J06`, `J07`, `J08`.
- **Výstup:** Bezpečné HTTP DTO, snapshoty a delta SSE s omezenými frontami.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M03 — Úsporná letová historie

- **Etapa:** G-05; **technologie:** Rust/SQLx/PostgreSQL.
- **Primární vlastněná KEEP ID (1):** `J09`.
- **Výstup:** Nová izolovaná DB, vzorkování, dávkový zápis a retence bez migrace V1.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M04 — Živá mapa a základní ovládání

- **Etapa:** G-06; **technologie:** Next.js/React/MapLibre.
- **Primární vlastněná KEEP ID (9):** `R01`, `R02`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`.
- **Výstup:** Mapa, symboly, popisky, seznam, stopa, follow, letiště a podklady. Bez detailního panelu, pokročilých barev a HUD.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M05 — Letiště a NAVAID/FIX

- **Etapa:** G-07; **technologie:** Rust/SQL a web.
- **Primární vlastněná KEEP ID (3):** `A01`, `A02`, `T10`.
- **Výstup:** Lehký katalog letišť včetně drah/frekvencí a navigačních bodů, bez ATS tratí, ATC sektorů či automatického importního systému.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M06 — Vybrané meteorologické informace

- **Etapa:** G-08; **technologie:** Rust providers/MapLibre.
- **Primární vlastněná KEEP ID (4):** `W03`, `W06`, `W08`, `W09`.
- **Výstup:** ČHMÚ radar, SIGMET/AirSIGMET, Mode-S BDS 4,4 a kvalita pozorování, bez METAR/TAF.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M07 — Nouzové squawk a doručení upozornění

- **Etapa:** G-10; **technologie:** Rust/web/SQL podle potřeby.
- **Primární vlastněná KEEP ID (3):** `L06`, `L11`, `L12`.
- **Výstup:** Detekce 7500/7600/7700 a volitelné Web Push/Pushover, bez obecného rule enginu.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M08 — Lehké statistiky a provozní zdraví

- **Etapa:** G-11; **technologie:** Rust/SQL/web.
- **Primární vlastněná KEEP ID (11):** `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`.
- **Výstup:** Čítače, typy/aerolinky, logbook a diagnostiky včetně SSE; bez 7/30 dashboardů, coverage analýz a T58 produktu.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

### M09 — Responzivní web, PWA a lokalizace

- **Etapa:** G-12; **technologie:** Next.js/React.
- **Primární vlastněná KEEP ID (6):** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`.
- **Výstup:** Desktop/mobil, PWA, poctivě označená offline cache, čeština/angličtina, hledání a zkratky.
- **Přejímka:** jednotlivá ID potvrď deterministickými scénáři, křížovými kontrakty a pouze reálně relevantními výkonovými metrikami. Evidenční linky ukládej do JSON matice a českého reportu. Bez testů nejde o dokončenou funkci.

## Výslovně zrušené plánované moduly

- **Původní G-03** (ADSBHub, ADSB.lol, E01–E03) — **ZRUŠENO**. Nesmí se připojovat ani vytvářet jejich adaptery. Původní R02 přepínač LOCAL/EXTENDED je zatím zachován, ale bez externích zdrojů musí zobrazovat poctivý stav „externí pokrytí není k dispozici“.
- **Původní G-09** (OGN/FLARM, SoftRF, RXW, FlightAware, ADSBDB, Planespotters) — **ZRUŠENO**.
- **Pokročilá historie a analýza** (H07, H14, S02, S03, S04, T13, T14) — **ZRUŠENO**. J09 zůstává pro úsporné nové vzorky; S06 a S09 mohou z nich vytvářet jednoduché agregace.
- **Pokročilé ATC/ATS** (R17, T08, T09 a další REMOVE) — **ZRUŠENO**. A02 obsahuje publikované letištní frekvence, ale nesmí se z toho automaticky stavět samostatný ATC systém.
- **METAR/TAF mapové značky** (W01/W02) a univerzální overlay (R18) — **ZRUŠENO**, meteorologický radar ČHMÚ a SIGMET zůstávají specifickými funkcemi.
- **O09/O10/O11/O12/O13/O15** už nejsou samostatnými produkty. Nezbytné technické testy, bezpečné přístupové kontroly, zálohy a manuální bezpečný release zůstávají jako nepřekročitelné vývojové/provozní podmínky.

## Architektonické hranice

1. Jediný vlastník live letadel = Rust engine. V2 nebude automaticky zobrazovat network-only/OGN cíle.
2. Živý tok `readsb/BEAST → RAM → SSE` nesmí čekat na databázi či počasí.
3. Nová PostgreSQL bez migrace V1; ukládat jen potřebné, vzorkované údaje; nevyvíjet nevyžádané historické dashboardy.
4. **L06** spouští pouze bezpečně rozpoznané události squawk 7500/7600/7700. Doručování L11/L12 je samostatně povolené a testované, nesmí se automaticky zapnout během shadow běhu.
5. Nové API, knihovna, tabulka nebo vrstva musí být přiřaditelná k jednomu z 46 KEEP ID nebo k nezbytné bezpečnostní/QA infrastruktuře. Nezařazené funkce nepřebírat.
6. G-13 kvalita, G-14 shadow a G-15 bezpečný přechod jsou nadále povinné inženýrské brány, **nikoliv znovuzařazení O09–O12**.

**Související zdroje:** [hlavní zadání](ZADANI-AIRRADAR-V2.md), [anti-legacy pravidla](CO-NEPREBIRAT-Z-V1.md), [přehled etap](ETAPY-A-STAV.md), [podrobné popisy 178 funkcí](PODROBNY-KATALOG-FUNKCI.md).
