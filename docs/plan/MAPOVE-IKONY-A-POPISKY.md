# AirRadar V2 — ikony letadel a popisky mapy

**24. 10. 2026 · Specifikace pro G-05.03/.04 (R03/R04), návazně R01/R06/R08/R09/R14 a G-10.**  
**Stav 24. 10. 2026:** v PR #29 byly implementovány původní Path2D siluety; [PR #30](https://github.com/Boym323/AirRadarV2/pull/30) implementuje volitelná validovaná metadata `emitterCategory` ze skutečného readsb JSON i CRC-validních DF17 BEAST zpráv. **Finální G-05 vizuální/GPU přejímka zůstává otevřená**. Rozhodnutí popisuje [ADR 0015](../adr/0015-g05-emitter-category.md). [Hlavní UX](WEB-UX-UI-SPECIFIKACE.md) · [Vizuální audit](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md).

## 1. Historická výchozí implementace před G-05.03 / PR #29 a #30

`apps/web/src/radar/RadarMap.tsx` má jedinou bílou siluetu `radar-plane` z 64×64 canvasu při číselném `telemetry.trackDegrees`; bez směru ukazuje modrou tečku. Vybraný cíl se zvětšuje. Popisek je `callsign`; GeoJSON properties mají `id, label, heading, selected`. Toto byl stav **před PR #29**, které přidalo vlastní katalog a symbolové vrstvy. `contracts/aircraft-public.schema.json`, Rust `PublicAircraft` a webový `Aircraft` **před PR #30 neobsahovaly** ověřenou kategorii; PR #30 doplňuje zpětně kompatibilní volitelný údaj. Nelze nyní právem tvrdit, že rozlišujeme dopravní letadlo, vrtulník a kluzák.

## 2. Vizuální jazyk vlastních siluet

Sada původních vector/source ikon, vyobrazených **shora**, jednotný geometrický střed, špička směrových symbolů vzhůru při 0°, dostatečně široké plochy i ve 24 px; bez emoji, fotografií, kopírování cizích sprite, externího fontu nebo DOM markeru na každý cíl.

| ID vlastního symbolu | Tvar / pravidlo | Použití teprve při ověřené kategoriové hodnotě |
|---|---|---|
| `generic-fixedwing` | Neutrální letadlo s křídly, ne konkrétní model | Kategorie neznámá, **směr znám** |
| `large-fixedwing` | Delší trup, šípová křídla | A3/A4/A5, ne „zaručeně dopravní linka“ |
| `light-fixedwing` | Malý trup, spíše rovná křídla | A1/A2 |
| `high-performance` | Šípová zkrácená křídla | A6; **ne** automaticky vojenské |
| `rotorcraft` | Trup a příčný rotor | A7 |
| `glider` | Dlouhá úzká křídla | B1 |
| `ultralight` | Jednoduchá silueta lehkého prostředku | B4 |
| `lighter-than-air` | Balon/vzducholoď, nevyvolává zdání letového tracku těla | B2 |
| `uav` | Malý neutrální bezpilotní symbol | B6 |
| `ground-unit` | Odlišný obdélníkový pozemní tvar | C1/C2, jen s reálnou validní polohou |
| `unknown-direction` | Kruh s kontrastním obrysem | Kdykoliv není platný směr, **bez ohledu na kategorii** |

`A0/B0/C0`, rezervované kódy, jiné hodnoty, neplatné metadata → generický symbol jen při známém tracku; bez tracku bod. Symbol dopravního letadla **nepředpokládá** skutečnou aerolinku, typ, počet motorů či trasu.

**Grafický výstup G-05:** v PR #29 byly implementovány vlastní vektorové Path2D siluety (nikoli externí SVG soubory) a registrované MapLibre ikony; opticky porovnat čitelnost při 16/24/32 CSS px; mít variantu dark/light a highlight bez duplikace modelu. **Samostatné originální SVG soubory zůstávají otevřeným grafickým výstupem; dosavadní Path2D siluety jsou funkční zdroje.**

## 3. Pravdivý datový původ (blokace)

Readsb JSON může obsahovat `category` (ADS-B emitter category) a s externě nakonfigurovanou databází i `t` (typ). `type=adsb_icao` apod. označuje původ zprávy, **nikoliv typ letadla**. Použít pouze skutečně validovaný a čerstvý emitter category, který Rust přijme a G-03 bezpečně zveřejní. Změna veřejného DTO a jeho kontraktů je **samostatná práce** s golden tests pro BEAST/JSON, snapshot/delta, invalidní nebo rezervované hodnoty, reconnect a zpětnou kompatibilitu. Databázové `t` neimportovat bez samostatného ověření licence/provenance/mappingu. Neodvozovat typ z callsignu, výšky či rychlosti.

Přesné mapování jen při schváleném a testovaném poli:

- `A1/A2` → light; `A3/A4/A5` → large; `A6` → high-performance; `A7` → rotorcraft
- `B1` → glider; `B2` → lighter-than-air; `B4` → ultralight; `B6` → uav
- `C1/C2` → ground-unit; jiné/unknown → generic fallback; bez tracku vždy unknown-direction.

**BEAST-only režim:** dokud bez duplicitního ingestu neumí poskytnout důvěryhodnou kategorii, musí fungovat generický symbol, nikoliv falešná specializovaná ikona. Absence emitter category **nesmí blokovat mapu**.

## 4. Směr, velikosti a interakční cíl

