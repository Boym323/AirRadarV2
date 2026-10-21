# Pravidla české Markdown dokumentace AirRadar V2

**Výchozí jazyk:** čeština `cs-CZ`. Platí pro README, AGENTS, hlavní zadání, etapy, ADR, reporty, runbooky, API vysvětlení a výchozí UI. Technické názvy a strojové identifikátory se nepřekládají.

## Zdroj pravdy

| Soubor | Účel |
|---|---|
| `AGENTS.md` | Závazná pravidla agentů |
| `docs/plan/ZADANI-AIRRADAR-V2.md` | Produkt a architektura |
| `docs/plan/MODULY-A-ROZSAH.md` | Vlastnictví přesně 45 aktivních ID |
| `contracts/feature-matrix.json` | Strojový stav pouze aktivních funkcí |
| `docs/plan/MATICE-FUNKCI.md` | Automaticky generovaná česká matice |
| `docs/plan/PODROBNY-KATALOG-FUNKCI.md` | Slovní popisy aktivních funkcí |
| `docs/plan/ETAPY-A-STAV.md` | Aktuální 14etapový plán |
| `docs/ukoly/G-XX-*.md` | Jednotlivá proveditelná zadání |
| `docs/reports/G-XX-*.md` | Skutečné výsledky a důkazy |
| `CHANGELOG.md` | Evidované změny vývoje |
| `docs/backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md` | Jediná izolovaná evidence dosud neschválených nápadů |

## Postup

1. Přečíst jen požadovanou etapu, její aktivní ID a související kód.
2. Při změnách doplnit testy, bezpečnostní invarianty, vlastníka a odhad provozní ceny.
3. Po práci aktualizovat český report, matici, etapový plán i `CHANGELOG.md` se skutečným testovacím důkazem.
4. Spustit `node tools/generate-feature-matrix.mjs --check` a `node tools/check-docs.mjs`.
5. Neoznačovat hypotetické výsledky za PASS ani dokumentované zadání za hotový kód.

Rozšíření se z odděleného archivu do aktivního rozsahu přesunuje jen po výslovném rozhodnutí vlastníka a následné aktualizaci všech návazností.
