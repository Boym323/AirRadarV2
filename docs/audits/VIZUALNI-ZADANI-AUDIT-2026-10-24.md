# AirRadar V2 — audit úplnosti vizuálního zadání

**24. 10. 2026 · Výchozí commit `1511877e98f6`.** Posouzeno: `apps/web/src/{App.tsx,styles.css,radar/RadarMap.tsx}`, veřejné `contracts/aircraft-public.schema.json`, [map-first UX](../plan/WEB-UX-UI-SPECIFIKACE.md), [konkurenční rešerše](KONKURENCNI-UX-PRUZKUM-2026-10-24.md), etapová zadání G-05 až G-10, MapLibre výkon a browser testy.

**Závěr: PŘED AUDITEM NEBYLO VŠE PRO FINÁLNÍ VZHLED SPECIFIKOVÁNO.** Funkční plán a map-first princip existují, ale zejména ikony, podrobný design systém, vrstvy a vizuální přejímka nebyly kompletní. V tomto PR se konkretizuje [ikonografický kontrakt](../plan/MAPOVE-IKONY-A-POPISKY.md) a doplňují [vizuální pravidla UX](../plan/WEB-UX-UI-SPECIFIKACE.md). **Nejde o implementaci, assety, testované FPS ani úplnou grafickou akceptaci.**

## 1. Matice pokrytí a zbývajících rozhodnutí

| Oblast | Před auditem | Po doplnění | Zůstává otevřeno |
|---|---|---|---|
| Mapa jako hlavní pracovní plocha | Definováno | Map-first princip a layout | End-to-end akceptace G-11 |
| Kompaktní desktop, kontextové panely | Částečně | Doplněny návrhové rozměry a panelové priority | Reálný screenshot/zvolený breakpoint |
| Mobil a tablet, spodní panel | Částečně | Kontrakty pro výšku, touch/focus, safe area | Skutečná zařízení, gesta G-10 |
| Katalog ikon podle typu | Nedostatečné | **Nový dokument** kategorií a atlasu | Veřejné DTO kategorii **neobsahuje**, SVG neexistují |
| Směr a neznámý track | Částečně | Přesné pravidlo tracku a bodu, map bearing | Browser test 0/90/180/270 |
| Velikost, vybraný cíl a hit target | Částečně | Návrh px, halo a dotyku | Ověřit s WCAG/GPU |
| Popisky, výšky a kolize | Částečně | Zoom, text priority, fallbacky | Hustota 2000, výsledný look |
| Tmavý/světlý design systém | Částečně | Návrhové tokeny, typografie, focus/stavy | Kontrast a finální grafická paleta |
| Běžná/vybraná/nouzová barva | Částečně | Priority a pravidla bez falešných dat | G-08 není hotová; validace |
| Pořadí vrstev a legenda | Částečně | Pravidla zobrazení nad mapou a legendy | G-06/G-07 a licence |
| Letištní/NAVAID ikony/detail | Funkčně zadáno | Pravidla odlišení a pravdivosti dat | Originální letištní assety v G-06 |
| ČHMÚ/SIGMET grafika a čas | Funkčně zadáno | Viditelná platnost, vyznačení stale | Skutečné meteo podklady a overlay testy |
| Live, reconnect, offline, prázdno | Částečně | Stavová typologie a české texty | E2E přechody a PWA G-10 |
| Letadlo/detail/telemetrie | Částečně | Pravidla jednotek a chybějících hodnot | Nezaměnit track a heading |
| Statistiky a diagnostika | Funkčně zadáno | Sekundární kontextové rozhraní | G-09 realizace |
| Globální hledání, klávesnice | Částečně | Princip jediného hledání, focus/ESC | G-10 realizace |
| PWA, lokalizace, přístupnost | Funkčně zadáno | Vizuální stavy, WCAG a a11y scénáře | Reálná kontrola G-10 |
| WebGL/tile chyby a návrat | Částečně | Degradace mapy, seznam trvale použitelný | Browser test |
| Logo, favicon a brand | Nedokončeno | Evidováno jako design závislost | **Finální vlastní logo/favikona nebyly graficky schváleny** |
| Práva k ikonám a mapě | Částečně | Vlastní vektorové assety, atribuce | Assety a licence teprve ověřit |
| Referenční screenshot baseline | Nedokončeno | Seznam povinných pohledů | **Žádné finální schválené baseline zatím nejsou** |
| Desktop/mobile GPU benchmark | Pouze plán | Převzetí dle G-05.11/G-10.11 | **Zatím NEOVĚŘENO** |

Status „doplněno“ znamená **existuje návrhová specifikace**, nikdy „hotový kód“.

## 2. Konkrétní zjištění ze zdrojových souborů