| Parametr | Návrhová výchozí hodnota k otestování |
|---|---|
| Zdrojová mřížka | 64×64, nos směrových ikon přesně nahoru (0°) |
| Běžná velikost | Opticky cca **24 px**; upravit podle hustoty, DPI a screen testu |
| Vybrané letadlo | Cca **30–32 px**, jasný vnější obrys/halo |
| Neznámý směr | **10–12 px** kruh s obrysem, nikoli šipka |
| Směrování | `telemetry.trackDegrees`: true ground track 0° sever, clockwise; neodvozovat z headingu, letecké rychlosti ani volacího znaku |
| Map bearing | Test pro otočenou mapu: `icon-rotation-alignment=map`, žádná dvojitá rotace |
| Interakce | Grafika může být malá, ale hit target / ekvivalentní seznam splní WCAG 2.2; návrh min. 24×24 px nebo přípustná prostorová výjimka, na mobilu raději přibližně 40–44 px |
| Stopa | Krátká pravdivá stopa **za** ikonou, stávající max 64 bodů/20 minut, žádné prodloužení |

Velikosti jsou návrh k přejímce, nikoli důkaz správné dotykové ergonomie.

## 5. Barevná a stavová hierarchie

- **Běžný cíl:** neutrální světlý symbol s tmavým obrysem nad tmavým podkladem, na světlém mapovém stylu jednoznačně kontrastní obrys. Neznámá výška ≠ „nízký let“.
- **Vybraný:** nezaměnitelné tyrkysové/modré halo a zvětšení + současně výraznější kontrast, ne pouze barva.
- **7500/7600/7700:** až po G-08 validaci červené výstražné orámování **plus číselné označení**, které je přístupné i přes seznam/detail; neimprovizovat další druh alertu.
- **`onGround`** je provozní údaj v detailu, nikoli důkaz pozemní kategorie C1/C2.
- **Stale/offline:** živá mapa nezobrazuje ztracený cíl jako nový/aktuální; poslední snapshot výslovně s časem a offline označením.
- Výškový barevný gradient inspirovaný tar1090/ADS-B Exchange **není výchozí povinností**: koliduje s nouzovou červenou a vyžadoval by samostatnou legendu, metriky kontrastu a ověření dostupné výšky.

## 6. Popisky R04: informační úrovně

- **Základ:** krátký validní `callsign`; pokud chybí, vybraný cíl má v detailu bezpečné ID. Bez odhadovaného typu/registrace. `altitudeBaroFt` pouze je-li reálná, s `ft`; neznámá pole **vynechat**.
- **Zoom 0–7 (pracovní návrh):** minimální popisky; vybraná identita vždy dostupná v detailu.
- **Zoom 8–10:** jen callsign u nekolidujících cílů, přednost vybranému.
- **Zoom ≥11:** volitelně doplnit existující výšku; při hustotě skrýt **text**, nikoli správná letadla. Zoom hranice a počet řádků kalibrovat v G-05.11.
- Text 11–12 px, vybraný ~13 px; kontrastní halo a čitelný odstup od siluety. `text-optional` a sorting/collision pravidla ověřit na skutečně použité verzi MapLibre. Rozhodnutí nesmí trvale vytvářet DOM marker ani React komponentu na každé letadlo.

## 7. MapLibre implementační pravidla

Jedna stálá MapLibre instance a jeden GeoJSON aircraft source se stabilní `feature.id`; registrované ikony obnovit po `style.load`, fallback při chybějící ikoně; `icon-image` podle ověřené category, `icon-rotate` jen skutečný track. Zvolený/highlight event nesmí množit per-aircraft komponenty. Pro **`icon-color` je nutné SDF**, nelze předpokládat přebarvení libovolné canvas RGBA bitmapy. Vrstvy počasí/SIGMET pod leteckými symboly, vysoká priorita výběru a validní nouze.

## 8. Testovací a přejímací matice

1. Unit test A1–A7/B1/B2/B4/B6/C1/C2, rezervované, neplatné a chybějící kategorie; neznámý track.
2. Snapshot/delta/epoch/reconnect s kategorií a změnou identifikátoru; žádné duplicity, žádné odhalení interních polí.
3. Bearing 0/45°, track 0/90/180/270°, `onGround`, invalidní stopa a žádná falešná predikce.
4. Screenshot dark/light, default/selected/emergency-po-G-08, zoom 7/9/12 a hustota 25/100/500/2000.
5. Text kolize, label při neznámé výšce, mobile tap, keyboard/list alternativa, focus, reduced-motion a kontrast.
6. Přepnutí stylu a výpadek dlaždic/WebGL: ikony se nepřestanou zobrazovat nebo dostupný seznam zůstane použitelný.
7. G-05.11: FPS/CPU/RSS/p95, `setData` proti `updateData` na skutečně uvedeném zařízení. Headless CI screenshot **není GPU benchmark**.

**Zápis v této specifikaci nesplňuje žádné checkboxy.** Až při realizaci vydat vlastní SVG assety, implementaci mapování, kontrakty DTO, testy a důkazy v G-05 reportu.

## 9. Ověřené zdroje

- [readsb README JSON — `category`, `t`, `track`](https://github.com/wiedehopf/readsb/blob/dev/README-json.md)
- [ADS-B Exchange — emitter categories](https://support.adsbexchange.com/hc/en-us/articles/44705224053517-Emitter-Category-ADS-B-DO-260B-2-2-3-2-5-2)
- [ADS-B Exchange — Map Help](https://support.adsbexchange.com/hc/en-us/articles/44653064937741-Map-Help)
- [MapLibre — Style Spec, icon/label/collision](https://maplibre.org/maplibre-style-spec/layers/)
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
