# AirRadar V2 — UX/UI specifikace „mapa na prvním místě“

**Datum rozhodnutí:** 24. 10. 2026  
**Schválené produktové rozhodnutí:** AirRadar V2 je především **živá interaktivní mapa leteckého provozu**. Mapa je hlavní pracovní plochou webu, nikoliv jednou z několika rovnocenných dashboardových stránek.  
**Stav tohoto dokumentu:** schválený princip *map-first*, navržená podrobnější podoba UI pro postupnou implementaci a testování. Samotné sepsání dokumentu **není implementací ani akceptací** G-05 až G-10.

## 1. Rozsah a vazby

- Dokument konkretizuje pouze stávajících **45 schválených funkcí** v `contracts/feature-matrix.json`. Nepřidává nová ID, zdroje dat, samostatné produktové moduly ani nové backendové služby.
- Hlavní vlastník: **M04 / G-05** (`R01/R03/R04/R06/R08/R09/R12/R14`) a **M09 / G-10** (`U01/U02/U03/U04/U05/U08`).
- Navazující konzumenti: **G-06** letiště/NAVAID (`A01/A02/T10`), **G-07** počasí (`W03/W06/W08/W09`), **G-08** upozornění (`L06/L11/L12`), **G-09** statistiky a diagnostika (`S01/S06/S09/O01–O08`).
- Autorita živých dat zůstává v Rust RAM; klient využívá jediné G-03 HTTP/SSE DTO. **Žádný výběr jiného zdroje letadel, druhý ingest, další mapa ani skryté simulované cíle.**
- Existující první rozložení `apps/web/src/App.tsx` (hlavička, mapa vlevo, seznam vpravo) je **přechodná G-05 implementace**, nikoli závazný finální wireframe. Jakýkoliv refaktor musí zachovat funkční SSE, výběr, stopu a follow a projít regresními testy.

## 2. Informační architektura

**Výchozí pohled a návrat do aplikace = radarová mapa.** Každá schválená funkce se otevře tak, aby bylo možné rychle pokračovat v práci s mapou.

| Funkční oblast | Cílové umístění / chování | Etapa a ID |
|---|---|---|
| Živá mapa | Dominantní plocha dostupná hned po otevření | G-05 `R01` |
| Letadla a hledání | Sbalitelný boční seznam na desktopu, vysouvací panel na mobilu; jeden stav s mapou | G-05 `R06`, G-10 `U05` |
| Detail letadla | Kontextový panel po výběru cíle, bez zakládání nové hlavní mapové stránky | G-05 `R03/R04/R08/R09` |
| Podklady a vrstvy | Jedno místo pro existující/implementované volby; neaktivní vrstvy se nenabízejí jako hotové | G-05 `R12/R14` |
| Letiště, dráhy, frekvence, NAVAID/FIX | Výběr prvku v mapě nebo hledání → stručný kontextový detail | G-06 `A01/A02/T10` |
| ČHMÚ radar a SIGMET | Průhledné mapové vrstvy s ovládáním platnosti; počasí nikdy nepřekryje význam živého provozu | G-07 `W03/W06` |
| Meteorologie konkrétního letadla | Sekce v detailu vybraného letadla, jen pro skutečně přijaté a důvěryhodné údaje | G-07 `W08/W09` |
| Nouzové squawk a upozornění | Čitelný stav cíle na mapě/seznamu a volitelné nastavení doručení | G-08 `L06/L11/L12` |
| Statistiky, logbook, diagnostika | Sekundární stručný panel/pohled dosažitelný z mapy; nikoliv druhý výchozí dashboard ani duplicita mapy | G-09 `S01/S06/S09/O01–O08` |
| Mobil, PWA, jazyky, klávesnice | Stejný produkt a mapa, adaptivní ovládání bez druhého stavu aplikace | G-10 `U01/U02/U03/U04/U05/U08` |

**Nezavádět** samostatné stránky nebo moduly jen kvůli vizuálnímu zaplnění. Je-li rozsáhlejší detail skutečně nutný pro existující schválenou funkci, musí mít jasnou cestu zpět na mapu, zachovaný výběr a test. Názvy URL ani router nejsou tímto dokumentem předepsané.

