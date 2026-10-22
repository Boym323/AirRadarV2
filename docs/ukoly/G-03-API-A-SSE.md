# G-03 — Rust HTTP API a SSE full/delta

**Stav:** **10/10 kroků TESTED v izolované CI**, provozní DEV LXC a release **NEPŘEVZATY (NO-GO pro merge)**.  
**Odpovědnost:** M02 — Axum.  
**Závislosti:** G-01 a G-02.  
**Primární KEEP ID:** `J06`, `J07`, `J08`.  
**Odhad realizace:** nepodložený, nedávat termín bez změřené kapacity týmu.

## Výstup etapy

Samostatně otestovaný modul odpovídající uvedeným schváleným ID; zbytečné části systému se nepřidávají.

## Implementační kroky a důkaz přejímky

Každý krok realizuj v **samostatném malém PR nebo logicky uceleném commitu**; pokud má krok vlastní závislost, nejdříve ověř jeho předchozí části. Zaškrtávej jen s odkazy na implementaci a reálné testy, nikdy pouze za napsání tohoto plánu.

- [x] **G-03.01 — Stabilizovat public DTO**  
  **Implementace:** Odděl interní RAM struktury od veřejných DTO; vynech interní souřadnice přijímače, secrets a raw payloady.  
  **Ověření:** Snapshot JSON fixture splňuje OpenAPI a privacy test.

- [x] **G-03.02 — HTTP endpointy**  
  **Implementace:** Doplň GET /api/v1/aircraft, GET /api/v1/health a konzistentní chyby, timeouty a parametry filtrů pouze dle schválené smlouvy.  
  **Ověření:** Integrační test pokrývá prázdný i plný stav a 4xx/5xx.

- [x] **G-03.03 — Plné SSE snapshoty**  
  **Implementace:** Odesílej úplný stav se sequence a časem při připojení jako kompatibilní formát.  
  **Ověření:** EventSource test dostane validní snapshot a spojení lze ukončit.

- [x] **G-03.04 — Delta producent**  
  **Implementace:** Vytvářej upsert/removed změny jednou pro všechny klienty, nikoli znovu pro každého odběratele.  
  **Ověření:** Replay test prokáže, že následný delta stav odpovídá full snapshotu.

- [x] **G-03.05 — Sequence a resynchronizace**  
  **Implementace:** Specifikuj monotónní pořadí a řízený resync při zmeškané sekvenci; nepředstírej neomezený server-side replay.  
  **Ověření:** Odpojení během více delt skončí novým korektním snapshotem.

- [x] **G-03.06 — Heartbeat a lifecycle**  
  **Implementace:** Přidej heartbeat/keepalive, cleanup po disconnectu a správné hlavičky text/event-stream.  
  **Ověření:** Ověřený reconnect nemá uniklé úlohy ani rostoucí počet subscriberů.

- [x] **G-03.07 — Backpressure a kapacita**  
  **Implementace:** Každý klient má ohraničenou frontu a deadline; při pomalém klientovi provést coalescing/resync/odpojení; nový klient může dostat 503.  
  **Ověření:** Pomalý klient neblokuje producer a RSS roste pouze do deklarovaného limitu.

- [x] **G-03.08 — Síťový web kontrakt**  
  **Implementace:** Ve Vite dev proxy a budoucím Nginx konfiguraci zajisti stejný origin /api, vypnutý SSE buffering a přiměřené timeouty.  
  **Ověření:** curl -N a browser smoke ověří postupné doručování, nikoliv dávkové flush po ukončení.

- [x] **G-03.09 — Regresní testy**  
  **Implementace:** Doplň concurrency, reconnect, malformed payload, slow consumer a konverzi veřejných jednotek.  
  **Ověření:** Rust unit/integration a TS kontraktové testy PASS.

- [x] **G-03.10 — Měření SSE**  
  **Implementace:** Na syntetickém radarovém replay změř 1/10/100 klientů, bytes/s, CPU, RSS, p95/p99 a počet discard/resync.  
  **Ověření:** Výsledky s hardwarem a SHA v českém reportu, bez nevysvětlené ztráty pozic.

