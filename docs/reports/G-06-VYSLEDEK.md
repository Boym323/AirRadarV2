# G-06 — průběžná implementace veřejného katalogu letišť pro G-05.10

**24. 10. 2026 · Stav: ROZPRACOVÁNO, nikoli dokončeno.**  
**Výchozí commit:** `0f6b76002c63e29a14fcb94a473d84403c29e95e`.

## Provedená izolovaná část

- Aktivní ID `A01/A02/R12/R14`, kroky `G-06.01/.02/.03/.07` a návaznost `G-05.10`.
- Zdroj: veřejný `davidmegginson/ourairports-data`, [licence The Unlicense](https://github.com/davidmegginson/ourairports-data/blob/3b10d0b967cc04c62eb6411939e4e9936a88e701/LICENSE), dostupná na GitHubu. Přesné upstream commit `3b10d0b967cc04c62eb6411939e4e9936a88e701` a blob SHA-1 `1efc0826092c91e47add98ad1f0fa700bcd8b805`. Žádné vyjednané/omezované eAIP licence ani vymyšlené souřadnice.
- `apps/web/scripts/import-airports.mjs` jednorázově před Vite buildem stáhne tento **přesně připnutý 12.8 MB CSV** nebo načte explicitní lokální soubor; zkontroluje git blob SHA-1, parsuje CSV quoting/CRLF a validuje identitu, typ, souřadnice a duplicity.
- Import filtruje pouze otevřená velká/střední/malá letiště pro `CZ/SK/AT/PL/HU`, vytváří **jeden statický JSON** `/data/airports-ourairports.json` se zdrojem a verzí. NAVAID, frekvence a dráhy nejsou předstírané. Stáří dat určuje připnuté upstream SHA, nikoli čas stahování.
- `airport-catalog.ts` kontroluje typy a limity (max. 4 000 letišť) při jednom načtení. MapLibre vykreslí samostatný GeoJSON source **mimo 1Hz aircraft SSE**, kruhové letištní značky a zoomové labely. Přepínač vrstvy a kontextový detail lze vyvolat klikem; nejde o nové odchozí API ani backend závislost.
- Spouštěcí testy: Node `node --test scripts/import-airports.test.mjs`, Vitest `g05-airports.test.ts`, Chromium syntetický offline dataset s přepínačem mapy/detailu; build-time import reálného datasetu je součástí všech `npm run build`.

## Provozní hranice

- Není potřeba instalace softwaru na uživatelův počítač, žádný daemon, žádná nová DB ani autentizační mechanismus. Veřejný JSON obsahuje pouze public-domain referenční záznamy.
- HTTP zdroj je **statická webová data cesta**, nikoli finální verzované vyhledávací REST `/api/v1/airports` z G-06.05. Je bezpečně oddělen od G-03 a lokálního přijímače; při chybě importu build selže a do DEV se nenasadí neověřený dataset.
- Oficiální eAIP má nadále přednost pro skutečnou navigaci, provozní frekvence a aktuální letištní informace.

## Co zůstává pro úplné G-06 a G-05

- G-06.04 frekvence, G-06.05 REST full API/vyhledávání, G-06.06 NAVAID/FIX, G-06.08 měření RAM/latencí a G-06.09 finální akceptace.
- G-05.10 přepnout na finálně schválený G-06 katalog kontrakt; označit `TESTED` až po dokončení zdrojové/datové přejímky.
- G-05.11 skutečný desktop i mobil GPU/FPS výkon mimo GitHub SwiftShader CI.
- **CI/DEV hodnoty a PASS uvést až po dokončení workflow**, ne po samotném commitování této dokumentace.
