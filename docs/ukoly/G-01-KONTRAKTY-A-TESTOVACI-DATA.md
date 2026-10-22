# G-01 — Kontrakty a deterministická testovací data

**Stav:** NEZAHÁJENO (podrobné zadání; nejde o implementaci).  
**Odpovědnost:** Průřezový kontrakt pro všech 45 aktivních funkcí (neznamená jejich implementaci).  
**Závislosti:** G-00; schválená matice a Vite/Rust scaffold.  
**Primární KEEP ID:** průřezový kontrakt všech aktivních funkcí; nepřebírá vlastnictví modulů.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

## Výstup etapy

Kontrakty a fixture pro 45 aktivních funkcí, bez připsání statusu dokončení produktovým funkcím.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [ ] **G-01.01 — Zafixovat rozsah a vlastnictví**  
  **Implementace:** Načti právě 45 aktivních ID ze strojové matice; pro každé zachovej jednoho vlastníka z 9 modulů. Vytvoř přehled požadovaných vstupů, výstupů a chování.  
  **Ověření:** Kontrolní skript prokáže 45 různých ID a 1:1 přiřazení k modulům.

- [ ] **G-01.02 — Definovat doménové typy**  
  **Implementace:** Navrhni typy AircraftId, AircraftObservation, AircraftState, position/altitude/speed/squawk, timestamp a dostupnost pole. Rozliš identitu ICAO a ne-ICAO, chybějící a neplatné údaje.  
  **Ověření:** Rust a TS kontrakt explicitně rozlišují null/neznámé a mají validační testy.

- [ ] **G-01.03 — Upřesnit jednotky a časy**  
  **Implementace:** Specifikuj jednotky, UTC časové značky, age/seen, desetinná omezení souřadnic, validitu azimutu a pořadí aktualizací; žádné odhady chybějících hodnot.  
  **Ověření:** Golden test odmítne NaN, nesmyslný rozsah, zastaralé a časově obrácené měření.

- [ ] **G-01.04 — Vytvořit readsb JSON fixture**  
  **Implementace:** Vytvoř syntetické JSON snapshoty: více letadel, postupné změny, ztrátu pozice, callsign, nehledanou identitu a prázdný snapshot.  
  **Ověření:** Fixture jsou verzované, reprodukovatelné, s expected normalizovaným stavem a validovaným JSON.

- [ ] **G-01.05 — Vytvořit BEAST/Mode-S fixture**  
  **Implementace:** Sestav syntetické rámce včetně escape 0x1A, krátkého/chybného rámce, CRC24, různých DF, CPR a chybějících částí dvojice; nepoužívej soukromá data.  
  **Ověření:** Očekávané zprávy jsou ručně ověřitelné a rozbité rámce nezpůsobí panic.

- [ ] **G-01.06 — Navrhnout pravidla continuity**  
  **Implementace:** Napiš scénáře BEAST→JSON→BEAST, reconnect, deduplikaci, změnu callsign, TTL a letadlo bez platné polohy.  
  **Ověření:** Každý scénář má deterministický očekávaný state diff a hranici pro stale data.

- [ ] **G-01.07 — Uzavřít API OpenAPI**  
  **Implementace:** V contracts/openapi.yaml doplň /api/v1/aircraft, health, verzi a chyby; DTO jsou veřejná a bezpečně sanitizovaná.  
  **Ověření:** Validátor OpenAPI a fixture pro odpověď prokážou stabilní tvar a absenci interních secrets.

- [ ] **G-01.08 — Definovat SSE smlouvu**  
  **Implementace:** Popiš snapshot/delta/removed, sekvenci, reconnect, heartbeat, resync a horní limity zpráv/front; nezaváděj nový transport.  
  **Ověření:** Rust emitované zprávy i TS parser používají stejné golden fixtures.

- [ ] **G-01.09 — Zavést kontraktové testy**  
  **Implementace:** Doplň Rust jednotkové/integrační testy a TS Vitest nad společnými daty. Backend i web testují chyby bez spojení s produkčním readsb.  
  **Ověření:** cargo test a npm test projdou s věrnými fixtures při opakovaném běhu.

- [ ] **G-01.10 — Připravit akceptaci etap**  
  **Implementace:** Ke každému aktivnímu ID zapiš scénář PASS/FAIL, odkaz na zamýšlený automatizovaný test a vlastní etapu; neoznačuj produktové funkce za hotové.  
  **Ověření:** Přehled obsahuje všech 45 ID, ale statusy v JSON zůstávají NOT_STARTED, dokud je skutečný kód neprokáže.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-01-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.

## Návrhové technické specifikace (22. 10. 2026)

- [readsb/BEAST přesné vstupní typy a deduplikace](../technical/READSB-BEAST-KONTRAKTY.md)
- [SSE události, seq a memory limity](../technical/SSE-PROTOKOL-A-LIMITY.md)
- [Playwright browser smoke plán](../technical/PLAYWRIGHT-BROWSEROVE-TESTY.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.

## Minimální původ lokálního příjmu — pravidlo návrhu (ADR 0008)

V kontraktu normalizovaného lokálního pozorování odděl `receiverId` (stabilní neveřejný konfigurační identifikátor skutečného příjmu) od `transport` (readsb JSON / BEAST). **Nesmí se měnit identita letadla při fallbacku a jedna stanice nesmí být započtena dvakrát.** Jde o minimální obecný typ pro současných 45 funkcí, nikoli o nový produktový modul nebo úkolový krok. Přejímací fixture: `icao:abc123` přes oba transporty jednoho `receiverId` stále pouze jeden RAM track a bez duplicitního SQL vzorku. Viz [ADR 0008](../adr/0008-identita-puvodu-lokalniho-prijmu.md).
