# CHANGELOG — AirRadar V2

Sledujeme **skutečné změny kódu, dokumentace a provozních kontraktů**; všechny zápisy jsou česky. Jednotlivé implementační etapy mají vlastní reporty a přesné odkazy na CI.

## [Nepublikováno]

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
