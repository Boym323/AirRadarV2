# CHANGELOG — AirRadar V2

Sledujeme **skutečné změny kódu, dokumentace a provozních kontraktů**; všechny zápisy jsou česky. Jednotlivé implementační etapy mají vlastní reporty a přesné odkazy na CI.

## [Nepublikováno]

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
