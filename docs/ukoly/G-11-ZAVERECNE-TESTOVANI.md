# G-11 — Kompletní regresní, bezpečnostní a výkonová přejímka

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** Neproduktová kvalitativní brána pro celý projekt.  
**Závislosti:** G-01 až G-10 na schválené cílové větvi; skutečná DEV DB a replay.  
**Primární KEEP ID:** žádná; pouze ověřovací/provozní brána, žádné nové produktové funkce.

## Cíl

Schválit funkční správnost, výkon a bezpečnost celého rozsahu 45 ID, nebo vydat zdokumentované NO-GO.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-11.01 — Inventura 45 ID**  
  **Implementace:** Zkontroluj zdrojovou matici, vlastníka a konkrétní důkaz každé funkce, bez automatického doplnění statusu.  
  **Ověření:** Každé ze 45 ID má test nebo report výslovně označí NO-GO.

- [ ] **G-11.02 — Rust statické kontroly**  
  **Implementace:** Proveď fmt, Clippy s warnings-as-errors, cargo test --locked, dependency audit podle politiky a reprodukovatelný build.  
  **Ověření:** CI finálního přesného SHA PASS s uchovanými logy.

- [ ] **G-11.03 — Web statické kontroly**  
  **Implementace:** Spusť npm ci, ESLint, TypeScript, Vitest, Vite build a kontrolu statického HTML i všech hlavních lokalizací.  
  **Ověření:** CI nahraditelnost checkoutu prokáže bez opakovaného npm install.

- [ ] **G-11.04 — Protokolová regrese**  
  **Implementace:** Replay validní/poškozené JSON a BEAST, identity, CPR, chybné CRC, restart receiveru a BEAST↔JSON.  
  **Ověření:** Zkontrolovaný expected state a žádný panic/ztráta continuity.

- [ ] **G-11.05 — Kontrakty a SSE**  
  **Implementace:** Proveď OpenAPI/JSON Schema, full/delta, nesouhlas sekvence, heartbeat, reconnect, pomalého klienta a burst klientů.  
  **Ověření:** Rust a browser interpretují stejný stav; omezené fronty.

- [ ] **G-11.06 — Databáze a retence**  
  **Implementace:** V bezpečné DEV PostgreSQL testuj SQLx migrace, batch, indexy, časové dotazy, DB restart, restore a retenci.  
  **Ověření:** Obnovená DEV data odpovídají kontrolnímu součtu a RAM ingest se nezastaví.

- [ ] **G-11.07 — Mapa a E2E**  
  **Implementace:** Playwright desktop/tablet/mobil: připojení, výpadek, filtrování, trail, follow, letiště, počasí, PWA offline a i18n.  
  **Ověření:** Bez kritického regresního problému, screenshot/video artefakty v CI.

- [ ] **G-11.08 — Notifikační dry-run**  
  **Implementace:** Ověř emergency squawk a Web Push/Pushover na mock transportu s chybnými odpověďmi, retry a restartem.  
  **Ověření:** Nulové reálné odeslání, idempotence a absence credentials ve webu.

- [ ] **G-11.09 — Bezpečnost API a infrastruktury**  
  **Implementace:** Test veřejných DTO bez tajemství, TLS/PG oprávnění, závislostí, CORS/origin, rate limit, proxy header a payload limitů.  
  **Ověření:** Bez neopraveného kritického/velmi závažného zjištění nebo formální NO-GO.

- [ ] **G-11.10 — Zátěž a dlouhodobý běh**  
  **Implementace:** Reprodukovatelně měř 1×/2×/4× ingest, SSE 1/10/100 klientů, React/MapLibre 25–2000 cílů a bounded RSS.  
  **Ověření:** Výsledky p95/p99, disk/WAL, CPU/RSS v reportu s metodikou a baseline.

- [ ] **G-11.11 — Zálohy a provozní scénáře**  
  **Implementace:** Simuluj výpadek DB/provideru/přijímače, restart Rust procesu, nezávislost na V1 a restore/test rollback.  
  **Ověření:** Vypnutí V2 neovlivní V1 a důkazy identifikují zbývající rizika.

- [ ] **G-11.12 — Finální GO/NO-GO**  
  **Implementace:** Sestav akceptační protokol a checklist s odkazy na 45 ID a skutečné výsledky; chybějící podmínka blokuje postup.  
  **Ověření:** G-12 začíná jen při doloženém GO a explicitním souhlasu pro read-only shadow.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-11-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Brána se uzavírá pouze písemným GO se všemi odkazovanými důkazy, jinak zůstává NO-GO/BLOKOVÁNO. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.
