# G-04 — PostgreSQL historie a bezpečná izolace DB LXC

**Stav:** NEZAHÁJENO (podrobné zadání; nejde o implementaci).  
**Odpovědnost:** M03 — Rust/SQLx/PostgreSQL.  
**Závislosti:** G-02 a G-03, samostatný DEV přístup k existujícímu PostgreSQL LXC schválený správcem.  
**Primární KEEP ID:** `J09`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [ ] **G-04.01 — Prověřit existující DB LXC**  
  **Implementace:** Read-only ověř verzi, kapacitu a zálohy PostgreSQL. Plán výhradně nové V2 DB, DEV a omezených rolí předlož správci; nic neměň bez oprávnění.  
  **Ověření:** Evidence schváleného názvu nové DB, přístupového modelu a potvrzené izolace od jiných aplikací.

- [ ] **G-04.02 — Síť a TLS**  
  **Implementace:** Nastav pouze s oprávněním konkrétní firewall allowlist, pg_hba, SCRAM a ověření identity serveru sslmode=verify-full; nezapisuj credentials do GitHubu.  
  **Ověření:** Read-only handshake prokáže správnou V2 identitu a šifrování, jiná DB zůstává nedostupná.

- [ ] **G-04.03 — DEV databáze a role**  
  **Implementace:** Pověřený správce vytvoří samostatnou DEV DB, migrační a runtime roli s nejnižšími nutnými právy, bez SUPERUSER/CREATEDB/CREATEROLE.  
  **Ověření:** Ověřit current_database/current_user a odmítnutí přístupu do jiné databáze.

- [ ] **G-04.04 — Minimální SQLx schéma**  
  **Implementace:** Navrhni migration SQL pro vzorkované pozice a jen skutečně potřebné indexy/dotazy J09, S06/S09; žádná kopie V1 Prisma.  
  **Ověření:** Migrace tam/zpět otestována na prázdné DEV DB; nezávislý DDL runtime účet.

- [ ] **G-04.05 — Connection pool a hranice**  
  **Implementace:** Přidej SQLx Postgres pool s timeouty, omezeným počtem spojení a fail-soft chováním při ztrátě DB.  
  **Ověření:** Ztráta připojení neblokuje readsb→RAM→SSE a nezacyklí retry.

- [ ] **G-04.06 — Sampling politika**  
  **Implementace:** Vzorkuj pouze validní změny polohy v řízeném intervalu a se stejnou identitou jako live stav; žádné raw BEAST dumpy.  
  **Ověření:** Test vzorkování a deduplikace pro konstantní i pohybující se letadlo.

- [ ] **G-04.07 — Bounded batch writer**  
  **Implementace:** Zaveď asynchronní ohraničenou frontu a dávkové UPSERT/INSERT s retry a metrikami ztrát při přetížení.  
  **Ověření:** Test saturace a restartu nevede k nekonečné frontě ani dvojímu zápisu.

- [ ] **G-04.08 — Query-first čtení**  
  **Implementace:** Připrav pouze query pro schválené souhrny a logbook; nic neprojektuj pro budoucí rozšíření.  
  **Ověření:** SQL dotazy mají test korektního časového okna a EXPLAIN/BUFFERS v DEV.

- [ ] **G-04.09 — Retence a indexy**  
  **Implementace:** Retenční interval odvoď z reálného provozu, zdokumentuj variantu DELETE/partitioning podle naměřeného objemu; nevol složité partitioning předčasně.  
  **Ověření:** Retenční test i test po smazání uchovají indexy a plánovaný limit dat.

- [ ] **G-04.10 — Výpadek a zotavení**  
  **Implementace:** Otestuj restart DB LXC, timeout, dočasný packet loss a zotavení, bez destruktivního zásahu do produkční instance.  
  **Ověření:** RAM a SSE zůstávají aktivní, SQLx metrics ukážou degradaci a návrat.

- [ ] **G-04.11 — Záloha a skutečné obnovení**  
  **Implementace:** Připrav pro V2 oddělené pg_dump/restore či schválené fyzické zálohy, obnovu ověř v DEV a bez přístupu do cizích databází.  
  **Ověření:** Zkouška obnovení prokáže integritu záznamů a dokumentované RPO/RTO.

- [ ] **G-04.12 — Výkonnost PostgreSQL**  
  **Implementace:** Změř 1×/2×/4× zápis, batch size, pool, WAL, IOPS, síťovou latenci a p95/p99 SQL dotazů na identickém datasetu.  
  **Ověření:** Český report doloží SHA, metody, naměřené limity a provozní bezpečnost.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-04-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.

## Návrhové technické specifikace (22. 10. 2026)

- [SQLx schéma, indexy, retence a testy záloh](../../database/POSTGRES-DATOVY-MODEL.md)
- [TLS, firewall, role a bezpečná obnova](../runbooks/POSTGRES-LXC-SECURITY-BACKUP.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.