## První CI důkazy G-03.03–.05 (22. 10. 2026)

Na commitu `dbd9af7ceefe9455aae1297691f94cf2eeb9ea63` prošla [CI #39803605266](https://github.com/Boym323/AirRadarV2/actions/runs/39803605266): Rust fmt/Clippy/test, offline G-02 replay, Vite lint/TypeScript/test/build a česká dokumentace, 3/3 PASS. Nové testy `sse_snapshot_delta_and_resync_after_coalesced_seq` a `deltas_reconstruct_exact_canonical_full_state` přímo ověřují HTTP SSE snapshot, delta upsert/removed, resynchronizaci při ztraceném watch okně a shodu výsledné replica RAM s kanonickým stavem. SSE klient se na žádná data z V1 nepřipojoval.

Zaškrtnuté **G-03.03/.04/.05 jsou unit/integračně TESTED**, nikoli reálně provozně ACCEPTED. G-03.01/.02/.06/.07 mají část implementace, ale ještě nejsou přejaty v rozsahu všech ověřovacích kritérií. G-03.08–.10 nebyly **v tomto historickém mezistavu** implementovány. V té době se release nenasadil; aktuální dokončení syntetických CI testů je doloženo v závěru dokumentu.

## Povinné výsledky a testovací brána

- Ověřit Rust `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.
- Ověřit Vite `npm ci`, `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`; podle etapy kontraktové, integrační, browser a replay testy.
- Spustit `node tools/check-docs.mjs` a `node tools/generate-feature-matrix.mjs --check`, aktualizovat jen skutečně ověřené testovací důkazy.
- Každý nový modul/DB tabulka/datový zdroj potřebuje konkrétní aktivní ID, test, limity paměti/CPU a provozní důvod.
- Založit český `docs/reports/G-03-VYSLEDEK.md` s baseline, SHA, PASS/FAIL, omezeními a zbývajícími riziky; upravit `CHANGELOG.md` a `docs/plan/ETAPY-A-STAV.md`.
- **Zákaz:** přenos nebo mazání historie V1, zásah do produkce/V1 LXC/DB, skutečné push v replay/shadow a nevyžádané funkce mimo [aktivní matici](../../contracts/feature-matrix.json). Historické nápady se otevírají pouze na výslovnou žádost vlastníka.

## Definice HOTOVO

Všechny kroky mají skutečné odkazy na důkazy a všechny této etapě primárně přiřazené KEEP funkce jsou `TESTED` nebo `ACCEPTED` po doložení odpovídajících testů. U G-01 stačí ověřit kontrakty a připravenost pro implementaci — **žádná produktová funkce tím automaticky není hotová**. CI finálního SHA je úspěšné, report a changelog popisují realitu.

## Návrhové technické specifikace (22. 10. 2026)

- [SSE protokol a limity](../technical/SSE-PROTOKOL-A-LIMITY.md)
- [Browser reconnect testy](../technical/PLAYWRIGHT-BROWSEROVE-TESTY.md)

Výchozí číselné limity jsou **neověřené konfigurace pro první test**, nikoli PASS. Všechny checkboxy a stav KEEP zachovat, dokud nevznikne reálný kód, fixture a testovací důkazy.

## Integrační pokračování

**Historický mezistav před CI browser testy:** G-03.08 tehdy obsahoval pouze izolovaný Nginx vzor a Vite proxy; cílový DEV streaming přes proxy stále nebyl ověřen. Checkbox zůstává neoznačen.

G-03.01/.02/.06/.07: doplněn oddělený runtime OpenAPI kontrakt a Rust regresní scénáře; výsledky se stanou TESTED až po zelené CI a případných provozních měřeních.

## Doložená dílčí přejímka G-03.01 a G-03.02

[CI #39814106622](https://github.com/Boym323/AirRadarV2/actions/runs/39814106622) na `d7483417f80ce1820d84b2e377e3b709db93a153`: 3/3 PASS, samostatná G-03 OpenAPI 3.1 + validační fixture a Rust HTTP 200/400/503 včetně privacy. Proto .01/.02 jsou TESTED. G-03.06/.07 stále potřebují heartbeat, skutečný RSS/backpressure; .08 proxy browser, .09 další regresi a .10 zátěž.

## Nová přejímka G-03.06 a G-03.09 — CI 39835102766

Doložen [GitHub Actions #39835102766](https://github.com/Boym323/AirRadarV2/actions/runs/39835102766), přesný SHA `db4078fa854e84ec741ba21c72bbc30301f19a65`: Rust/web/docs 3/3 PASS, 15s SSE heartbeat přímo i přes Nginx, reconnect a uvolnění permitů, pomalý klient a 1/10/100 spojení, malformovaný syntetický readsb JSON a pokračující aktualizace bez poškození RAM. J07/J08 mají důkaz `TESTED`, nejde o schválení G-03 v produkci.

**Zbývá:** .07 explicitní deadline při skutečně zablokovaném pomalém TCP konzumentovi a definitivní backpressure/RSS hranice; .08 browser EventSource přes cílový DEV Nginx; .10 rozšíření testu 1000/5000 letadel a reálné LXC p95/p99, CPU/RSS. CI test není náhradou za provozní přejímku.

## Pokračování G-03.07/.10 — skutečný CI důkaz, ale ne provozní převzetí

[CI #39845031595](https://github.com/Boym323/AirRadarV2/actions/runs/39845031595) 3/3 PASS, SHA `ea64497d7079f65e474969a8a2f44df5d49a6b8a`, 42/42 Rust testů. Nový `ClientLease` po 30s bez HTTP Body poll (5s perioda watchdogu) nezávisle uvolní permit i při zablokovaném downstreamu; neprokazuje fyzické ukončení TCP. Syntetická G-03.10 měří 1/10/100 SSE klientů při 5000 sparse letadlech (VmRSS max 43396 KiB, P95 2033 ms u 100 klientů, doba od JSON souboru přes 1Hz poll+publish, nikoliv čistá SSE RTT). Původní 2048 FIFO při 5000 JSON jednotlivých zpráv ztrácela řádky; nahrazena jednou koaleskovanou ohraničenou `JsonBatch` se zachováním jediné RAM autority. **Checkboxy .07/.08/.10 zůstávají otevřené**, dokud nebude ověřen OS buffer/RSS, browser a rollback na skutečném DEV LXC.

## Finální implementační CI přejímka G-03 — 10/10 TESTED (22. 10. 2026)

[**GitHub Actions #39848794079**](https://github.com/Boym323/AirRadarV2/actions/runs/39848794079), SHA **`c43ab9d543a740020ac9133941f1a4829b2c7dea`**: všechny tři joby Rust, Vite a dokumentace PASS. G-03.07: 90 nečtoucích TCP klientů, 1 zdravý klient dostal všech pět následujících aktualizací; Rust VmRSS přírůstek **15 112 KiB**, pod samostatně deklarovaným limitem **25 600 KiB** (bez OS socket bufferů). G-03.08: skutečný Chromium EventSource přes Vite i Nginx (snapshot, delta, full reconnect). G-03.10: 1/10/100 klientů s až **5 000 kompaktními letadly**; při 100×5k všech 100 aktualizací, p95 **1 576 ms**, p99 **1 630 ms**, Rust VmRSS peak **48 764 KiB**; žádné neodůvodněné discards. Metodika a další omezení v [reportu](../reports/G-03-VYSLEDEK.md).

**Rozlišení stavu:** všechny implementační checkboxy jsou testově splněné v izolovaném GitHub runneru; **to neznamená provozní ACCEPTED na cílovém DEV LXC**. Nelze zatím bezpečně provést G-03 cutover, dokud není ověřen root-owned instalátor, systemd přepnutí z G-00 na G-03 a atomický rollback v DEV. V1 nedotčena. **PR #11 zůstává draft a NO-GO pro merge do `main`.**
