# G-12 — Neprodukční shadow v novém LXC bez dopadu na V1

**Stav:** NEZAHÁJENO (jde o implementační zadání, nikoliv hotový kód).  
**Odpovědnost:** Provozní ověřovací brána, nikoli nový produkt.  
**Závislosti:** G-11 GO a explicitní oprávnění k izolovanému V2 LXC a případnému read-only zdroji.  
**Primární KEEP ID:** žádná; pouze ověřovací/provozní brána, žádné nové produktové funkce.

## Cíl

Porovnat budoucí V2 v izolaci a zabránit jakémukoli negativnímu vlivu na V1.

## Implementační kroky a testovatelná kritéria

Kroky lze řešit malými samostatnými PR / ucelenými commity. Zaškrtávej pouze po skutečné implementaci a doloženém PASS; pořadí změň jen s dokumentovaným důvodem.

- [ ] **G-12.01 — Schválit shadow hranice**  
  **Implementace:** Potvrď vlastníka LXC, samostatnou V2 DB/DEV, rozsah read-only zdroje a zakázané činnosti vůči V1.  
  **Ověření:** Písemně evidovaný GO pro shadow a kontaktní osoba pro rollback.

- [ ] **G-12.02 — Prověřit izolaci**  
  **Implementace:** Ověř samostatné UID, IP/porty, repo checkout, tajemství, systemd a PostgreSQL V2 DB/role.  
  **Ověření:** Žádný zapisovací přístup na V1 ani sdílená cesta či port.

- [ ] **G-12.03 — Zachytit baseline V1**  
  **Implementace:** Neinvazivně a jen oprávněně zaznamenej V1 CPU/RAM, readsb odezvu, reconnect před spuštěním shadow.  
  **Ověření:** Baseline se opírá o měření, ne odhad.

- [ ] **G-12.04 — Připravit V2 artefakt**  
  **Implementace:** Sestav přesné SHA, statický web a Rust v izolaci, bezpečně nastav pouze V2 service config.  
  **Ověření:** Ověřen hash buildu, žádná webová DB credentials.

- [ ] **G-12.05 — Zakázat skutečné push**  
  **Implementace:** Enforce dry-run ještě před povolením vstupního streamu, včetně Pushover, Web Push a retry workera.  
  **Ověření:** Mock/packet capture prokáže nulové produkční notifikační požadavky.

- [ ] **G-12.06 — Read-only replay jako první**  
  **Implementace:** Pusť syntetický replay a změř V2 RAM/SSE/DB bez dotyku produkčního readsb.  
  **Ověření:** Reprodukce testů G-11 v novém LXC PASS.

- [ ] **G-12.07 — Připojení skutečného readsb jen se souhlasem**  
  **Implementace:** Po ověření limitu konzumentů připoj read-only mirror nebo autorizovaný odběr; nijak neovlivňuj nastavení V1 přijímače.  
  **Ověření:** V1 zůstane v baseline a readsb nepřetíží nový klient.

- [ ] **G-12.08 — Dlouhodobý soak**  
  **Implementace:** Zaznamenej CPU/RSS, síť, p95 ingest/SSE, DB WAL, úniky paměti a restart po dobu schválené relace.  
  **Ověření:** Časová řada a report se zdrojem vzorků, bez poškození cizích dat.

- [ ] **G-12.09 — Simulovat omezené výpadky**  
  **Implementace:** Pouze na V2 otestuj ztrátu DB sítě, restart V2 procesu, výpadek provideru a omezený reconnect.  
  **Ověření:** Místní RAM/SSE se zotaví a V1 nedotčená.

- [ ] **G-12.10 — Ukončení a porovnání**  
  **Implementace:** Odpoj V2, změř návrat V1 na baseline, uzavři incidenty a připrav GO/NO-GO.  
  **Ověření:** Konečný report eviduje srovnání před/po a možnost opakování.

## Povinné testy a dokumentační výstupy

- Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` a příslušné replay/integration testy.
- Web `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a E2E jen pokud ho etapa potřebuje.
- `node tools/check-docs.mjs`, `node tools/generate-feature-matrix.mjs --check`, akceptační důkazy pro vlastní ID (u G-11 až G-13 průřezové důkazy).
- Český report `docs/reports/G-12-VYSLEDEK.md` obsahuje výchozí a výsledný SHA, CI link, data, metodiku, PASS/FAIL, rizika, případně naměřené CPU/RSS/latence; aktualizovat `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- Žádná automatická změna V1, její databáze nebo LXC; nikdy produkční nasazení ani skutečné notifikace bez odděleného schválení.
- Nepřebírat žádnou funkci mimo [aktivní matici](../../contracts/feature-matrix.json), budoucí nápady jsou izolované v [backlogu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Převzetí

Brána se uzavírá pouze písemným GO se všemi odkazovanými důkazy, jinak zůstává NO-GO/BLOKOVÁNO. Finální CI na přesném SHA PASS, bez kritických regresí. Změny provádět pouze v odděleném prostředí V2.
