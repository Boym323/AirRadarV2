# G-05 — MapLibre mapa v již připraveném Vite webu

**Stav:** ROZPRACOVÁNO; [PR #22](https://github.com/Boym323/AirRadarV2/pull/22) sloučen 23. 10. 2026, DEV nasazení [#40028968380](https://github.com/Boym323/AirRadarV2/actions/runs/40028968380) PASS. 4 z 11 kroků jsou formálně ověřené, G-05.08 má navazující opravu a čeká na nové CI; úplný benchmark a letiště nejsou hotové.  
**Odpovědnost:** M04 — Vite/React/MapLibre.  
**Závislosti:** G-03; letištní vrstva se dokončí po G-06, ostatní části lze provést dříve.  
**Primární KEEP ID:** `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

**UX návaznost:** [Map-first UX/UI specifikace](../plan/WEB-UX-UI-SPECIFIKACE.md) stanoví mapu jako výchozí hlavní plochu a seznam/výběr/detail jako kontextové ovládání. První G-05 dvojpanelové rozložení je přechodný stav; přestavba nesmí narušit SSE ani měření G-05.11. Specifikace nepřidává nové ID ani nezaškrtává neověřené kroky.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [x] **G-05.01 — Založit MapLibre klienta**  
  **Implementace:** Přidej MapLibre do Vite a jedinou stabilní Map instanci; SSR není potřeba, nepřeváděj Next.js ani vytvářej druhý mapový engine.  
  **Ověření:** Browser smoke potvrdí mapu bez opakovaných remountů a bez chyb WebGL.

- [x] **G-05.02 — SSE stavový adapter**  
  **Implementace:** Založ úzký klient nad /api/v1/stream; aplikuj snapshot/delta na vlastní browser store s kontrolou sekvence.  
  **Ověření:** Unit test normalizuje, maže a obnoví tracky při resync.

- [ ] **G-05.03 — Symbolová vrstva**  
  **Implementace:** Použij jeden GeoJSON source a layer pro ikony dle skutečně známého typu a směru, stabilní feature IDs.  
  **Ověření:** Správné směry, bez fiktivních defaultních vlastností a duplicit.

- [ ] **G-05.04 — Čitelné popisky**  
  **Implementace:** Zobraz stručné popisky jen pro dostupný callsign/výšku, MapLibre collision a minimalizovanou práci React.  
  **Ověření:** Test hustého provozu nepřekrývá obrazovku nekontrolovanými DOM markery.

- [x] **G-05.05 — Seznam a vyhledávání v provozu**  
  **Implementace:** Vytvoř živý seznam letadel se základními filtry a řazením; používej stejný state jako mapa.  
  **Ověření:** Seznam a mapa jsou konzistentní po ztrátě/reconnect streamu.

- [x] **G-05.06 — Výběr cíle**  
  **Implementace:** Kliknutí v mapě nebo seznamu aktivuje výběr pro stopu/follow; nevytvářet zbytečný samostatný analytický panel.  
  **Ověření:** Výběr reaguje správně i při zániku tracku a opětovném výskytu.

- [ ] **G-05.07 — Skutečná letová stopa**  
  **Implementace:** Kresli krátkou omezenou stopu jen z validních přijatých poloh, s TTL a maximálním počtem bodů.  
  **Ověření:** Test retence odřízne staré body a nevytváří budoucí predikce.

- [ ] **G-05.08 — Follow kamera**  
  **Implementace:** Volitelně sleduj vybrané letadlo s možností okamžitě vypnout sledování interakcí uživatele.  
  **Ověření:** Playwright ověří výběr, follow stop a nezaseknutou kameru. Doplněno vypnutí follow při ručním wheel/touchmove/keyboard a drag/rotate/pitch; nový Chromium test čeká na CI a G-05.08 zůstává neuzavřená do doložení všech scénářů.

- [ ] **G-05.09 — Výběr mapového podkladu**  
  **Implementace:** Implementuj jen funkční schválené mapové styly a jejich bezpečné URL; preference mohou být lokálně uložené.  
  **Ověření:** Test změny stylu zachová live tracky a výběr.

- [ ] **G-05.10 — Vrstva letišť**  
  **Implementace:** Po dokončení G-06 zobraz letištní body nad stabilním zdrojem; neduplikuj katalog ve webu.  
  **Ověření:** Zobrazené letiště vede ke správnému katalogovému detailu a API.

- [ ] **G-05.11 — Výkon a regresní E2E**  
  **Implementace:** Porovnej setData a updateData pouze podle skutečných dat; změř 25/100/500/2000 cílů, 1/2 Hz, FPS, re-renders a paměť.  
  **Ověření:** Důkaz s modelem zařízení, SHA a minimálně desktop+mobil browser scénář.

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
