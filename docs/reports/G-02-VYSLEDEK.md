# G-02 — průběžný výsledkový report implementace lokálního ingestu

**Datum:** 22. 10. 2026. **Stav:** **ROZPRACOVÁNO; zatím NEPŘEVZATO jako celá etapa**.  
**Nasazená verze v DEV LXC:** `227a8619b590d2c536761ffefc75cd0a5633936c` — zůstává G-00/G-01, tento nový G-02 kód **není nasazen**.  
**Ověřený historický základ G-02:** [`18fcbed9039207b3b83fcc28558ca6c5b0cc363f`](https://github.com/Boym323/AirRadarV2/commit/18fcbed9039207b3b83fcc28558ca6c5b0cc363f).  
**Historické CI:** [#39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657), 22 Rust testů PASS. **Novější CI:** [#39769778992](https://github.com/Boym323/AirRadarV2/actions/runs/39769778992), 26 Rust testů PASS, 3/3 joby PASS (včetně dokumentace); oba běhy bez reálného ADS-B receiveru a DB.

## Historický mezistav před doplněním implementace

Následující tabulka zachycuje starší průběžnou kontrolu; **aktuální testové výsledky a 10/11 doložených kroků jsou v závěrečné části „Implementační dokončení a měření CI“ níže.**

| Krok | Historický stav | Důkaz a tehdejší omezení |
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

## Implementační dokončení a měření CI — 22. 10. 2026

**Kód SHA:** [`81a27295d78753a6a3d987e7d68d6b8394c12ef5`](https://github.com/Boym323/AirRadarV2/commit/81a27295d78753a6a3d987e7d68d6b8394c12ef5). **Testy:** [GitHub Actions #39769323592](https://github.com/Boym323/AirRadarV2/actions/runs/39769323592), 3/3 joby PASS; `cargo fmt`, Clippy `-D warnings`, **26 Rust testů**, offline JSON+BEAST replay, Vite lint/typecheck/test/build a dokumentační validace PASS.

**Nově doložené G-02.04/.05/.06/.08/.10:** konzervativní DF17 CRC a barometrická výška (Q=1), TC19 rychlost/heading/vertikální rychlost; odmítnutí nedůvěryhodných AP rámců, lokální CPR jen s čerstvou trusted JSON polohou; typovaná validovaná readsb telemetrie; jeden RAM konzument pro oba transporty; test přechodu JSON/BEAST, TCP reconnectu a zdraví zdrojů. Deterministické CLI `--g02-replay` nyní pokrývá oba transporty a chybný CRC případ.

### GitHub CI syntetické replay měření (debug profil, Ubuntu 24.04 x86-64, nikoli cílové LXC)

| Test | Opakování/pozorování | Celkem | p95 | RSS na konci | CPU ticks |
|---|---:|---:|---:|---:|---:|
| Fixture 1× | 1 000 průchodů | 174,7 ms | 182 µs/průchod | 5 920 KiB | 18 |
| Fixture 2× | 2 000 průchodů | 348,6 ms | 182 µs/průchod | 5 944 KiB | 34 |
| Fixture 4× | 4 000 průchodů | 697,4 ms | 181 µs/průchod | 5 972 KiB | 70 |
| 5 000 identit 1× | 45 000 syntetických pozorování | 101,8 ms | 10 808 µs/dávka 5 000 | 8 812 KiB | 10 |
| 5 000 identit 2× | 85 000 syntetických pozorování | 184,7 ms | 10 708 µs/dávka 5 000 | 8 812 KiB | 18 |
| 5 000 identit 4× | 165 000 syntetických pozorování | 355,5 ms | 10 812 µs/dávka 5 000 | 8 812 KiB | 36 |

**Metodika:** `--g02-benchmark` čte `/proc/self/status` pro RSS a `/proc/self/stat` jako raw ticks. První tři řádky časují celý syntetický JSON+BEAST replay 3 letadel. Poslední tři řádky časují dávky 5 000 unikátních identit během 8/16/32 cyklů (bez SDR, TCP, databáze a webu); po naplnění stavu úmyslně vloží 5 001. identitu a ověří 1 odmítnutí kapacity i úplné TTL odstranění. Nejde o měření skutečného rádio toku ani o důkaz provozního CPU/RAM limitu cílového 1vCPU DEV LXC. Žádná úspora SSD/IOPS nebyla měřena. Šest funkcí M01 je **TESTED**, ne **ACCEPTED**.

**Zbývající povinná přejímka G-02.11:** bezpečně a izolovaně spustit tuto binárku `--g02-benchmark` a dlouhodobý soak/reconnect na cílovém DEV LXC; zaznamenat SHA binárky, počet vCPU/RAM, CPU/RSS, ztráty a p95 a ověřit chování na vybraném lokálním testovacím vstupu. Stávající službu G-00/G-01 zatím nevyměňovat. Bez testu na cílovém LXC **není G-02 finálně HOTOVO**.

**Reprodukovatelný postup testu na cílovém 1vCPU DEV LXC:** [G-02.11 runbook](../runbooks/G-02-DEV-LXC-PREJIMKA.md). Dokud není doložen výstup těchto kroků, G-02 zůstává 10/11.

### Automatizace přejímky G-02.11 (návrh; NENASAZENO)

Bezpečné [automatické DEV ověření](../runbooks/G-02-AUTOMATICKA-PREJIMKA.md) zahrnuje SHA-pripnuté prerelease balíčky z důvěryhodného GitHub CI a odchozí rootless pull časovač v samostatném DEV LXC. JSON report přes existující privátní Nginx. Samotná instalace skriptu v LXC, syntetické výsledky přímo v LXC a reálný dlouhodobý BEAST/JSON soak zůstávají **NEOVĚŘENO**. Nepovažovat za přijetí G-02.

## Výsledek automatického offline benchmarku přímo z DEV LXC — 22. 10. 2026

**První brána G-02.11 = PASS (syntetická zátěž).** Uživatel 22. 10. 2026 doložil výstup `curl -i https://airradarv2.pomykal.cz/g02-status.json` z Macu: HTTP 200, JSON `status=PASS_SYNTHETIC_LXC`, `replay=PASS`, `benchmark=PASS`. Přímé nezávislé načtení veřejného endpointu z prostředí asistenta nebylo možné (síťový přístup omezen). Tento záznam vychází z **uživatelem předaného skutečného výstupu**, nikoli z odhadu nebo reprodukce na GitHub runneru.

- **Testovaný SHA binárky:** `daa15703610b219b5b413319a4920659e19a8694`.
- **UTC:** `2026-10-22T12:02:06.838135+00:00` (= 14:02 SELČ), publikovaný HTTP výstup ze 14:10 SELČ.
- **Prostředí:** samostatný DEV LXC; `visibleCpuCount=1`; `cpuQuota=max 100000` (bez cgroup CPU quota; přímá verifikace affinity/cpuset zatím chybí).
- **Sestavení:** [GitHub Actions #39774198432](https://github.com/Boym323/AirRadarV2/actions/runs/39774198432) (opakovaný PASS 26 Rust + 3 Python testy), [SHA-pripnutý release](https://github.com/Boym323/AirRadarV2/releases/tag/g02-dev-daa15703610b219b5b413319a4920659e19a8694).
- **Metoda:** release Rust binárka `--g02-replay` a `--g02-benchmark` mimo běžící systemd aplikaci; žádný živý readsb/BEAST, DB, API, SSE ani zápisy historických dat.

| Offline scénář | Celkem | p95 na průchod/dávku | RSS | CPU raw ticks |
|---|---:|---:|---:|---:|
| Fixture replay 1× (1 000 průchodů) | 24,2 ms | 26 µs | 2 332 KiB | 2 |
| Fixture replay 2× (2 000 průchodů) | 47,5 ms | 24 µs | 2 364 KiB | 5 |
| Fixture replay 4× (4 000 průchodů) | 99,5 ms | 38 µs | 2 392 KiB | 10 |
| 5 000 identit 1× (45 000 pozorování) | 23,4 ms | 2 399 µs/dávka 5 000 | 5 236 KiB | 2 |
| 5 000 identit 2× (85 000 pozorování) | 40,9 ms | 2 546 µs/dávka 5 000 | 5 256 KiB | 4 |
| 5 000 identit 4× (165 000 pozorování) | 77,0 ms | 2 467 µs/dávka 5 000 | 5 256 KiB | 8 |

`--g02-replay`: 3 tracky / 3 aktualizace / 1 duplicita / 0 konfliktů. U všech tří 5 000-track variant `drops=1` je **úmyslný negativní kapacitní test 5 001. identity**. RSS z `/proc/self/status` je hodnota příslušného krátkého testu; raw ticks z `/proc/self/stat` nepřevádět na procenta CPU bez metodiky HZ a wall-clock/cgroup evidence. Neprokazuje maximální paměť ani CPU za hodinový soak. Není metodicky správné přímo porovnávat výsledky s **debug** profilem předchozích GitHub CI měření: LXC testuje `--release` optimalizovanou binárku.

**Přejímací závěr:** Offline replay a syntetické testy na cílovém LXC úspěšné, bez požadavku na ruční přenos binárky. **Druhá brána G-02.11 zůstává NEOVĚŘENO:** bezpečný reálný JSON/BEAST long-running soak, reconnect, bounded RAM/CPU a kontrola provozních dropů. Pořád **G-02 = 10/11, ROZPRACOVÁNO; J01/J02/J03/J04/J05/J11 = TESTED**, ne ACCEPTED. Nejde o doklad automatického nasazení G-02 do běžícího webu a nebyl proveden zásah do V1.

## Nový automatizovaný izolovaný TCP/JSON soak — čeká na důkaz z cílového LXC

V kroku **G-02.11 (J02/J03/J04/J05/J11)** rozšiřuje `--g02-benchmark` následný integrační scénář `isolated-loopback soak`: skutečný `poll_beast` přes TCP `127.0.0.1:0`, fragmentované golden rámce, šest uměle ukončených socketů, automatický reconnect, skutečný `poll_json` nad náhodným izolovaným testovacím souborem, jeho poškození a oprava, jediná RAM identita, bounded fronta, TTL a výpis p95 do `benchmarkLog`. Test nečte V1, reálný readsb, DB ani nepřepíná DEV service. **Zapsaná specifikace není sama o sobě PASS:** zatím čeká na úspěšné nové CI a výstup na cílovém DEV LXC pro přesný nový SHA. Dlouhý hodinový soak fyzického přijímače je samostatná nepřevzatá část.
