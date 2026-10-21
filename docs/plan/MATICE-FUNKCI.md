# Matice funkcí AirRadar V2

**Výchozí jazyk: čeština.** Tento soubor se generuje ze `contracts/feature-matrix.json`; neupravuj jej ručně. Použij `node tools/generate-feature-matrix.mjs`.

**Rozsah:** 79 požadovaných funkcí (KEEP) a 99 vyřazených funkcí (REMOVE). Označení `NOT_STARTED` znamená, že funkce dosud není implementovaná.

## Požadované funkce

| ID | Oblast | Funkce | Stav | Důkazy |
|---|---|---|---|---|
| `J01` | Živý příjem a stav | Příjem lokálních letadel přes readsb JSON · základ systému | `NOT_STARTED` | — |
| `J02` | Živý příjem a stav | Přímý příjem lokálních BEAST TCP rámců | `NOT_STARTED` | — |
| `J03` | Živý příjem a stav | Rust akcelerace BEAST parseru a CRC24 | `NOT_STARTED` | — |
| `J04` | Živý příjem a stav | Automatické přepínání BEAST / JSON při výpadku | `NOT_STARTED` | — |
| `J05` | Živý příjem a stav | Správa živého stavu letadel v RAM · základ systému | `NOT_STARTED` | — |
| `J06` | Živý příjem a stav | Veřejné API aktuálních letadel · základ systému | `NOT_STARTED` | — |
| `J07` | Živý příjem a stav | Živý stream SSE V1 – celé snapshoty · základ systému | `NOT_STARTED` | — |
| `J08` | Živý příjem a stav | Optimalizovaný SSE V2 – delta aktualizace | `NOT_STARTED` | — |
| `J09` | Živý příjem a stav | Ukládání vzorkované historie do PostgreSQL | `NOT_STARTED` | — |
| `J11` | Živý příjem a stav | Kontinuita identity letadel, detekce výpadků a deduplikace · základ systému | `NOT_STARTED` | — |
| `R01` | Mapa a vizualizace | Základní interaktivní mapa MapLibre · základ systému | `NOT_STARTED` | — |
| `R02` | Mapa a vizualizace | Přepínání LOCAL / EXTENDED pokrytí | `NOT_STARTED` | — |
| `R03` | Mapa a vizualizace | Ikony letadel podle typu a směru | `NOT_STARTED` | — |
| `R04` | Mapa a vizualizace | Popisky letadel a řízení jejich kolizí | `NOT_STARTED` | — |
| `R05` | Mapa a vizualizace | Barevné režimy letadel podle provozních údajů | `NOT_STARTED` | — |
| `R06` | Mapa a vizualizace | Živý seznam provozu a filtrování letadel | `NOT_STARTED` | — |
| `R07` | Mapa a vizualizace | Detail vybraného letadla v mapovém panelu | `NOT_STARTED` | — |
| `R08` | Mapa a vizualizace | Zobrazení živé letové stopy (trail) | `NOT_STARTED` | — |
| `R09` | Mapa a vizualizace | Sledování letadla kamerou mapy (follow) | `NOT_STARTED` | — |
| `R10` | Mapa a vizualizace | Současné sledování více letadel | `NOT_STARTED` | — |
| `R11` | Mapa a vizualizace | Zobrazení trasy mezi letišti a jejího průběhu | `NOT_STARTED` | — |
| `R12` | Mapa a vizualizace | Vrstvy okolních letišť | `NOT_STARTED` | — |
| `R14` | Mapa a vizualizace | Mapové podklady, vzhled a předvolby vrstev | `NOT_STARTED` | — |
| `R15` | Mapa a vizualizace | Mapová HUD lišta a rychlé akce | `NOT_STARTED` | — |
| `R17` | Mapa a vizualizace | ATC sektory a vysílače přímo v mapě | `NOT_STARTED` | — |
| `R18` | Mapa a vizualizace | Volitelné počasí a vzdušné prostory přímo v mapě | `NOT_STARTED` | — |
| `H03` | Detail letadla | Fotografie letadel z Planespotters | `NOT_STARTED` | — |
| `H07` | Detail letadla | Výškový, rychlostní a vertikální profil letu | `NOT_STARTED` | — |
| `H14` | Detail letadla | Souhrny letadla za 7/30 dní a celé období | `NOT_STARTED` | — |
| `A01` | Letiště | Katalog letišť a vyhledávání podle ICAO | `NOT_STARTED` | — |
| `A02` | Letiště | Detail letiště – dráhy, poloha, vybavení a frekvence | `NOT_STARTED` | — |
| `T08` | ATC, ATS a navigace | ATS tratě České republiky | `NOT_STARTED` | — |
| `T09` | ATC, ATS a navigace | ATS tratě Slovenska a Rakouska | `NOT_STARTED` | — |
| `T10` | ATC, ATS a navigace | NAVAID/FIX databáze a navigační průzkumník | `NOT_STARTED` | — |
| `T13` | ATC, ATS a navigace | Navigation Integrity – podezřelé anomálie ADS-B | `NOT_STARTED` | — |
| `T14` | ATC, ATS a navigace | Navigation Integrity – regionální mapa, historie a analýza | `NOT_STARTED` | — |
| `W01` | Počasí | Aktuální METAR a TAF letišť | `NOT_STARTED` | — |
| `W02` | Počasí | METAR značky a informace přímo v mapě | `NOT_STARTED` | — |
| `W03` | Počasí | Meteorologický radar ČHMÚ s časovou osou | `NOT_STARTED` | — |
| `W06` | Počasí | SIGMET a AirSIGMET výstrahy | `NOT_STARTED` | — |
| `W08` | Počasí | Počasí měřené letadly z Mode-S / BDS 4,4 | `NOT_STARTED` | — |
| `W09` | Počasí | Profil a kvalita počasí pozorovaného letadlem | `NOT_STARTED` | — |
| `L11` | Upozornění | Push notifikace do prohlížeče | `NOT_STARTED` | — |
| `L12` | Upozornění | Externí Pushover upozornění | `NOT_STARTED` | — |
| `E01` | Datové zdroje a integrace | ADSBHub – příjem externího síťového provozu | `NOT_STARTED` | — |
| `E02` | Datové zdroje a integrace | ADSB.lol – surový BEAST/SBS stream | `NOT_STARTED` | — |
| `E03` | Datové zdroje a integrace | ADSB.lol – HTTP geografická záloha | `NOT_STARTED` | — |
| `E04` | Datové zdroje a integrace | OGN/FLARM – letadla, kluzáky a další cíle | `NOT_STARTED` | — |
| `E05` | Datové zdroje a integrace | OGN Device Database – identifikace a ochrana soukromí | `NOT_STARTED` | — |
| `E06` | Datové zdroje a integrace | SoftRF SQLite – nouzová OGN whitelist databáze | `NOT_STARTED` | — |
| `E07` | Datové zdroje a integrace | ACARS/VDL2 metadata přes RXW Hub | `NOT_STARTED` | — |
| `E09` | Datové zdroje a integrace | FlightAware – doplňování letových plánů a diagnostika využití | `NOT_STARTED` | — |
| `E10` | Datové zdroje a integrace | ADSBDB – externí metadata letadel | `NOT_STARTED` | — |
| `S01` | Statistiky | Aktuální počítadla letadel a zpráv za sekundu | `NOT_STARTED` | — |
| `S02` | Statistiky | Denní/7denní/30denní agregované statistiky – API | `NOT_STARTED` | — |
| `S03` | Statistiky | Receiver Coverage – polární mapa dosahu | `NOT_STARTED` | — |
| `S04` | Statistiky | Analýza slabých sektorů a spolehlivosti příjmu | `NOT_STARTED` | — |
| `S06` | Statistiky | Statistiky typů letadel, aerolinek a tratí – API | `NOT_STARTED` | — |
| `S09` | Statistiky | Logbook a agregované souhrny přijímače | `NOT_STARTED` | — |
| `O01` | Diagnostika a provoz | System Dashboard – stav všech komponent | `NOT_STARTED` | — |
| `O02` | Diagnostika a provoz | Veřejná kontrola zdraví aplikace a verze · základ systému | `NOT_STARTED` | — |
| `O03` | Diagnostika a provoz | Receiver Monitoring – zdraví SDR/readsb příjmu | `NOT_STARTED` | — |
| `O04` | Diagnostika a provoz | Diagnostika externích poskytovatelů a integrací | `NOT_STARTED` | — |
| `O05` | Diagnostika a provoz | CPU, RAM, GC a event-loop telemetrie | `NOT_STARTED` | — |
| `O06` | Diagnostika a provoz | Historie výkonu a aktuálních bottlenecků | `NOT_STARTED` | — |
| `O07` | Diagnostika a provoz | Databázová latence, fronty a diagnostika zapisování | `NOT_STARTED` | — |
| `O08` | Diagnostika a provoz | Systémové diagnostické SSE streamy | `NOT_STARTED` | — |
| `O09` | Diagnostika a provoz | T58 production soak, stall canary a CPU profiling | `NOT_STARTED` | — |
| `O10` | Diagnostika a provoz | Výkonové benchmarky a zátěžové testy | `NOT_STARTED` | — |
| `O11` | Diagnostika a provoz | Automatické CI, testy, CodeQL a bezpečnostní kontroly · základ systému | `NOT_STARTED` | — |
| `O12` | Diagnostika a provoz | Automatické nasazení, health check a rollback · základ systému | `NOT_STARTED` | — |
| `O13` | Diagnostika a provoz | Import/synchronizace eAIP, ATS a letištních dat | `NOT_STARTED` | — |
| `O15` | Diagnostika a provoz | Správa přihlášení administrátora a bezpečnostních oprávnění · základ systému | `NOT_STARTED` | — |
| `U01` | Uživatelské rozhraní | Responzivní rozhraní mobil/tablet/desktop · základ systému | `NOT_STARTED` | — |
| `U02` | Uživatelské rozhraní | Instalovatelná PWA aplikace | `NOT_STARTED` | — |
| `U03` | Uživatelské rozhraní | Offline cache posledního známého stavu | `NOT_STARTED` | — |
| `U04` | Uživatelské rozhraní | Česká a anglická lokalizace | `NOT_STARTED` | — |
| `U05` | Uživatelské rozhraní | Globální Command Search a navigace | `NOT_STARTED` | — |
| `U08` | Uživatelské rozhraní | Klávesové zkratky a rychlé ovládání | `NOT_STARTED` | — |

