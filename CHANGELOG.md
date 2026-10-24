# CHANGELOG — AirRadar V2

## 24. 10. 2026 — Kratší názvy GitHub Actions

- Zkráceny pouze viditelné názvy workflow, jobů a kroků v CI, DEV release, G-04 PostgreSQL a denních statistikách. Jednotné české prefixy `CI`, `G01`–`G05`, `Rust`, `Web`, `DEV` a `Statistiky`.
- Beze změny zůstaly ID jobů, povinný status check `Česká dokumentace`, názvy artefaktů, testovací příkazy, podmínky i samotné nasazování.
- Regresní test hlídá maximální délku 35 znaků a zachování identifikátorů důležitých pro CI.

## 24. 10. 2026 — Provozní audit issue #41 a stabilizace DEV LXC

- Ověřen DEV release `43e20d8ce460a5bd3f12503db22ae98c4d02db85`, health G-04, živý BEAST ingest, PostgreSQL na odděleném DB LXC, SSE snapshot/delta a mapový fallback.
- Opravena instalovaná DEV systemd evidence: `ConditionPathExists` a účinné `TimeoutStartSec` pro oneshot testovací služby; bez zásahu do V1, PP Studio nebo PostgreSQL konfigurace.
- Runner po úspěšném SHA-bound deployi atomicky publikuje skutečný `current/VERSION` do `/dev-deploy-status.json`; doplněn regresní test.
- Retence G-04 zůstává deaktivovaná do DBA schválení; fyzický GPU/FPS test a release-security evidence zůstávají NEOVĚŘENO/BLOKOVÁNO podle reportu [issue #41](docs/reports/ISSUE-41-PROVOZNI-BRANY-2026-10-24.md).

## 24. 10. 2026 — Omezení opakovaných PR/main testů bez oslabení release gate

- Po úspěšném `pull_request` CI a odděleném PostgreSQL CI se ukládá krátkodobý artefakt s PR číslem, přesným HEAD SHA, ID a pokusem běhu a Git tree skutečně otestovaného merge checkoutu.
- Následný `main` workflow zkontroluje vazbu sloučeného PR na konkrétní commit, úspěšný výsledek odpovídajícího CI a shodu Git tree s výsledným commitem. Pouze v takovém případě přeskočí **stejnou** kompletní testovací sadu. Testy G-04 vyžadují samostatný důkaz; jejich provedení není nahrazeno obecným CI.
- Přímý push do `main`, změna výsledného stromu, neplatný/expirující artefakt, odlišný běh nebo výpadek GitHub API znamenají **fail-closed** návrat k plným testům. Sestavení, vydání, DEV deployment a jeho provozní kontroly se nepřeskakují.
- Regresní testy pokrývají úspěšnou shodu, odlišný tree, jiný pokus CI, přímý push i oddělení databázového důkazu. G-04 ruší také zbytečné rozpracované PR běhy při novém commitu.


## 24. 10. 2026 — Oddělená retence G-04 DEV (připraveno, NEAKTIVOVÁNO)

- Auditní oprava G-04.09/J09: root/DBA-managed script `database/maintenance/g04_retention_bounded.sh` čistí pouze V2 DEV, max. 20 dávek po 1 000 řádcích a 90 s. Vyžaduje dedikovanou SQL roli a explicitní GO, běžná Rust runtime role nezískává `DELETE`.
- Přidány samostatné systemd šablony pro DB LXC, bezpečnostní [ADR 0017](docs/adr/0017-g04-dba-retence.md), [runbook](docs/runbooks/G-04-RETENCE-30-DNI.md) a izolovaný PostgreSQL 18 CI test. Po merge se nic na DB LXC neinstaluje a timer se automaticky nezapíná; provozní schválení a měření stále chybí.


## 24. 10. 2026 — Auditní opravy mapy G-05.08 a obnovy podkladu

- `R09/G-05.08`: kamera `follow` nespouští animaci každou sekundu při stejné poloze; nový výběr nebo opětovné zapnutí stále provede úvodní přesun. Respektováno `prefers-reduced-motion`.
- `R14/G-05.09`: MapLibre odstraní zastaralé upozornění při úspěšném `style.load` a při dokončení načítání dlaždic. Stream i seznam zůstávají nezávislé na mapovém podkladu.
- Regresní testy: Vitest `followTargetChanged` a offline Chromium emulace chyby a obnovy podkladu; průběžná CI není dokladem fyzického GPU výkonu. G-05.11 zůstává otevřena.


## 24. 10. 2026 — Provozní oprava auditu G-03/G-04 (ověření probíhá)

- `J09` — zdravotní stav PostgreSQL nyní odpovídá read-only `SELECT 1` sondě s intervalem 15 s; HTTP při selhání přiznává `degraded`, namísto statického `database_connected=true`.
- `J06/J07/J08` — společný dohled HTTP/ingest a řízené vypnutí na SIGTERM/SIGINT, G-04 umožňuje 65s SQL drain. Neočekávané selhání ingestu vyvolá chybový exit pro systemd restart.
- Přidány Rust regrese `g04_health_reflects_database_up_and_down` a `unexpected_ingest_end_closes_http` a [ADR 0016](docs/adr/0016-g04-runtime-health-a-dozor-ingestu.md). Dokud neskončí CI a cílená DEV přejímka, nelze tvrdit PASS; ostatní G-04/G-05 kapacitní limity zůstávají otevřené.


## [Nepublikováno]

### Issue #41 a stabilita mapového podkladu
- Ochrana `main` je aktivní přes GitHub Branch Protection API; snapshot nad 1 MiB je fail-closed a retenční SQL je ověřené v izolaci i read-only proti DEV DB. Podrobný český audit je v [reportu](docs/reports/ISSUE-41-PROVOZNI-BRANY-2026-10-24.md).
- MapLibre při výpadku externího OpenFreeMap stylu nebo při třech chybách tiles v krátkém okně použije lokální background-only fallback; jednotlivá chyba pouze stabilně zobrazí upozornění po omezenou dobu. Letadla a seznam zůstanou dostupné bez remountu. Chromium regrese pokrývá úplný i opakovaný výpadek podkladu.

### G-05 vizuální doladění hlavičky a identity (24. 10. 2026)
- Vlastní inline SVG symbol AirRadar místo systémového znaku letadla.
- Zjednodušený podtitul a patička bez trvalého označení interní etapy G-05.
- Seskupené ovládání letišť a mapového podkladu; méně dominantní hlášení chyby WebGL.

### Dokumentační oprava aktuálního stavu G-05/G-06 (24. 10. 2026)
- Sjednocen aktuální stav v README, `ZACNI-ZDE-NOVE-VLAKNO.md` a `ETAPY-A-STAV.md`.
- Opraveny zastaralé údaje o G-04, G-05 (10/11 kroků), G-06 a počtu funkcí `TESTED/ACCEPTED`.
- Výslovně ponecháno otevřené pouze G-05.11 z důvodu chybějícího fyzického GPU/FPS měření; full REST/frekvence/NAVAID zůstávají v G-06.

### G-05.11 — CI MapLibre benchmark a adaptivní diff (24. 10. 2026)
- [PR #34](https://github.com/Boym323/AirRadarV2/pull/34) sloučen `b8691f8b`, [CI #40095503907](https://github.com/Boym323/AirRadarV2/actions/runs/40095503907) 3/3 PASS; 32 skutečně naměřených software Chromium scénářů, `setData/updateData`, 25/100/500/2 000 cílů při 1/2 Hz, desktop/mobile emulation. [Celý p95 report](docs/reports/G-05-VYKON-MERENI-2026-10-24.md) s odkazy na JSON artefakt.
- [PR #36](https://github.com/Boym323/AirRadarV2/pull/36) sloučen `3bf49297`: podmíněné GeoJSON updateData pro hustý provoz, skip idle stavu a bezpečný fallback při nové epoše/stylu. [CI #40096318946](https://github.com/Boym323/AirRadarV2/actions/runs/40096318946) 3/3 PASS, včetně 200/2 000 SSE delta.
- **G-05.11 stále otevřená**: software RAF měření není ověřený fyzický GPU/FPS desktop/telefonu.



### G-06.01/.02/.03/.07 → G-05.10 — legální letištní data a mapová vrstva (24. 10. 2026)
- OurAirports The Unlicense, přesný SHA upstream CSV ověřený při Vite buildu; statický regionální JSON CZ/SK/AT/PL/HU s validovanými pozicemi a testy importéru. Žádná externí runtime síťová služba.
- Oddělený statický GeoJSON source MapLibre, přepínač letištní vrstvy a klikací kontextový detail. NAVAID, frekvence a úplný G-06 REST jsou mimo tento základ; G-05.10/R12 pro mapovou vrstvu jsou TESTED: PR #33 `bfa48c3f`, [CI #40094724356](https://github.com/Boym323/AirRadarV2/actions/runs/40094724356) 3/3 PASS a [DEV #40095020297](https://github.com/Boym323/AirRadarV2/actions/runs/40095020297) SUCCESS. Plná G-06 REST/NAVAID/frekvence dál rozpracovaná.
- [Průběžný report](docs/reports/G-06-VYSLEDEK.md). CI a DEV nasazení pro mapový rozsah G-05.10 ověřeno, viz předchozí bod.



### G-05.04/.07 — čitelné popisky a přerušená stopa (24. 10. 2026, CI před přejímkou)
- R04: štítky jen z validovaného krátkého callsignu; chybějící/nereálnou výšku neuvádíme, selected má prioritu proti ostatním nekolidujícím textům.
- R08: GeoJSON MultiLineString bez falešných spojnic přes výpadek >120 s nebo antimeridián; browserově udržovaná stopa max. 64 bodů / 20 min zůstala.
- Vitest pro segmentaci, podvržený text, retenci a odstranění; Chromium WebGL barevný obrazový test vykreslení stopy s omezeným testovacím Pillow balíčkem. PR #32 `87838925`, [CI #40087996178](https://github.com/Boym323/AirRadarV2/actions/runs/40087996178) 3/3 PASS a [DEV #40088791328](https://github.com/Boym323/AirRadarV2/actions/runs/40088791328) SUCCESS; R04 a R08 TESTED.



### G-05.03 — veřejné validované ADS-B emitter category, PR #30 (24. 10. 2026)
- [CI #40084080300](https://github.com/Boym323/AirRadarV2/actions/runs/40084080300) 3/3 PASS, [izolovaný G-04 SQL #40084080298](https://github.com/Boym323/AirRadarV2/actions/runs/40084080298) PASS; squash merge `9c398c2f`.
- Přes readsb JSON a CRC-validní identifikační BEAST DF17 přidána validovaná `emitterCategory` do jediné Rust RAM a volitelného veřejného G-03 SSE DTO. Staré snapshoty zůstávají kompatibilní; rezervované a nepovolené kódy se vynechají.
- JSON Schema/OpenAPI + G-01 TS parser, G-05 browser store, Rust/TypeScript testy a izolovaný Chromium smoke se 2 000 symboly. Zdroj a bezpečné pravidlo merging zapsáno v [ADR 0015](docs/adr/0015-g05-emitter-category.md).
- **PR #30 sloučen `9c398c2f`, CI #40084080300 3/3 PASS**, G-05.03/R03 a G-05.08/R09 nyní TESTED dle zaznamenaných browser/kontraktových důkazů; G-05.10 (G-06) a skutečný GPU/FPS G-05.11 zůstávají NEOVĚŘENO. Žádné změny V1 ani G-04 SQL.



### G-05.03/.04 — původní mapové siluety a adaptivní štítky (24. 10. 2026, PR #29)
- [CI #40082045615](https://github.com/Boym323/AirRadarV2/actions/runs/40082045615) **3/3 PASS**, včetně Chromium MapLibre/SSE smoke; merge `8309f069`. Plná R03/R04 akceptace bez category DTO a GPU benchmarku stále otevřená.
- Nový `apps/web/src/radar/aircraft-icons.ts`: 10 vlastních vektorových Path2D ikon registrovaných v MapLibre, bezpečný fallback při chybějícím typu/směru, jediný GeoJSON source.
- `RadarMap.tsx` přidává zvýraznění vybraného cíle a kolizní popisky podle zoomu (callsign, validní výška), zachovává G-03 SSE, follow, stopu, přepnutí podkladu a živý seznam.
- Kategoriové metadata jsou pouze volitelná příprava ve webovém store; G-03 API ani Rust nebyly upraveny. Full R03 přejímka a G-05.11 GPU benchmark jsou **otevřené**, žádné automatické povýšení `TESTED`.


### G-05 až G-10 — vizuální specifikace, ikony a audit (24. 10. 2026)
- Nová [specifikace mapových ikon a popisků](docs/plan/MAPOVE-IKONY-A-POPISKY.md) a [audit vizuální úplnosti](docs/audits/VIZUALNI-ZADANI-AUDIT-2026-10-24.md), návrhové rozměry/typografie/paleta/stavy/vrstvy a akceptační scénáře v [map-first UX](docs/plan/WEB-UX-UI-SPECIFIKACE.md).
- Vazby na G-05 až G-10 doplněny, bez změny aktivních 45 funkcí, runtime/API/databází i dosavadních stavů `TESTED/ACCEPTED`. Veřejné DTO zatím nemá kategorii letadla a originální SVG teprve vzniknou při implementaci.
 — map-first UI (24. 10. 2026)
- G-05 (R01/R06/R08/R09/R14): místo statického dashboardu jedna dominantní mapa, kompaktní navigační lišta, sbalitelný seznam a kontextový panel letu, na telefonu spodní overlay. Jedna MapLibre instance a jeden G-03 SSE store zachovány.
- Detail používá jen reálná DTO pole (výška, rychlost, heading, vertikální rychlost), stale výběr neumožní follow. Chromium E2E: desktop/mobil, zavření panelu/focus, stejný canvas, styl/SSE; [CI #40079234896](https://github.com/Boym323/AirRadarV2/actions/runs/40079234896) 3/3 PASS; PR #26 sloučen do main jako `c17601d5`.
- Není implementována G-05.10 letištní vrstva ani potvrzen G-05.11 referenční GPU benchmark. V1, databáze a produkční prostředí nedotčeny.


Sledujeme **skutečné změny kódu, dokumentace a provozních kontraktů**; všechny zápisy jsou česky. Jednotlivé implementační etapy mají vlastní reporty a přesné odkazy na CI.

## [Nepublikováno]

### UX/UI — schválen princip „mapa na prvním místě“ (24. 10. 2026)
- Nová [specifikace UX/UI](docs/plan/WEB-UX-UI-SPECIFIKACE.md) sjednocuje cílovou informační architekturu, desktop/mobilní panely, stavy a akceptační scénáře G-05 až G-10. Jediná živá mapa je výchozí pracovní plocha, doplňkové informace jsou kontextové.
- Jde **jen o dokumentační návrh** (schválený hlavní princip, detailní vizuální volby otevřené), bez změny kódu, počtu 45 aktivních funkcí, stavů TESTED/ACCEPTED, DB, V1 či nasazení.
- Doplněn [konkurenční UX průzkum](docs/audits/KONKURENCNI-UX-PRUZKUM-2026-10-24.md) se zdrojovaným srovnáním, prioritami P0/P1 a výslovně nepřebíranými prvky; rozhodnutí zůstávají uvnitř aktuálního schváleného rozsahu.

### G-05.08 — navazující ochrana ručního ovládání follow (24. 10. 2026)
- Po nasazení první mapy do DEV opravena konkurence ručního zoomu/pohybu s automatickým follow (R09): wheel, touchmove, šipky, drag/rotate/pitch nyní režim vypnou; programové easeTo jej samo nevypne.
- Tlačítko sledování hlásí `aria-pressed`. Izolovaný Chromium test pokrývá myš/klávesnici/dotyk a zachování MapLibre instance; **CI nové změny se teprve ověří**, R09 i etapa G-05 zůstávají IN_PROGRESS.
- PR #22 již sloučen (23. 10.) a první G-05 mapa nasazena na DEV přes [#40028968380](https://github.com/Boym323/AirRadarV2/actions/runs/40028968380); V1/produkce/DB bez změny.

### G-05 — první ověřená MapLibre mapa (PR #22)
- CI #40024971130 PASS: Chromium MapLibre canvas a izolovaný SSE snapshot/delta fixture bez real ADS-B/DB/external tiles, bez remountu; interakční výběr, smazání a změna stylu. R01 a R06 `TESTED` na základě prohlížečové přejímky, G-05.01/.02/.05/.06 mají jednotkové a browser důkazy.
- Na samostatné větvi Vite/React klient MapLibre GL JS 6.15.0 s připnutým npm lockfile, Vite workerem a OpenFreeMap podkladem Liberty/Fiord. Jeden GeoJSON source, čitelné symboly, známý track/heading, mapová stopa, follow a klikací výběr.
- G-03 SSE adapter s atomickým snapshot/delta, verzovaným epoch/seq resync, bounded 5000 cílů a 1Hz UI publikací; jediný stav sdílený mapou a seznamem. Jednotkové testy identity, smazání, reconnect, validace a retence 64 bodů/20 minut.
- G-05 stále IN_PROGRESS; G-05.10 letiště po G-06, E2E/MapLibre GPU výkonnost G-05.11 neověřena. Bez zásahu do G-03, DB a AirRadar V1 či automatického DEV deploye.


### G-04 — opraven automatický DEV runtime podle hlavní větve
- Release z `main` nyní nese verzovaný `runtime-stage` marker. Po přijetí J09 root-owned instalátor atomicky přepne immutable release na `--g04-live-history`, předá pouze chráněné DB prostředí a ověří `stage=G-04`/`database_connected=true` přes přímé i proxované HTTP/SSE.
- G-04 canary zůstává oddělený; živý RAM/SSE příjem není blokován databází. V1, její LXC/DB, firewall a readsb konfigurace se nemění. Důkaz: ADR 0014, Rust health test, stage boundary, workflow a shell/Python regrese.

### G-04 — etapa uzavřena v DEV (J09 ACCEPTED, 23. 10. 2026)
- Vlastník přijal skutečný 1817,498s DEV canary: `eligible=committed=1312`, 61/61 SQL dávek, 0 dropů/SQL timeoutů, `pending=inflight=0`, `drained=true writer_finished=true`, systemd `Result=success`/`ExecMainStatus=0`/`ActiveState=inactive`, PostgreSQL 1490 → 2802 (+1312). Rust/Web/docs a izolované PostgreSQL CI PASS.
- **DEV ACCEPTED s výslovnou evidenční výjimkou:** doložena vstupní DB baseline, chybí průběžné 5min host disk/WAL/CPU/RAM záznamy a závěrečný G-03 SSE. To není dodatečně vydáváno za naměřené hodnoty ani za PASS úplného evaluatoru. Vlastník na tomto podkladě uzavřel DEV G-04 a nepožaduje další SQL soak; omezení evidence se nepřenáší na produkční akceptaci.
- J09 stav `ACCEPTED`, G-04 `DEV ACCEPTED`. PR #21 zatím bez merge, trvalé spouštění G-04, DB konfigurace, G-03, AirRadar V1 a PP Studio beze změny. Starší níže uvedené stavy `IN_PROGRESS` patří k historickým etapám testování.

### G-04 — opakovaný 30min DEV canary: historický writer a graceful drain PASS
- Na novém immutable G-04 releasu z implementačního SHA `e186c54f8304c3e437f6ed9a54c6bd31722dde7c` (binární SHA-256 `7b7cbf291141bdfbfa4e3a4e8ea7c8914d687cbf1824089721afdcd43e5f5d1e`) správce doložil skutečný 1817,498s DEV canary: `eligible=1312`, `committed=1312`, `pending=0`, `inflight=0`, `coalesced=15239`, 61/61 SQL dávek bez chyby, nulové dropy/SQL timeouty, `connection=Up`.
- Autonomní drain dokončil s `drained=true writer_finished=true`, `Result=success`, `ExecMainStatus=0`, `ActiveState=inactive`, bez systemd timeoutu. Read-only DB readback se změnil z 1490 na 2802 řádků (+1312), po běhu 326 identit; G-04 CPU time 6,085s, peak 3,8MB. **J09 označeno funkčně TESTED.**
- Předtestová DB baseline přes Proxmox Console prokázala 3,1GiB volného disku, WAL 59MiB a 796MiB dostupné RAM; **časovaná 5min DB host evidence a závěrečné G-03/SSE důkazy nejsou dosud doloženy**. Kompletní provozní `G-04 ACCEPTED` proto stále čeká na tyto důkazy a plný evaluator. PR #21 draft, žádný merge ani produktový release.

### G-04 — řízené vypuštění SQL writeru po 30 minutách (ADR 0013)
- Nová opt-in G-04 implementace po 1 802 s vypne jen vlastní příjem, nejvýše 60 s čeká na uzavření 30s bucketů a potvrzení SQL ack, nejvýše 3 s na konec writeru; vypíše finální diagnostiku a při úspěchu ukončí proces s návratovým kódem 0. `systemd RuntimeMaxSec=1875`, nouzový rychlý STOP, G-03 a PostgreSQL zůstávají beze změn.
- Doplněny Rust testy drainu (PASS ack, FAIL ack, nedoručený ack) a izolovaná PostgreSQL/SQLx regrese se skutečným závěrečným drainem. Nová systemd šablona odkazuje na zdrojový commit `e186c54f8304c3e437f6ed9a54c6bd31722dde7c` v odděleném immutable release. Binární SHA nové verze dosud neověřeno na DEV.
- Aktualizovaný read-only postup pro ruční DB host snapshots, G-03 SSE heartbeat, nové SQL řádky a PASS/FAIL evaluator. Fyzické DEV nasazení a opakovaný canary **neprovedeny**, G-04/J09 zůstává IN_PROGRESS.

### G-04 — skutečný BEAST/CPR 31min DEV canary, dílčí PASS
- Ve skutečném BEAST-only DEV provozu vznikaly reálné SQL historické vzorky (snímek tabulky doložen vlastníkem); poslední diagnostika `eligible=1502`, `committed=1489`, `coalesced=17754`, 63/63 úspěšných SQL dávek, 0 SQL timeoutů, 0 zaznamenaných `dropped_*`, `connection=Up`. G-03 po testu `active`.
- Systemd dosažením `RuntimeMaxSec=1875` ukončil canary výsledkem `timeout` (nikoli SQL timeout). V poslední diagnostice zůstalo `pending=13` (0,87 % kandidátů), bez potvrzeného závěrečného flush; G-04 nemá prokazatelný graceful drain. CPU canary 6,499 s, RSS peak 3,8 MB. Bez kompletního DB host readback/metrik a plné evaluace neoznačovat G-04/J09 `ACCEPTED`. Detaily v `docs/reports/G-04-VYSLEDEK.md`.

### G-04 — opravená BEAST/CPR binárka musí být odděleně nasazena
- Šablona ručně dozorovaného canary nyní ukazuje na SHA-pinned release `232d92840a0b053feba3ee13868d8b28be8ed821` (binární SHA-256 `fdb9e3e3c96372637e770d03f22fca1e0b947b1ab614a2ca65f413cab46b774a`). Deployment boundary test výslovně odmítá původní nefunkční `fe0ac66` cestu. V GitHubu není žádný důkaz, že byl nový release fyzicky instalován na DEV; nový live canary proto musí počkat na samostatnou kontrolu a lidský dohled.

### G-04 — oprava BEAST-only ukládání historie v PR #21
- Opravena skutečná příčina `eligible=0`: validní polohy rekonstruované z DF17/CPR jsou označeny `positionOrigin=adsb_cpr`, ale čas zůstává pravdivě `timeBasis=server_arrival`; obecné serverové pozice zůstávají odmítnuté.
- Zachovány jsou `rx-primary`, `beast_tcp`, shodný čerstvý čas, CRC/CPR a fyzikální plausibilita, jediný RAM stav, adaptivní sampler, bounded fronty a SQLx writer. Přidána regrese BEAST-only → RAM → sampler → SQLx → izolovaný PostgreSQL.
- Lokální Rust testy: 57/57 PASS; izolované PostgreSQL CI `39984664523` a hlavní CI `39984664522` PASS. Release binární SHA-256 pro commit `232d92840a0b053feba3ee13868d8b28be8ed821`: `fdb9e3e3c96372637e770d03f22fca1e0b947b1ab614a2ca65f413cab46b774a`.
- Canary nebyl spuštěn; G-03, G-03 LXC, V1, PP Studio a sdílená PostgreSQL konfigurace nebyly měněny. Viz ADR 0012 a průběžný report G-04.

### G-04 — skutečný DEV preflight po přerušeném SSH pokusu: NO-GO
- Ověřeno přímo na `lo-dev-airradarv2`: G-04 zůstává `inactive/dead` bez listeneru na 4114; G-03/API/SSE a Nginx jsou aktivní. Nebyl proveden restart ani zásah do `current`, G-03, V1 nebo jiné DB.
- Potvrzena binárka SHA-256 `1dd11387635eeb16040b69f33e09a6108d1fecf2491dde56699cc9b5971ba21b`, env `root:root 0600`, SQL audit existující DB bez TLS, 1 historický řádek, 4 připojení a `wal_bytes=163397425457`.
- Reprodukované selhání testu zapisovatelnosti (`airradarv2` nemá zápis do `root:airradarv2 0750`); změna práv nebyla provedena, protože G-04 do adresáře nezapisuje. Aktuální fyzické metriky DB LXC a zajištěný nepřetržitý lidský dohled chybí, proto canary nebyl spuštěn a G-04/J09 zůstává `IN_PROGRESS`.

### G-04 — ADR 0011: ruční dohled místo automatického SSH DB monitoringu
- Vlastník zvolil pouze jednorázový osobně dozorovaný DEV canary; unit spouští přímo SHA-pinned binárku, bez `monitor_g04_canary.sh` a bez nového SSH účtu. Zachovány CPUQuota 50 %, MemoryMax 2 GiB, RuntimeMaxSec 1875 s, Restart=no, port 4114 a nedestruktivní rollback. DB disk/WAL/CPU/RAM a zdraví G-03/PP Studio se během testu kontrolují osobně nejpozději každých 5 minut. Riziko neautomatického zastavení při naplnění disku je vědomě přijaté jen pro tento dozorovaný DEV test. G-04 nespouštěna.

### G-04 — DEV preflight canary 23. 10. 2026: NO-GO
- Na DEV LXC byl z přesného SHA `fe0ac66fdebea25d21a2d211cd74cab922dd7d05` sestaven a mimo `current` atomicky nainstalován x86-64 artefakt SHA-256 `1dd11387635eeb16040b69f33e09a6108d1fecf2491dde56699cc9b5971ba21b`; G-04 unit je `static/inactive`, nikdy nespouštěná.
- Doplněn funkčně testovaný fail-closed monitor pro read-only SQL/DB-LXC metriky, disk pod 2 GiB, WAL, CPU/RAM, SQL timeouty, stáří diagnostiky a degradaci G-03. Monitor nepoužívá `pg_ls_waldir()` ani `df --output`; připraven je forced-command helper a schvalovací postup pro samostatný neprivilegovaný SSH účet, který na DB LXC nebyl vytvořen. Kapacita proto zůstává NEOVĚŘENA a preflight NO-GO. Canary nebyl spuštěn; G-04/J09 zůstává `IN_PROGRESS`.

### G-04 — ADR 0010: postradatelná historie V2 bez povinných záloh
- Vlastník výslovně přijal možnost ztráty celé historické databáze V2; denní dump, PITR a provozní RPO/RTO nejsou přejímací bránou. Izolovaný pg_dump/pg_restore zůstává technickým důkazem G-04.11, backup timer se nezavádí. Zůstává povinné sledovat volné místo/WAL/CPU/RAM sdíleného LXC a bezpečný canary/rollback. Zálohování PP Studio, AirRadar V1 a ostatních databází se nemění.


### G-04 — příprava bezpečného DEV canary
- Přidána neinstalovaná systemd šablona G-04 canary na odděleném loopback portu, chráněný env vzor, automatické PASS/FAIL vyhodnocení 30minutového běhu a nedestruktivní rollback na G-03.
- Přidán read-only `pg_dump -Fc` helper s pevným cílem `airradar_v2_dev`; skutečný dump byl uložen mimo DB LXC a obnoven pouze v izolaci. Provozní RPO/RTO, volné místo a CPU/RAM DB LXC zůstávají NEOVĚŘENO.
- Statická brána potvrzuje, že merge/main auto-deploy neobsahuje `--g04-dev-history` a G-03 unit zůstává `--g03-live-api`. G-04/J09 zůstává `IN_PROGRESS`, canary nebyl spuštěn.

### G-04 — provozní audit a izolovaná výkonnostní brána (23. 10. 2026)
- Na auditním SHA `4659d63822d842a69fb0e141b9dcf1c5480e381e` ověřena skutečná DEV DB `airradar_v2_dev`: PostgreSQL 17.10, `pg_stat_ssl.ssl=false`, role a omezená práva bez změny serveru.
- V samostatném lokálním PostgreSQL clusteru PASS výpadek/reconnect writeru, JSON/RAM/G-03 API při nedostupné DB, `pg_dump -Fc`/`pg_restore` a replay 5000 cílů.
- Izolovaný dávkový benchmark 1×/2×/4× doložil wall, p95/p99, WAL, `pg_stat_io`, client CPU/RSS a dávky 128. Nejde o měření sdíleného DB LXC.
- G-04/J09 zůstává `IN_PROGRESS`: chybí serverové CPU, 30denní retenční profil, srovnání 15/30/60 s a SQLx-writer benchmark. Writer nebyl aktivován v běžícím DEV.

### G-04 — izolovaný writer, retence a sampling replay
- Na testovacím kódu `36afdda5e752cc320496f44672a94e9434eea9eb` proběhl Rust→RAM→bounded→SQLx→PostgreSQL replay 1×/2×/4×; limit 2048 způsobil 0/452/2952 řízených ztrát, živé SSE zůstalo dostupné.
- Izolovaná retence odstranila 2501 starých řádků dávkami 1000/1000/501 a zachovala nový řádek i jediný PK index. Srovnání stejné rekonstruované trasy 15/30/60/adaptive doložilo prostorovou chybu a počet řádků.
- Audit sdílené DEV DB zjistil `archive_mode=off` a vypnutý `archive_command`; skutečná backup politika a provozní RPO/RTO nejsou z runtime účtu známé. G-04/J09 zůstává `IN_PROGRESS`.

### G-04 — diagnostika a časově rozložený replay
- Implementační SHA `0daeb2f755bff81ce518a0415942a7e6b82579b0` přidává interní 15s diagnostiku pending/inflight/commit/drop/timeout/connection bez tajných údajů a Rust regresní replay 64 letadel.
- Časově rozložený replay 16 snapshotů po 10 s zachoval 3 vzorky na každé ze 64 identit. Bounded benchmark ale vykázal ztráty 18,1 % při 2× a 59,0 % při 4×; první DEV canary proto zůstává NO-GO.

### G-04 — stávající PostgreSQL pravidla beze změn
- Vlastník odmítl konfigurovat nové `hostnossl` ani měnit `pg_hba.conf` či síťová pravidla. Ověření používá pouze existující autorizaci; při odmítnutí zůstane G-04.02 BLOKOVÁNO bez automatických serverových změn. Aktualizováno ADR 0009, runbook, kroky a technické kontrakty.


### G-04.02 — výslovně zvolená důvěryhodná LAN bez TLS (ADR 0009)
- SQLx používá `PgSslMode::Disable`, přihlašovací jméno a heslo z chráněných proměnných, bez CA souboru. Rozhodnutí je omezené na izolovanou V2 DEV LAN a existující serverová pravidla. CI i read-only DEV ověření kontrolují `pg_stat_ssl.ssl=false`. Skutečný databázový server a V1 se nemění.


### G-04 — přihlášení k PostgreSQL jménem a heslem
- SQLx DEV připojení přijímá samostatné proměnné HOST/PORT/USER/PASSWORD, bez hesla ve spojovacím URL, a používá samostatnou databázi `airradar_v2_dev`. Přidán negativní test chybějících údajů a hesel se speciálními znaky. Skutečná DB se nemění.


### G-04 — izolovaný SQLx historický modul, rozpracované PR #21
- Nový Rust RAM-first sampler a volitelný `--g04-dev-history`; omezený SQLx pool, 2048 coalescing, dávka 128, potvrzené SQL watermarks, 30s buckety, čtení historie v časovém okně bez dalších tabulek.
- Oddělená PostgreSQL CI kontrola a nový retenční/backup runbook. Na stávajícím sdíleném DB LXC nebylo nic spuštěno; G-04.01–.03/.10/.12 zůstávají blokované provozními důkazy a fyzickým DEV měřením.


### 23. 10. 2026 — oprava kolize automatických statistik s DEV releasem
- Při [hlavním deploymentu #39909065853](https://github.com/Boym323/AirRadarV2/actions/runs/39909065853) se po zelené CI a publish změnil `main` jen bot commitem statistik `6162c00c`; runner odmítl starší HEAD. Opraveno sdíleným concurrency lockem a přísnou výjimkou jen pro ověřené lineární bot commity, které mění výhradně README/metriky. Bez obecného oslabení immutable SHA nebo zásahu do V1/DB; nová CI/deploy přejímka musí ověřit nový commit.


### 23. 10. 2026 — G-03 uzavřena: HOTOVO 10/10, DEV ACCEPTED
- **Referenční přijatý runtime:** `29ee2ee75feab9279123b77d50895d648c5419d7`. [Main deployment #39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759) **6/6 PASS**, včetně self-hosted runneru a post-deploy ověření HTTP/SSE. Předchozí opravy a nasazení [#39905085484](https://github.com/Boym323/AirRadarV2/actions/runs/39905085484) a [#39905632012](https://github.com/Boym323/AirRadarV2/actions/runs/39905632012) také SUCCESS.
- Ve skutečném DEV provozu doložen `stage=G-03`, `status=live`, read-only BEAST, snapshot/delta, prioritní ~15s heartbeat a reconnect. G-02 testovací služby inactive, V1/DB nezměněny.
- **Omezení:** úmyslný rollback aktuálního živého G-03 releasu ani fyzický OS TCP socket buffer/RSS limit nebyly nově měřeny. `J06/J07/J08` zůstávají `TESTED`; produkční G-11/G-13 bez schválení. Dokumenty aktualizovány bez přepisování historických CI incidentů.

---

**Starší položky níže jsou historický průběh vývoje.** Záznamy s tehdejším `NO-GO`, `PR draft` nebo `NEOVĚŘENO` nepopisují dnešní stav G-03.


### G-03 — oprava SSE heartbeat a odolnosti DEV deploymentu
- Opraven heartbeat SSE: samostatný 15s ticker se po každé deltě neresetuje, takže aktivní stream nezůstane bez keepalive při živém 1Hz příjmu.
- Nginx DEV dostává no-buffer hlavičky a systemd retry po DHCP boot race; rollback nepoužívá destruktivní `restart` transakci během shutdownu LXC.
- Přidány diagnostické návratové kódy pro obsazený deployment lock a kontrola Nginx před přepnutím releasu. Změny se týkají G-03.06/.08 a G-13 bridge; V1, DB a receiver zůstávají nedotčené.
- Následná oprava dává heartbeat explicitní prioritu před stále připravenými živými deltami; tím je odstraněna i možnost starvation při trvalém 1Hz provozu.

### G-03 — DEV ACCEPTED po automatickém nasazení
- Commit `749ffb942407f7c7cd88f3d983a72841664f52ee`, GitHub Actions run [#39905632012](https://github.com/Boym323/AirRadarV2/actions/runs/39905632012): validační CI, immutable release, publish a self-hosted DEV deployment **SUCCESS**.

### G-03 — obnoven automatický main → V2 DEV deployment
- Release workflow po PASS CI opět publikuje SHA256 prerelease a spouští již registrovaný runner. Stage-aware root instalátor automaticky přepne `--g03-live-api` i immutable release a provede kompletní systemd+binary rollback při chybě. Source čte již schválený read-only G-02 BEAST (privátní LAN TCP 30005). Jednorázový root helper připravuje prostředí bez restartu; samotné automatické vydávání už nevyžaduje ruční schválení každého commitu.


### G-04.04 — zahájení izolované PostgreSQL historie (CI k ověření)
- Přidána skutečná SQLx migrace s jednou J09 tabulkou, 30s UTC bucketem a jediným PK. Reverzibilní SQL a separátní CI workflow pro PostgreSQL 18 testují integritu, nezměnitelnost historie, role a rollback pouze ve vytvořené efemérní DB.
- Žádné spojení se skutečným DB LXC, žádné nasazení; G-04.01–.03, reálný TLS, writer, retence a měření zůstávají otevřené. Podrobnosti v [průběžném reportu](docs/reports/G-04-VYSLEDEK.md).


### G-04.04 — zahájení izolované PostgreSQL historie (DB CI #39856030008 PASS)
- Přidána skutečná SQLx migrace s jednou J09 tabulkou, 30s UTC bucketem a jediným PK. Reverzibilní SQL a separátní CI workflow pro PostgreSQL 18 testují integritu, nezměnitelnost historie, role a rollback pouze ve vytvořené efemérní DB.
- Žádné spojení se skutečným DB LXC, žádné nasazení; G-04.01–.03, reálný TLS, writer, retence a měření zůstávají otevřené. Podrobnosti v [průběžném reportu](docs/reports/G-04-VYSLEDEK.md).

- Test topologie workflow doplněn pro čtvrté, samostatné DB-only CI; release/deploy zůstává fail-closed.

### Audit P0 před G-03 merge: fail-closed hlavní release pipeline
- Doloženo riziko: každý push do `main` by jinak zveřejnil G-03 prerelease a pokusil se spustit automatický DEV deployment bez ověřeného systemd/rollbacku. Workflow nyní bezpečně rozlišuje obsah G-03 balíčku, při G-03 publikuje **jen Github Actions artefakt** a zablokuje prerelease i závislý self-hosted deployment. Přidána regresní kontrola release topologie; G-03 kód lze sloučit bez živého cutoveru.

### G-03 — 10/10 implementačních kroků TESTED v CI; DEV cutover zatím NO-GO
- [CI #39848794079](https://github.com/Boym323/AirRadarV2/actions/runs/39848794079), SHA `c43ab9d543a740020ac9133941f1a4829b2c7dea`: 3/3 PASS; 5k/100 SSE klientů bez ztracené aktualizace (p95 1 576 ms, p99 1 630 ms, Rust RSS max 48 764 KiB); 90 nečtoucích klientů s 5/5 aktualizací zdravého a RSS +15 112 KiB; Chromium EventSource Vite + Nginx snapshot/delta/reconnect PASS.
- Oprava ztráty 5000 JSON identit v 2048-elementovém mpsc používá jeden bounded `JsonBatch` se single-pending atomickou branou, žádná duplikace RAM stavu a žádné DB přístupy; doplněny regresní testy.
- PR #11 zůstává draft, **NO-GO** pro merge kvůli dosud neověřenému DEV přepnutí systemd a rollbacku; V1 a její produkce nedotčeny.

### G-03.07 — 90 skutečně nečtoucích SSE klientů
- Syntetický HTTP TCP zátěžový test drží 90 spojení bez čtení body a kontroluje 5 úplných aktualizací zdravého klienta nad 5k cíli při přírůstku Rust RSS ≤25 MiB. Test odlišuje Rust RSS od OS socket bufferů; čeká na CI a cílovou DEV přejímku.

### G-03.07/.08 — nečtoucí prohlížeče a rychlejší reálný browser CI
- Do připraveného DEV Nginx SSE vzoru přidán downstream `send_timeout 30s`, zatímco heartbeat a 75s upstream proxy timeout zůstávají. Chromium browser smoke používá Playwright s nainstalovaným Google Chrome na izolovaném GitHub runneru, bez stahování vlastní browser binárky; nezměnilo to uživatelský PC ani DEV LXC.

### G-03.07/.10 — slow reader lease, atomický JSON batch a 5000 tracků (CI #39845031595 PASS)
- SSE lease watchdog s monotónním 30s deadlinem bez HTTP Body poll; 5s kontrolní interval a bezpečné předání aplikačního permitu. Žádná změna kontraktu ani produkce.
- Izolovaný benchmark rozšířen na 1/10/100 odběratelů s 5000 úspornými identitami; unit testy přetížení přes 1 MiB zachovány. Provozní cílový TCP/RSS test zůstává otevřený.


### G-03.08 — Chromium EventSource, dva stejno-origin proxy průchody
- Izolovaný browser test Chromium přes Vite 4100 → Rust 4102 a Nginx 8082 → Rust 4102. Ověřuje plný snapshot při připojení, delta upsert během otevřeného streamu, monotónní epoch/seq a full snapshot po reconnectu. Playwright se instaluje pouze do dočasného CI venv, nikoli do uživatelova počítače ani do DEV LXC.

### G-03.10 — skutečný test 5 000 kompaktních letadel (CI)
- Syntetický readsb replay rozšířen o 5 000 kompaktích letadel pro 1 a 100 souběžných klientů, report CPU/RSS/p95/p99/bytes/s se explicitním čítačem nedoručených aktualizací. Přetížení nad 1 MiB ověřuje separátní Rust regresní test; plný produkční stav pěti tisíc letadel nelze garantovat při deklarovaném 1MiB limitu.

### P1/P2 audit G-03 — CI ověřeno, zatím pouze draft G-03
- Sdílené SSE payloady a 16KiB odesílací chunky, explicitní 503/degraded/unavailable při překročení 1MiB a návrat novým full snapshotem.
- Stage-aware G-03 kontrola bez falešného bootstrap PASS: direct SSE + Nginx snapshot/heartbeat, fail-closed preflight root instalátoru. První systemd/config rollback stále NO-GO, žádná produkční změna.
- Stav G-00/G-01/G-02 a rozpracované G-03 opraven v úvodní dokumentaci.


### G-03 — průběžná dílčí přejímka 7/10 TESTED (22. 10. 2026)
- [CI #39835102766](https://github.com/Boym323/AirRadarV2/actions/runs/39835102766) pro `db4078fa`: Rust, web, dokumentace 3/3 PASS; SSE 1/10/100 klientů, validní delta/snapshot, 15s heartbeat přes skutečný testový Nginx, 101. klient 503, JSON invalid/recovery.
- Pro 100 souběžných SSE klientů a 250 syntetických letadel p95 **1 786 ms** od JSON rename do přijetí (včetně dvou 1Hz cyklů), RSS peak **11 448 KiB**, CPU **72,39 % jednoho jádra** na GitHub runneru. Není to výkon cílového DEV LXC.
- G-03.06/.09 nově TESTED, J07/J08 v aktivní matici TESTED. G-03.07/.08/.10 stále otevřené; žádný G-03 release ani zásah do V1.


### G-03.08 — skutečný průchod SSE přes Nginx v izolované CI (před přejímkou)
- CI připravuje dočasný Nginx podle odděleného G-03 vzoru a testuje HTTP `/api/v1/health`, průběžné doručení SSE snapshotu a heartbeat přes 127.0.0.1:8082; neinstaluje ani nepřepíná skutečný DEV web. Browser EventSource a LXC zůstávají NEOVĚŘENO.

### G-03.09 — kontrola poškozeného readsb a přesnější SSE měření
- Benchmark odlišuje čisté SSE update bytes/s od prvního snapshotu, měří peak RSS během aktualizace a ověřuje, že atomicky publikovaný chybný JSON neporuší kanonický Rust RAM stav ani další doručování. Výsledky aktualizuje CI finálního SHA.

### G-03.07/.09/.10 — izolovaný 1/10/100 SSE zátěžový scénář (čeká CI)
- Přidán standard-library Python harness spouštějící pouze opt-in Rust API na loopback s atomicky obnovovaným syntetickým readsb JSON. Ověřuje 1/10/100 SSE klientů, delta/snapshot, limit 101. klienta a návrat kapacity; reportuje p95/p99, CPU, RSS a bytes/s. Čísla nejsou výkon cílového DEV LXC, proxy ani prohlížeče.

### G-03.01/.02 — 5/10 testově uzavřeno na CI #39814106622
- Public API J06 označeno `TESTED` po formální OpenAPI 3.1 validaci, HTTP 200/400/503, privacy a SSE reconnect testech (3/3 CI PASS); neznamená to provozní G-03 release.

### G-03.01/.02/.06/.07 — formální API a regresní testy (čeká nové CI)
- Oddělený OpenAPI kontrakt opt-in API, odmítnutí neschválených query parametrů HTTP 400, JSON `CONTRACT_INVALID` a při 100 SSE klientů JSON 503 `UNAVAILABLE`.
- Přidány Rust testy reconnect se starým Last-Event-ID, uvolnění SSE permitů a záměrně zpožděného čtení s resynchronizací. Výkonové RSS/p95/p99 zůstává NEOVĚŘENO.

### G-03.08 — neaktivované SSE proxy a izolovaný HTTP smoke
- Připraven volitelný Vite cíl 4102 a oddělený Nginx SSE vzor; CI doplněna o síťový kontrakt a HTTP/SSE smoke bez skutečného přijímače. Provozní brána zatím NEOVĚŘENO.

### G-03 — izolované HTTP/SSE rozhraní nad G-02 (22. 10. 2026)
- Přeneseny skutečné změny [PR #8](https://github.com/Boym323/AirRadarV2/pull/8) na aktuální `main` bez historických G-02 změn; 3/10 G-03 kroků doloženy [CI #39804030160](https://github.com/Boym323/AirRadarV2/actions/runs/39804030160).
- Rust opt-in API a SSE sdílí jeden RAM stav, veřejná DTO a 1 Hz snapshot/delta; provozní proxy/zátěž dosud NEOVĚŘENO.
- Bez změny V1, PostgreSQL a vývojového LXC. Merge a auto-DEV release závisí na zeleném CI a schválené bráně.


### Automatický DEV deploy z ověřeného main — zatím pouze implementace
- Nový root-owned pull agent přijímá jen GitHub botem vydaný release k přesnému současnému main SHA a validuje balíček, SHA256, adresy assetů, tar allowlist a VERSION.
- Jednorázový instalátor vytvoří izolovaný systemd timer každých 5 minut a bezpečný read-only DEV status. Healthcheck a rollback se provádí bez zásahu do V1, přijímače nebo DB.
- **Není zatím potvrzena reálná instalace v DEV LXC**, provozní G-13 produkční etapa zůstává nerozpracovaná.




### Úklid GitHub Actions: 3 workflow místo 5 (22. 10. 2026)
- Jediná kanonická CI sada (`ci.yml`) pro PR a jako reusable workflow před automatickým `main` release; odstraněny duplicitní `main` testy.
- Jediný DEV release/deploy přes repo-scoped Runner zachovává G-02 replay/benchmark, Rust/Vite, OpenAPI, SHA256, healthcheck i rollback. Historický ruční build a jednorázové G-02 LIVE workflow odstraněny z aktivních YAML; auditní testy a výsledky zachovány.
- Denní statistiky zůstávají, ale commit obsahující pouze README/metrics již nespouští release ani restart DEV LXC. Aktualizován README CI badge na skutečný stav DEV main pipeline.



- G-02.11: LIVE přejímka se již nezablokuje po vypnutí offline G-02 timeru. Schválení je svázáno s nasazeným DEV SHA, aktuálním main, botem vydaným immutable release a jednorázovým request ID. Historický offline status zůstává zachován a LIVE gate čte výlučně `liveSoak.sha`. Doplněny negativní regresní testy a postup jednorázové aktualizace root-owned LIVE skriptů; nová hodinová přejímka ještě nebyla spuštěna.

- G-13: běžný DEV deployment už nečeká na periodický veřejný `g02-status.json`
  ani `PASS_SYNTHETIC_LXC`. Historický verify job byl odstraněn a deploy nyní
  přímo na self-hosted runneru ověřuje přesný `VERSION`, release symlink, Rust
  health API, Nginx, web a krátkou stabilitu engine. G-02 replay/benchmarky v CI
  zůstávají zachované a oba staré polling timery zůstávají vypnuté.
- G-13: opraven autonomní DEV deployment pro již registrovaný runner. Workflow
  používá SHA/run izolovaný staging bez mazání cizích souborů, kontrolu SHA-256
  z reálného adresáře a robustní kontrolu `VERSION`; root bridge ověřuje
  vlastníka stagingu. Instalátor již nemaže registraci ani nevytváří duplicitní
  systemd runner službu.
- G-02.11: validátor veřejného DEV reportu už nepředpokládá natvrdo jeden
  viditelný CPU; vyžaduje kladnou hodnotu a zachovává skutečné měření hosta.
- G-13: zdokumentována a regresně testována korektní validace kořenových záznamů GNU tar (`.`/`./`) pro DEV auto-deploy; bezpečnostní kontroly zůstávají povinné.
- G-13: doplněn repo-scoped self-hosted runner `airradarv2-dev`, root-owned deployment bridge, krátkodobá tokenová registrace, sdílený G-02 lock a deploy job pouze pro úspěšný `main` prerelease. První registrace a skutečné nasazení čekají na GitHub runner oprávnění.
- G-13: řízený rollback test root bridge PASS; nefunkční kandidát byl odmítnut healthcheckem, aktivní symlink i Rust API se vrátily na předchozí release. Vadný release zůstal zachován pro audit.

### Automatický DEV deploy z ověřeného main — zatím pouze implementace
- Nový root-owned pull agent přijímá jen GitHub botem vydaný release k přesnému současnému main SHA a validuje balíček, SHA256, adresy assetů, tar allowlist a VERSION.
- Jednorázový instalátor vytvoří izolovaný systemd timer každých 5 minut a bezpečný read-only DEV status. Healthcheck a rollback se provádí bez zásahu do V1, přijímače nebo DB.
- **Není zatím potvrzena reálná instalace v DEV LXC**, provozní G-13 produkční etapa zůstává nerozpracovaná.



### G-02.11 — úspěšná hodinová LIVE přejímka, etapa 11/11 HOTOVO (22. 10. 2026)
- [GitHub run #39795592425](https://github.com/Boym323/AirRadarV2/actions/runs/39795592425) — skutečný 60min read-only BEAST ingest na DEV LXC: 109 154 pozorování, 77 max tracků, `channelDrops=0`, `capacityDrops=0`, průměr CPU 0,36 %, peak RSS 2 492 KiB; plný G-02 SHA `d0cefc4a5c3332eadccf8ab21c96bae51987abe9`, request ID `39795592425`. Oba GitHub joby `SUCCESS` v 17:29 SELČ.
- Kompletní G-02 má **11/11 testových kroků PASS**; šest KEEP ID zůstává `TESTED`. G-03/živé HTTP/SSE stále není produktově nasazeno. Samostatně zbývá propis opraveného `TimeoutStartSec=3900` do instalované oneshot jednotky po skončení testu.



### G-02.11 — oprava systemd time limitu a důkaz živého příjmu (RUNNING)
- V DEV LXC správce potvrdil funkční GitHub požadavek a skutečný BEAST TCP stream: 591 `preferred=Beast` vzorků, 16 532 aktualizací, nula queue/capacity drops, maximum 66 tracků. 60min test ještě běží; G-02 stále 10/11.
- Oprava instalátoru: `Type=oneshot` vyžaduje `TimeoutStartSec=3900`, protože `RuntimeMaxSec=3900` je ignorováno. Přidán regresní test; instalovanou jednotku aktualizovat bezpečně až po dokončení aktuálního testu.


### G-02.11 — ovládání 60min LIVE přejímky z GitHubu (připraveno, neaktivováno)
- Odchozí pull-only agent čte výhradně botem vydaný request release s přesným SHA/run ID a věkem do 45 minut; spouští schválený read-only BEAST test se systémovým limitem 3900s.
- Jednorázový instalační skript pro izolovaný DEV LXC vytváří samostatný systemd timer a omezenou službu. GitHub workflow pro ruční spuštění je dodán v odděleném PR určeném do `main`.
- Výsledek je v `liveSoak` připojen k workflow run ID; defaultní přijímač a V1 se nemění, na LXC ještě nic nového neběží. G-02 10/11.


### G-02.11 — připraven hodinový reálný BEAST read-only soak (NEPROVEDENO)
- Samostatný rootless diagnostický proces připojí existující G-02 engine pouze k BEAST TCP OUTPUT 30005. Bez změny V1, DB či běžícího webu.
- Po dobu 3600 sekund měří tracky, přijaté zprávy, dropy, RSS a CPU. Veřejný status dostane oddělenou sekci liveSoak bez adres nebo dat letadel.
- Provedení a posouzení výsledků stále čeká. **G-02 10/11.**


### G-02: oprava výběru nejnovějšího GitHub prerelease
- Github API nevracelo seznam vydání v chronologickém pořadí; dosavadní pull agent vybíral první starší SHA místo nového, takže automatický DEV LXC 180s test zůstával nespouštěný.
- Nově se řadí validní připnuté releasy podle `published_at`. Regresní testy pokrývají nesetříděnou odpověď. Vyžaduje jednorázovou aktualizaci nainstalovaného agenta v DEV LXC (bez restartu běžícího radaru).


### G-02.11 — automatický loopback reconnect soak (čeká na CI a LXC)
- Do existujícího `--g02-benchmark` přidán izolovaný skutečný TCP BEAST + JSON poll scénář s fragmentací, opakovanými EOF, obnovením spojení i chybným/opravovaným JSON souborem, kontrolami deduplikace, fronty, RSS a TTL. Release sestavení měří 180s, rychlý CI debug test nejméně 5s.
- Stávající timer na DEV LXC při vydání nové verze spustí test bez přístupu k produkční V1; výsledky se objeví v `benchmarkLog` veřejného `/g02-status.json`. Skutečná čísla se doplní až po ověření, live-soak fyzického přijímače zůstává nepřevzatý.


### Automatická G-02.11 acceptance — offline test na DEV LXC PASS, live-soak čeká
- GitHub Actions při zelených testech vydá immutable SHA-pripnutý DEV prerelease bez nasazení.
- Samostatný rootless systemd timer v DEV LXC stahuje ověřené balíčky přes HTTPS a spouští offline replay + 5000-track benchmark. Uživatel doložil HTTP 200 i přes veřejnou reverse proxy `/g02-status.json`.
- Doložený build `daa15703` v DEV LXC: PASS replay i benchmark, 5 000 tracků p95 2,40–2,55 ms, RSS do 5 256 KiB. Live-soak/reconnect a provozní G-02.11 přejímka stále NEOVĚŘENO. Žádný self-hosted runner ani automatické přepnutí webu/V1.


### G-02 — rozšíření testového pokrytí na 10/11 kroků (22. 10. 2026)
- [CI #39769323592](https://github.com/Boym323/AirRadarV2/actions/runs/39769323592), commit `81a27295d78753a6a3d987e7d68d6b8394c12ef5`: Rustfmt, Clippy, 26 Rust testů a 3/3 joby PASS.
- Doplněna validovaná letová telemetrie, TC19 rychlost/heading/vertikální rychlost, DF17 výška, lokální CPR s trusted reference, source health a test JSON↔BEAST fallbacku.
- Offline replay nyní zahrnuje JSON i BEAST, negativní CRC a syntetickou zátěž 5 000 identit při 1×/2×/4× bez připojení na V1.
- Šest KEEP funkcí M01 označeno `TESTED`, nikoli `ACCEPTED`. **G-02 není HOTOVO**, dokud nebude změřena G-02.11 na izolovaném 1vCPU DEV LXC. Nebylo nasazeno do LXC.


### G-02 — první izolovaná implementace readsb/BEAST (22. 10. 2026)
- Opraven úklid per-aircraft `position_quality` pomocné RAM mapy při TTL, regresně ověřeno [CI 39763140573](https://github.com/Boym323/AirRadarV2/actions/runs/39763140573) 3/3 PASS na `be564960`.
- [Průběžný G-02 report](docs/reports/G-02-VYSLEDEK.md) a [CI 39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657): 22 Rust testů PASS, syntetický TCP reconnect, JSON temp-file, DF17/CPR a 1×/2×/4× CPU/RSS/p95 replay. **5/11 úkolů bezpečně potvrzeno, G-02 dosud HOTOVO není**.
- Implementovány Rust moduly pro bounded JSON snapshoty, inkrementální BEAST TCP framing, konzervativní DF17 CRC/CPR párování, jediný RAM state, TTL/dedup a oddělený bounded Tokio supervisor.
- Přidán `--g02-replay` pro offline syntetický test a explicitní `--g02-live` pouze na ruční vyžádání s konfigurací zdroje; běžný systemd HTTP stále poctivě G-00 bootstrap a žádný skutečný receiver se automaticky nepřipojuje.
- **Etapa zůstává rozpracována** do ověření CI, testů, výkonnostních měření a přejímacích funkcí. Nový build se do LXC nenasadil.


### První úspěšné DEV LXC nasazení — 22. 10. 2026
- Správce potvrdil i otevření vývojového webu v prohlížeči z LAN. Zbývají kontroly Proxmox firewallu a dlouhodobého běhu; G-02 není zatím implementována.
- Dodatečný konzolový HTTP test potvrdil českou HTML stránku přes Nginx a `ss -lntp` prokázal lokální Rust `127.0.0.1:4101` a Nginx jen na privátní IPv4:8080, bez IPv6 listeneru na těchto portech. Přístup z jiného LAN zařízení a Proxmox firewall/IPv6 pravidla zůstávají neověřené.
- Správce doložil nasazený balíček `227a8619b590d2c536761ffefc75cd0a5633936c`, SHA-256 OK, Rust i Nginx aktivní, bootstrap health a korektní `NOT_IMPLEMENTED` aircraft API.
- [Přejímací report](docs/reports/DEV-LXC-PRVNI-NASAZENI-2026-10-22.md) rozlišuje ověřený lokální provoz a dosud neověřený prohlížeč, IPv6 firewall a soak. G-02/G-03/G-04 a V1 beze změny.


### Připravený DEV LXC balíček — 22. 10. 2026
- Potvrzen existující izolovaný Debian 13.7 LXC (1 vCPU, 8 GiB RAM, 20 GiB disk) bez potřeby kompilovat Rust/Node uvnitř.
- Přidáno ruční GitHub Actions balíčkování G-00/G-01, Debian bootstrap a bezpečná instalace přes SHA-256, oddělený systemd/Nginx pouze pro LAN IPv4:8080 a rollback.
- **Neprovedeno:** skutečné vzdálené spuštění v LXC, produkční V1, PostgreSQL, readsb, proxy do internetu, přístupové klíče, automatický deploy.


### G-01 — dokončené kontrakty a syntetické testy (22. 10. 2026)
- **G-01 HOTOVO — 10/10 kroků**. Validátor v izolované GitHub CI kontrole ověřuje OpenAPI 3.1 bez duplicitních YAML klíčů, JSON Schema Draft 2020-12 a pozitivní/negativní fixtures. [CI 39748325104](https://github.com/Boym323/AirRadarV2/actions/runs/39748325104) 3/3 PASS na commitu `14f591c4`.
- Pouze smlouvy a testy; 45 produktových funkcí nadále `NOT_STARTED`, runtime příjem a SSE až G-02/G-03, databáze G-04.
- Ověřený [CI 39747410591](https://github.com/Boym323/AirRadarV2/actions/runs/39747410591) se všemi třemi joby PASS a aktualizovaná česká evidence G-01: **10/10** kroků; nezávislý validátor OpenAPI 3.1 dodán a doložen CI. Kontrakty a fixture jsou testované, nikoli živě připojené.
- Doplněny JSON Schema veřejného AircraftV1/SSE snapshot/delta, cílové OpenAPI komponenty při zachování pravdivého G-00 stubu a syntetické BEAST/stream fixture.
- Přidány sdílené syntetické JSON golden případy, Rust/TS validace identity, provenance, času a bezpečných SQL vzorků; modelové testy sekvence SSE.
- G-01 je dokončena jako kontraktová etapa; skutečný streaming BEAST parser, browserové E2E a runtime SSE se teprve implementují v navazujících etapách. 45 produktových funkcí se tím nestalo dokončenými.


### Architektonický audit 22. 10. 2026
- Přidán [audit připravenosti V2](docs/audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md) na stavu `f97ba6e0`; 3 kontraktové blokace G-01 a další etapová rizika, zejména časová semantika BEAST/readsb, identita, SSE race, SQL ack, logbook/statistiky a bezpečná správa notifikačních odběrů.
- Opraven zastaralý údaj o Next.js v aktuální G-00 tabulce; historické ADR výslovně označeno jako supersedované v části sampling.
- Žádné implementace 45 KEEP funkcí, SQL migrace, produktové rozšíření ani produkční nasazení.


### Pravidla původu lokálního příjmu — 22. 10. 2026
- [ADR 0008](docs/adr/0008-identita-puvodu-lokalniho-prijmu.md): oddělit stabilní neveřejný `receiverId` od JSON/BEAST transportu; stejný readsb nezapočítávat jako dvě RF stanice a nespojovat receiver s identitou letadla.
- Scénáře dalších přijímačů a případných externích služeb jsou zapsány **jen v odděleném backlogu**, mimo 45 aktivních funkcí. Původní přepínač LOCAL/EXTENDED se nevrací.
- Jde výhradně o úpravu specifikací a o budoucí akceptační test, nikoli o implementaci, připojení dalšího zdroje nebo zásah do infrastruktury.


### Návrh RAM-first a ochrany SSD — 22. 10. 2026
- Audit návrhu G-04: SQL unikátní klíč chránil jen stejný 15s časový bucket, ale nezabraňoval zbytečným zápisům nezměněné polohy mezi buckety.
- Návrh revidován na **primární deduplikaci v Rust RAM**, adaptivní sampling (běžně nejdříve 60s; významné manévry nejdříve 30s), coalescing max 2048 pending/30s, dávkování max 128 vzorků/5s a pouze nezbytný výchozí SQL PK.
- Druhý B-tree, BRIN a denní partitioning podmíněny EXPLAIN a měřením objemu/retence; denní agregace nanejvýš 1 UPSERT/5 min, živý SSE bez SQL.
- Přidán deterministický model `tools/check-eco-sampling.mjs` a [ADR 0007](docs/adr/0007-ram-first-deduplikace-a-usporny-zapis.md); **modelová úspora řádků není měření fyzických SSD zápisů**.
- Žádné DB migrace, produkční zásahy, benchmark PostgreSQL ani změny 45 aktivních funkcí.


### Průzkum 22. 10. 2026
- Dokončeny české technické návrhy pro PostgreSQL fyzický model, indexy a retenci 30/365 dní, off-LXC zálohy, readsb/BEAST a deduplikaci, 1Hz SSE s bounded frontami, MapLibre profilování, Playwright browser E2E a TLS/SCRAM spojení k existujícímu DB LXC.
- Přidán [rozcestník](docs/plan/TECHNICKE-KONTRAKTY-A-LIMITY.md) a ADR 0006. Neprovedeno žádné SQL, nasazení, reálné Playwright E2E ani výkonový benchmark. Výběr 45 funkcí zůstává nezměněn.


### Přidáno
- Detailní český krokový implementační plán `docs/plan/KROKOVY-IMPLEMENTACNI-PLAN.md`: **140 konkrétních kroků** v G-01 až G-13 s návaznostmi, definicí výstupu a odpovídajícími testy.
- Všech 13 etapových MD zadání rozšířeno o zaškrtávací kroky `G-XX.YY`, bezpečnostní hranice a pravidla evidence PASS/FAIL; žádná z 45 funkcí není touto dokumentací označena jako hotová.
- Automatická kontrola úplnosti a jednoznačnosti kroku, evidence vlastnictví 45 ID a rozcestník pro nová vlákna.

### Změněno
- Webový základ G-00 převeden na **Vite + React + TypeScript** se statickým `dist/`, českým UI a HTTP smoke.
- Odstraněn Next.js runtime a závislosti; CI používá uzamčené `npm ci`.
- PostgreSQL definitivně zvolena na existujícím separátním DB LXC; nové V2 připojení a migrace až G-04.
- Žádný produkční deploy ani změna LXC/DB nebyly provedeny.

### Přidáno
- Důkladná česká rešerše technologií a architektury: `TECHNOLOGICKY-PRUZKUM.md`, `ARCHITEKTURA-V2.md`, `BENCHMARKY-TECHNOLOGII.md` a `TECHNOLOGICKA-ROZHODNUTI.md`.
- Rozhodovací brány a reprodukovatelné benchmarky Rust/Go/Node, Next.js/Vite, SQLite/PostgreSQL, BEAST/JSON, SSE a MapLibre.
- Architektonické doporučení Vite/React místo produkčního Next.js serveru a podmíněný kandidát SQLite WAL; skutečný výběr DB a výkonové výsledky zůstávají neověřené.

### Změněno
- Vývojová platforma modernizována na Node.js 26.12.0/npm 11, Next.js 16.4.2, React 19.3.1, TypeScript 6 (poslední podporovaná řada), ESLint 10 a Vitest 5. Regenerace lockfilu a CI neprovádí nasazení.
- Produktový plán je nyní **místní radar s 45 aktivními funkcemi**. Rozšíření jsou přesunuta do jediného odděleného souboru pro budoucí posouzení; aktivní zdrojová matice neobsahuje nevybrané požadavky.
- Aktualizováno rozdělení devíti logických modulů, navazující české implementační úkoly a souvislé číslování etap G-00 až G-13.
- Zesílena kontrola synchronizace matice, české dokumentace a vývojových statistik bez pevně zakódovaného historického rozsahu.
- Základní přijímací API ani UI implementace se tímto rozhodnutím nestaly živým radarem.

### Ověření
- Modernizace runtime ověřena [CI 39673418390](https://github.com/Boym323/AirRadarV2/actions/runs/39673418390), commit `d59f98d4`: Node 26, web lint/types/test/build/smoke, Rust testy a česká dokumentace prošly.
- Stav tohoto dokumentačního čištění musí potvrdit CI na jeho finálním SHA. Žádné produkční nasazení se neprovádí.

## 2026-10-21 — Založení nezávislého AirRadar V2

### Přidáno
- Samostatný GitHub repozitář a počáteční Rust/Tokio/Axum HTTP scaffold s neimplementovaným seznamem letadel.
- Next.js vývojová stránka s češtinou jako výchozím jazykem.
- Připnuté závislosti v Cargo/npm lockfile, dokumentace, OpenAPI, počáteční matice požadavků a CI bez deploye.
- Automatizované statistiky vývoje zdrojového kódu s historickými snapshoty, které se zpětně nepřepisují.

### Ověřeno
- [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202): Rust + web unit/build a HTTP smoke při počátečním ověření.
- [GitHub Actions 39662365349](https://github.com/Boym323/AirRadarV2/actions/runs/39662365349): Rust, web a česká dokumentace.
- Živá data, databáze, LXC a produkční deployment tímto nebyly zřízeny.

## Pravidlo zápisu

Každá skutečná implementační změna vyžaduje položku v části Nepublikováno, český etapový report a zdrojové důkazy. Historická označení verzí vzniknou teprve po skutečném vydání. Tento soubor není náhradou za automatizované testy.
# 2026-10-23

- G-04/PR #21: opraven P1 canary evaluator (povinné spojení, časová řada fronty,
  jmenovatel ztrát podle Rust `eligible`, skutečný 30min interval, DB readback a
  G-03 SSE); přidány negativní regresní testy.
- G-04 canary unit je připnutý k samostatnému SHA-bound artefaktu s `MemoryMax=2G`,
  `RuntimeMaxSec=1875` (1800 s důkaz + uzavření bucketu/drain) a bez automatického
  restartu. G-03/4101 zůstává nedotčené.
- Přidán read-only SQL audit DB/WAL a návrh externích záloh s RPO 24 h/RTO 2 h.
