# G-11 — Základní statistiky, logbook a systémový dohled

**Stav:** NEZAHÁJENO
**Závislosti:** G-02, G-04, G-05 a aktivní weather moduly
**Primární KEEP ID:** `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`

## Cíl

Průhledné čítače, S06/S09, low-overhead status a diagnostické SSE pro aktivní části V2.

## Postup

1. S01 zprávy/s a aktuální letadla, S06 agregace typů/aerolinek/tratí pouze z dostupných skutečných dat, S09 stručný logbook.
2. O01–O08 implementovat jako levné diagnostiky pouze existujících systémů. O04 se týká jen aktivních providerů, např. ČHMÚ/SIGMET.
3. O05 správně rozlišit Rust CPU/RSS/fronty a Node runtime; O06 omezená historie výkonu; O07 DB stav bez těžkých dotazů.
4. O08 shared SSE s bounded fanout, bez per-client sond a zbytečného zahřívání cache.

## Testy a měření


- Status bez DB, bez readsb, failed provider, anonymní read-only, ochrana citlivých detailů.
- Měřený počet probe/DB dotazů podle klientů, paměť/CPU, retention diagnostiky.

## Co neimplementovat

S02 7/30 statistiky, coverage S03/S04, T58 O09 a samostatné benchmark centrum O10.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
