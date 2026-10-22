# G-08 — Nouzový squawk a bezpečné volitelné notifikace

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** M07 — Rust + Web Push/Pushover.  
**Závislosti:** G-02, G-03; G-04 pouze pokud je potřebná odolná fronta doručení; service worker lze dokončit společně s G-10.  
**Primární KEEP ID:** `L06`, `L11`, `L12`.

## Cíl

Implementovat schválenou funkční část s minimem závislostí, testy a transparentním provozním dopadem.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-08.01 — Squawk doménový stav**  
  **Implementace:** Z validních lokálních pozorování odliš běžný squawk a 7500/7600/7700; sleduj čas a identitu, změnu stavu.  
  **Ověření:** Golden replay potvrdí jedinou událost při přechodu, ne každé opakované hlášení.

- [ ] **G-08.02 — Ochrana proti falešným eventům**  
  **Implementace:** Zpracuj chybné/opožděné rámce, přeskoky stavů a restart tracku; nedoplňuj smyšlenou nouzovou událost.  
  **Ověření:** Test nevalidního a stale squawk nevytvoří alert.

- [ ] **G-08.03 — Idempotentní event model**  
  **Implementace:** Připrav minimální event ID a bounded deduplikaci; události neztrácej kontrolu při krátkém reconnectu.  
  **Ověření:** Test opakované sekvence nedoručí stejný event dvakrát.

- [ ] **G-08.04 — Výchozí dry-run**  
  **Implementace:** Zaveď explicitní konfiguraci SEND_ENABLED=false, která zakazuje skutečné volání transportů v DEV/CI/shadow.  
  **Ověření:** Integrační test odchytí každý pokus o outbound HTTP, nulové skutečné zásilky.

- [ ] **G-08.05 — Web Push opt-in**  
  **Implementace:** Připrav pouze potřebnou VAPID konfiguraci na backendu, subscription s ověřením souhlasu prohlížeče a bezpečné uložení citlivých dat.  
  **Ověření:** Test odmítnutí souhlasu, expired subscription a absence klíčů ve Vite assetech.

- [ ] **G-08.06 — Service worker hranice**  
  **Implementace:** Ve spolupráci s G-10 dodej minimální příjem push ve service workeru bez zbytečných offline funkcí navíc.  
  **Ověření:** Browser E2E potvrdí přijetí testovací push v izolovaném prostředí.

- [ ] **G-08.07 — Pushover adapter**  
  **Implementace:** Implementuj minimální HTTP klient s oddělenými secrets, validací response body i HTTP statusu a limity transportu.  
  **Ověření:** Mock 2xx/error, 4xx, 429, 5xx, timeout a chybná odpověď správně ovlivní stav.

- [ ] **G-08.08 — Retry a kapacita**  
  **Implementace:** Retry s konečným limitem, jitter/backoff a bezpečnou deduplikací; bez neomezené DB fronty a opakovaného spamu.  
  **Ověření:** Test výpadku dodavatele nezahltí CPU, síť ani DB.

- [ ] **G-08.09 — Soukromí a audit**  
  **Implementace:** Minimalizuj data oznámení, nesděluj interní receiver lokaci ani neveřejné identifikátory; diagnostiku odděl od externích zpráv.  
  **Ověření:** Test veřejných DTO a payloadů bez tajemství a historie osobních subscription údajů.

- [ ] **G-08.10 — Koncová akceptace**  
  **Implementace:** V izolaci proveď scénář squawk→událost→mock push/Pushover→deaktivace a restart; bez zapnutí reálného doručení.  
  **Ověření:** L06/L11/L12 pokryté důkazy, všechna zařízení během CI/shadow zůstávají bez skutečného upozornění.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-08-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Všechna primárně přidělená ID mají vlastní testovací důkaz a odpovídající status; dokumentace nevydává neprovedený krok za hotový. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.

## Návrhové technické specifikace (22. 10. 2026)

- [Validace squawk a idempotence alarmu](../technical/READSB-BEAST-KONTRAKTY.md)
- [Dry-run browser testy](../technical/PLAYWRIGHT-BROWSEROVE-TESTY.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.
