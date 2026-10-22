# G-13 — Samostatně schválený release V2 s obnovením

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** Bezpečnostní release brána bez automatického zásahu do V1.  
**Závislosti:** G-11 a G-12 GO, potvrzená záloha/restore V2 a výslovné samostatné schválení nasazení.  
**Primární KEEP ID:** žádná; pouze ověřovací/provozní brána, žádné nové produktové funkce.

## Cíl

Bezpečně připravit a pouze s výslovným souhlasem provést vlastní V2 nasazení s ověřeným návratem.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-13.01 — Schválit release a cutover**  
  **Implementace:** Vyžádej explicitní povolení pro konkrétní V2 hostname/doménu, LXC a čas, zvlášť pro jakýkoliv dotyk V1.  
  **Ověření:** Existuje písemné GO; bez něj pouze připravit dokumentaci, neprovádět deploy.

- [ ] **G-13.02 — Zmrazit commit a artefakty**  
  **Implementace:** Vyber přesný release SHA, zamkni Rust/npm závislosti, proveď finální CI a dolož kontrolní součty binárky a Vite dist.  
  **Ověření:** Stejné artefakty lze nezávisle sestavit/ověřit.

- [ ] **G-13.03 — Předrelease záloha**  
  **Implementace:** Před DB změnou ověř zálohu izolované V2 DB, jejich retenci a obnovu v jiném prostředí; neovlivňuj cizí DB.  
  **Ověření:** Zdokumentovaný restore PASS, konkrétní RPO/RTO.

- [ ] **G-13.04 — Schválit SQLx migrace**  
  **Implementace:** Vykonej jen dopředu bezpečné změny V2 schématu s oddělenou migrační rolí; při nevratném DDL definuj návrat.  
  **Ověření:** Ukončená migrace a kontrola DB verze na V2 DB, žádné jiné databáze.

- [ ] **G-13.05 — Nasadit Rust službu**  
  **Implementace:** Pod neprivilegovaným uživatelem systemd, read-only filesystem kde lze, health endpoint, správné CPU/RAM limity.  
  **Ověření:** Stabilní start/restart a žádná eskalace přístupových práv.

- [ ] **G-13.06 — Nasadit statický web**  
  **Implementace:** Publikuj schválený Vite dist přes Nginx/Caddy s TLS, cache politikou a stejným originem /api.  
  **Ověření:** Statické assety a index fungují bez běžícího Node procesu.

- [ ] **G-13.07 — Ověřit proxy a SSE**  
  **Implementace:** Zkontroluj forward headers, vypnutý buffering SSE, timeouty, HTTPS, reconnect a Web Push oprávnění.  
  **Ověření:** HTTP/SSE/browser smoke v cílové topologii PASS.

- [ ] **G-13.08 — Notifikace explicitně řídit**  
  **Implementace:** Podle zvláštního souhlasu nastav doručování; výchozí režim zůstává dry-run, dokud není produkční subscribe/consent ověřen.  
  **Ověření:** Žádná skutečná push bez opt-in a auditní evidence.

- [ ] **G-13.09 — Release smoke a observabilita**  
  **Implementace:** Ověř health, lokální letadla, SSE, mapu, katalog, počasí a DB frontu; sleduj CPU/RAM/disk.  
  **Ověření:** Stav všech kritických cest a rollback trigger jsou doložené.

- [ ] **G-13.10 — Otestovat rollback**  
  **Implementace:** V připravené zkušební variantě vrať web i Rust artefakt a ověř kompatibilitu PostgreSQL schématu; nekopíruj V1 data.  
  **Ověření:** Rollback test PASS s měřeným časem a ověřenou integritou V2.

- [ ] **G-13.11 — Izolace vůči V1**  
  **Implementace:** Pokud V2 použije jinou doménu, neměň původní proxy/DNS bez zvláštního schválení; porovnej V1 health před/po.  
  **Ověření:** V1 dostupná beze změny nebo vydané explicitní rozhodnutí o odděleném cutover.

- [ ] **G-13.12 — Uzavřít vydání**  
  **Implementace:** Doplň release tag, český changelog, postrelease report, runbook alarmů, zálohy a známých omezení.  
  **Ověření:** Link na přesný SHA, CI, health evidence a souhlas; jinak release neoznačovat HOTOVO.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-13-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Brána se uzavírá pouze písemným GO se všemi odkazovanými důkazy, jinak zůstává NO-GO/BLOKOVÁNO. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.