## 3. Cílové rozložení — desktop

Přednost má souvislá mapa přes hlavní obsahovou oblast. Ovládání tvoří krátká horní lišta a kontextové, zavíratelné panely. Panel nesmí trvale zabírat mapu, pokud jej uživatel nepotřebuje.

```text
┌──────────────────────────────────────────────────────────────────┐
│ AirRadar V2  ● živý příjem  [Hledat]  [Vrstvy]  [Stav] [Menu]   │
├──────────────────────────────────────────────────────────────────┤
│ ┌────────────────┐                              ┌─────────────┐ │
│ │ Letadla /      │         ŽIVÁ MAPA             │ Detail cíle │ │
│ │ filtrování     │                              │ stopa       │ │
│ │ (sbalitelné)   │       letadla + vrstvy        │ follow      │ │
│ └────────────────┘                              └─────────────┘ │
│                 Mapové ovládání / přepnutí podkladu              │
└──────────────────────────────────────────────────────────────────┘
```

- **Horní lišta:** kompaktní značka, skutečný stav streamu, přístup k hledání a vrstvám. Žádný velký marketingový nadpis vytlačující mapu pod přehyb obrazovky.
- **Seznam:** otevřený podle potřeby, lze jej skrýt; výběr funguje oběma směry (mapa ↔ seznam). Pozice panelu vlevo je výchozí **návrh**, nikoliv požadavek na konkrétní CSS framework.
- **Detail:** zobrazí se až po výběru; obsahuje jen pole skutečně obsažená v kontraktu. Zavření detailu nezastaví živý radar. Zobrazení follow je explicitní a vypnutelné.
- **Ovládání vrstev:** jedno konzistentní místo; nenutit k souběžnému otevření více velkých panelů. Změna stylu nezruší výběr ani stav SSE.
- **Informace o zdroji a atribuce mapy** zůstávají čitelné, ale co nejméně překážejí mapě.

## 4. Cílové rozložení — mobil a tablet

- Na telefonu je **mapa výchozím celoobrazovkovým kontextem**. Horní ovladače jsou zhuštěné; seznam a detail letadla se otevírají jako **spodní vysouvací panel** (návrh pro `G-10.01/.02`). Bez trvalého dvousloupcového layoutu.
- Otevření panelu nevyvolá novou mapovou instanci ani reset připojení; při zavření se vrací focus na příslušný ovladač.
- Dotykové pan/zoom/pinch musí fungovat bez konfliktu s gesty panelů. Při ručním posunu mapy se ukončí volitelný follow, ne výběr cíle.
- Tablet může využít boční panel podle skutečně dostupné šířky; breakpoint se volí podle testů, ne podle pevně předpokládaného zařízení.
- Všechny hlavní akce musí být dostupné dotykem i klávesnicí; bez horizontálního scrollování ovládání a s dostatečnými klikacími cíli.

## 5. Jednoznačné interakce

### Letadlo (G-05)
1. Kliknutí na platný symbol v mapě nebo položku seznamu vybere stejnou identitu.
2. Vybraný cíl dostane jednoznačné zvýraznění; zobrazí se **skutečná** omezená stopa a dostupný detail (callsign/ICAO, výška, rychlost, směr jen pokud je datově podložen).
3. Akce **Sledovat** aktivuje kameru follow, nikoli automaticky každé kliknutí. Ruční pan follow vypne.
4. Ztráta identity, zastaralá poloha nebo SSE reconnect musí mít zřejmou prezentaci; nikdy nevydávat starý bod za aktuální nebo stopu za predikci trasy.
5. Zavření detailu vrátí pozornost k mapě, stav letadel a spojení zůstane zachován.

### Letiště / navigační bod (G-05.10 a G-06)
- Pouze licencovaná/kontrolovaná data a existující schválený katalog. Kliknutí na letiště ukáže především identifikaci, publikované dráhy a frekvence tam, kde jsou dostupné; nic se neodhaduje.
- Výběr letiště nesmí smazat živá letadla, aktualizovat hlavní SSE ani vytvářet samostatnou mapu.

