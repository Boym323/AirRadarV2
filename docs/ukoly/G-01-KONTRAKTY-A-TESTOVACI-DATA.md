# G-01 — API kontrakty, testovací data a přejímací scénáře

**Stav: NEZAHÁJENO**  
**Závislost:** G-00 (ověřené Rust/Next.js sestavení a HTTP smoke)  
**Rozsah:** pouze kontrakty a testovací základ; **neimplementovat celý ingest, databázi ani produkční nasazení**.

## Cíl

Vytvořit jednoznačné a testovatelné definice chování AirRadar V2 před vývojem živého přijímače. Odstranit nejasnosti z V1, aniž by se do V2 kopíroval její monolit.

## Zadání

1. Projdi všech **46 KEEP** v `contracts/feature-matrix.json`, rozděl je na odpovědné moduly a pro každou funkci napiš měřitelný přejímací scénář: vstup, očekávaný výstup, chyba, související bezpečnostní invariant a odkaz na budoucí test.
2. Doplň verzované OpenAPI/JSON Schema pro health, aircraft, single aircraft, SSE full/delta a chybové odpovědi. Navrhni pevné DTO typy Rust a generovaný TS klient. Veřejné chyby musí být sanitizované. Nepřekládej klíče/stavové konstanty, vysvětlení piš česky.
3. Připrav *syntetické nebo bezpečně anonymizované* referenční readsb JSON a BEAST sekvence: normální let, ztráta pozice, opakovaná zpráva, změna callsign, výpadek zdroje, chybné ICAO, ne-ICAO identita, wrap timestampu, více lokálních letadel a přepnutí BEAST/JSON (bez simulace externích síťových poskytovatelů).
4. Založ strojově spustitelné golden/contract/property testy, kontrolu stability DTO a replay bez vnější sítě. Žádné testovací fixture nesmí obsahovat identifikátory tajných účtů, credentials nebo přesné soukromé souřadnice.
5. Urči chybějící vstupy jen pro zachované funkce: letiště/NAVAID, ČHMÚ/SIGMET/BDS, emergency squawk L06 a vzorkovanou historii J09. Uveď zdroje dat, licence a omezení při jejich veřejném ukládání.
6. Zveřejni v českém ADR případné odchylky V2 od starých API (zachování produktové funkce je nadřazené nevyžádané byte-level kompatibilitě).
7. Výsledek zapiš do `docs/reports/G-01-VYSLEDEK.md`; aktualizuj `docs/plan/ETAPY-A-STAV.md` a stavy funkcí pouze pokud mají skutečný implementační důkaz.

## Hotovo když

- Množina všech 46 KEEP má akceptační scénáře (L06 je nově zahrnuta, 34 vyřazených ID neimplementovat); 132 REMOVE nepřibylo do rozsahu.
- Kontrakty jsou syntakticky a behaviorálně otestované proti deterministickým datům.
- Příklady explicitně rozlišují skutečný live stav a současný G-00 `NOT_IMPLEMENTED`.
- Dokumentace a názvy testovacích scénářů pro uživatele jsou česky; identifikátory zůstávají stabilní.
- Všechny testy a CI prošly na přesném SHA, nebo je etapa označena **ČÁSTEČNĚ/BLOKOVÁNO** s důkazy.

## Pokyn pro Codex

> Realizuj pouze **G-01** v repozitáři `Boym323/AirRadarV2` podle tohoto českého Markdown zadání. Nejdřív ověř aktuální `main`, přečti `AGENTS.md` a závazné `docs/plan/ZADANI-AIRRADAR-V2.md`. Vytvoř skutečné verzované kontrakty, bezpečné fixture a automatické testy, dolož jejich výsledky, aktualizuj matici a český report. Nepracuj v produkčním AirRadar V1, nepřipojuj živý přijímač ani nevytvářej deploy.
