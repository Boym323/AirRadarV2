# Pravidla Markdown dokumentace AirRadar V2

**Výchozí a povinný jazyk: čeština (`cs-CZ`).** Platí pro README, AGENTS, zadání, reporty, ADR, provozní runbooky, vysvětlující texty OpenAPI a výchozí webové UI.

## Zdroj pravdy

| Soubor nebo adresář | Jak se používá |
|---|---|
| `AGENTS.md` | Pravidla agentů, bezpečnost a jazyk |
| `docs/plan/ZADANI-AIRRADAR-V2.md` | Jedno závazné hlavní zadání |
| `docs/plan/ETAPY-A-STAV.md` | Aktuální stav G-00…G-15 |
| `docs/ukoly/G-XX-*.md` | Samostatná zadání s akceptačními kritérii |
| `docs/reports/G-XX-*.md` | Důkazy testů, měření, revize a blokéry |
| `docs/adr/NNNN-*.md` | Trvalá architektonická rozhodnutí |
| `docs/runbooks/*.md` | Postupy provozu, záloh, LXC a obnovy |
| `contracts/feature-matrix.json` | Strojový stav jednotlivých 79 KEEP a 99 REMOVE |
| `docs/plan/MATICE-FUNKCI.md` | Automaticky vytvořená lidská matice z JSON |

## Při každé samostatné etapě

- Předem: přečíst zadání, aktuální stav, související ADR a kontrakt. Nedávat do vstupu agenta všechny historické reporty bez potřeby.
- Během: zaznamenat přesné změny, nové/změněné API, dotčené KEEP ID, testy a měřicí metodiku. Kód a dokumentace musí odpovídat skutečnosti.
- Po implementaci: spustit relevantní testy a CI, upravit Markdown dokumenty, vygenerovat matici, aktualizovat `ETAPY-A-STAV.md`, dodat report s ověřeným SHA a riziky.
- Pokud se něco nepovedlo, ponechat stav **ČÁSTEČNĚ / BLOKOVÁNO**, přesně popsat překážku a nevymýšlet čísla.
- Rozsáhlé zadání vždy uložit jako soubor `.md` v repozitáři; chatový prompt má obsahovat primárně cestu, ID etapy a požadovaný výsledek.
- Soubory nepoužívej jako nekonečný deník: průběžné poznámky patří do reportu etapy, dlouhodobá rozhodnutí do ADR, trvalé požadavky do hlavního zadání.

## Jazyk a kompatibilita

- Dokumentační proza, nadpisy, závěry, UI hlášení a uživatelská nápověda musí být **česky**.
- Technické identifikátory, názvy příkazů, tabulek, JSON klíčů, formátů a stavových kódů v kódu se nepřekládají. Vlastní vysvětlující text a příklady piš česky.
- U04 požaduje později volitelný anglický překlad UI; výchozí jazyk musí zůstat čeština. Není důvod vytvářet anglickou projektovou dokumentaci.
- Nevkládej do veřejného repozitáře tokeny, adresy citlivých systémů, screenshoty s osobními údaji ani surová soukromá data přijímače.

## Kontrola

~~~bash
node tools/check-docs.mjs
node tools/generate-feature-matrix.mjs --check
~~~

Při změně dokumentů uprav jejich relativní odkazy. Nezaměňuj akceptované dokumenty s implementovanými funkcemi: text požadavku nezaručuje existenci kódu.

