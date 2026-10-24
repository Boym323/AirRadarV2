# AirRadar V2 — konkurenční UX a vizuální průzkum

**Datum:** 24. 10. 2026  
**Účel:** Převzít vybrané dobré principy leteckých radarových rozhraní, nikoli zdrojové kódy, cizí designové assety nebo neověřené funkce.  
**Závazný kontext:** [map-first UX/UI specifikace](../plan/WEB-UX-UI-SPECIFIKACE.md), 45 schválených ID, lokální readsb/BEAST. **Toto je produktový/UX průzkum, ne výsledky měření, implementace, použití UI v autentizovaném prohlížeči ani schválení nových KEEP ID.**

## Metoda a omezení

Posouzeny veřejně dostupné produktové dokumentace, návody, oficiální stránky a publikované snímky rozhraní. Část screenshotů je historická a nemusí přesně odpovídat aktuálně nasazené verzi. Dynamickou aplikaci každého poskytovatele jsme netestovali interaktivně, chování z klávesnice a na mobilu vychází z uvedené dokumentace. Rychlost, FPS ani ergonomické rozdíly nebyly nezávisle měřeny. Před přejímkou UI je nutné validovat návrh přímo v AirRadar V2 přes G-05/G-10 E2E a skutečné mobilní viewporty.

## Konkurenční vzory

| Služba | Ověřené, užitečné principy | Co se u nás nehodí |
|---|---|---|
| **Flightradar24** | Živá mapa jako centrální kontext, otevření informačního panelu letu bez ztráty mapy, základní/technická telemetrie, filtry včetně přednastavených kategorií a změny zvýraznění ostatního provozu | Předpokládané gate/ETA, plánované linky, cestující, globální pokrytí a náročná paleta marketingových widgetů nemají v lokálním readsb automaticky zdroj |
| **ADS-B Exchange** | Přímé mapové zkratky, vyhledání ICAO/callsign/typu, follow, reset kamery, popisky, ovládání mapových vrstev a seznamu, přepínatelné zobrazení stop | Hustá řada jednopísmenných tlačítek, technické příkazy bez vysvětlení a rozsáhlá paleta filtrů nesmí zahlcovat základní obrazovku |
| **AirNav Radar (redesign 29. 7. 2026)** | Čitelnější navigace, konzistentní světelný/tmavý/systémový režim, map-first priorita a omezení vizuálního šumu | Nový globální 3D globe nepřináší do cíle místního 2D radaru měřitelný užitek; nejde o schválené ID |
| **Plane Finder** | Hierarchicky členěná karta informací, jednoznačné hledání, map focus, uložené filtry, volitelné počasí | Předplatitelská „premium“ navigace, 3D onboard a dlouhodobý globální playback nejsou důvod k rozšíření V2 |
| **tar1090 / Airplanes.live** | Účelné živé ADS-B technické UI, jednoduchá tabulka letadel, barvení symbolů dle dostupné výšky, omezení popisků, stopa, volba velikosti ikon, dostatečně rychlý výběr v seznamu | Trvale exponované desítky ovládacích tlačítek, zkratková abeceda bez přístupných názvů, persistence ztraceného letu vydávaná za živá data |
| **FlightAware** | Srozumitelný oddělený živý/historický pohled, doplňkové počasí nad mapou | Komerční integrační/nástupní web a globální datové produkty, které G-05/G-10 vůbec nepotřebují |

## Co přebrat — jednoznačná priorita a schválený rozsah

### P0 — bezprostředně při dokončení G-05 (pouze existující funkce)

1. **Mapa zaplní hlavní pracovní plochu:** nad mapou kompaktní lišta stav/hledání/vrstvy, ne velký hero/title. V základu nechat mapě maximum místa; seznam lze skrýt. `R01/R06`, `G-05.01/.05`; ergonomii definitivně validovat v `G-10.01`.
2. **Výběr bez ztráty kontextu:** klik v mapě i seznamu vybere stejný cíl. Stručný detail s dostupným callsign/ICAO, výškou, rychlostí, headingem a platností pouze dle DTO; stopa a explicitní follow na jednom místě. `R06/R08/R09`, `G-05.05–.08`.
3. **Jasná vizuální hierarchie:** neutrálnější tmavá mapa, světlé čitelné texty, vybrané letadlo s kontrastním zvýrazněním, střídmé popisky a kolize. Symbol nesmí tvrdit neznámou kategorii nebo heading. `R03/R04`, `G-05.03/.04`. Přesné barvy/typografie zůstávají návrhem k vizuálnímu a a11y testu.
4. **Jednoduchá vrstva/map styl nabídka:** nyní funkční světlý/tmavý podklad; zachovat výběr a SSE po přepnutí. Neskrytě zobrazit atribuci. `R14`, `G-05.09`.
5. **Výrazný, pravdivý stav dat:** spojeno / reconnect / odpojeno / bez validní polohy. Odlišovat barvou **a textem/ikonou**; neživit stale polohu opticky jako aktuální. Stávající G-03/G-05 kontrakt, `G-10.04/.05/.10`.

### P1 — nad G-05, podle existujících G-06 až G-10

