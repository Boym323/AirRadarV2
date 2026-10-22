# Plán etap a aktuální stav AirRadar V2

**Aktualizováno 22. 10. 2026.** Tento plán řeší **pouze 45 aktuálně vybraných funkcí**, bez vedlejších a nepoužívaných modulů. Výchozí jazyk: čeština. Zápis „NEZAHÁJENO“ není nedostatečný test — práce ještě nezačala.

| Etapa | Výstup a zadání | Stav |
|---|---|---|
| G-00 | Samostatný základ Rust + Vite/React | **HOTOVO — pouze scaffold** |
| G-01 | [Kontrakty a deterministické fixture](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) | **HOTOVO — 10/10, CI PASS** |
| G-02 | [Lokální ADS-B ingest a RAM](../ukoly/G-02-LOKALNI-INGEST.md) | **HOTOVO — 11/11 TESTED, reálný 60min BEAST PASS; G-02 HTTP/SSE zatím nenasazeno** |
| G-03 | [Verzované HTTP API a SSE](../ukoly/G-03-API-A-SSE.md) | **10/10 TESTED v CI; automatické main → DEV nasazování obnoveno, první fyzická instalace/rollback NEOVĚŘENO** |
| G-04 | [Úsporná historie](../ukoly/G-04-POSTGRESQL-HISTORIE.md) | **ROZPRACOVÁNO — G-04.04 SQLx migrace + izolovaný CI test připraven; skutečný DB LXC, TLS, writer a restore NEOVĚŘENO** |
| G-05 | [Živá mapa a základní ovládání](../ukoly/G-05-ZIVA-MAPA.md) | NEZAHÁJENO |
| G-06 | [Letiště a NAVAID/FIX](../ukoly/G-06-LETISTE-NAVAID.md) | NEZAHÁJENO |
| G-07 | [Vybraná meteorologie](../ukoly/G-07-VYBRANE-POCASI.md) | NEZAHÁJENO |
| G-08 | [Nouzové squawk a doručení](../ukoly/G-08-NOUZOVE-KODY-A-NOTIFIKACE.md) | NEZAHÁJENO |
| G-09 | [Statistiky a provozní diagnostika](../ukoly/G-09-STATISTIKY-DIAGNOSTIKA.md) | NEZAHÁJENO |
| G-10 | [Webová platforma](../ukoly/G-10-PWA-LOKALIZACE.md) | NEZAHÁJENO |
| G-11 | [Akceptace všech aktivních funkcí](../ukoly/G-11-ZAVERECNE-TESTOVANI.md) | NEZAHÁJENO |
| G-12 | [Oddělený shadow provoz](../ukoly/G-12-ODDELENY-SHADOW.md) | NEZAHÁJENO |
| G-13 | [Schválené nasazení a rollback](../ukoly/G-13-BEZPECNY-RELEASE.md) | PRODUKČNÍ ETAPA NEZAHÁJENA — samostatný DEV auto-deploy připraven, neaktivován |

## Technologická údržba G-00 (bez změny funkčního rozsahu)

