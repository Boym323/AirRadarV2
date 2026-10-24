# G-05 — audit aktuálního vzhledu a iterativní vizuální přejímka (24. 10. 2026)

**Zdroj výchozího stavu:** screenshot veřejně nasazeného `airradarv2.pomykal.cz` (24. 10., cca 20:11), `apps/web/src/App.tsx`, `styles.css`, `RadarMap.tsx` v main `2964e8040ba91`; původní [map-first UX](../plan/WEB-UX-UI-SPECIFIKACE.md), [ikony](../plan/MAPOVE-IKONY-A-POPISKY.md) a [zadání G-05](../ukoly/G-05-ZIVA-MAPA.md). Poznámka: screenshot je skutečný uživatelský důkaz, nikoliv vykreslená CI fixture. Přímé otevření veřejného webu z auditorského prostředí selhalo; skutečné externí HTTP a browser výsledky se musí evidovat zvlášť, ne tvrdit bez důkazu.

## Zjištění a priority

| Priorita | Důkaz / problém | Změna, etapa a aktivní ID | Přejímka |
|---|---|---|---|
| P0 | Výchozí styl `dark` je téměř černý, silnice i města mají minimální vizuální váhu. Ztěžuje lokalizaci letadel. | G-05.09/R14: atlasový modrošedý podklad `fiord` jako základ, `dark` a `liberty` jako alternativy | Reálná mapa má zřetelnou geografii i pod běžnými siluetami, přepnutí neodpojí stream ani výběr. |
| P0 | Blokovitý celoplošný 66px header odděluje a opticky zmenšuje hlavní mapu. | G-05.01/R01 + G-05.05/R06: transparentní plovoucí horní ovladače nad mapou, jeden mapový viewport | Jeden WebGL canvas, žádný layout shift/remount, 1440×900 + 390×844 bez horizontálního přetékání. |
| P1 | Všechny siluety stejnou bílou barvou, v husté oblasti se překrývají a ztrácejí hranice. | G-05.03/R03 + G-05.06/R06: vizuální oddělení výběru s halo, jemnější běžné ikony; nezměnit tvar bez validované `emitterCategory` | Ikony při 500/2000 cílech zůstanou čitelné, bez DOM markerů či falešných kategorií. |
| P1 | V screenshotu téměř žádný callsign a slabé názvy míst; daná RF data mohou opravdu postrádat callsign. | G-05.04/R04: kontrastnějším halo a vhodnější offsetem zlepšit **pouze validní** štítky; doplnit informaci o popisku až do detailu. | Při testovací známé identitě je validní callsign na zoom >=8, bez zaměňování chybějících dat za odhad. |
| P1 | MapLibre zoom působí jako defaultní samostatný nástroj. | G-05.01/R01: vizuálně sjednotit navigační control s horními plovoucími skupinami | Touch, klávesnice, zoom, compass stále fungují. |
| P1 | V horní liště chybí zřejmý vstup do hledání; seznam vyžaduje více kroků. | G-05.05/R06: hledat pouze existující letadla, mobil bez zhoršení | Přístupný search input otevře jediný seznam, bez druhé kopie dat. |
| P1 | Další technická patička `AirRadar V2 · ... Nasazeno SHA` byla vrácena na spodní okraj. | G-05.01/R01: odstranit patičku z výchozího zobrazení; mapová atribuce zůstává povinně ve standardním prvku MapLibre | Footer není v DOM výchozího pohledu; atribuce zůstává funkční. |
| P2 | Předchozí Chromium pouze funkčně testoval mapu, nikoliv kvalitativně vzhled po DEV release. | G-05.11: povinné před/po důkazy 1440×900, 390×844, state live/selected/layers, verifikace nasazeného SHA | Samostatné screenshoty a měřitelná kontrola layoutu; fyzický GPU benchmark nadále nezměřen. |

## Navržený výsledek a omezení
- Styl **Aviation Atlas**: map-first, sofistikovaná modrošedá kartografie, ostrý cyan výběr, matné plovoucí nástroje, omezit rámečky a nechat co nejvíce mapy. Není to přidání analytického dashboardu.
- Zachovat `cs-CZ`, potvrzený lokální readsb/BEAST, jeden Rust RAM vlastník, jediný MapLibre canvas, SSE epoch/seq/stream, schválených 45 funkcí a přístupnou seznamovou alternativu.
- V žádné iteraci neměnit ani nenasazovat V1; `airradarv2.pomykal.cz` je DEV veřejný náhled V2, **nikoliv produkční schválení G-13**.
- Výsledky Playwright/SwiftShader neoznačovat jako fyzické GPU FPS; G-05 formálně **10/11**.
- Externí mapové chyby nelze „vyřešit“ pouze skrytím banneru; vlastní analýza sítě/tile CDN je samostatné riziko.

## Postup iterací

1. **I1 — rozložení / kartografie / hledání:** samostatné PR s funkčními testy, následný automatický V2 DEV release a kontrola nasazeného SHA. Ověřit proti dodanému reálnému screenshotu a pokud dostupné i externímu HTTP/Chromium.
2. **I2 — ikonografie / detail / map controls:** druhé malé PR až po I1, reálný browser screenshot desktop+mobile, kontrola ztráty SSE a map style při změnách.
3. **I3 — finální vizuální přejímka** teprve s důkazem skutečného přístupu do nasazeného webu a fyzického zařízení. Nemačkat G-05.11 do HOTOVO bez výkonu.

## Stav evidence
Audit proveden z aktuálního zdrojového kódu a uživatelského screenshotu. Důkazy změn a konkrétní CI/DEV URL doplnit po jejich skutečném úspěchu; toto není potvrzení finálního vzhledu na veřejném URL.
