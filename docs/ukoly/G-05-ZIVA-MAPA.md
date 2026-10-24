# G-05 — MapLibre mapa v již připraveném Vite webu

**Navazující G-05 map-first UI (24. 10.):** implementováno nové kompaktní rozložení mapy, skrývatelný seznam a kontextový detail v izolované větvi; nová Chromium regresní CI #40079234896 PASS, PR #26 sloučen do main na `c17601d5`. Tento krok realizuje již existující R01/R06/R08/R09/R14 a schválenou `WEB-UX-UI-SPECIFIKACE.md`, nikoli nové produktové ID. Formálně **10/11** po CI #40087996178 a #40094724356 a DEV #40095020297. G-05.11 zůstává otevřená pro prokazatelné FPS/p95/heap a fyzický GPU test.

**Stav:** ROZPRACOVÁNO; [PR #22](https://github.com/Boym323/AirRadarV2/pull/22) sloučen 23. 10. 2026, DEV nasazení [#40028968380](https://github.com/Boym323/AirRadarV2/actions/runs/40028968380) PASS. **10 z 11 kroků** je formálně otestováno; mapová vrstva letišť, popisky a stopa mají Chromium a DEV důkazy. Zbývá G-05.11 výkon (SwiftShader baseline a skutečné GPU).  
**Odpovědnost:** M04 — Vite/React/MapLibre.  
**Závislosti:** G-03; letištní vrstva se dokončí po G-06, ostatní části lze provést dříve.  
**Primární KEEP ID:** `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

**UX návaznost:** [Map-first UX/UI specifikace](../plan/WEB-UX-UI-SPECIFIKACE.md) stanoví mapu jako výchozí hlavní plochu a seznam/výběr/detail jako kontextové ovládání. První G-05 dvojpanelové rozložení je přechodný stav; přestavba nesmí narušit SSE ani měření G-05.11. Specifikace nepřidává nové ID ani nezaškrtává neověřené kroky.

**Ikony a vizuální přejímka (24. 10.):** závazný podklad [MAPOVE-IKONY-A-POPISKY.md](../plan/MAPOVE-IKONY-A-POPISKY.md) a [audit vizuálního zadání](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md). Specifikovány vlastní siluety podle **validované** emitter category, track a bezsměrový fallback, velikosti, barvy, kolize a testy. **PR #30** sloučen jako `9c398c2f`; zpětně kompatibilní validované `emitterCategory` přichází z readsb JSON a BEAST DF17 (ADR 0015), [CI #40084080300](https://github.com/Boym323/AirRadarV2/actions/runs/40084080300) **3/3 PASS**, izolovaný SQL test [#40084080298](https://github.com/Boym323/AirRadarV2/actions/runs/40084080298) PASS. Tato dokumentační změna neuzavírá G-05.03/.04 ani žádné jiné checkboxy.

## Výstup etapy

**Pokračování implementace G-05.03/.04 (24. 10. 2026; [PR #29](https://github.com/Boym323/AirRadarV2/pull/29), [CI 3/3 PASS](https://github.com/Boym323/AirRadarV2/actions/runs/40082045615), sloučeno jako `8309f069`):** nově existuje `apps/web/src/radar/aircraft-icons.ts` s vlastními Path2D siluetami, bezpečnou kategoriovou volbou/fallbackem a mapová vrstva s halo/štítky podle zoomu. **PR #30 sloučen:** G-05.03 má testované originální ikony, validovaný datový kontrakt a browser smoke pro 2 000 cílů. G-05.04 a G-05.11 stále potřebují měřenou kolizní/grafickou přejímku. [Průběžný report](../reports/G-05-VYSLEDEK.md).



Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [x] **G-05.01 — Založit MapLibre klienta**  
  **Implementace:** Přidej MapLibre do Vite a jedinou stabilní Map instanci; SSR není potřeba, nepřeváděj Next.js ani vytvářej druhý mapový engine.  
  **Ověření:** Browser smoke potvrdí mapu bez opakovaných remountů a bez chyb WebGL.

- [x] **G-05.02 — SSE stavový adapter**  
  **Implementace:** Založ úzký klient nad /api/v1/stream; aplikuj snapshot/delta na vlastní browser store s kontrolou sekvence.  
  **Ověření:** Unit test normalizuje, maže a obnoví tracky při resync.

- [x] **G-05.03 — Symbolová vrstva**  
  **Implementace:** Použij jeden GeoJSON source a layer pro ikony dle skutečně známého typu a směru, stabilní feature IDs; mapování a fallback podle [ikonografického kontraktu](../plan/MAPOVE-IKONY-A-POPISKY.md).  
  **Ověření:** Správné směry, bez fiktivních defaultních vlastností a duplicit.

- [x] **G-05.04 — Čitelné popisky**  
  **Implementace:** Zobraz stručné popisky jen pro dostupný callsign/výšku, MapLibre collision a minimalizovanou práci React. Použít [priority a kolizní testy](../plan/MAPOVE-IKONY-A-POPISKY.md) na zoom a hustý provoz.  
  **Ověření:** Test hustého provozu nepřekrývá obrazovku nekontrolovanými DOM markery. PR #32, [CI #40087996178](https://github.com/Boym323/AirRadarV2/actions/runs/40087996178) PASS: 2 000 cílů, sanitizace, výška, MapLibre text priority; DEV #40088791328 SUCCESS.

- [x] **G-05.05 — Seznam a vyhledávání v provozu**  
  **Implementace:** Vytvoř živý seznam letadel se základními filtry a řazením; používej stejný state jako mapa.  
  **Ověření:** Seznam a mapa jsou konzistentní po ztrátě/reconnect streamu.

- [x] **G-05.06 — Výběr cíle**  
  **Implementace:** Kliknutí v mapě nebo seznamu aktivuje výběr pro stopu/follow; nevytvářet zbytečný samostatný analytický panel.  
  **Ověření:** Výběr reaguje správně i při zániku tracku a opětovném výskytu.

- [x] **G-05.07 — Skutečná letová stopa**  
  **Implementace:** Kresli krátkou omezenou stopu jen z validních přijatých poloh, s TTL a maximálním počtem bodů.  
  **Ověření:** Test retence odřízne staré body a nevytváří budoucí predikce. PR #32, [CI #40087996178](https://github.com/Boym323/AirRadarV2/actions/runs/40087996178) PASS: stopa skutečně vykreslena (124 pixelů), segmentace >120 s a antimeridiánu, odstranění ztracené identity; DEV #40088791328 SUCCESS.

- [x] **G-05.08 — Follow kamera**  
  **Implementace:** Volitelně sleduj vybrané letadlo s možností okamžitě vypnout sledování interakcí uživatele.  
  **Ověření:** Playwright ověří výběr, follow stop a nezaseknutou kameru. Doplněno vypnutí follow při ručním wheel/touchmove/keyboard a drag/rotate/pitch; Chromium CI #40084080300 PASS; G-05.08 je TESTED pro popsaná gesta, plná fyzická mobilní přejímka zůstává v G-10.

- [x] **G-05.09 — Výběr mapového podkladu**  
  **Implementace:** Implementuj jen funkční schválené mapové styly a jejich bezpečné URL; preference mohou být lokálně uložené.  
  **Ověření:** Test změny stylu zachová live tracky a výběr.

- [x] **G-05.10 — Vrstva letišť**  
  **Implementace:** Zobraz letiště z verzovaného veřejného OurAirports katalogu připraveného build-time G-06.01/.02/.03/.07, bez duplicitního přenosu v G-03 SSE.  
  **Ověření:** Zobrazené letiště vede ke správnému katalogovému detailu a API. [PR #33](https://github.com/Boym323/AirRadarV2/pull/33), [CI #40094724356](https://github.com/Boym323/AirRadarV2/actions/runs/40094724356) PASS: 884 reálných validovaných letišť, /data/airports-ourairports.json, pixelový test bodu, kliknutí/detail a zachování při style switch; [DEV #40095020297](https://github.com/Boym323/AirRadarV2/actions/runs/40095020297) SUCCESS. G-06.05 full vyhledávací REST, frekvence a NAVAID zůstávají výslovně v navazující G-06, nikoli G-05.

- [ ] **G-05.11 — Výkon a regresní E2E**  
  **Implementace:** Porovnej setData a updateData pouze podle skutečných dat; změř 25/100/500/2000 cílů, 1/2 Hz, FPS, re-renders a paměť.  
  **Ověření:** Důkaz s modelem zařízení, SHA a minimálně desktop+mobil browser scénář. První [report 32 scénářů](../reports/G-05-VYKON-MERENI-2026-10-24.md) z [CI #40095503907](https://github.com/Boym323/AirRadarV2/actions/runs/40095503907) PASS poskytuje software Chromium SwiftShader baseline a p95 `setData` vs `updateData`; **nenahrazuje fyzický desktop/mobil GPU test, pan/zoom/follow a delší měření**. G-05.11 ponechat [ ] do této brány.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-05-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.

## Návrhové technické specifikace (22. 10. 2026)

- [MapLibre výkon a benchmark](../technical/MAPLIBRE-VYKON-A-ZATEZ.md)
- [Playwright E2E scénáře](../technical/PLAYWRIGHT-BROWSEROVE-TESTY.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.