- Node.js 26 a aktualizace webu: [zadání](../ukoly/G-00-AKTUALIZACE-STACKU.md), [report](../reports/G-00-AKTUALIZACE-STACKU-VYSLEDEK.md).
- Stav: **HOTOVO**, výsledky doloženy [CI 39673418390](https://github.com/Boym323/AirRadarV2/actions/runs/39673418390) na `d59f98d4`. Nejde o novou produktovou etapu.

## Technologický průzkum před pokračováním

- Dokončené **zdrojované srovnání návrhů**: [průzkum](TECHNOLOGICKY-PRUZKUM.md), [cílová architektura](ARCHITEKTURA-V2.md), [rozhodnutí](TECHNOLOGICKA-ROZHODNUTI.md).
- **Výkonové benchmarky zatím nebyly provedeny**. [Protokol B01–B07](BENCHMARKY-TECHNOLOGII.md) určuje společné vstupy, důkazy a přejímku.
- Rust + Axum a MapLibre zůstávají. **G-00 web je převeden na Vite/React**, se statickým výstupem a npm lockfile. **PostgreSQL byla zvolena definitivně** na již existujícím DB LXC, ale žádné V2 DB schéma zatím neexistuje.
- Číslování G-00 až G-13 se nemění; G-01 připraví fixture/kontrakty, G-04 připraví oddělenou PostgreSQL databázi V2 a G-05 implementuje MapLibre ve Vite webu. Měření nikdy nespouštět na V1.

## Podrobné implementační kroky

Každá níže uvedená etapa G-01 až G-13 má konkrétní kroky `G-XX.YY` s checkboxem, testovacím výsledkem a kritériem převzetí. Souhrn pořadí a vazeb: **[KROKOVY-IMPLEMENTACNI-PLAN.md](KROKOVY-IMPLEMENTACNI-PLAN.md)** — 140 kroků, dosud nepovažovaných za provedené. V plánových dokumentech nebyl při této změně implementován žádný nový produktový modul.

## Ověřená výchozí základna

- G-00: Rust, Next.js, připnuté závislosti a skutečné HTTP smoke testy: [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202).
- Tato základna **nepřijímá živá letadla**; veřejné API správně vrací `NOT_IMPLEMENTED`.
- Uživatel potvrdil samostatný Debian 13.7 DEV LXC (1 vCPU, 8 GiB RAM, 20 GiB disk), **G-00/G-01 aplikace byla skutečně instalována do DEV LXC; integrita a služby ověřeny**; produkční V1 zůstává nedotčená.
- Každá nová etapa musí mít skutečné testy i český report podle [šablony](../sablony/REPORT-ETAPY.md), odkazy na přesný commit a aktualizovaný [CHANGELOG](../../CHANGELOG.md).

## Závislosti

G-01 → G-02 → G-03 → G-04/G-05; G-06 doplní mapový katalog; G-07 přidá počasí; G-08 přidá nouzové kódy; G-09 diagnostiku; G-10 dokončí web; G-11 je bezpečnostní a výkonnostní přejímka; G-12 odpojený shadow; G-13 odděleně schválený release. Při vývoji lze bezpečně paralelizovat nezávislé moduly, ale funkce označit `ACCEPTED` pouze po testu.

**Zdroj pravdy:** `contracts/feature-matrix.json`, [moduly](MODULY-A-ROZSAH.md), a [závazné zadání](ZADANI-AIRRADAR-V2.md). Rozšíření mimo tento rozsah se nepřidávají automaticky.

## Detailní technické kontrakty před realizací

Podklad pro G-01 až G-11: [PostgreSQL, readsb/BEAST, SSE, MapLibre, Playwright a propojení LXC](TECHNICKE-KONTRAKTY-A-LIMITY.md). Jde výlučně o **návrhové parametry s měřicími kritérii**, nikoli o implementovaný produktový modul. Počet a pořadí kroků G-XX.YY se nemění.

## RAM-first optimalizace — návaznost bez změny etap

Audit úspornosti vedl k [adaptivní deduplikaci v Rust RAM](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md) a [revidovanému PostgreSQL modelu](../../database/POSTGRES-DATOVY-MODEL.md). Nová politika snižuje **potenciální** počet historických vzorků; nesmí se vykazovat jako skutečně naměřená úspora. G-01/G-02 definují a implementují per-field freshness/dedup; G-04 experimentálně uzavře sampling, batch, indexy, partitioning a obnovu. Funkce zůstávají NEZAHÁJENO, počet kroků beze změny.

## Audit před přejímkou G-01 — 22. 10. 2026

[**Úplný architektonický audit**](../audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md) konstatuje **GO pro zahájení G-01**, avšak **NO-GO pro její převzetí**, dokud nebudou testem doloženy: (1) `timeBasis/positionOrigin` a BEAST-only režim, (2) slučování identit a konfliktních pozorování, (3) formální DTO a atomický SSE snapshot/delta. Navazující úkoly G-04/G-08/G-09 mají další nezávislé brány. G-01 až G-13 jsou nadále NEZAHÁJENO.

## Dokončení G-01 — 22. 10. 2026

[Finální G-01 report](../reports/G-01-VYSLEDEK.md) a [CI 39748325104](https://github.com/Boym323/AirRadarV2/actions/runs/39748325104) prokazují OpenAPI 3.1, Draft 2020-12, readsb/BEAST/Mode-S a SSE golden fixtures i Rust/TypeScript kontraktové testy. **G-01 = HOTOVO**, G-02 až G-13 beze změny NEZAHÁJENO. Číslo 45 označuje budoucí produktové funkce, nikoli 45 implementovaných funkcí.

## Dev infrastruktura po G-01

Připraven [ruční balíčkovací workflow](../../.github/workflows/dev-package.yml) a [bezpečný instalační runbook](../runbooks/DEV-LXC-PRVNI-NASAZENI.md) pro potvrzený DEV LXC. GitHub Actions vytváří oddělený artefakt, ale **sám se k LXC nepřipojuje a nic nenasazuje**. První G-00/G-01 web a Rust health ještě neznamená dokončení G-02/G-03 ani produktových 45 KEEP.

## První skutečné DEV LXC nasazení — 22. 10. 2026

[**Záznam nasazení a důkazy**](../reports/DEV-LXC-PRVNI-NASAZENI-2026-10-22.md): artefakt `227a8619b590d2c536761ffefc75cd0a5633936c`, SHA-256 OK, systemd Rust/Nginx active, health `bootstrap`, aircraft `NOT_IMPLEMENTED`. **Web v prohlížeči v LAN uživatel potvrdil jako funkční; IPv6 firewall a dlouhodobý soak ještě nejsou ověřeny.** G-02 až G-13 stále NEZAHÁJENO; nedotčeno V1, PostgreSQL a externí zdroje.

## Průběžný G-02 replay — 22. 10. 2026

[Report G-02](../reports/G-02-VYSLEDEK.md): 5/11 kontrolovaných kroků PASS, zbytek rozpracován. [CI 39762483657](https://github.com/Boym323/AirRadarV2/actions/runs/39762483657) 3/3 PASS včetně 22 Rust testů a syntetických 1×/2×/4× měření. Nasazený LXC stále používá stabilní G-00/G-01 release. Žádné V1/readsb/DB změny.

## Novější CI G-02 a oddělená přejímka

Kód `81a27295d78753a6a3d987e7d68d6b8394c12ef5` má [CI #39769323592](https://github.com/Boym323/AirRadarV2/actions/runs/39769323592) **3/3 PASS**, **26 Rust testů PASS** a benchmark 5 000 syntetických identit. Kódy `J01/J02/J03/J04/J05/J11` mají v aktivní matici `TESTED`. Implementačních 10/11 kroků má testové důkazy. **G-02.11 a celá etapa zůstávají ROZPRACOVÁNO**, protože nebyla ověřena skutečná zátěž na cílovém DEV LXC. Současná instalace DEV LXC je nadále G-00/G-01, produkční V1 nedotčena.

## Automatická první brána DEV LXC G-02.11 — 22. 10. 2026

Doložen uživatelský výstup z `https://airradarv2.pomykal.cz/g02-status.json` (HTTP 200). Syntetický offline replay + 5 000 unikátních letadel na skutečném DEV LXC pro build `daa15703610b219b5b413319a4920659e19a8694` **PASS**, viditelný 1 CPU, p95 dávek 2,40–2,55 ms, RSS do 5 256 KiB. [Měření, provenance, metodika](../reports/G-02-VYSLEDEK.md#výsledek-automatického-offline-benchmarku-přímo-z-dev-lxc--22-10-2026). Nejde o HTTP/SSE/live-ingest test; zůstává G-02 **10/11, ROZPRACOVÁNO** do bezpečného dlouhodobého JSON/BEAST reconnect soak a vyhodnocení CPU/RAM. Produkční V1 se nemění.

## G-02 závěrečná brána úspěšně uzavřena — 22. 10. 2026 v 17:29 SELČ

Historické výše uvedené stavy `10/11` a `čeká LIVE` jsou nahrazeny skutečným výsledkem [GitHub run #39795592425](https://github.com/Boym323/AirRadarV2/actions/runs/39795592425) (`SUCCESS`): neměnný G-02 commit `d0cefc4a5c3332eadccf8ab21c96bae51987abe9`, 60min skutečný read-only BEAST stream, 109 154 pozorování, 77 peak tracků, nulové ztráty, CPU 0,36 %, RSS peak 2 492 KiB. Zároveň dříve potvrzený syntetický G-02 replay + 180s reconnect [#39781158565](https://github.com/Boym323/AirRadarV2/actions/runs/39781158565) PASS. **G-02 = 11/11 HOTOVO na úrovni vývojové/testovací etapy**, KEEP ID zůstávají `TESTED`; nasazení G-02 jako živého HTTP API, G-03 a provozní oprava systemd timeoutu zůstávají samostatné činnosti. [Závěrečný report](../reports/G-02-VYSLEDEK.md#závěrečná-provozní-přejímka-g-0211--pass-22-10-2026).

## Ověřený DEV auto-deploy z main (mimo produkční G-13)

Repo-scoped runner `airradarv2-dev` je aktivní a po úspěšném CI publikuje
immutable SHA-bound release do DEV LXC přes root-owned bridge se sdíleným
G-02 lockem, atomickým release a rollbackem. Deploy job nyní přímo ověřuje
služby, přesný `VERSION`, symlink, Rust health API a web; není závislý na
periodickém veřejném `PASS_SYNTHETIC_LXC`. `airradarv2-g02-auto.timer` a
`airradarv2-main-deploy.timer` jsou vypnuté, LIVE G-02 zůstává oddělený.
Aktivní důkazní release: `a1319067ae9fb5d01be3851eb3f899d25686e1e3`.
Nejde o produkční cutover, DB migraci ani změnu V1.

## Údržba LIVE G-02 po oddělení od autonomního CI/CD

G-02 LIVE workflow nyní používá vlastní `liveSoak.sha` a požadavek
`requestRunId`, zatímco historický offline `g02-status.json.sha`
se nepřepisuje. Autorizace vyžaduje aktivní DEV release shodný
s GitHub `main` a immutable bot release. Offline G-02 timer se
nezapíná; LIVE timer se zachovává. Jde o opravu testovací infrastruktury,
nikoli o novou produktovou funkci nebo nový 60min PASS.
