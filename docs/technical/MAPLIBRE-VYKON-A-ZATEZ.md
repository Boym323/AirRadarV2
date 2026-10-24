# MapLibre GL JS — vysoká hustota letadel, výkon a ergonomie

**Stav:** technické zadání G-05 s prvním [naměřeným Chromium baseline 24. 10. 2026](../reports/G-05-VYKON-MERENI-2026-10-24.md). Headless SwiftShader **není fyzický GPU benchmark** a formální G-05.11 zůstává otevřený. **Aktivní ID:** R01, R03, R04, R06, R08, R09, R12, R14, U01. Použít **Vite + React + MapLibre**, nikoli další framework.

Při realizaci R03/R04 respektovat [ikonografii](../plan/MAPOVE-IKONY-A-POPISKY.md): vlastní atlas, pravdivá emitter category, fallback pro neznámý směr, kontrastní popisky a validované sprite po `style.load`. Screenshot CI není důkaz GPU výkonu.

## První měření: CI #40095503907 — software GPU, nikoli fyzické zařízení

Reprodukovatelných **32 scénářů**: 25/100/500/2 000 cílů, 1/2 Hz, `setData` vs `updateData`, viewport 1440×900 a mobilní emulace 390×844; 10 % změněných geometrií na tick, 4 vzorky na scénář. Všechny testy prošly. Při 2 000 cílech / 1 Hz bylo p95 metody `setData` 6,6 ms desktop a 9,8 ms mobile emulation; `updateData` 2,1 ms v obou profilech. Při 2 000 / 2 Hz na mobilní emulaci se diff nezlepšil (6,7 proti 6,1 ms).

