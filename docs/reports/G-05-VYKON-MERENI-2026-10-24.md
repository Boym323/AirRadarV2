# G-05.11 — první skutečné měření MapLibre v Chromium (24. 10. 2026)

**Výsledek:** CI **PASS** v 32 opakovatelných syntetických scénářích, **pouze software-renderovaná baseline**. **Fyzický desktopový ani mobilní GPU benchmark není doložen** a G-05.11 nelze zatím označit za úplně přijaté podle [mapového výkonového kontraktu](../technical/MAPLIBRE-VYKON-A-ZATEZ.md).

**Původ:** [PR #34](https://github.com/Boym323/AirRadarV2/pull/34), implementace `b8691f8b` v `main`; [GitHub Actions #40095503907](https://github.com/Boym323/AirRadarV2/actions/runs/40095503907) 3/3 PASS a [archivovaný JSON artefakt](https://github.com/Boym323/AirRadarV2/actions/runs/40095503907/artifacts) (32 scénářů, ref head SHA `664ce2757e4020d014249151ae0a837e781d6359`). Následující optimalizační [PR #36](https://github.com/Boym323/AirRadarV2/pull/36) byl sloučen jako `3bf49297`, [CI #40096318946](https://github.com/Boym323/AirRadarV2/actions/runs/40096318946) **3/3 PASS**; upravuje běžný render na základě těchto hodnot, ale neospravedlňuje tvrzení o fyzickém GPU.

## Metodika

- **Zařízení:** GitHub-hosted runner `ubuntu-24.04`, headless Google Chrome přes ANGLE **SwiftShader (software)**, stejné čerstvé CI prostředí pro oba režimy; MapLibre ze zamčeného projektu (`maplibre-gl@6.15.0`). Nešlo o fyzický mobil ani GPU počítače uživatele.
- **Profily:** desktop browser viewport 1440×900 a mobilní **emulace** viewportu 390×844 s dotykovým profilem, stejný Chrome engine.
- **Zátěž:** 25/100/500/2 000 identit, 1 Hz a 2 Hz, 4 aktualizace na kombinaci, přesně **10 % geometrií mění skutečné souřadnice** na tick; zbývající ID i štítky jsou stabilní. Celkem `4 × 2 × 2 × 2 = 32` scénářů, 128 měřených aktualizací.
- **Mapové podmínky:** jeden MapLibre canvas, lokální neměnný/legální styl bez tile requests třetích stran; skutečné registrované siluety, vybrané popisky, testováno bez živého RF nebo DB. Inkrementální `updateData` vyžaduje stabilní unikátní feature IDs.
- **Měřítka:** p95 `await source.setData()/updateData()` v ms (latence worker aktualizace, **nikoliv hotový obraz**), requestAnimationFrame intervaly a jejich proxy frekvence, Chrome CDP TaskDuration, `performance.memory.usedJSHeapSize`, DOM mutace, počet canvasů/DOM markerů. **Pouze 4 vzorky na scénář:** hodnoty jsou indikativní baseline, nikoli statistická garance dlouhodobého p95 nebo systémové RAM.

## Naměřená aktualizace GeoJSON — p95 v ms

| Cílů | Desktop 1 Hz set/diff | Desktop 2 Hz set/diff | Mobil emul. 1 Hz set/diff | Mobil emul. 2 Hz set/diff |
|---:|---:|---:|---:|---:|
| 25 | 1,2 / 2,0 | 1,0 / 1,0 | 1,3 / 2,1 | 1,1 / 0,9 |
| 100 | 1,8 / 1,0 | 1,3 / 0,9 | 1,3 / 2,4 | 1,1 / 1,7 |
| 500 | 4,6 / 1,3 | 4,7 / 1,7 | 2,4 / 2,3 | 2,6 / 1,2 |
| 2 000 | **6,6 / 2,1** | **6,3 / 2,5** | **9,8 / 2,1** | **6,1 / 6,7** |

Zkratka **set/diff** = `GeoJSONSource.setData` / `GeoJSONSource.updateData`.

- Při 2 000 cílech a 1 Hz je v tomto běhu diff významně rychlejší: desktop ~3,1×, mobilní emulace ~4,7× podle naměřeného p95 pracovní operace. **Při mobilní emulaci a 2 Hz diff nebyl rychlejší** (6,7 oproti 6,1 ms), tudíž nejde o absolutní výhodu.
- Desktop `requestAnimationFrame` proxy přibližně 42–52 snímků/s, mobilní emulace kolem 60; **nejde o FPS skutečného fyzického GPU ani měření kompletně vykreslených mapových snímků**. P95 RAF odstup desktop nejčastěji ~33,4 ms, mobilní emulace ~16,7 ms, u desktop 2 000 / 2 Hz diff až 50 ms.
- DOM markery **0**, DOM mutace během zdrojových aktualizací **0**, MapLibre canvas instance **1** ve všech 32 scénářích. Koncový Chrome JS heap cca 24–60 MiB podle scénáře; není to celková procesová RSS ani konstantní horní mez.
- Workload je deterministický pouze ve vygenerovaných ID a přesunech; proměnlivá výkonnost hostovaného runneru a headless renderingu může ovlivnit další běh.

## Implementační závěr pro G-05

1. `setData` zachovat při malém provozu a větších změnách/doplnění epochy/stylu; je jednodušší a u 25 letadel může být i rychlejší.
2. `updateData` využít selektivně při min. 250 stabilních cílech a nejvýše 35 % změněných features, s plným fallbackem na `setData`. Konkrétní implementace a regresní testy ve sloučeném [PR #36](https://github.com/Boym323/AirRadarV2/pull/36), [CI #40096318946](https://github.com/Boym323/AirRadarV2/actions/runs/40096318946) PASS. Je to opatrná první heuristika, **ne poslední univerzální benchmarkem dokázané optimum**.
3. Při novém `style.load`, nové SSE epoše a konfliktním ID vždy resetovat předchozí zdroj a provést full set. Při stejném `epoch/seq/selected` neposílat redundantní GeoJSON worker update.
4. Pro G-05.11 **zbývá fyzický výkonový test** na konkrétním desktopovém GPU a mobilním telefonu při pan/zoom/follow s 500 a 2 000 cíli, současné měření reakce UI a reálných 1/2Hz SSE, p95 FPS/render tick, RAM, tile fallback a porovnání s tímto CI baseline. Nevydávat PR #34 za toto potvrzení.
5. G-05.01–10 mají samostatné fungující regresní a DEV důkazy; formálně G-05 **10/11**, zbývající výkonová přejímka je oddělena od další etapy G-06 a produkční V1.

## Jak benchmark opakovat

`npm ci --prefix apps/web && npm run build --prefix apps/web`; na systému s Chrome a připraveným Playwright prostředím spustit `python tools/benchmark_g05_map.py`. Skript lokálně obslouží statický Vite web a v `.artifacts/g05-map-benchmark.json` uloží přesnou matici včetně commit SHA. **Výchozí skript explicitně používá SwiftShader**, proto pro fyzické GPU musí být přizpůsoben hardware a řádně znovu ověřeny výsledky; dostupná CI fakta se tím nezmění.
