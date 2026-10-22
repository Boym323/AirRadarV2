# G-03 — výsledkový report Rust HTTP API a SSE

**Datum:** 22. 10. 2026. **Stav:** ROZPRACOVÁNO, dosud NEPŘEVZATO. **Vývoj izolován ve větvi**, bez nasazení a bez změny produkční V1 či G-02.11 live-soak.

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
