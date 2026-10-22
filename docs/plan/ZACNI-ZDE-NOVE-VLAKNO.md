# ZAČNI ZDE — nové vlákno nebo Codex běh AirRadar V2

**Repozitář:** https://github.com/Boym323/AirRadarV2. **Výchozí jazyk: čeština.** Jde o nový samostatný projekt; V1 je jen read-only reference. Celý implementační kontext je dostupný z GitHubu, není nutná historie chatu.

## Povinné soubory a pořadí

1. [AGENTS.md](../../AGENTS.md): pravidla, bezpečnost a český jazyk.
2. [ZADANI-AIRRADAR-V2.md](ZADANI-AIRRADAR-V2.md): závazný cíl a architektura.
3. [MODULY-A-ROZSAH.md](MODULY-A-ROZSAH.md): devět modulů a jejich přesná aktivní ID.
4. [TECHNOLOGICKA-ROZHODNUTI.md](TECHNOLOGICKA-ROZHODNUTI.md): Rust, **již převedený Vite/React web a definitivní PostgreSQL** na existujícím DB LXC.
5. [KROKOVY-IMPLEMENTACNI-PLAN.md](KROKOVY-IMPLEMENTACNI-PLAN.md): **140 kroků**, závislosti, pravidla malých PR a nejbližší G-01.01; [stav etap](ETAPY-A-STAV.md).
6. `contracts/feature-matrix.json` a [MATICE-FUNKCI.md](MATICE-FUNKCI.md): **45 schválených funkcí**, při implementaci jen tento rozsah.
7. [CO-NEPREBIRAT-Z-V1.md](CO-NEPREBIRAT-Z-V1.md): pravidla greenfield minimalismu.
8. [CHANGELOG.md](../../CHANGELOG.md), pouze reporty související s aktuální etapou, a jediný úkol z `docs/ukoly/`.

**Povinná technická příloha před G-01 až G-11:** [přesné kontrakty PostgreSQL, readsb/BEAST, SSE, MapLibre, Playwright a zabezpečení LXC](TECHNICKE-KONTRAKTY-A-LIMITY.md). Jsou to výchozí návrhové limity a jejich měřicí brány, nikoliv důkaz provozu.

**Aktuální provozní optimalizace:** [RAM-first deduplikace, cache a adaptivní minimalizace zápisů](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md). G-04 nesmí implementovat starou pevnou 15s historii, neprokázané indexy ani neomezenou write queue. Související [ADR 0007](../adr/0007-ram-first-deduplikace-a-usporny-zapis.md).

**Oddělený archiv budoucích návrhů není určen pro běžný vývoj**. Čti jej pouze na výslovnou žádost vlastníka, nikdy jej neimportuj do současné aplikace automaticky.

## Technologie: důležitý rozdíl mezi plánem a kódem

K 21. 10. 2026 webový skeleton běží na **Vite + React** a build produkuje statické `dist/`. **PostgreSQL je definitivní** volbou pro existující separátní DB LXC, ale vlastní databáze V2 a SQLx migrace budou vytvořeny až v G-04. Viz [průzkum](TECHNOLOGICKY-PRUZKUM.md), [benchmarky](BENCHMARKY-TECHNOLOGII.md), [cílová architektura](ARCHITEKTURA-V2.md).

## Co je skutečně dokončeno

- G-00: Rust + Vite/React scaffold a testy.
- G-01: 10/10 kontrakty/fixtures, CI #39748325104.
- G-02: 11/11 ověřený lokální readsb/BEAST ingest a jediný Rust RAM stav, 60min reálný read-only LIVE soak [#39795592425](https://github.com/Boym323/AirRadarV2/actions/runs/39795592425).
- G-03: izolovaný opt-in Rust HTTP/SSE PR #11 rozpracován; dílčích 5/10 kroků TESTED na [CI #39814106622](https://github.com/Boym323/AirRadarV2/actions/runs/39814106622), bez živého DEV nasazení.
- V2 PostgreSQL role, tabulky a historie zatím neexistují; G-04 teprve čeká. Produkční V1 je nedotknutelná.

## Postup při navazování

Ověř aktuální HEAD/branch/CI. V izolovaném checkoutu implementuj pouze konkrétní `G-XX.YY` krok ze schválené etapy. Proveď Rust fmt/Clippy/testy, Vite lint/typecheck/test/build, dokumentační kontrolu a případné integrační testy. Aktualizuj český `.md` report etapy, `ETAPY-A-STAV.md`, `CHANGELOG.md` a důkazy u aktivních KEEP. Závěr musí uvést skutečný commit, měření, PASS/FAIL a rizika.

## Zadání k vložení do dalšího vlákna

> Pracuj v `Boym323/AirRadarV2`. Přečti `AGENTS.md`, `docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md`, aktuální plán a jediný následující úkol. Aplikace je nový **místní ADS-B radar**, má pouze **45 aktivních funkcí**, žádné volby zdroje dat. Všechny dokumenty i výchozí uživatelské texty piš česky. Nepřebírej zdrojový strom V1. Zkontroluj aktuální main/CI a [krokový plán](KROKOVY-IMPLEMENTACNI-PLAN.md), implementuj jediný `G-XX.YY` krok s testy, aktualizuj `CHANGELOG.md` a český etapový report. Původní V1 a její DB/LXC neměň a bez samostatného souhlasu nic nenasazuj do produkce.

## Aktuální audit před vývojem

Před převzetím G-01 použij [audit architektury z 22. 10. 2026](../audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md): P0-01 čas a původ polohy, P0-02 identita/konflikty, P0-03 SSE atomické DTO. Neznamená změnu 45 aktivních funkcí ani implementaci dalších zdrojů.

## Aktuální etapa k převzetí

G-02 je HOTOVO 11/11. Pokračuj G-03.06–.10 dle [úkolu](../ukoly/G-03-API-A-SSE.md) a [reportu](../reports/G-03-VYSLEDEK.md); před dokončením a merge posuzuj skutečný Nginx/browser streaming, backpressure a p95/p99/RSS v DEV. PR #11 je draft, samotný merge do `main` spouští DEV auto-deploy podle samostatného runbooku, proto jej neprováděj bez dokončené G-03 brány.
