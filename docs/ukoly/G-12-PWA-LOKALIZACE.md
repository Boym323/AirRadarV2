# G-12 — Mobilní UX, PWA, offline cache a lokalizace

**Stav:** NEZAHÁJENO
**Závislosti:** G-06 a relevantní read API
**Primární KEEP ID:** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`

## Cíl

Moderní mobilní i desktopový UI bez přebírání staré aplikace.

## Postup

1. Responzivita a ovládání mapy, PWA manifest, instalace a bezpečně označená offline cache.
2. Výchozí čeština cs-CZ a později volitelná angličtina, bez překládání API konstant.
3. Globální command search jen pro skutečně implementované obrazovky a klávesové zkratky, žádné položky skryté analytiky.

## Testy a měření


- Playwright desktop/tablet/mobile, offline zobrazení starých dat s označením, PWA.
- Kontrola českých textů a anglického přepnutí UI, přístupnost klávesnicí.

## Co neimplementovat

U06/U07, barevné experimenty, samostatný prezentční mód.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