1. `RadarMap.tsx` má jednu canvas ikonu `radar-plane` a bod bez směru; **R03 rozlišení kategorií dosud neplní**. Výběr zvětšuje ikonu, ale bez celého katalogu a vizuálního fallback testu.
2. `contracts/aircraft-public.schema.json`, `apps/web/src/radar/aircraft-store.ts`, Rust `PublicAircraft` nemají `emitterCategory` ani ověřenou databázovou informaci typu. Před ikonami podle typu je potřebné prověřit readsb JSON/BEAST kontrakt, validaci, stáří údajů a kompatibilitu SSE.
3. `text-field` čerpá jen `callsign`, výška v mapovém štítku se nyní nevykresluje. Nejsou pravidla pro zoom a priority ve velké hustotě (pouze `text-optional`).
4. `styles.css` má hexa hodnoty v jednotlivých selektorech; neexistuje sjednocená produktová design-token paleta. Dark-first vzhled funguje, ale nemáme přijatý světlý kontrastní design.
5. G-05 map-first layout z PR #26 je již v kódu, ale navazující panely G-06 až G-10 se musejí řídit stejným kontextem bez druhé mapy nebo zbytečného dashboardu.
6. Funkční Chromium CI a první mapové interakce **nejsou totéž co zlatý screenshot baseline / fyzický telefon / měřený GPU benchmark**.

## 3. Co doplnit při implementaci, se skutečnými důkazy

| Priorita | Konkrétní krok | Etapa / ID |
|---|---|---|
| **P0** | Ověřit emitter category zdroj, čas, BEAST/JSON, public DTO a SSE kompatibilitu; nevydávat metadata za automaticky dostupná | G-02/G-03 + G-05.03 / R03 |
| **P0** | Vytvořit vlastní SVG katalog, fallback, správný výběr/rotaci a kategorii | G-05.03 / R03 |
| **P0** | Doplnit zoom popisků, kolize, skutečné `ft`, vybraný cíl, map bearing | G-05.04 / R04 |
| **P0** | 25/100/500/2000, 1/2 Hz, CSS px, FPS/render/paměť, map-style regrese | G-05.11 |
| **P1** | Přijmout jednotné design tokeny a srovnat skutečný kontrast dark/light | G-10.10 / U01 |
| **P1** | Letiště/FIX/NAVAID grafická pravidla nad věrohodnými katalogy | G-05.10/G-06 / R12/A01/A02/T10 |
| **P1** | Zobrazit časové meteo vrstvy/legendu a selhání pouze své vrstvy | G-07 / W03/W06 |
| **P1** | Nouzové kódy s validovaným symbolem/číslem, push dry-run | G-08 / L06 |
| **P1** | Diagnostika a statistiky jako sekundární panely bez nového default dashboardu | G-09 / S01/O01 |
| **P1** | Responsive focus/keyboard/touch, offline i18n a screenshot E2E | G-10 / U01–U08 |
| **P2** | Finální originální logo/favikona a grafické schválení variant | Designová součást existující webové platformy, nikoli nové ID |
| **P2** | Schválit screenshot baseline na různých režimech a validační matici G-11 | G-11 |

## 4. Doporučené scénáře vizuálního schválení

- Desktop **1440×900 a 1920×1080**: mapa přednostně, kompaktní hlavička, zavření panelu, dark/light.
- Tablet **768×1024**, mobil **390×844** a malý **320×568**: bez horizontálního scrollování, otevření a zavření panelu, navigace přes klávesnici, focus a pinch/map pan.
- Mapové cíle při zoomu **7/9/12**, bearing **0°/45°**, true track **0°/90°/180°/270°**, známá a neznámá kategorie, známý a neznámý směr.
- Symboly při **25/100/500/2000** cílech, zvolené letadlo, přeplněná mapa, kolize a výběr; nouzový stav až po implementaci G-08.
- `no data` vs `no results` vs připojování/reconnect vs odpojeno, WebGL/tile outage, skutečně označený offline snapshot.
- ČHMÚ/SIGMET/letiště až když budou G-06/G-07 opravdu implementované a legální; nedemonstrovat placeholder jako produkční data.
- Porovnání výsledného vzhledu s referenčními screenshoty, axe/WCAG kontrast, textový seznam a stav `prefers-reduced-motion`. CI screenshot ≠ naměřené FPS.

## 5. Závěr auditu

**Po této úpravě je zadání podstatně konkrétnější a lze podle něj bezpečně pokračovat v návrhu/implementaci.** Nelze ale říci, že je celý finální design již odsouhlasen: chybí samotné originální SVG, finální logo, přijatá barvová sada po testu kontrastu, skutečné kategoriové pole API, referenční vizuální screenshoty a výkonové měření. Audit tyto závislosti výslovně udržuje jako otevřené.

## Zdroje

- [readsb — JSON `category/t/track`](https://github.com/wiedehopf/readsb/blob/dev/README-json.md)
- [ADS-B Exchange — kategorie emitentů](https://support.adsbexchange.com/hc/en-us/articles/44705224053517-Emitter-Category-ADS-B-DO-260B-2-2-3-2-5-2)
- [MapLibre — symboly, label a overlap](https://maplibre.org/maplibre-style-spec/layers/)
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
