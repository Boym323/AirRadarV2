# ZAČNI ZDE — nové vlákno nebo Codex běh AirRadar V2

**Repozitář:** https://github.com/Boym323/AirRadarV2. **Výchozí jazyk: čeština.** Jde o nový samostatný projekt; V1 je jen read-only reference. Celý implementační kontext je dostupný z GitHubu, není nutná historie chatu.

## Povinné soubory a pořadí

1. [AGENTS.md](../../AGENTS.md): pravidla, bezpečnost a český jazyk.
2. [ZADANI-AIRRADAR-V2.md](ZADANI-AIRRADAR-V2.md): závazný cíl a architektura.
3. [MODULY-A-ROZSAH.md](MODULY-A-ROZSAH.md): devět modulů a jejich přesná aktivní ID.
4. [ETAPY-A-STAV.md](ETAPY-A-STAV.md): G-00 až G-13, doložený stav a návaznosti.
5. `contracts/feature-matrix.json` a [MATICE-FUNKCI.md](MATICE-FUNKCI.md): **45 schválených funkcí**, při implementaci jen tento rozsah.
6. [CO-NEPREBIRAT-Z-V1.md](CO-NEPREBIRAT-Z-V1.md): pravidla greenfield minimalismu.
7. [CHANGELOG.md](../../CHANGELOG.md), pouze reporty související s aktuální etapou, a jediný úkol z `docs/ukoly/`.

**Oddělený archiv budoucích návrhů není určen pro běžný vývoj**. Čti jej pouze na výslovnou žádost vlastníka, nikdy jej neimportuj do současné aplikace automaticky.

## Co je skutečně dokončeno

- G-00: Rust/Next.js scaffold, lockfily, build, testy a HTTP smoke — [historický důkaz GitHub CI](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202).
- G-01: první neimplementovaná etapa. Rozpracuj kontrakty/fixture, teprve potom příjem.
- V2 nemá prokázaný živý ingest, aktivní PostgreSQL ani release do samostatného LXC.
- Produkční V1, její DB i LXC se nesmějí měnit.

## Postup při navazování

Ověř aktuální HEAD/branch/CI. V izolovaném checkoutu implementuj pouze schválenou etapu. Proveď Rust fmt/Clippy/testy, Next.js lint/typecheck/test/build, dokumentační kontrolu a případné integrační testy. Aktualizuj český `.md` report etapy, `ETAPY-A-STAV.md`, `CHANGELOG.md` a důkazy u aktivních KEEP. Závěr musí uvést skutečný commit, měření, PASS/FAIL a rizika.

## Zadání k vložení do dalšího vlákna

> Pracuj v `Boym323/AirRadarV2`. Přečti `AGENTS.md`, `docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md`, aktuální plán a jediný následující úkol. Aplikace je nový **místní ADS-B radar**, má pouze **45 aktivních funkcí**, žádné volby zdroje dat. Všechny dokumenty i výchozí uživatelské texty piš česky. Nepřebírej zdrojový strom V1. Zkontroluj aktuální main/CI, implementuj vybranou etapu s testy, aktualizuj `CHANGELOG.md` a český etapový report. Původní V1 a její DB/LXC neměň a bez samostatného souhlasu nic nenasazuj do produkce.
