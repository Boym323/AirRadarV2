# ZAČNI ZDE — předání AirRadar V2 do nového vlákna

Toto je **povinný kontextový vstup** při zahájení nové konverzace, nového Codex běhu nebo předání mezi vývojovými agenty. **Není potřeba načítat historii předchozího chatu.** Nejdříve přečti uvedené soubory v tomto repozitáři, aktuální HEAD a související zdrojový kód.

## Kde se pracuje

- Nový GitHub: https://github.com/Boym323/AirRadarV2, vývoj `main`, výchozí jazyk **čeština**.
- Starý GitHub: https://github.com/Boym323/AirRadar — **jen read-only reference**, v jeho produkčním LXC ani databázi nic neměň.
- Pro V2 je určen **jiný, vlastní LXC**; jeho založení, IP/SSH přístup a produkční deployment nejsou v aktuálním repozitáři doloženy. Nevymýšlej host/uživatele/credentials; tajemství nepatří do GitHubu.
- V2 starou historickou databázi z V1 nepřebírá. Nová historie bude záznamy ukládat až při G-05.
- Stack: Rust/Tokio/Axum jako jediný vlastník RAM state a streamu; Next.js/React/MapLibre pro UI; PostgreSQL/SQLx pro vzorkovanou historii. Volitelné jazyky pouze s naměřeným přínosem.

## Přečti v tomto pořadí (povinné)

1. [AGENTS.md](../../AGENTS.md) — český jazyk, bezpečnost a evidence
2. [Závazné zadání](ZADANI-AIRRADAR-V2.md) — produkt a architektura
3. [Katalog modulů](MODULY-A-ROZSAH.md) — konkrétně co implementovat a ke kterým KEEP ID
4. [Co nepřebírat z V1](CO-NEPREBIRAT-Z-V1.md) — seznam nechtěného dluhu
5. [Etapy a stav](ETAPY-A-STAV.md) — co je skutečně hotové a co dál
6. [Matice funkcí](MATICE-FUNKCI.md), [podrobné popisy všech 178 funkcí](PODROBNY-KATALOG-FUNKCI.md) a `contracts/feature-matrix.json` — rozsah k revizi, nikoli automaticky schválený implementační seznam
7. [CHANGELOG](../../CHANGELOG.md) — dokončené změny a doložené vydané/verifikované výsledky
8. **Jediné zadání G-XX**, na kterém máš právě pracovat, v `docs/ukoly/`, a případné související ADR/reports

**Nečti automaticky celou složku** `docs/` ani historické audity V1. Pokud konkrétní dokument chybí, oprav nejdřív dokumentaci, nevymýšlej obsah.

## Aktuální bezpečný stav (reviduj vůči HEAD)

- **G-00:** ověřený základ Rust HTTP a Next.js s lockfile, [CI 39662365349](https://github.com/Boym323/AirRadarV2/actions/runs/39662365349) úspěšně na `d72fad1` (platí pro tento konkrétní SHA).
- **G-01:** kontrakty/fixture zatím `NEZAHÁJENO`; [zadání](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md).
- **G-02 až G-15:** zadání existují, implementace zatím nebyla potvrzena. Přesný stav vždy ověř v `ETAPY-A-STAV.md` a kódu.
- G-00 `/api/v1/health` vrací `bootstrap`; `/api/v1/aircraft` vrací `NOT_IMPLEMENTED`; nic z toho neprokazuje živý příjem.
- Žádný produkční deploy, změna DNS/proxy, čtení V1 DB, nové alerty či V1 LXC.
- Důkazy o vývoji ukládej do **českých MD** a aktualizuj `CHANGELOG.md`; netvrď GO/PASS bez stejného SHA CI.

## Postup při každém novém vlákně

1. Ověř `git branch --show-current`, `git rev-parse HEAD`, `git status --short` a aktuální GitHub Actions.
2. Zjisti jednu následující neuzavřenou etapu a její KEEP ID. Nesměš přeskočit závislosti.
3. V izolovaném vývojovém checkoutu implementuj **pouze schválený rozsah**. Starý kód neimportuj plošně.
4. Proveď Rust fmt/clippy/testy, web lint/typecheck/test/build a `node tools/check-docs.mjs`. U DB/SSE/UI přidej odpovídající integrační testy.
5. Doplň české Markdown zadání/report, matici s důkazy, stav etap a changelog. CI musí běžet na přesném commitu.
6. Připrav stručný závěrečný report: změny, testy a CI s SHA, měření, regrese, chybějící důkazy a rozhodnutí HOTOVO / ČÁSTEČNĚ / BLOKOVÁNO.

## Prompt k vložení do nového vlákna

> Pracuj na **AirRadar V2** v repozitáři https://github.com/Boym323/AirRadarV2. Přečti `AGENTS.md` a `docs/plan/ZACNI-ZDE-NOVE-VLAKNO.md`, potom jen relevantní zadání etapy, katalog modulů, matici funkcí a poslední changelog. Všechny pracovní dokumenty i viditelné texty aplikace piš **česky**. Ověř aktuální `main` a GitHub CI, pokračuj pouze v první nedokončené bezpečně realizovatelné etapě. Aktualizuj `docs/plan/ETAPY-A-STAV.md`, český report, `CHANGELOG.md` a skutečně ověřené KEEP ID. Nedělej nic v AirRadar V1 ani jeho databázi/LXC. Nepoužívej staré moduly bez testovaného důvodu a žádný produkční deploy bez výslovného samostatného souhlasu.
