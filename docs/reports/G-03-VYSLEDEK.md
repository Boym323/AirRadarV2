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