### Počasí (G-07)
- ČHMÚ snímek musí mít čitelný čas měření a ovladatelnou časovou osu. Staré snímky jsou jednoznačně označené.
- SIGMET/AirSIGMET zobrazit jako oddělenou volitelnou vrstvu s validitou. Nedostupnost zdroje není důvodem skrýt letadla.
- Pole Mode-S BDS lze zobrazit jen na základě skutečných měření a jejich kvality.

### Stav systému, squawk a upozornění (G-08 a G-09)
- Živý provoz, spojování, reconnect, ztráta streamu a offline režim se vizuálně liší i textem/ikonou, nejen barvou.
- Nouzové kódy `7500/7600/7700` mají zřetelnou indikaci teprve po validaci; neinterpretovat běžné stavy jako nouzové.
- Web Push/Pushover zůstávají **opt-in** a výchozí doručení v DEV/CI/shadow vypnuté.
- Diagnostika zobrazuje jen skutečně běžící komponenty, nevymýšlí „Node server“ pro statický Vite build a neposkytuje tajemství.

## 6. Vizuální pravidla (návrh design systému)

- Vizuální priorita: **mapa → vybraný cíl → živý stav → seznam/vrstvy → sekundární statistiky**.
- Zachovat současný tmavý základ jako výchozí návrh; světlejší mapový podklad je již dostupný. Definitivní UI paleta a typografie se doladí podle kontrastu a čitelnosti; tento dokument neschvaluje nové barvy ani logo.
- Stavový význam musí být konzistentní; barvu nikdy nepoužít jako jediný nosič informace. Letecké značky, trajektorie a meteorologie musí být rozeznatelné.
- Jednotná terminologie v češtině (`cs-CZ`), volitelná angličtina v G-10. Žádné nově přidané viditelné defaultně anglické ovladače.
- Nepoužívat mnoho překrývajících se plovoucích karet, trvalou druhou navigaci nebo zbytečné animace. Respektovat `prefers-reduced-motion`.
- Zachovat přístupný textový seznam jako alternativu k WebGL mapě.

## 7. Stavy a chybové scénáře

| Situace | Co smí UI tvrdit a ukázat |
|---|---|
| Čerstvý SSE snapshot/delta | „Živý provoz“, skutečné polohy; zachovat výběr podle identity |
| Připojování / reconnect | Viditelné „Připojování/Obnovuji spojení“, žádný falešný live indikátor |
| SSE ztraceno | „Spojení přerušeno“; případná poslední data jasně označit jako neaktuální |
| Letadlo bez validní polohy | Seznam může uvádět identitu, ale mapa nevykreslí odhadnutý bod |
| Nedostupný mapový podklad/WebGL | Srozumitelná chyba; živý seznam dále použitelný |
| Nedostupné počasí / katalog | Degradovat jen příslušnou vrstvu, ne ingest či seznam letadel |
| Offline PWA | Jen omezený poslední snapshot s časem a explicitním offline štítkem; nikdy nevydávat cache za živé letadlo |
| Žádné viditelné cíle | Jasný prázdný stav; nezaměňovat s chybou připojení |

## 8. Výkon, zdroje a hranice implementace

- V mapě má být **jediná stabilní MapLibre instance** a sdílený browser store; žádná paralelní mapa pro každý panel.
- Render letadel používá společný GeoJSON source/symbol vrstvy, nikoliv DOM marker pro každé letadlo. Není-li známa kategorie/směr, ikona nesmí předstírat konkrétní typ/heading.
- UI nemá přímý PostgreSQL klient, druhý readsb ingest ani opakovaný server polling na každou otevřenou kartu.
- Mapové vrstvy a atributy načítat nezávisle; výpadek externího poskytovatele nesmí měnit RAM-first živé letadlo.
- Výkonnostní testy `G-05.11`: 25/100/500/2000 letadel, 1/2 Hz, FPS, re-renders, paměť; mobile a desktop. Hodnoty vyhodnotit na skutečně popsaném zařízení a SHA. Není zde deklarovaný výkonový PASS.
- G-10 mobilní zátěž, přístupnost, offline/focus a regresní E2E jsou samostatné brány.