6. **Letiště, frekvence, FIX/NAVAID v kontextovém detailu bez nové mapy**; jen doložené datasety. `R12/A01/A02/T10`, `G-05.10/G-06`.
7. **Weather vrstvy s malou legendou a jasnou platností**: ČHMÚ timeline, SIGMET overlay; neohrozit čitelnost letadel. `W03/W06/W08/W09`, `G-07`.
8. **Jedno globální vyhledávání a ovládání z klávesnice:** neklonovat náročné regex UI; hledat jen existující schválené objekty a akce, zkratky popsat tooltipem. `U05/U08`, `G-10.08/.09`; G-05 seznam ponechat jednoduchý.
9. **Mobilní vysouvací panel:** mapa zůstává základ, seznam/detail si sdílejí tentýž stav. Zajistit dotyk, návrat focusu, skrývání seznamu a zachování SSE. `U01`, `G-10.01/.02/.10/.11`.
10. **Stručné živé metriky, logbook a diagnóza mimo centrální mapu:** informativní stavový panel bez druhého výchozího dashboardu. `S01/S09/O01–O08`, `G-09`.
11. **Jasné upozornění na validní 7500/7600/7700** a volitelné explicitně zapnuté doručení. Bez výchozího push v DEV/CI/shadow. `L06/L11/L12`, `G-08`.
12. **Předvolby vzhledu a podkladu** jen v mezích `R14/U04`; případný „systémový režim“ nejprve ověřit jako preferenci stávajícího vzhledu, nikoli novou službu.

### NEPŘEBÍRAT / neimplementovat bez nového výslovného schválení

- 3D globe, AR a 3D kokpit; další mapový engine by narušil záměr úsporného jediného MapLibre proudu.
- Globální feedy, přepínání ADS-B zdrojů, odhadované ztracené cíle, aproximované routy a letiště bez kvalitního podkladu.
- Predikované ETA, gate, fotografie, aerolinky nebo typy předstírané z nedostatečného lokálního DTO.
- Všechny stopy najednou, masivní nezávislé filtrační režimy a multiselect jako automatická součást základu; případnou změnu rozsahu evidovat odděleně a schvalovat.
- Napodobování cizího loga, konkrétních grafických assetů, CSS či chráněných mapových stylů.

## Návrh informačního vzhledu k vizuálnímu testu

```text
┌──────────────────────────────────────────────────────────────┐
│ AirRadar   ● živě  26 letadel  [Hledat] [Vrstvy] [Stav]     │
├──────────────────────────────────────────────────────────────┤
│ [☰ Letadla]      MAPA, SYMBOLY A POPISKY      [Detail cíle] │
│  Kompaktní       vybrané letadlo zvýraznit     Callsign      │
│  sbalitelný      krátká skutečná stopa         výška, směr   │
│  seznam          volba podkladu                sledovat      │
│                  atribuce mapy                              │
└──────────────────────────────────────────────────────────────┘
Telefon: mapa na celou pracovní plochu → seznam/detail do spodního panelu.
```

Zobrazený počet je **ilustrativní**, nikoliv výsledek live přijímače. Ovládací prvky je třeba zohlednit pro českou lokalizaci i pro přístupnost. Finální rozměry a barevnost neurčuje výzkum, ale browser test návrhu v projektu.

## Navržené akceptační scénáře navíc k existujícím testům (bez nových ID)

- Desktop 1440×900: první pohled zobrazuje mapu a kompaktní stav, seznam jde skrýt, otevření/zavření detailu nerestartuje MapLibre a nemění SSE.
- Telefon 390×844: mapa dostupná bez horizontálního scrollu, otevření/spuštění spodního panelu neblokuje pinch a follow, focus se po zavření vrací.
- Hustota 500/2000: test kolizí a čitelnosti popisků, žádný nesmyslný počet jednotlivých DOM markerů; skutečné FPS/RSS před/po s metodikou a SHA.
- Výpadek readsb/SSE nebo externích vrstev: nevytvořit falešnou čerstvost, zachovat seznam a čitelné chybové zprávy.
- Přepnutí podkladu: vybraný cíl, skutečná stopa a follow se zachovají dle aktuálního kontraktu.
- Změna zobrazení a přístupnost: kontrast, reduced-motion, screenreader text a klávesnicové ovládání.

Všechny tyto body jsou **návrhové acceptance scénáře**; PASS musí teprve dodat G-05/G-10. Pro mobilní výkon a mapovou vrstvu nadále platí `G-05.11`, `G-10.11` a [technické limity](../plan/TECHNICKE-KONTRAKTY-A-LIMITY.md).

## Ověřitelné zdroje (veřejná dokumentace)

1. Flightradar24 — funkcionality mapy, panel letu, filtry: https://www.flightradar24.com/blog/product-features/
2. Flightradar24 — nové filtry, zachování kontextu: https://support.fr24.com/support/solutions/articles/3000120424-how-to-use-filters
3. ADS-B Exchange — map help (aktualizováno 15. 5. 2026): https://support.adsbexchange.com/hc/en-us/articles/44653064937741-Map-Help
4. ADS-B Exchange — detail letadla (29. 4. 2026): https://support.adsbexchange.com/hc/en-us/articles/45478440676365-How-To-Track-a-Specific-Aircraft
5. AirNav Radar — redesign (29. 7. 2026): https://www.airnavradar.com/blog/meet-the-new-airnav-radar-a-new-way-to-explore-live-aviation
6. Plane Finder — seznam nabízených funkcí: https://planefinder.net/plans
7. tar1090 — dokumentace open-source webového rozhraní: https://github.com/wiedehopf/tar1090/blob/master/README.md
8. Airplanes.live — dokumentované panely a mapové zkratky: https://airplanes.live/map-help/
9. FlightAware — integrace živé mapy, historie, počasí: https://www.flightaware.com/commercial/integrated-maps/

**Poznámka k původu:** srovnání principů je naše vlastní syntéza; funkcionality a data nelze na základě cizího seznamu funkcí vydávat za dostupné v AirRadar V2.
