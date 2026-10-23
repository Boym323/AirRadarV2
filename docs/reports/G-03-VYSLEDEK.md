# G-03 — výsledkový report Rust HTTP API a SSE

**Datum:** 23. 10. 2026. **Stav:** **G-03 10/10 TESTED v izolované CI; provozní přejímka NEOVĚŘENO do dokončení nového main CI/deploy běhu**. V1, DB a produkční receiver zůstaly nedotčené.

## Incident 23. 10. 2026 — kořenová příčina a oprava

Journal prokázal dvě oddělené příčiny:

- V 07:00 se G-03 release `03be43a4a5793ebf73fad7ff5243f5148f4f3a3e` skutečně přepnul a nastartoval jako `--g03-live-api`. V 07:07 proběhlo vypnutí/restart LXC; runner obdržel standardní `Ctrl-C`, nikoli samovolné ukončení deploymentem. Rollback v téže shutdown transakci narazil na systemd chybu `restart is destructive`.
- Po bootu DHCP přidělilo `192.168.1.126` až po startu Nginx. Jednorázový bind na konkrétní adresu proto selhal a Nginx zůstal inactive. Druhý Actions pokus tak skončil před G-03 healthcheckem zavádějícím exit 3.
- Při živém 1Hz streamu se heartbeat timer vytvářel znovu po každé deltě. Heartbeat se tedy nikdy nedostal na řadu; verifier s 20s limitem mohl skončit timeoutem/předčasným EOF. To je skutečná příčina SSE chyby, nikoli G-02.

Oprava patří ke krokům **G-03.06/G-03.08**, konzumenty jsou Rust HTTP/SSE, DEV Nginx a root-owned deployment bridge. Přidán je persistentní ticker, Nginx no-buffer konfigurace, retry po DHCP a rollback bez destruktivního restartu. Kontrolovaný ruční cutover existujícího immutable releasu na DEV proběhl a vrátil `stage=G-03`, `status=live`; finální důkaz musí pocházet z nového pushu do `main`.

## Provozní přejímka DEV — PASS 23. 10. 2026

