# G-02 — Lokální readsb/BEAST ingest a jediný RAM state

**Stav:** **10/11 testově doložených kroků; G-02.11 má PASS offline benchmarku na DEV LXC, čeká dlouhodobý živý JSON/BEAST soak/reconnect**. [Průběžný report](../reports/G-02-VYSLEDEK.md), [aktuální CI #39769778992](https://github.com/Boym323/AirRadarV2/actions/runs/39769778992).  
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

- [x] **G-02.04 — Mode-S a CRC24**  
  **Implementace:** Dekóduj pouze potřebná pole a DF, správně odliš PI/AP/CRC podle typu zprávy; chybný rámec bez důkazu nepovyšuj na spolehlivý údaj.  
  **Ověření:** Ověř soulad s fixture a známými vektory Mode-S, včetně špatné parity.

- [x] **G-02.05 — Poloha a CPR**  
  **Implementace:** Zpracuj korektní párování CPR a časovou/fyzikální plausibilitu; pokud pozici nejde spolehlivě získat, vynechej ji.  
  **Ověření:** Test pokryje jednoprvkový CPR, lokální/globální dekódování a neplatné souřadnice.

- [x] **G-02.06 — Kanonický stav letadla**  
  **Implementace:** Založ jeden Rust vlastník track mapy, stabilní ID, freshness, výšku, rychlost, squawk a odlišená missing data.  
  **Ověření:** Syntetický replay generuje správný stav bez duplicit při změně callsign.

- [x] **G-02.07 — TTL a deduplikace**  
  **Implementace:** Nastav konfigurovatelné bounded TTL a odstranění stale tracků, spoj duplicitní pozorování a změř počet aktivních cílů.  
  **Ověření:** Replay se zpožděnými a obrácenými časy neobnoví starou chybnou polohu.

- [x] **G-02.08 — Arbitráž a fallback**  
  **Implementace:** Jeden supervisor volí platný BEAST nebo JSON při výpadku, restartuje spojení s omezeným backoff/jitter; obě cesty zapisují do stejné RAM.  
  **Ověření:** Test změní dostupnost zdrojů vícekrát a zachová jedinou identitu tracku.

- [x] **G-02.09 — Backpressure a limity**  
  **Implementace:** Omez velikost TCP bufferu, JSON payloadu, počtu tracků a front; při zátěži zaznamenej drop a health bez blokace.  
  **Ověření:** Při syntetickém přetížení nepřibývá paměť neomezeně a klient dostane správný health.

- [x] **G-02.10 — Deterministický replay**  
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

## Historický průběžný stav před rozšířenými testy

Při předchozí kontrole byly G-02.04/.05/.06/.08/.10/.11 pouze částečně ověřené. **Tento historický stav byl pro G-02.04/.05/.06/.08/.10 aktualizován níže**, nikoli pro G-02.11. Kompletní provozní přejímka šesti funkcí M01 dosud neproběhla. G-02 není nasazena v DEV LXC, kde běží G-00/G-01.

## Dodatečná přejímka implementace (22. 10. 2026)

**CI:** [#39769323592](https://github.com/Boym323/AirRadarV2/actions/runs/39769323592), kód `81a27295d78753a6a3d987e7d68d6b8394c12ef5`: 3/3 joby PASS, **26 Rust testů PASS**, web/TS/build a validační dokumentace PASS. Offline `--g02-replay` nyní obsahuje skutečné syntetické JSON i BEAST DF17 zprávy, CRC negativní případ a jediný RAM state. Výška, rychlost, směr, vertikální rychlost a ground jsou volitelná validovaná pole. Lokální CPR vyžaduje čerstvou důvěryhodnou polohu stejného letadla. Kontrola source-health/fallback i loopback reconnect prochází testy.

- **G-02.04/.05/.06/.08/.10: testováno** v tomto vymezeném DF17/readsb rozsahu, bez přidávání neschválených AP/DF dekodérů.
- **G-02.11: nedokončeno.** GitHub CI měří navíc 5 000 různých identit při 1×/2×/4× syntetické zátěži; skutečné profilování na samostatném DEV LXC s 1 vCPU, stabilita při přetížení a dlouhodobý soak zatím nedoloženy.
- `J01/J02/J03/J04/J05/J11` mají stav **TESTED**, nikoli `ACCEPTED`; celá etapa se do `HOTOVO` nepřeklápí před skutečnou G-02.11 přejímkou.
- Nevznikla žádná produkční integrace, nasazení G-02 na DEV LXC ani změna V1, databáze či readsb.

**Postup zbývajícího měření:** [G-02.11 — bezpečná DEV LXC přejímka](../runbooks/G-02-DEV-LXC-PREJIMKA.md).

## Doložená první brána G-02.11 — DEV LXC 22. 10. 2026

Přes automatický pull timer uživatel doložil `PASS_SYNTHETIC_LXC` na SHA `daa15703610b219b5b413319a4920659e19a8694`: 1 v systému viditelný CPU, p95 při dávce 5 000 unikátních identit 2,399/2,546/2,467 ms (1×/2×/4×), RSS 5 236–5 256 KiB a očekávané `drops=1` při vložení 5 001. tracku. [Kompletní metodika a čísla](../reports/G-02-VYSLEDEK.md#výsledek-automatického-offline-benchmarku-přímo-z-dev-lxc--22-10-2026). **Checkbox G-02.11 zůstává nevyplněný**, protože live-soak s reconnectem nebyl proveden. Měření není nasazení webového G-02 ingestu.
