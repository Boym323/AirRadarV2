# G-09 — Úsporné statistiky a diagnostika běžících komponent

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** M08 — Rust/SQLx/web.  
**Závislosti:** G-02 až G-08 podle aktivovaných funkcí; pouze existující komponenty.  
**Primární KEEP ID:** `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`.

**Vizuální návaznost (24. 10. 2026):** [map-first UX](../plan/WEB-UX-UI-SPECIFIKACE.md), [ikony a mapové popisky](../plan/MAPOVE-IKONY-A-POPISKY.md), [audit úplnosti](../audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md): Statistiky a diagnostika jsou sekundární panely nad mapou, se skutečnými `OK/degraded/unavailable` stavy, nikoli další hlavní dashboard. Tento dokumentační odkaz neuzavírá žádný krok.

## Cíl

Implementovat schválenou funkční část s minimem závislostí, testy a transparentním provozním dopadem.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-09.01 — Počítadla živého příjmu**  
  **Implementace:** Měř počet aktivních letadel, messages/s a validní/odmítnuté zprávy z jediného Rust state ownera.  
  **Ověření:** Golden replay dává očekávané čítače s časovou granularitou.

- [ ] **G-09.02 — Dostupná klasifikace provozu**  
  **Implementace:** Agreguj jen skutečně známý typ, aerolinku a trať; chybějící údaje uveď jako neznámé, neodvozuj je z přání.  
  **Ověření:** SQL/fixture test pro neznámou aerolinku a duplicitní polohu.

- [ ] **G-09.03 — Logbook přijímače**  
  **Implementace:** Z uložených vzorků poskytni omezené denní souhrny a základní přehled činnosti přijímače.  
  **Ověření:** G-04 data a výstup logbooku se shodují i po retenci.

- [ ] **G-09.04 — Jediný systémový dashboard**  
  **Implementace:** Založ stručnou kartu receiver/API/DB a aktivních meteorologických dat; žádné další vedlejší analytické stránky.  
  **Ověření:** UI E2E při výpadku části systému ukáže dílčí degradaci.

- [ ] **G-09.05 — Health a build metadata**  
  **Implementace:** Veřejný /api/v1/health musí vracet sanitizovaný stav a SHA/verzi, nikoli tajné IP, DB URL či interní konfiguraci.  
  **Ověření:** Security snapshot test ověřuje safe-list polí.

- [ ] **G-09.06 — Příjem readsb**  
  **Implementace:** Měř last seen, reconnect, BEAST/JSON aktivní cestu a počet vadných rámců.  
  **Ověření:** Test odpojeného přijímače správně označí degradovaný stav.

- [ ] **G-09.07 — Aktivní provider monitoring**  
  **Implementace:** Sleduj timeouty, poslední úspěch a chybovost **jen skutečně používaných** meteorologických providerů.  
  **Ověření:** Nedostupný provider nezastaví lokální radar a health sdělí poslední platný údaj.

- [ ] **G-09.08 — Runtime CPU/RAM**  
  **Implementace:** V Rust sleduj CPU/RSS, queue depth a počet úloh. Node 26 je jen build nástroj webu; **není** zde dlouhodobý serverový GC/event-loop runtime.  
  **Ověření:** Telemetrie nepublikuje neexistující Node server ani smyšlené GC údaje.

- [ ] **G-09.09 — Krátká historie výkonu**  
  **Implementace:** Ukládej pouze ohraničené časové metriky pro ověřený účel O06; retence a kapacita jsou explicitní.  
  **Ověření:** Soak test neprokazuje neomezený nárůst storage.

- [ ] **G-09.10 — DB pool a latence**  
  **Implementace:** Monitoruj SQLx pool waiting/active, query p95, délku bounded batch fronty a drop/retry.  
  **Ověření:** Simulovaný DB outage se zobrazí bez vlivu na SSE letadel.

- [ ] **G-09.11 — Diagnostické SSE**  
  **Implementace:** Přes sdílený low-rate stream poskytuj stavové změny, bez per-client pollingu DB/receiveru.  
  **Ověření:** 1/10/100 klientů nespustí násobně stejný drahý probe.

- [ ] **G-09.12 — Benchmark a security přejímka**  
  **Implementace:** Změř režii metrik na CPU/RSS, síť a DB, testuj citlivá data a chování po restartu.  
  **Ověření:** Testovací report pokrývá všech 11 primárních ID a zaznamenává pozorované limity.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-09-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Všechna primárně přidělená ID mají vlastní testovací důkaz a odpovídající status; dokumentace nevydává neprovedený krok za hotový. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.
