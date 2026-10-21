# G-13 — Akceptace 46 KEEP a bezpečnostní brány

**Stav:** NEZAHÁJENO
**Závislosti:** G-02, G-04–G-08, G-10–G-12
**Primární KEEP ID:** žádná; interní brána nebo zrušený modul

## Cíl

Nepřidávat O11 jako produkt, ale provést nezbytné CI, bezpečnostní a regresní testy 46 KEEP.

## Postup

1. Zkontrolovat všechny KEEP ID s konkrétním důkazem ve strojové matici; REMOVE se neobnovuje.
2. Prověřit Rust/TS dependency audit, privacy/sanitizaci DTO, nepovolené mutace, výpadek DB/receiveru a paměťové limity.
3. Spustit reprodukovatelné integrační, Playwright a zátěžové testy, posoudit CPU/RSS/P95/P99 a návratový postup.

## Testy a měření


- Skutečná evidence u všech 46 ID nebo explicitní NO-GO.
- Nulový leak tajných údajů a ověřená absence nevyžádaných externích providerů.

## Co neimplementovat

O11 jako samostatná produktová stránka a O09/O10 jako nově poskytovaná diagnostická služba; testy jsou inženýrským minimem.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