## 9. Akceptační UX scénáře navázané na existující etapy

| Scénář | ID / krok | Testovatelné očekávání |
|---|---|---|
| Otevření webu | `R01`, `G-05.01` | Výchozí je mapa (ne dashboard), jedna instance |
| Výběr / zrušení cíle v mapě i seznamu | `R06/R08/R09`, `G-05.05–.08` | Jediný stav, stopa/follow bez nechtěných resetů |
| Přepnutí mapového podkladu | `R14`, `G-05.09` | Letadla, výběr a SSE zůstanou |
| Přidání vrstvy letišť | `R12/A01/A02`, `G-05.10/G-06` | Katalog se načítá zvlášť, bez zásahu do letového streamu |
| Radar ČHMÚ a výstrahy | `W03/W06`, `G-07` | Lze zapnout/vypnout; viditelné časové údaje |
| Nouzový squawk | `L06/L11/L12`, `G-08` | Vizualizace jen validní události, doručování v DEV vypnuto |
| Statistiky / systém | `S01/S09/O01–O08`, `G-09` | Z mapy dosažitelný sekundární panel, žádný nový default dashboard |
| Telefon / tablet / desktop | `U01`, `G-10.01/.02` | Mapa dominuje, panely ovladatelné, bez horizontálního scrollu |
| Hledání a klávesnice | `U05/U08`, `G-10.08/.09` | Jediné hledání schválených objektů, focus a zkratky funkční |
| Offline, lokalizace, přístupnost | `U02/U03/U04`, `G-10.03–.07/.10` | Žádná falešná živost, přístupný seznam, cs/en a čitelné stavy |

**Pravidlo převzetí:** výše uvedené body jsou specifikací požadovaného chování, **ne novými PASS výsledky**. Stav funkcí i etap zůstává v `contracts/feature-matrix.json`, příslušných reportech a `docs/plan/ETAPY-A-STAV.md`.

## 10. Implementační pořadí a otevřené volby

1. **G-05:** dotáhnout mapové stávající ID a browser/GPU měření, sjednotit výběr a překrývání ovladačů; nepřepisovat už funkční SSE store jen kvůli vzhledu.
2. **G-06/G-07:** navázat detail letišť a mapové počasí na stejný panelový a vrstvový princip.
3. **G-08/G-09:** nouzové stavy i provozní diagnostiku připojit k mapě jako sekundární informace bez nového hlavního dashboardu.
4. **G-10:** definitivně sjednotit responzivní geometrii, mobilní panel, PWA, globální hledání, a11y a lokalizaci.
5. **G-11:** validovat průchod všemi existujícími funkčními scénáři a výkon proti skutečnému releasu.

**Otevřené a nikoliv předem schválené detaily:** přesné breakpointy, šířka bočních panelů, výsledná barevná paleta, mobilní výška spodního panelu, podoba ikonek a počet současně zobrazitelných sekundárních panelů. Přijímat je podle návrhu a E2E přejímky, bez změny aktivního rozsahu.

## 11. Související dokumentace

- [Závazné zadání](ZADANI-AIRRADAR-V2.md)
- [Schválené moduly a ID](MODULY-A-ROZSAH.md)
- [G-05 — Živá mapa](../ukoly/G-05-ZIVA-MAPA.md)
- [G-06 — Letiště a NAVAID](../ukoly/G-06-LETISTE-NAVAID.md)
- [G-07 — Počasí](../ukoly/G-07-VYBRANE-POCASI.md)
- [G-08 — Nouzové kódy](../ukoly/G-08-NOUZOVE-KODY-A-NOTIFIKACE.md)
- [G-09 — Diagnostika](../ukoly/G-09-STATISTIKY-DIAGNOSTIKA.md)
- [G-10 — Mobilní web a PWA](../ukoly/G-10-PWA-LOKALIZACE.md)
- [MapLibre výkon a testování](../technical/MAPLIBRE-VYKON-A-ZATEZ.md)
