# G-06 — Lehký katalog letišť, frekvencí a NAVAID

**Stav 24. 10. 2026:** ROZPRACOVÁNO — build-time veřejný OurAirports katalog pro G-05.10. Jednotlivé G-06 checkpointy zůstávají otevřené do CI a úplného REST/NAVAID/frekvenčního modelu. [Průběžný report](../reports/G-06-VYSLEDEK.md).  
**Odpovědnost:** M05 — Rust + schválené statické datové podklady.  
**Závislosti:** G-01, G-03; G-04 jen pokud konkrétní čtení skutečně potřebuje DB.  
**Primární KEEP ID:** `A01`, `A02`, `T10`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

**Vizuální návaznost (24. 10. 2026):** [map-first UX](../plan/WEB-UX-UI-SPECIFIKACE.md), [ikony a mapové popisky](../plan/MAPOVE-IKONY-A-POPISKY.md), [audit úplnosti](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md): Letiště, dráhy a NAVAID/FIX musejí mít vlastní odlišitelný tvar a přístupný kontextový detail; nevydávat neověřené frekvence/zdroje, nezahlcovat mapu a nechat funkční živá letadla. Tento dokumentační odkaz neuzavírá žádný krok.

**Předběžný implementovaný základ:** statický read-only JSON `/data/airports-ourairports.json` generovaný při buildu z přesné revize public-domain OurAirports, bez další služby či synchronizačního daemona; vykreslení v MapLibre jako samostatná vrstva nad mapou a nezávisle na SSE. Není to ještě plný REST G-06.05 ani dokončená etapa.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [ ] **G-06.01 — Přijmout zdroje a licence**  
  **Implementace:** Dolož původ, formát, frekvenci aktualizací a povolení použití pro letiště, dráhy/frekvence a NAVAID/FIX; bez licence data nevydávej.  
  **Ověření:** V repozitáři je původ/verze datasetu a test, že aplikace nevydává nepovolená data.

- [ ] **G-06.02 — Minimální katalogový model**  
  **Implementace:** Definuj ICAO/IATA jen kde existuje, souřadnice, dráhy, publikované frekvence a FIX/NAVAID s přesnými jednotkami.  
  **Ověření:** Model odlišuje neznámý údaj od falešné nuly či odhadnuté frekvence.

- [ ] **G-06.03 — Verzovaný import**  
  **Implementace:** Implementuj jednorázový kontrolovaný import nebo build-time generátor z validovaného podkladu; žádný běžící synchronizační daemon.  
  **Ověření:** Parser odmítá chybnou hlavičku, duplicity, neplatné souřadnice a chybnou verzi.

- [ ] **G-06.04 — Důvěryhodnost frekvencí**  
  **Implementace:** U zveřejněných frekvencí eviduj zdroj, službu, rozsah platnosti a datum; nevymýšlej další ATC funkce.  
  **Ověření:** Fixture prokáže zachování publikovaných frekvencí bez fiktivního přiřazení letadla.

- [ ] **G-06.05 — REST katalogu letišť**  
  **Implementace:** Zprovozni vyhledání ICAO/názvu a detail základních letištních dat s limity stránkování.  
  **Ověření:** Kontraktový test pro neexistující kód, duplicitu a validní detail.

- [ ] **G-06.06 — NAVAID a FIX**  
  **Implementace:** Připrav seznam/bodové vyhledání navigačních prostředků pouze dle schválených dat.  
  **Ověření:** Porovnání golden datasetu a prostoru ukáže správné typy/pozice.

- [ ] **G-06.07 — Interakce s mapou**  
  **Implementace:** Doplň API GeoJSON/kompaktní DTO potřebné pro R12 a případný výběr v G-05; neposílej kompletní katalog při každé SSE deltě.  
  **Ověření:** Katalog se načte samostatně, letový stream se nezvětší o statická data.

- [ ] **G-06.08 — Výkon a aktualizace**  
  **Implementace:** Změř velikost souboru, memory footprint a latenci hledání; velký katalog nedrž duplicitně ve více procesech.  
  **Ověření:** Report s verzí datasetu, pokrytím, časem importu a vyhledávání.

- [ ] **G-06.09 — Kontroly a akceptace**  
  **Implementace:** Test importu, lokalizace názvů a chybějících údajů v CI; zdokumentuj ruční aktualizační postup.  
  **Ověření:** A01/A02/T10 mají konkrétní testovací důkaz a nulový vliv na přijímání letadel.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-06-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.
