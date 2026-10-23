# Aktivní matice funkcí AirRadar V2

Tento soubor se generuje ze `contracts/feature-matrix.json` a obsahuje výhradně aktuálně schválené funkce.

**Rozsah: 45 aktivních funkcí.** Budoucí rozšíření eviduje [oddělený archiv](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

| ID | Oblast | Funkce | Stav | Testovací důkazy |
|---|---|---|---|---|
| `J01` | Místní příjem | Příjem lokálních letadel přes readsb JSON · základ systému | `TESTED` | G-02 JSON fixture+temp-file a společný replay, CI #39769323592 |
| `J02` | Místní příjem | Přímý příjem lokálních BEAST TCP rámců | `TESTED` | G-02 BEAST TCP fragmentation+loopback reconnect, CI #39769323592, G-02 realny BEAST 60min read-only DEV LXC, GitHub #39795592425, SHA d0cefc4a |
| `J03` | Místní příjem | Rust akcelerace BEAST parseru a CRC24 | `TESTED` | G-02 DF17 CRC24, TC19, CPR a negativní parity, CI #39769323592, G-02 realny BEAST 60min read-only DEV LXC, GitHub #39795592425, SHA d0cefc4a |
| `J04` | Místní příjem | Automatické přepínání BEAST / JSON při výpadku | `TESTED` | G-02 JSON/BEAST interleaving, connect/disconnect a source health, CI #39769323592 |
| `J05` | Místní příjem | Správa živého stavu letadel v RAM · základ systému | `TESTED` | G-02 jediný RadarState, TTL a stres 5000 identit, CI #39769323592, G-02 realny BEAST 60min read-only DEV LXC, GitHub #39795592425, SHA d0cefc4a |
| `J06` | Místní příjem | Veřejné API aktuálních letadel · základ systému | `TESTED` | G-03 HTTP GET/400/503 + OpenAPI 3.1 + privacy, CI #39814106622, SHA d7483417 |
| `J07` | Místní příjem | Živý stream SSE V1 – celé snapshoty · základ systému | `TESTED` | G-03 SSE snapshot, reconnect, 15s heartbeat Nginx curl -N; CI #39835102766, SHA db4078fa |
| `J08` | Místní příjem | Optimalizovaný SSE V2 – delta aktualizace | `TESTED` | G-03 SSE delta/upsert/removed, gap resync, 100-client benchmark; CI #39835102766, SHA db4078fa |
| `J09` | Místní příjem | Ukládání vzorkované historie do PostgreSQL | `IN_PROGRESS` | — |
| `J11` | Místní příjem | Kontinuita identity letadel, detekce výpadků a deduplikace · základ systému | `TESTED` | G-02 identita, dedup, plausibilita, neplatné CRC a stale polohy, CI #39769323592, G-02 realny BEAST 60min read-only DEV LXC, GitHub #39795592425, SHA d0cefc4a |
| `R01` | Mapa | Základní interaktivní mapa MapLibre · základ systému | `NOT_STARTED` | — |
| `R03` | Mapa | Ikony letadel podle typu a směru | `NOT_STARTED` | — |
| `R04` | Mapa | Popisky letadel a řízení jejich kolizí | `NOT_STARTED` | — |
| `R06` | Mapa | Živý seznam provozu a filtrování letadel | `NOT_STARTED` | — |
| `R08` | Mapa | Zobrazení živé letové stopy (trail) | `NOT_STARTED` | — |
| `R09` | Mapa | Sledování letadla kamerou mapy (follow) | `NOT_STARTED` | — |
| `R12` | Mapa | Vrstvy okolních letišť | `NOT_STARTED` | — |
| `R14` | Mapa | Mapové podklady, vzhled a předvolby vrstev | `NOT_STARTED` | — |
| `A01` | Letiště | Katalog letišť a vyhledávání podle ICAO | `NOT_STARTED` | — |
| `A02` | Letiště | Detail letiště – dráhy, poloha, vybavení a frekvence | `NOT_STARTED` | — |
| `T10` | Navigace | NAVAID/FIX databáze a navigační průzkumník | `NOT_STARTED` | — |
| `W03` | Počasí | Meteorologický radar ČHMÚ s časovou osou | `NOT_STARTED` | — |
| `W06` | Počasí | SIGMET a AirSIGMET výstrahy | `NOT_STARTED` | — |
| `W08` | Počasí | Počasí měřené letadly z Mode-S / BDS 4,4 | `NOT_STARTED` | — |
| `W09` | Počasí | Profil a kvalita počasí pozorovaného letadlem | `NOT_STARTED` | — |
| `L11` | Upozornění | Push notifikace do prohlížeče | `NOT_STARTED` | — |
| `L12` | Upozornění | Externí Pushover upozornění | `NOT_STARTED` | — |
| `S01` | Statistiky | Aktuální počítadla letadel a zpráv za sekundu | `NOT_STARTED` | — |
| `S06` | Statistiky | Statistiky typů letadel, aerolinek a tratí – API | `NOT_STARTED` | — |
| `S09` | Statistiky | Logbook a agregované souhrny přijímače | `NOT_STARTED` | — |
| `O01` | Provoz | System Dashboard – stav všech komponent | `NOT_STARTED` | — |
| `O02` | Provoz | Veřejná kontrola zdraví aplikace a verze · základ systému | `NOT_STARTED` | — |
| `O03` | Provoz | Receiver Monitoring – zdraví SDR/readsb příjmu | `NOT_STARTED` | — |
| `O04` | Provoz | Diagnostika externích poskytovatelů a integrací | `NOT_STARTED` | — |
| `O05` | Provoz | CPU, RAM, GC a event-loop telemetrie | `NOT_STARTED` | — |
| `O06` | Provoz | Historie výkonu a aktuálních bottlenecků | `NOT_STARTED` | — |
| `O07` | Provoz | Databázová latence, fronty a diagnostika zapisování | `NOT_STARTED` | — |
| `O08` | Provoz | Systémové diagnostické SSE streamy | `NOT_STARTED` | — |
| `U01` | Uživatelské prostředí | Responzivní rozhraní mobil/tablet/desktop · základ systému | `NOT_STARTED` | — |
| `U02` | Uživatelské prostředí | Instalovatelná PWA aplikace | `NOT_STARTED` | — |
| `U03` | Uživatelské prostředí | Offline cache posledního známého stavu | `NOT_STARTED` | — |
| `U04` | Uživatelské prostředí | Česká a anglická lokalizace | `NOT_STARTED` | — |
| `U05` | Uživatelské prostředí | Globální Command Search a navigace | `NOT_STARTED` | — |
| `U08` | Uživatelské prostředí | Klávesové zkratky a rychlé ovládání | `NOT_STARTED` | — |
| `L06` | Upozornění | Emergency squawk 7500/7600/7700 | `NOT_STARTED` | — |

## Aktualizace

- Jednotlivá KEEP ID musí mít právě jeden vlastní modul.
- Stav `TESTED` nebo `ACCEPTED` musí mít doklad v poli `tests`.
- Používej `node tools/generate-feature-matrix.mjs` a kontrolu `--check`.
