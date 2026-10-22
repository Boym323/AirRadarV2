# G-10 — Mobilní web, PWA, offline režim a lokalizace

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** M09 — Vite/React/TypeScript.  
**Závislosti:** G-05 živá mapa a potřebná veřejná API; případně G-08 pro Web Push service worker.  
**Primární KEEP ID:** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`.

## Cíl

Implementovat schválenou funkční část s minimem závislostí, testy a transparentním provozním dopadem.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-10.01 — Responzivní rozložení**  
  **Implementace:** Navrhni adaptivní mapu, list/panely a ovládání pro mobil/tablet/desktop; žádný zdvojený UI state.  
  **Ověření:** Playwright scénáře tří viewportů, bez horizontálního scrollu a nepřístupných ovladačů.

- [ ] **G-10.02 — Dotykové ovládání**  
  **Implementace:** Zajisti touch/pinch mapu, bezpečné rozbalení seznamu a focus po návratu z panelu.  
  **Ověření:** Test touch a keyboard nemá konfliktní event handlery.

- [ ] **G-10.03 — PWA manifest**  
  **Implementace:** Doplň manifest, správné ikony, start_url/scope, display a český název; neinstaluj nepotřebný background sync.  
  **Ověření:** Lighthouse/automatická kontrola manifestu a browser installability.

- [ ] **G-10.04 — Service worker strategie**  
  **Implementace:** Cache statické soubory verzovaně, ale síťový API/SSE stream nikdy nepředstírej jako live cache.  
  **Ověření:** Test offline odliší statický web od nedostupného živého streamu.

- [ ] **G-10.05 — Poslední známý stav offline**  
  **Implementace:** Zobraz pouze schválený omezený snapshot s viditelným datem poslední aktualizace a výrazným štítkem neaktuálnosti.  
  **Ověření:** Při offline režimu nelze zaměnit cached polohy za živý provoz.

- [ ] **G-10.06 — České texty**  
  **Implementace:** Centralizuj cs-CZ texty chyb, menu, času, výšky a rychlosti; žádná nepřeložená viditelná výchozí stránka.  
  **Ověření:** Automatický test všech základních obrazovek v češtině.

- [ ] **G-10.07 — Volitelná angličtina**  
  **Implementace:** Přidej jednoduché přepnutí cs/en, uloženou preferenci a fallback českého textu; API klíče nepřekládej.  
  **Ověření:** E2E přepínání jazyka nepřepisuje data letadel ani jednotky.

- [ ] **G-10.08 — Globální hledání**  
  **Implementace:** Jedno pole/command menu hledá pouze existující letadla, letiště a schválené akce v mapě.  
  **Ověření:** Test nenabízí neimplementované funkce nebo položky bez skutečného cíle.

- [ ] **G-10.09 — Klávesové zkratky**  
  **Implementace:** Zaved zkratky pro hledání, mapu a panely, s vyloučením fokusovaných inputů a konfliktních browser kláves.  
  **Ověření:** Keyboard E2E správně ovládá web i se screenreader texty.

- [ ] **G-10.10 — Přístupnost a pohyb**  
  **Implementace:** Ověř aria-label, focus visibility, reduced-motion a barevný kontrast; mapa musí mít přístupný textový seznam.  
  **Ověření:** Automatická axe/Playwright kontrola a manuální scénář se základní klávesnicí.

- [ ] **G-10.11 — Mobilní zátěž**  
  **Implementace:** Při měřeném počtu cílů otestuj FPS, LCP/INP, paměť, offline návrat a reconnect po změně sítě.  
  **Ověření:** Report odlišuje browser výkon od backend SSE a neslibuje data bez měření.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-10-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Všechna primárně přidělená ID mají vlastní testovací důkaz a odpovídající status; dokumentace nevydává neprovedený krok za hotový. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.