Finální commit `749ffb942407f7c7cd88f3d983a72841664f52ee` je shodný s `main` i s `/opt/airradar-v2/current/VERSION`. GitHub Actions [run #39905632012](https://github.com/Boym323/AirRadarV2/actions/runs/39905632012) skončil `SUCCESS`; přímý runner journal potvrzuje start deploymentu v 07:29:05, přepnutí releasu v 07:29:08 a dokončení celého jobu v 07:29:44 bez ručního opakování.

Na cílovém LXC byly po deploymentu ověřeny:

- `airradarv2-engine.service`, Nginx a GitHub runner jsou `active`;
- systemd používá `/opt/airradar-v2/current/airradarv2-engine --g03-live-api` a BEAST `192.168.1.50:30005`;
- `/api/v1/health` vrací `stage=G-03`, `status=live`, `live_ingest=true`, `database_connected=false`;
- root verifier prošel direct i proxované SSE: snapshot, epoch/seq, `aircraft=67`, `proxySeq=87`;
- samostatný test přes `192.168.1.126:8080` obdržel při živých deltách heartbeat za **15,000 ms** a po reconnectu znovu snapshot, heartbeat a stejnou epochu;
- G-02 služby během finální kontroly neběžely; G-04 databáze nebyla aktivována.

Rollback důkaz: při incidentním selhání `03be...` journal zachytil obnovení předchozího symlinku `812a858237f7` a odstranění G-03 drop-inu; po bootu běžel G-00 health bez falešného `PASS`. Nová implementace navíc před rollback restartem kontroluje stav systemd a při shutdownu obnoví pointer i režim bez destruktivní transakce. Izolovaný úmyslný rollback na živém DEV nebyl proveden, aby se nerušil právě přijatý stav; riziko je kryto existujícími unit/regresními testy a incidentním důkazem.

**Výsledek:** G-03 DEV HTTP/SSE a automatické `main → CI → immutable release → runner → LXC → healthcheck` jsou ACCEPTED pro vývojový LXC. Produkční AirRadar V1, jeho LXC, databáze a konfigurace readsb nebyly měněny.

## První implementační celek — G-03.01 až G-03.06 (CI PASS, dílčí přejímka)

- **G-03.01 / J06:** Veřejná `PublicAircraft` projekce z již existující G-01 typové smlouvy; nevystavuje `receiverId`, transport, IP, raw BEAST zprávu ani privátní čas doručení. Stabilní epoch a monotónní sequence, RFC3339 UTC čas snímku.
- **G-03.02 / J06:** Opt-in `--g03-live-api` poskytuje `GET /api/v1/health` a `GET /api/v1/aircraft` s verzovaným snapshotem. Stav je `live` pouze s pozorováními nejvýše 5 s starými; jinak explicitní `waiting_for_observations`. Defaultní start běžící G-00 service zůstává `NOT_IMPLEMENTED`.
- **G-03.03 / J07:** HTTP `GET /api/v1/stream` začne úplným `snapshot` s `epoch`/`seq` a pokračuje SSE. Hlavičky `text/event-stream`, `Cache-Control: no-cache, no-transform`, `X-Accel-Buffering: no`, 15s keepalive.
- **G-03.04 / J08:** Jeden Rust RAM vlastník volá `publish()` **maximálně 1 Hz**. Produkuje atomický sdílený snapshot, upsert/removed deltu podle `PublicAircraft` a origin `seq`. Změny se nekalkulují v každém klientovi.
- **G-03.05:** `watch` nemá historii a koalescuje; po chybějící sekvenci klient dostane úplný snapshot. Restart má novou epochu. `Last-Event-ID` není důkaz pokračování z historie.
- **G-03.06/.07 (částečně):** 15s heartbeat; max 100 SSE klientů `Semaphore`, poslední frame v `watch` místo nekonečné per-client fronty; základní test vyčerpání permitů. Limit serializovaného snapshotu 1 MiB, odmítnutí větší publikace s chybovým logem. Pomalý klient vyžaduje další skutečné zátěžové ověření.
- **G-03.08–.10 neprovedeno:** Nginx/Vite integrace v DEV, 100-client load test, CPU/RSS, p95/p99, produkční release a přejímka čekají. Web UI neimplementováno (to patří G-05).

## Bezpečný test opt-in runtime

Jen v izolovaném checkoutu / CI, bez instalace webu a bez přepnutí aktuálního serveru:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace

# POUZE na ověřeném samostatném vývojovém receiveru s povolením čtení,
# ne během souběžné G-02.11 přejímky:
AIRRADAR_V2_BIND=127.0.0.1:4102 \
AIRRADAR_V2_BEAST_ADDR=PRIVATNI_READONLY_RECEIVER:30005 \
cargo run --locked -p airradarv2-engine -- --g03-live-api

curl -fsS http://127.0.0.1:4102/api/v1/aircraft
curl -Ns http://127.0.0.1:4102/api/v1/stream
```

**Tento report nepovoluje spuštění proti V1 ani další odběr ze sdíleného přijímače během G-02.11.** Připravený runtime režim se nikam sám nenasazuje.

## Přesná přejímací pravidla

- G-03.01–.06 lze označit jako testově doložené **teprve po zeleném CI finálního SHA a kontroly snapshot/delta/gap/privacy integrace**; do té doby se nezaklikávají checkboxy.
- `PublicAircraft` export využívá stávající G-01 kontrakt, bez nového JSON schema/modelového rozšíření.
- U existujícího endpointu defaultního G-00 bootstrapu se nic nemění, proto se nemění jeho původní OpenAPI kontrakt. Opt-in runtime kontrakt API/SSE bude výslovně dokumentován odděleně před G-03.08 přejímkou.
- Podmínka plného `HOTOVO`: CI, 1/10/100 klientů včetně chyb/slow reader, p95/p99 a RSS, bezpečné provozní ověření a schválené nasazení. **Momentálně G-03 rozhodně není HOTOVO.**

## Skutečný CI důkaz prvního celku (22. 10. 2026)

[CI #39803605266](https://github.com/Boym323/AirRadarV2/actions/runs/39803605266) na `dbd9af7ceefe9455aae1297691f94cf2eeb9ea63` má **3/3 joby PASS** (Rust fmt, Clippy -D warnings, testy včetně snapshot/delta/gap, 5s izolovaného G-02 replay soak, Vite a dokumentace). Přímý HTTP test sleduje empty/full/health + SSE hlavičky; `deltas_reconstruct_exact_canonical_full_state` aplikuje skutečně publikované delty do G-01 `Replica` a srovnává s canonical snapshotem včetně TTL removal. Tyto testy **nejsou** 100-client HTTP/SSE benchmark a nedokládají proxy průchod ani provozní RSS.

**Dílčí stav:** G-03.03/.04/.05 = TESTED v izolovaném CI. Zbývající kroky dokončit podle zadání, G-03 celá ROZPRACOVÁNO. Žádný zásah do DEV/V1/G-02 LIVE.

## Přenos na základ dokončené G-02

Tento report je přenesen do samostatné integrační větve od `main` po G-02 11/11. Rozpracované HTTP/SSE zatím není součástí hlavní větve ani nasazené DEV služby. Opětovná přejímka nového SHA a provozní G-03.07–.10 chybí.

## Pokračování G-03.08 — síťový kontrakt (NEOVĚŘENO provozně)

- Oddělený budoucí Nginx příklad `deploy/nginx-g03.example.conf` je pouze loopback, bez SSE bufferingu, s 75s timeoutem a API na opt-in portu 4102; instalovaný G-00 Nginx se nemění.
- Vite přepne dev proxy na 4102 jen při `AIRRADAR_V2_G03_API=1`, jinak zůstává 4101.
- CI spouští statický síťový test a HTTP/SSE `curl --no-buffer` s neexistujícím lokálním fixture souborem — bez přístupu k reálnému readsb a bez DB.
- Skutečná Vite/Nginx browser integrace, pomalý klient, CPU/RSS/p95/p99 a 1/10/100 klientů zůstávají NEOVĚŘENO. Checkbox G-03.08 je proto otevřený.

## Další regresní a formální kontraktové změny (čekají CI)

- `contracts/openapi.g03.yaml` odděluje opt-in živý kontrakt od historicky pravdivého bootstrap `contracts/openapi.yaml`; formální kontrola `tools/validate-g03-openapi.py` ověřuje JSON schema/veřejné DTO a typované odpovědi.
- HTTP nepřijímá neodsouhlasené query filtry (400 `CONTRACT_INVALID`); SSE kapacita vrací 503 `UNAVAILABLE` ve veřejném `ErrorV1` namísto prázdné odpovědi.
- Nové testy zkouší reconnect s cizím Last-Event-ID, uvolnění permitů a 64 aktualizací pro pomalého klienta, aby namísto nekompletní delty přišel úplný snapshot.
- Tyto změny nyní čekají na **CI finálního SHA**. Skutečná RSS, proxy, přenos 100 pomalým klientům a konečný release jsou NEOVĚŘENO, proto checkboxy zatím neměním.

## Dílčí přejímka G-03.01/.02 — 22. 10. 2026

[CI #39814106622](https://github.com/Boym323/AirRadarV2/actions/runs/39814106622), přesné SHA `d7483417f80ce1820d84b2e377e3b709db93a153`: Rust fmt/Clippy/test, Vite lint/typy/test/build, OpenAPI 3.1, JSON Schema a izolované HTTP/SSE curl smoke — **3/3 joby PASS**. Ověřeno odmítnutí nepovolených query parametrů 400/`CONTRACT_INVALID`, překročení 100 SSE spojení 503/`UNAVAILABLE`, reconnect na novém snapshotu, cleanup permitů a koalescence změn při pomalém konzumentovi. **J06 = TESTED** se skutečným testovacím důkazem; G-03 nyní **5/10 testově doložených kroků**, nikoliv HOTOVO.

Limitace: testy v CI jsou syntetické bez skutečného přijímače; nejsou měření 100 klientů přes Nginx/browser, heartbeat v reálné síti, p95/p99 přenosu ani RSS cílového LXC. Provozní G-03.06–.10 zůstává otevřeno.

## Pokračování — měřicí brána G-03.07/.09/.10

Připraven `tools/benchmark_g03_sse.py`: izolační test s vlastní Rust instancí, syntetickými readsb JSON soubory a 1/10/100 HTTP SSE klienty. Po prvním snapshotu měří reakci na nová pozorování, kontroluje 503 u 101. klienta, restart capacity a vypisuje CPU (Linux `/proc`), RSS, bytes/s, p95 a p99. Měření je **CI na sdíleném runneru**, nikoli cílové 1vCPU DEV LXC nebo browser/Nginx; aktuální stav pro finální report **NEOVĚŘENO do dokončení CI**. Checkbox .07/.09/.10 zůstává otevřený.

## Revidovaná metodika CI zátěžového testu

První [CI #39833985212](https://github.com/Boym323/AirRadarV2/actions/runs/39833985212) 3/3 PASS naměřila doručení 1/10/100 klientům. Měřená latence je **od atomického zápisu syntetického JSON souboru do klientského přijetí aktualizace**, zahrnuje dva 1Hz cykly (readsb poll a SSE publish). **Není to čistá latence fanoutu SSE ani údaj z browseru**. V původní verzi číslo `bytes_per_sec` chybně započítávalo také první snapshot mimo měřený čas; nepoužívej je pro plánování kapacity. Nyní je oddělen přírůstek SSE bytes během měřeného okna, peak RSS a malformovaný JSON/recovery; nové hodnoty čekají na CI.

## G-03.08 — Nginx streaming smoke v CI (čeká výsledek)

Nový nezávislý CI krok dočasně instaluje Nginx jen na izolovaný GitHub runner, načítá `deploy/nginx-g03.example.conf` (loopback:8082 → Rust:4102), ověřuje health a přes `curl -N` postupný snapshot i 15s heartbeat. Nejde o nasazení do DEV LXC ani browser EventSource přejímku, G-03.08 proto zůstává otevřené.

## Uzavřená dílčí CI přejímka 22. 10. 2026 — 7/10 TESTED

[**Run #39835102766**](https://github.com/Boym323/AirRadarV2/actions/runs/39835102766) na **přesném SHA `db4078fa854e84ec741ba21c72bbc30301f19a65`**, joby Rust/web/česká dokumentace **3/3 PASS**. Podrobné logy obsahují každý `G03_SSE_SCENARIO`; výsledky jednoho konkrétního běhu:

| Klienti | Letadla | Aktualizace | p95 od zápisu JSON (ms) | p99 (ms) | CPU (% jednoho jádra) | RSS max Rust (KiB) | Nové SSE B/s |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 25 | 1 | 1 908 | 1 908 | 0,0 | 6 888 | 4 105 |
| 10 | 250 | 10 | 1 840 | 1 840 | 7,61 | 9 108 | 414 967 |
| 100 | 250 | 100 | 1 786 | 1 794 | 72,39 | 11 448 | 4 310 981 |
| 10 | 1 000 | 10 | 1 840 | 1 840 | 30,42 | 14 252 | 1 654 993 |

Metodika: Ubuntu GitHub-hosted runner, samostatný opt-in Rust binární proces, vygenerované lokální readsb JSON, 1Hz JSON poll a 1Hz RAM publish, měření od atomického rename souboru do přijetí validní SSE události; p95/p99 z jednotlivých klientů metodou nearest rank. Není to čistá síťová SSE RTT ani browser latency. CPU je spotřeba procesu vyjádřená jako procento jednoho CPU jádra z `/proc/PID/stat`; RSS pouze Rust proces z `VmRSS`, vzorkováno po 50 ms. B/s jsou nově přečtené SSE bytes ve stejné době, bez úvodních snapshotů a bez Nginx TCP režie. Hardware runneru není referenční 1vCPU DEV LXC.

PASS: všechna doručení 1/10/100, 101. klient HTTP 503, uvolnění capacity, malformovaný JSON bez ztráty RAM, obnovení aktualizací, Rust SSE 15s heartbeat, **reálná CI Nginx reverse proxy** 127.0.0.1:8082 → 4102 s `curl -N` snapshot + heartbeat. G-03.06/.09 jsou nyní TESTED; J06/J07/J08 jsou `TESTED`, bez akceptace produkce.

**Otevřeno:** G-03.07 deadline a závislost skutečně zablokovaných konzumentů; G-03.08 browser EventSource/streaming na izolovaném cílovém DEV Nginx; G-03.10 5k state/snapshot limit, 1/10/100 na cílovém 1vCPU LXC, RSS/CPU a p95/p99 vůči měřicím kritériím. Neoznačovat G-03 jako HOTOVO. Původní V1, její DB i LXC se neměnily.

## Oprava po auditu — P1 SSE a DEV stage gate / P2 dokumentace

- Nové sdílené `Bytes` a 16KiB SSE chunky zabraňují samostatné serializaci 1MiB eventu pro každého ze 100 klientů; nejnovější frame zůstává v jednom `watch` produceru. Reálný TCP buffering a RSS pro 100 pomalých klientů jsou stále **NEOVĚŘENO**.
- Při přetečení 1MiB snapshotu / překročení počtu 5000 přechod do `degraded`, HTTP 503 `CAPACITY_EXCEEDED`, explicitní `unavailable` pro stávající SSE a poté obnovení novým `snapshot`, nikoli ponechání starých LIVE letadel.
- G-03 DEV releasy nesou `web/g03-runtime-required.txt`. Runner odmítne starý root-owned installer ještě před instalací. Nový root rollback ověřuje `--g03-live-api` a skutečný snapshot/heartbeat přes Nginx; G-00 bootstrap není G-03 přejímka.
- **NO-GO pro sloučení G-03 do main** dokud není aktualizován root instalátor a ověřen návrat konfigurace systemd společně s binárkou. Při změně režimu nemusí pouhý rollback symlinku z G-03 na G-00 obnovit běh aplikace.
- Změny jsou ve vývojovém PR #16 bez přímého zásahu do LXC, V1 nebo G-04 databáze. CI regresních testů je potvrzeno [#39840890251](https://github.com/Boym323/AirRadarV2/actions/runs/39840890251), ale reálné cílové DEV měření stále chybí.

## P1 regresní PASS po sloučení opravy do G-03 — 22. 10. 2026

- [**GitHub Actions #39840890251**](https://github.com/Boym323/AirRadarV2/actions/runs/39840890251), commit `b1cdb3f23d57566d54a1cede08b42b64a59ec1c5`: Rust/web/dokumentace **3/3 PASS**, 38 Rust testů, včetně `five_thousand_aircraft_shared_snapshot_is_chunked_under_16k` a `oversize_snapshot_marks_api_degraded_not_stale_and_recovers`.
- Nový verifikátor stage-aware DEV má regresní negativní testy: `bootstrap` a `degraded` se nesmí přijmout jako G-03, SSE musí mít atomické `epoch:seq`. Statický test a syntetický Nginx heartbeat prošly v CI; **nainstalovaná root-owned G-03 brána v DEV LXC nebyla ověřena**.
- Ověřený syntetický 1/10/100 klientů nad izolovaným 127.0.0.1 Rust procesem z výsledného buildu. Jde o CI hardware, ne cílový 1vCPU LXC a ne browserový test. Tento PASS neznamená důkaz OS socket budgetu 256KiB/klient.
- **G-03 zůstává 7/10 TESTED a NO-GO pro merge do main**, dokud se nedokončí skutečné DEV provozní SSE měření a atomický rollback režimu systemd/binary. G-02, V1 a databáze zůstaly bez zásahu.

## Ověřená G-03.07/.10 CI brána — 22. 10. 2026

[**GitHub Actions #39845031595**](https://github.com/Boym323/AirRadarV2/actions/runs/39845031595), přesný SHA `ea64497d7079f65e474969a8a2f44df5d49a6b8a`, **3/3 PASS**, Rust fmt/Clippy `-D warnings`, **42/42 Rust unit**, Vite a české dokumenty, Nginx opt-in SSE snapshot + 15s heartbeat.

**Nalezená a opravená chyba:** První rozšíření benchmarku [CI #39844067265](https://github.com/Boym323/AirRadarV2/actions/runs/39844067265) nedokázalo vytvořit RAM snapshot 5000 letadel. Původní JSON poll posílal každý z 5000 řádků zvlášť do 2048-elementového `mpsc` s `try_send`. Oprava je atomická `Ingress::JsonBatch` (max. 5000 validovaných JSON identit ze `MAX_JSON_BYTES=4MiB`) s `AtomicBool` bránou **maximálně jedné čekající dávky**. Nová dávka při obsazeném slotu se zahodí vcelku a její pozorování se započítají jako drops; BEAST zůstává na původní ohraničené frontě, jediný RAM vlastník a časová arbitráž beze změny. Rust regrese `g03_10_json_batch_preserves_all_5000_tracks_without_fifo_loss` a `g03_10_json_batch_backpressure_discards_whole_sample_not_prefix` PASS; testován i původní G-02 offline replay a 5s loopback soak.

**G-03.07:** `ClientLease` vrací při 30s bez HTTP Body poll aplikační SSE permit watchdogem nezávislým na pokračování HTTP socketu, v intervalu 5s (nejpozději přibližně po 35s neaktivity). Watchdog drží jen `Weak` referenci a po dropu spojení mizí. Testy `g03_07_deadline_releases_permit_without_any_body_poll` a `g03_07_active_reader_refreshes_deadline_and_drop_cleans_up` PASS. **Nejde o garantované ukončení TCP socketu nebo limit kernelových bufferů**.

**G-03.10:** měření pouze na GitHub-hosted Ubuntu, syntetický readsb soubor atomicky nahrazovaný každou změnou, 1Hz JSON poll + 1Hz SSE publish. Čas P95/P99 je *od zápisu syntetického souboru do obdržení validní SSE aktualizace*, zahrnuje oba 1Hz cykly, není měřením čisté síťové SSE latence. CPU je proces Rustu z `/proc/PID/stat`, RSS z `VmRSS` periodicky po 50 ms, **bez kernelových TCP bufferů**. Nová plná zátěž 5000 řádků používá *sparse* identitu (bez souřadnic/telemetrie), aby byl serializovaný snapshot pod schváleným 1MiB limitem; 5000 dense nad limitem pokrývá samostatný negativní Rust overflow test.

| Klienti | Syntetická letadla | P95 od souboru (ms) | P99 (ms) | CPU (% 1 jádra) | Rust VmRSS peak (KiB) | Nové SSE B/s |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 5000 sparse | 1868 | 1868 | 10,17 | 37212 | 508691 |
| 10 | 5000 sparse | 1865 | 1865 | 11,79 | 42336 | 5096137 |
| 100 | 5000 sparse | 2033 | 2110 | 12,15 | 43396 | 44411539 |

**Přijímací omezení:** CI 5000/100 PASS neprokazuje 256KiB celkových TCP bufferů na klienta, odpojení skutečného pomalého socketu, browser EventSource, zátěž cílového DEV LXC ani atomické vrácení systemd režimu G-00. G-03.07/.08/.10 zůstávají nezaškrtnuté; G-03 **7/10 TESTED** a **NO-GO pro merge do main** do cílové provozní přejímky.

## G-03.07 — reálná síťová zkouška nečtoucích SSE klientů (nově zařazena do CI)

Test nyní drží **90 TCP SSE spojení bez jakéhokoliv čtení odpovědi**, ve stejnou chvíli musí zdravý SSE klient dostat pět aktualizací 5 000 kompaktních cílů. Zaznamenává a ověřuje přírůstek **Rust VmRSS ≤25 MiB** oproti době před připojením nečtoucích klientů. Kernel TCP receive/send buffery se z VmRSS Rust procesu **nezapočítávají**; samostatně platí sdílený payload + 16KiB chunky, 30s aplikační poll watchdog a Nginx `send_timeout 30s`. Test čeká na zelené CI; dokud neprojde, neoznačovat G-03.07 za přijaté.

## Závěrečné testové výsledky 10/10 — bez nasazení

[**CI #39848794079**](https://github.com/Boym323/AirRadarV2/actions/runs/39848794079), source SHA **`c43ab9d543a740020ac9133941f1a4829b2c7dea`**, GitHub-hosted `ubuntu-24.04` (přesný CPU model/logické CPU a fyzická RAM se od následujícího CI budou tisknout jako `G03_SSE_HOST`). Rust fmt, Clippy, unit/integrační testy, česká dokumentace, OpenAPI, Vite a skutečný Chromium browser: **3/3 PASS**. Žádná produkční V1, lokální ADS-B stanice ani DB při měření nebyla oslovena.

| Klienti | Letadla | Doručeno | p95 od atomického JSON (ms) | p99 (ms) | CPU % jednoho jádra | Peak Rust RSS (KiB) | Aktualizační SSE B/s |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 25 | 1/1 | 1 917 | 1 917 | 0,52 | 7 016 | 4 081 |
| 10 | 250 | 10/10 | 1 975 | 1 975 | 1,01 | 9 324 | 389 187 |
| 100 | 250 | 100/100 | 1 878 | 1 884 | 1,06 | 11 272 | 4 105 916 |
| 10 | 1 000 | 10/10 | 1 899 | 1 899 | 3,16 | 16 216 | 1 604 125 |
| 1 | 5 000 kompaktních | 1/1 | 1 777 | 1 777 | 9,00 | 36 848 | 534 284 |
| 10 | 5 000 kompaktních | 10/10 | 1 889 | 1 889 | 9,53 | 42 108 | 5 000 629 |
| 100 | 5 000 kompaktních | 100/100 | 1 576 | 1 630 | 20,52 | 48 764 | 57 005 762 |

**Pozor:** latence zahrnuje 1Hz JSON poll i SSE publish, CPU procenta platí pro samotný Rust proces ve sdíleném GitHub runneru (nikoli pro cílový 1vCPU LXC), VmRSS **nezahrnuje OS TCP buffery**, throughput se počítá z nových SSE bytes během měřené aktualizace (bez initial snapshotu). 5k sparse stav nemá souřadnice/telemetrii; 5k dense může přesáhnout 1MiB veřejný limit a skončí explicitním `degraded` / HTTP 503, nikoliv tichou ztrátou cílů. Čísla jsou jednotlivý izolovaný CI běh, ne výkonový SLA.

**G-03.07 další chaos důkaz:** 90 skutečných TCP klientů bez čtení response body + 1 rychlý čtenář doručil **5/5 aktualizací**, Rust RSS vzrostlo o **15 112 KiB** proti předběžné baseline (≤25MiB budget), špička **63 876 KiB**. Aplikační `ClientLease` vrací permit po 30s bez Body poll; Nginx vzor má `send_timeout 30s`. Neprokázána pevná hranice RAM pro kernel socket buffers ani skutečný cílový LXC.

**G-03.08 browser:** skutečný Chromium EventSource na `http://127.0.0.1:4100/` (Vite) i `http://127.0.0.1:8082/` (Nginx) předal snapshot, delta a nový full snapshot po reconnectu, bez výpadku (log `G03_BROWSER_EVENTSOURCE`).

**Před provozním GO:** nainstalovat schválený root-owned verifier a stage-aware systemd/instalátor na izolované V2 DEV LXC, ověřit reálný Nginx streaming/RSS/p95/p99 s explicitním souhlasem a nanečisto úspěšný návrat G-03→G-00 včetně systemd režimu. Automatická release pipeline na `main` by jinak mohla zahájit přepnutí služby, proto **NO-GO pro merge do main**, PR zůstává draft. Tato omezení nepopírají 10/10 implementačních testů v CI.

## Audit před merge do main — P0 DEV release gate

Kontrola [PR #11](https://github.com/Boym323/AirRadarV2/pull/11) před merge zjistila P0: samotný push do `main` spouští release pipeline včetně publikace prerelease a pokusu o self-hosted DEV deployment, ačkoliv aktuální root-owned G-00 systemd/rollback není připraven pro G-03. Proto nová *fail-closed* stage brána v `.github/workflows/g02-dev-candidate.yml` dovolí validovat a sestavit G-03 immutable artefakt v GitHub Actions, ale blokuje **publikaci prerelease i deploy** při `contracts/openapi.g03.yaml`. Doplněny statické Python regresní testy kontrolující přesné chování. Toto umožňuje splnit vlastníkovo zadání „sloučit kód do main“ bez neoprávněného přepnutí DEV služby. Důkaz bezpečného provozního cutoveru zůstává **NEOVĚŘENO**. Žádný test ani tento gate neospravedlňují prohlásit G-03 za živě nasazenou.


## Automatické vydávání a nasazování G-03 podle požadavku vlastníka

Původní zákaz publikace z `main` byl zrušen. Jediný schválený GitHub Actions řetězec nyní publikuje a nasazuje každý **úspěšně otestovaný** commit přes již existující self-hosted runner. Root-owned instalátor umí přepnout G-03 systemd mód a immutable Rust/Vite artefakt jako jednu transakci, ověřuje skutečný HTTP/SSE stav i Nginx a při selhání rollbackuje **systemd režim i binárku**. Jednorázové aktualizování root-owned installeru vyžaduje správce V2 DEV LXC podle runbooku, protože runner má jen úzce vymezené sudo pravidlo. Před prvním `deploy-dev=PASS` je skutečný provoz G-03 stále **NEOVĚŘENO**; žádná změna V1 ani DB.
