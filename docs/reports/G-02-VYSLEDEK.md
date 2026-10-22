# G-02 — průběžný výsledkový report implementace lokálního ingestu

**Datum:** 22. 10. 2026. **Stav:** **ROZPRACOVÁNO; zatím NEPŘEVZATO jako celá etapa**.  
**Nasazená verze v DEV LXC:** `227a8619b590d2c536761ffefc75cd0a5633936c` — zůstává G-00/G-01, tento nový G-02 kód **není nasazen**.  
**Ověřený GitHub základ G-02:** [`18fcbed9039207b3b83fcc28558ca6c5b0cc363f`](https://github.com/Boym323/AirRadarV2/commit/18fcbed9039207b3b83fcc28558ca6c5b0cc363f).  
**CI:** [#39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657), všechny **3/3 joby PASS**, 22 Rust testů PASS, Rustfmt, Clippy, Vite/TS a kontrakty. Bez reálného ADS-B receiveru a DB.

## Doložená implementace

| Krok | Stav | Důkaz a omezení |
|---|---|---|
| G-02.01 Rozdělení modulů | **PASS** | `apps/engine/src/ingest/{mod,json,beast,mode_s,state,supervisor}.rs`; testy izolované od V1 |
| G-02.02 JSON adapter | **PASS** | asynchronní čtení souboru, 4MiB limit, parse/clock validace, chybné soubory, fixture a skutečný temp-file test |
| G-02.03 BEAST TCP framing | **PASS** | incremental 0x1A/31/32/33, escaping, dělení TCP chunků, corrupt resync, bounded 256 output chunk, loopback TCP/reconnect test |
| G-02.04 CRC/Mode-S | **ČÁSTEČNĚ** | validované DF17 CRC24 a golden CPR; cizí DF/AP se **nepovyšují na platný ADS-B**; širší pokrytí AP/PI a subtype testů zatím chybí |
| G-02.05 CPR/poloha | **ČÁSTEČNĚ** | globální airborne even/odd CPR do 10 s s golden polohou; lokální CPR podle důvěryhodné reference, range guard a širší fuzz dosud nejsou implementovány |
| G-02.06 RAM state | **ČÁSTEČNĚ** | jedna `RadarState`, pozice/callsign/squawk, stáří a kvalita. Chybí explicitní další normalizované letové údaje — výška, rychlost, směr a vertikální rychlost |
| G-02.07 TTL/deduplikace | **PASS v syntetickém replay** | max. 5 000 tracků, 60s TTL, 15s freshness, přednost kvalitního JSON, fyzikálně nemožný skok nemění metadata; pending ani SQL zde nejsou |
| G-02.08 fallback/arbitráž | **ČÁSTEČNĚ** | JSON a BEAST bounded Tokio tasky, BEAST retry a jitter, loopback reconnect; kompletní end-to-end přepnutí s výpadky obou zdrojů ještě neověřeno |
| G-02.09 backpressure/limity | **PASS v CI** | 4MiB JSON, 5 000 cílů, 2 048 bounded mpsc zpráv, 256 BEAST rámců/volání, 4KiB TCP read, timeouty 2/3/5 s |
| G-02.10 deterministický replay | **ČÁSTEČNĚ** | spustitelné `--g02-replay`, syntetické JSON/BEAST testy a loopback, ale ne úplné pokrytí G-02.04/.05/.06 |
| G-02.11 měření | **ČÁSTEČNĚ** | skutečný 1×/2×/4× **syntetický** výkon v CI, ale ne profiling cílového 1vCPU LXC, 5 000 cílů ani soak/reconnect pod zátěží |

Kroky uvedené PASS jsou otestované ve specifikovaném omezeném rozsahu; **žádná aktivní produktová KEEP funkce tím automaticky nezískává status `ACCEPTED`**.

## Měřená syntetická baseline v CI

**Metoda:** GitHub Actions `rust-engine` job Ubuntu 24.04 / x86_64, debug `cargo run --locked --quiet`, 1 000/2 000/4 000 opakovaných trojic readsb fixture, vždy **3 stejné syntetické identity**; p95 na celý průchod `replay()` a RSS přibližně z `/proc/self/status`, CPU jako Linux raw `/proc/self/stat` ticks (bez předpokladu HZ). Nezahrnuje TCP network I/O, Vite mapu ani DB. Hodnoty nelze zobecňovat na reálný 1vCPU LXC.

| Zátěž | Opakování | Celkový čas | p95 replay | RSS na konci | CPU ticks |
|---|---:|---:|---:|---:|---:|
| 1× | 1 000 | 71,4 ms | 76 µs | 5 676 KiB | 7 |
| 2× | 2 000 | 142,6 ms | 76 µs | 5 692 KiB | 14 |
| 4× | 4 000 | 283,9 ms | 75 µs | 5 712 KiB | 29 |

Tyto hodnoty měří **opakování fixture**, nikoli skutečné 1×/2×/4× ADS-B radio traffic. Není důkaz úspory SSD, PostgreSQL vůbec není připojena. Replay vypsal **3 tracky, 3 aktualizace, 1 duplicitu, 0 konfliktů**.

## Bezpečnost a způsob spuštění

- Běžný systemd a dosud nasazený web zůstávají **G-00 stub**, `live_ingest=false`, `aircraft=NOT_IMPLEMENTED`. Skutečné REST/SSE a mapa až G-03/G-05.
- `--g02-replay` spouští pouze vestavěné syntetické vstupy bez přístupu do LAN.
- `--g02-benchmark` opakuje replay a vypíše lokální CPU/RSS/latenci; sám neotevírá DB.
- `--g02-live` **explicitní volitelný oddělený režim**, který lze spustit pouze po nastavení `AIRRADAR_V2_READSB_JSON` a/nebo `AIRRADAR_V2_BEAST_ADDR`; dnes není aktivován, **nepřipojovat jej na produkční V1 readsb bez schválení read-only odběru**.
- Žádná konfigurace V1, databáze, LXC, notifikací, externích providerů nebo aktuálního DEV releasu nebyla změněna.

## Zbývá pro bezpečné převzetí G-02

1. Doplnit konzervativní dekódování potřebných letových údajů (výška/rychlost/heading) s odpovídajícími G-01 DTO a unit testy a bezpečně ověřit další potřebné DF/AP případy.
2. Rozhodnout a otestovat lokální CPR pouze tehdy, je-li k dispozici důvěryhodná referenční poloha; doplnit range/plausibility a fuzz testy pro záměrně poškozené frame sekvence.
3. End-to-end test zdrojové arbitráže při interleaving JSON a BEAST, síťových výpadcích a přetížení včetně bounded RAM a metrik.
4. Skutečné 1×/2×/4× soak a baseline na cílovém DEV LXC, bez ovlivnění V1, teprve po bezpečné přípravě izolovaných testovacích vstupů.
5. Doložit všechny aktivní KEEP ID M01 v `contracts/feature-matrix.json`, přejímací evidenci kroků, CI finálního SHA a teprve potom měnit stav celé G-02 na HOTOVO.

**Konečný stav:** významná část G-02 existuje a synteticky funguje, ale **G-02 není dokončená**. Další vývoj se soustředí na uvedené blokace, nikoli na produkční nasazení.

## Závěrečná dodatečná kontrola bounded RAM

Při revizi bylo opraveno **uvolňování interní mapy `position_quality` společně s expirovaným `tracks`**. Bez toho mohla tato pomocná RAM evidence růst s počtem historicky pozorovaných identit i po vypršení 60s TTL. Regresní test `ingest::state::tests::replay_dedup_and_ttl` nyní kontroluje, že po expiraci jsou **obě mapy prázdné**. Důkaz: [CI 39763140573](https://github.com/Boym323/AirRadarV2/actions/runs/39763140573), **3/3 PASS** na [commitu `be564960`](https://github.com/Boym323/AirRadarV2/commit/be56496015f744a2d5b16fd68bebb1f8ca0d815d). Tento konkrétní test neprokazuje dlouhodobý soak ani všechny podmínky G-02.11.

**Stav zůstává ROZPRACOVÁNO**: dokončené a ověřené jsou pouze uvedené dílčí kroky, nasazené LXC stále provozuje dřívější stabilní základ.