## Funkce mimo rozsah

Tyto funkce mají v JSON stav `EXCLUDED` a nebudou do V2 automaticky přenášeny.

| ID | Funkce |
|---|---|
| `A03` | Airport Live Board – přehled provozu letiště |
| `A04` | Odvozené přílety, odlety a průlety |
| `A05` | Vyhodnocení použitých přistávacích drah |
| `A06` | Arrival Sequence / Approach Queue |
| `A07` | Arrival Flow, vytížení a stabilita drah (V8) |
| `A08` | Odhad terminálové poptávky / příletů (V9) |
| `A09` | Provozní výjimky a konfliktní údaje letiště |
| `A10` | Live Airport Network – okolní aktivní letiště |
| `A11` | Porovnání letišť |
| `A12` | Historická statistika provozu letiště za 7/30 dní |
| `D01` | Digital Twin – základní projekce letu na 30 minut |
| `D02` | 4D koridor – budoucí poloha, výška a čas |
| `D03` | Prediction Timeline v detailu letadla |
| `D04` | Budoucí průniky ATC sektorů a ATS tras |
| `D05` | Průniky s plánovanými AUP/UUP prostory |
| `D06` | Průniky s Navigation Integrity anomáliemi |
| `D07` | Operational Focus – prioritní situace letadla |
| `D08` | Operational Focus Change – změny priorit |
| `D09` | Regional Situation Graph – vazby mezi letadly |
| `D10` | Operational Attention – regionální upozornění |
| `D11` | Regional Focus Queue – pořadí prioritních letadel |
| `D12` | Regionální projekce za 5/15/30 minut |
| `D13` | Validace koridoru proti pozdějším pozicím |
| `D14` | Validace predikovaných událostí |
| `D15` | Truth-first Validation – vyhodnocování podle skutečných událostí |
| `D16` | Ukládání anonymních výsledků kalibrace |
| `D17` | Regional Attention Outcome – kontrola regionálních predikcí |
| `D18` | Regional Attention Graduation – pravidla povýšení modelu |
| `D19` | Trajectory Quality V2 – alternativní vertikální model |
| `D20` | Trajectory Quality V3 – výkonově orientovaný shadow model |
| `D21` | Outcome Validation pro modely V2 a V3 |
| `D22` | Graduation/Promotion – výběr schváleného modelu |
| `D23` | Admin Calibration Center – diagnostika modelů |
| `E08` | ACARS H1/FPN waypointy a mapová vrstva |
| `H01` | Samostatný detail letadla – registrace, typ, operátor |
| `H02` | Katalog a automatické doplňování metadat letadel |
| `H04` | Letový plán, volací znak a letiště OD/DO |
| `H05` | Historie konkrétního letadla a jeho letů |
| `H06` | Detail jednotlivého letu (Flight Story V2) |
| `H08` | Historické přehrávání konkrétního letu |
| `H09` | Seznam a vyhledávání historických letů |
| `H10` | Porovnání dvou letů |
| `H11` | Time Machine – přehrávání všech letadel v čase |
| `H12` | Event Replay – ±10 minut kolem události |
| `H13` | Historická identita NEW / RARE / RETURNING |
| `J10` | Demo provoz se simulovanými letadly |
| `L01` | Watchlist – pravidla sledování letadel |
| `L02` | Hlídání ICAO, registrace, callsignu, typu a aerolinky |
| `L03` | Upozornění podle vzdálenosti od přijímače |
| `L04` | Fleet Explorer – vlastní sledované flotily |
| `L05` | Geofencing – vlastní sledované geografické oblasti |
| `L06` | Emergency squawk 7500/7600/7700 |
| `L07` | Historie upozornění a událostí |
| `L08` | Pravidla cooldownu a odstranění duplicit |
| `L09` | Notification Center – přijaté/nepřečtené zprávy |
| `L10` | Ztlumení oznámení pro letadlo nebo pravidlo |
| `L13` | Delivery Health – stav doručování zpráv |
| `L14` | Alert Rule Simulator – testování pravidel |
| `L15` | Alert Effectiveness Analytics – účinnost upozornění |
| `O14` | Automatická dokumentace, changelog a metriky růstu kódu |
| `P01` | Flight Intelligence – automatické události letu |
| `P02` | Rozpoznávání holding, go-around, diversion, orbit a neobvyklých zatáček |
| `P03` | Přehled událostí a analytická stránka |
| `P04` | Route Intelligence – interpretace letového plánu |
| `P05` | Route Corridor Intelligence – odchylka od plánované trasy |
| `P06` | Trajectory Conformance – odklon, návrat a pravděpodobné DCT |
| `P07` | Predikce času příletu ETA |
| `P08` | Predikce přistávací dráhy |
| `P09` | Predikce změny přistávací dráhy |
| `P10` | Predikce budoucí trajektorie |
| `P11` | Explainable Prediction – vysvětlení podkladů predikcí |
| `P12` | Prospektivní sběr a validace predikcí |
| `P13` | Readiness a pravidla pro zveřejnění predikcí |
| `P14` | Kalibrace a porovnání predikcí s výsledky |
| `P15` | Predictive Operations Center – souhrn předpovědí |
| `R13` | Poloha přijímače, kružnice dosahu a měřítko |
| `R16` | Režim prezentace mapy |
| `S05` | Rekordy vzdálenosti a přijatých letadel |
| `S07` | Historická heatmapa a profil provozu – API |
| `S08` | Denní a týdenní recap – zbývající API |
| `T01` | ATC sektory včetně geometrie a výškových hranic |
| `T02` | Samostatný detail ATC sektoru |
| `T03` | ATC frekvence, vysílače a publikované služby |
| `T04` | Pravděpodobné ATC přiřazení vybraného letadla |
| `T05` | ATC handoff – odhad přechodu mezi sektory |
| `T06` | Historie provozu a přechodů mezi sektory – API |
| `T07` | Validace a diagnostika ATC dat |
| `T11` | Letové procedury SID/STAR a procedurální data |
| `T12` | AUP/UUP – plánovaná aktivace vzdušného prostoru |
| `U06` | Oblíbená letiště a lokální uživatelské preference |
| `U07` | Sdílené záložkové odkazy na filtrování a analýzy |
| `W04` | Echo Top – výška radarových odrazů |
| `W05` | Model větru ICON-EU v různých letových hladinách |
| `W07` | PIREP/AIREP hlášení pilotů |
| `W10` | Weather Fusion – spojení více zdrojů počasí |
| `W11` | Weather Avoidance – vyhodnocení možné odchylky kvůli počasí |
| `W12` | Počasí na trase zvoleného letu |
| `W13` | Historické mapové počasí v Time Machine |
| `W14` | Weather Corridor – počasí v předpokládaném koridoru |

## Údržba

- Aktualizuj skutečně ověřený stav v `contracts/feature-matrix.json`.
- Pro `TESTED` nebo `ACCEPTED` je nutný důkaz v poli `tests`.
- Spusť `node tools/generate-feature-matrix.mjs` a v CI ověř `node tools/generate-feature-matrix.mjs --check`.

Stav vývoje najdeš v [plánu etap](ETAPY-A-STAV.md).
