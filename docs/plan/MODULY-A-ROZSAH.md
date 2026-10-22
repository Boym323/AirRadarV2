# Implementační moduly AirRadar V2

**Schválená implementace:** 45 aktivních funkcí v [matici](../../contracts/feature-matrix.json). Devět níže uvedených modulů jsou **logické komponenty**, ne nové mikroservisy. Každá funkce má přesně jednoho vlastníka.

| Modul | Etapa | Technologie | Funkce | Implementovaný účel |
|---|---|---|---|---|
| M01 — Lokální ADS-B ingest a RAM | G-02 | Rust/Tokio | `J01`, `J02`, `J03`, `J04`, `J05`, `J11` | readsb JSON, BEAST TCP a CRC24, jediný autoritativní stav letadel, deduplikace a přepínání zdrojů |
| M02 — Verzované HTTP API a SSE | G-03 | Rust/Axum | `J06`, `J07`, `J08` | bezpečné DTO, snapshot a delta stream s ohraničenými frontami |
| M03 — Úsporná historie | G-04 | Rust/SQLx/PostgreSQL | `J09` | nová V2 DB, vzorkované pozice, batch zápisy a retence bez přenosu staré historie |
| M04 — Živá mapa a základní ovládání | G-05 | Vite/React/MapLibre | `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14` | jedna živá mapa, ikony a popisky, seznam, stopa, sledování kamery, letištní značky a mapové podklady |
| M05 — Letiště a NAVAID/FIX | G-06 | Rust/SQL/Vite/React | `A01`, `A02`, `T10` | lehký katalog letišť, jejich drah a frekvencí a navigačních bodů |
| M06 — Vybraná meteorologie | G-07 | Rust/MapLibre | `W03`, `W06`, `W08`, `W09` | meteorologický radar ČHMÚ s časem, platné výstrahy a skutečná Mode-S meteorologická měření |
| M07 — Nouzové squawk a doručení | G-08 | Rust/SQL/web | `L06`, `L11`, `L12` | detekce kódů 7500/7600/7700 a volitelné doručení Web Push/Pushover |
| M08 — Statistiky a provozní diagnostika | G-09 | Rust/SQL/web | `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08` | aktuální čítače, jednoduché agregace, logbook a levné health/metriky |
| M09 — Webová platforma | G-10 | Vite/React/PWA/i18n | `U01`, `U02`, `U03`, `U04`, `U05`, `U08` | responzivita, PWA, označená offline cache, výchozí čeština, volitelná angličtina, hledání a klávesové zkratky |

## Vlastnictví a přejímka

### M01 — Lokální ADS-B ingest a RAM

**Etapa:** G-02. **Technologie:** Rust/Tokio.  
**Primární vlastněná KEEP ID (6):** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`.

- Implementovat: readsb JSON, BEAST TCP a CRC24, jediný autoritativní stav letadel, deduplikace a přepínání zdrojů.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M02 — Verzované HTTP API a SSE

**Etapa:** G-03. **Technologie:** Rust/Axum.  
**Primární vlastněná KEEP ID (3):** `J06`, `J07`, `J08`.

- Implementovat: bezpečné DTO, snapshot a delta stream s ohraničenými frontami.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M03 — Úsporná historie

**Etapa:** G-04. **Technologie:** Rust/SQLx/PostgreSQL.  
**Primární vlastněná KEEP ID (1):** `J09`.

- Implementovat: nová V2 DB, vzorkované pozice, batch zápisy a retence bez přenosu staré historie.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M04 — Živá mapa a základní ovládání

**Etapa:** G-05. **Technologie:** Vite/React/MapLibre.  
**Primární vlastněná KEEP ID (8):** `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`.

- Implementovat: jedna živá mapa, ikony a popisky, seznam, stopa, sledování kamery, letištní značky a mapové podklady.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M05 — Letiště a NAVAID/FIX

**Etapa:** G-06. **Technologie:** Rust/SQL/Vite/React.  
**Primární vlastněná KEEP ID (3):** `A01`, `A02`, `T10`.

- Implementovat: lehký katalog letišť, jejich drah a frekvencí a navigačních bodů.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M06 — Vybraná meteorologie

**Etapa:** G-07. **Technologie:** Rust/MapLibre.  
**Primární vlastněná KEEP ID (4):** `W03`, `W06`, `W08`, `W09`.

- Implementovat: meteorologický radar ČHMÚ s časem, platné výstrahy a skutečná Mode-S meteorologická měření.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M07 — Nouzové squawk a doručení

**Etapa:** G-08. **Technologie:** Rust/SQL/web.  
**Primární vlastněná KEEP ID (3):** `L06`, `L11`, `L12`.

- Implementovat: detekce kódů 7500/7600/7700 a volitelné doručení Web Push/Pushover.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M08 — Statistiky a provozní diagnostika

**Etapa:** G-09. **Technologie:** Rust/SQL/web.  
**Primární vlastněná KEEP ID (11):** `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`.

- Implementovat: aktuální čítače, jednoduché agregace, logbook a levné health/metriky.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

### M09 — Webová platforma

**Etapa:** G-10. **Technologie:** Vite/React/PWA/i18n.  
**Primární vlastněná KEEP ID (6):** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`.

- Implementovat: responzivita, PWA, označená offline cache, výchozí čeština, volitelná angličtina, hledání a klávesové zkratky.
- Každé ID vyžaduje deterministický test, zdokumentované chyby a samostatný důkaz v matici.
- Nevytvářet nový proces, tabulku ani závislost, pokud není skutečně potřeba pro tento modul.
- Pro změny na živé cestě doložit bounded paměť/CPU, správnost přijatých údajů a bezpečnost; pro UI kontrolu interakce a výkonu.

## Architektonické invarianty

- Jediný vlastník živého stavu je **Rust**; zdrojem letadel je místní readsb/BEAST. Žádné UI přepínání původu dat.
- Stav v RAM je oddělen od PostgreSQL a pomocných providerů. Výpadek DB nebrání zobrazit živá letadla.
- PostgreSQL začíná prázdná; ukládá jen potřebné vzorky a informace pro aktivní funkce.
- Web má jeden přehledný MapLibre tok bez druhého přijímače, neomezeného pollingu či zbytečných paralelních stavů.
- Reálné odesílání upozornění je výchozím nastavením vypnuté. Testy, bezpečnost, zálohy a návrat jsou povinné inženýrské brány.
- Nové nápady se evidují **výhradně v [archivu rozšíření](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md)** a nesmějí pronikat do současného kódu ani zadání bez schválení.

## Konkrétní kroky

K realizaci všech devíti modulů použij [140 ověřitelných kroků](KROKOVY-IMPLEMENTACNI-PLAN.md). Primární vlastníci zůstávají stejní: G-02 až G-10 jsou jedinými etapami, které implementují 45 aktivních produktových ID; G-01, G-11, G-12 a G-13 jsou průřezové kontraktové a provozní brány.
