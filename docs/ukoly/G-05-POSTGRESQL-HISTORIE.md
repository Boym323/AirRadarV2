# G-05 — Nová PostgreSQL a úsporné vzorkování

**Stav:** NEZAHÁJENO
**Závislosti:** G-02, G-04
**Primární KEEP ID:** `J09`

## Cíl

Izolovaná PostgreSQL od nuly; pouze relevantní vzorkovaná historie J09 pro budoucí logbook a statistiky S06/S09.

## Postup

1. Query-first schéma s minimem tabulek; samostatná V2 DEV databáze a least-privilege role; migrace v SQLx.
2. Implementovat ohraničenou frontu a dávkové transakce, retenční politiku podle potřeby S06/S09, indexy dle EXPLAIN.
3. Fail-soft režim při výpadku DB a prázdné nové historii, bez přenosu původních letů.

## Testy a měření


- Zápisy/restart/coalescing/duplicitní let; DB offline; DST dle APP_TIMEZONE; bounded paměť.
- Měřené transakce, WAL, p95/p99 dotazů a velikost dat proti baseline.

## Co neimplementovat

Žádné H07/H14 profily/souhrny jednoho letadla, S02 7/30denní API, coverage ani kopie Prisma V1.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
