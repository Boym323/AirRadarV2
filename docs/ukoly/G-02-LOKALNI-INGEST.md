# G-02 — Lokální readsb/BEAST ingest a jediný RAM state

**Stav:** **ROZPRACOVÁNO — 5/11 testově doložených kroků; ostatní částečné**. [Průběžný report](../reports/G-02-VYSLEDEK.md), [CI #39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657).  
**Odpovědnost:** M01 — Rust/Tokio.  
**Závislosti:** G-01 kontrakty a golden fixtures.  
**Primární KEEP ID:** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [x] **G-02.01 — Oddělit normalizaci od transportu**  
  **Implementace:** Založ úzké Rust moduly pro JSON reader, BEAST reader, parser, state a supervisor; nepřidávej mikroservisy.  
  **Ověření:** Modulové testy lze spustit bez fyzického přijímače.

- [x] **G-02.02 — Readsb JSON snapshot adapter**  
  **Implementace:** Pravidelně čti lokální readsb JSON s timeouty, velikostním limitem a atomic read; zpracuj prázdný/neúplný snapshot.  
  **Ověření:** Fixture test nevyvolá panic ani falešnou polohu, stav zdroje je měřitelný.

- [x] **G-02.03 — BEAST TCP framing**  
  **Implementace:** Implementuj asynchronní čtení, escapování 0x1A, délky typů a rekonstrukci po rozděleném TCP chunku.  
  **Ověření:** Golden test zvládne fragmentaci, více rámců v chunku i záměrně poškozený vstup.

- [ ] **G-02.04 — Mode-S a CRC24**  
  **Implementace:** Dekóduj pouze potřebná pole a DF, správně odliš PI/AP/CRC podle typu zprávy; chybný rámec bez důkazu nepovyšuj na spolehlivý údaj.  
  **Ověření:** Ověř soulad s fixture a známými vektory Mode-S, včetně špatné parity.

- [ ] **G-02.05 — Poloha a CPR**  
  **Implementace:** Zpracuj korektní párování CPR a časovou/fyzikální plausibilitu; pokud pozici nejde spolehlivě získat, vynechej ji.  
  **Ověření:** Test pokryje jednoprvkový CPR, lokální/globální dekódování a neplatné souřadnice.

- [ ] **G-02.06 — Kanonický stav letadla**  
  **Implementace:** Založ jeden Rust vlastník track mapy, stabilní ID, freshness, výšku, rychlost, squawk a odlišená missing data.  
  **Ověření:** Syntetický replay generuje správný stav bez duplicit při změně callsign.

- [x] **G-02.07 — TTL a deduplikace**  
  **Implementace:** Nastav konfigurovatelné bounded TTL a odstranění stale tracků, spoj duplicitní pozorování a změř počet aktivních cílů.  
  **Ověření:** Replay se zpožděnými a obrácenými časy neobnoví starou chybnou polohu.

- [ ] **G-02.08 — Arbitráž a fallback**  
  **Implementace:** Jeden supervisor volí platný BEAST nebo JSON při výpadku, restartuje spojení s omezeným backoff/jitter; obě cesty zapisují do stejné RAM.  
  **Ověření:** Test změní dostupnost zdrojů vícekrát a zachová jedinou identitu tracku.

- [x] **G-02.09 — Backpressure a limity**  
  **Implementace:** Omez velikost TCP bufferu, JSON payloadu, počtu tracků a front; při zátěži zaznamenej drop a health bez blokace.  
  **Ověření:** Při syntetickém přetížení nepřibývá paměť neomezeně a klient dostane správný health.

- [ ] **G-02.10 — Deterministický replay**  
  **Implementace:** Připrav spustitelný lokální replay bez připojení na produkční readsb, který ověří všechna ID M01.  
  **Ověření:** CI cargo test a replay PASS, chybné zprávy způsobí explicitní chybu místo pádu.

- [ ] **G-02.11 — Výkonová evidence**  
  **Implementace:** Změř na stejných fixture 1×, 2× a 4× tok, CPU, RSS, p95 ingest latence a reconnect; baseline zapiš do reportu.  
  **Ověření:** Report obsahuje SHA, hardware, testovací data, chyby, naměřené hodnoty a reziduální rizika.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-02-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.

## Návrhové technické specifikace (22. 10. 2026)

- [Kontrakty readsb/BEAST a pravidla identity](../technical/READSB-BEAST-KONTRAKTY.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.

## Průběžná implementace 22. 10. 2026

Detaily důkazů a otevřených invariantů obsahuje [G-02 report](../reports/G-02-VYSLEDEK.md). Zatím **nezaškrtávat** G-02.04/.05/.06/.08/.10/.11 — jsou pouze částečně ověřené. Žádná z šesti funkcí M01 nemá schválenou kompletní přejímku; nová verze G-02 není nasazená do DEV LXC, ten běží na bezpečně ověřeném G-00/G-01 vydání.