**Naměřené RAF proxy FPS nejsou fyzické GPU FPS.** Kompletní čísla, hardware, verze, RAM omezení, p95 metodika a odkazy na JSON artefakt: [G-05 — výkonové měření](../reports/G-05-VYKON-MERENI-2026-10-24.md). Adaptivní diff v [PR #36](https://github.com/Boym323/AirRadarV2/pull/36) sloučen (`3bf49297`), [CI #40096318946](https://github.com/Boym323/AirRadarV2/actions/runs/40096318946) PASS; stále není důkazem finální mobilní fyzické přejímky.

## Renderovací architektura

1. **Jedna trvalá MapLibre instance** po celou dobu zobrazení stránky. Nedělat nový \`Map()\` při SSE zprávě nebo změně vybraného letadla; React je pouze životní cyklus a běžné ovládání.
2. Letadla jsou **jeden GeoJSON Source + symbol layer** s minimálními vlastnostmi \`id\`, \`lon\`, \`lat\`, \`heading\`, \`icon\`, \`callsign\`, \`selected\`, \`stale\`. Nepoužívat tisíce individuálních React/DOM Marker instancí. Stable \`feature.id\` = normalizovaná identita.
3. \`icon-image\` používat ze sprite atlasu či předem registrovaných SDF/ikon, \`icon-rotate\` pro skutečný směr, \`symbol-placement\`/overlap pravidla pro popisky. Nečíst React state všech letadel v každé renderované komponentě.
4. SSE delta aplikovat do malého JS store (\`Map<string, Aircraft>\`), sloučit změny do fronty a **max. 1 render update/s** na počátku. Změny seznamu, filtru a výběru publikovat odděleně; žádný \`setState\` tisíckrát za sekundu.
5. \`setData()\` (celý source) je nejjednodušší baseline. \`updateData()\` (diff) použít teprve s verzí MapLibre, která metodu nabízí, se **stabilními unikátními IDs** a po skutečném porovnání CPU/frame latency. Dokumentace uvádí povinnost unikátních ID, jinak jde o chybu.
6. Stopy R08, letiště R12, meteorologické vrstvy a značky držet odděleně, protože mají jiné frekvence aktualizací. Nesdílet high-frequency aircraft source se statickými letišti. Stopa v RAM max 20 minut / 64 bodů na identitu podle vstupních limitů.
7. Pokud uživatel pohybuje mapou, nepřerušovat drag či zoom kvůli častému \`fitBounds\`; režim follow R09 má jasnou aktivaci, výběr i ukončení při ručním gestu.
8. Popisky musí při přiblížení zůstat čitelné bez nekonečného překreslování. Délku callsign a volitelná metadata předat sanitizovaně; neznámá hodnota se neháda.

## Simulovaná testovací zátěž pro porovnání strategií

| Scénář | Letadel na mapě | Frekvence změn | Vrstvy navíc |
|---|---:|---:|---|
| M1 — malý lokální provoz | 50 | 1 Hz | pouze letadla |
| M2 — běžný provoz | 250 | 1 Hz | letadla + letiště |
| M3 — hustý provoz | 500 | 1 Hz | letadla + popisky + trail |
| M4 — vysoký provoz | 1 000 | 1–2 Hz | plné aktivní vrstvy |
| M5 — extrém | 2 000 | 2–5 Hz (zátěžový limit, ne plán) | navíc pan/zoom/follow |
| M6 — hraniční případ | 5 000 | 1 Hz | ověřit limit stavu/SSE a degradaci UI |

Každý dataset je syntetický a **deterministický** (stejná seed hodnota, počty a změny). Srovnávat \`setData\` a \`updateData\`, s popisky zapnutými/vypnutými, na stejném stylu mapy, GPU, prohlížeči a verzi MapLibre. Externí mapové podklady během benchmarků nahradit lokálním/fixovaným podkladem s legální licencí, aby síť poskytovatele neznehodnocovala FPS.

## Co měřit

- Browser \`requestAnimationFrame\` intervaly, p50/p95 frame time a *skutečné* FPS, JS long tasks, React Profiler commit count, WebGL worker/CPU a peak heap.
- Čas od přijetí delta do GeoJSON source a od \`setData/updateData\` po skutečně překreslený obraz.
- P95 klikání na letadlo, přepnutí filtru, přiblížení, pan a follow; kolize popisků.
- Ztrátu WebGL contextu, neúspěšný tile fetch, chybějící ikonu či deaktivovaný WebGL; zobrazit chybový stav a zachovat seznam provozu.
- Rozdíl desktop (skutečný GPU) a střední třída telefonu. **Playwright headless v CI není věrohodný referenční hardware pro GPU benchmark**, proto CI používáme především pro funkční regrese a vizuální konzistenci.

**Startovací rozpočet, nikoli naměřená propustnost:** pro M3 (500 letadel) preferujeme stabilní interakci bez dlouhých záseků; cílově desktop aspoň **30 FPS** a mobil aspoň **20 FPS** během pan/zoom s živým streamem, P95 render tick nejvýše 50 ms. Pokud skutečné testy limit nesplní, nejprve přejít na diff, snížit properties/popup count a frekvenci zobrazení. Neodstraňovat potichu provoz, neprovádět serverový sampling jen proto, aby výsledky vypadaly rychlejší.

## Přejímka G-05

- [ ] Stabilní identita mapových symbolů, správný úhel a \`removed\` event odstranit pouze správný cíl.
- [ ] React instance MapLibre je dlouhodobá, SSE update nezpůsobuje full remount ani tisíce React commitů.
- [ ] M1–M6 reprodukovatelné na stejné sadě fixture, výkon desktop/mobil s datem, verzí, GPU a metodikou.
- [ ] \`setData\` vs \`updateData\` porovnáno měřením; jednodušší varianta zvolena, není-li diff přínosný.
- [ ] UI není závislé na živém přijímači při testování; výpadek mapy nevyřadí seznam letadel.
- [ ] Vizuální kontrakty a mapové interakce mají funkční Playwright testy (viz další dokument).

## Zdroje

- [MapLibre — výkon velkých GeoJSON dat](https://maplibre.org/maplibre-gl-js/docs/guides/large-data/)
- [MapLibre — GeoJSONSource, setData/updateData a podmínka stabilních ID](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/)
