# G-07 — ČHMÚ radar, platné SIGMET a BDS počasí

**Stav:** NEZAHÁJENO (podrobné zadání; nejde o implementaci).  
**Odpovědnost:** M06 — Rust/MapLibre.  
**Závislosti:** G-02, G-03 a G-05; datové zdroje až po ověření licence.  
**Primární KEEP ID:** `W03`, `W06`, `W08`, `W09`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

**Vizuální návaznost (24. 10. 2026):** [map-first UX](../plan/WEB-UX-UI-SPECIFIKACE.md), [ikony a mapové popisky](../plan/MAPOVE-IKONY-A-POPISKY.md), [audit úplnosti](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md): ČHMÚ/SIGMET vrstvy patří pod letecké symboly, mají průkaznou legendu a časovou platnost. Při jejich výpadku zůstane lokální ADS-B a seznam letadel nedotčen. Tento dokumentační odkaz neuzavírá žádný krok.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [ ] **G-07.01 — Doložit meteorologické zdroje**  
  **Implementace:** Ověř aktuální dostupnost/licenci/geografické pokrytí radarových dat ČHMÚ a SIGMET; nevymýšlej endpointy.  
  **Ověření:** Ve vývojovém reportu je název poskytovatele, formát, limity a alternativa při výpadku.

- [ ] **G-07.02 — Oddělit cache od ingest**  
  **Implementace:** Zaveď oddělené async fetch úlohy s timeouty, rate-limity, TTL, bounded kapacitou a degradací.  
  **Ověření:** Test potvrdí, že výpadek poskytovatele nezbrzdí readsb ingest.

- [ ] **G-07.03 — ČHMÚ radar snapshot**  
  **Implementace:** Získávej radarové snímky s časem platnosti, kontrolou rozlišení a mapovou transformací.  
  **Ověření:** Snímek se zobrazí ve správném čase a souřadnicích bez vydávání stale za live.

- [ ] **G-07.04 — Časová osa radarových snímků**  
  **Implementace:** Doplň pouze nezbytné ovládání času v mapě, bez obecného mapového layer engine.  
  **Ověření:** Test posunu historie ukáže správný timestamp a odstranění starého obrázku.

- [ ] **G-07.05 — SIGMET kontrakt**  
  **Implementace:** Validuj polygony, horní/dolní hladinu, platnost a druh zprávy; nevalidní SIGMET bezpečně vynech.  
  **Ověření:** Golden test prostorové hranice, časového překryvu a prošlé výstrahy.

- [ ] **G-07.06 — Mapová vizualizace SIGMET**  
  **Implementace:** Kresli oddělenou lehkou vrstvu s vysvětlivkou a časovou platností; nezasahuj do mapového toku letadel.  
  **Ověření:** Screenshot/E2E ověří zapnutí, vypnutí a expired stav.

- [ ] **G-07.07 — Mode-S BDS 4,4 dekódování**  
  **Implementace:** Z lokálních validních rámců vyčítej skutečně dostupná meteorologická pole s přesnou definicí BDS a jednotek; bez falešných hodnot.  
  **Ověření:** Golden BDS test a kontrola, že nepodporovaný rámec žádné měření nevytvoří.

- [ ] **G-07.08 — Kvalita pozorování**  
  **Implementace:** Pro W09 ukládej/ukazuj jen časy, kvalitu a reálně přijatá měření; jasně označ chybějící hodnoty.  
  **Ověření:** Test nespojuje hodnoty různých letadel a nevyrábí nepodložené trendy.

- [ ] **G-07.09 — Chybové stavy providerů**  
  **Implementace:** Otestuj 429, 500, timeout, chybný formát, prošlý SIGMET, přerušený radarový feed a pozdější obnovu.  
  **Ověření:** Aplikace ukáže degradaci a zachová lokální letadla beze změny.

- [ ] **G-07.10 — Výkon a akceptace**  
  **Implementace:** Změř CPU/RSS/cache size/tile síť a latenci radaru se zapnutým i vypnutým počasím.  
  **Ověření:** Report obsahuje aktuální zdroje, licenční podmínky, testy a doklad o omezených nákladech.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-07-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.
