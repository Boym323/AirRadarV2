# G-06 — Interaktivní MapLibre mapa se základními prvky

**Stav:** NEZAHÁJENO
**Závislosti:** G-04, G-07 pro letištní vrstvu
**Primární KEEP ID:** `R01`, `R02`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`

## Cíl

Rychlá základní mapa, popisky a ikony, seznam, nedávná stopa, follow, přepínání podkladů.

## Postup

1. MapLibre lifecycle ve stabilním hooku bez remountů, SSE delta batch a šetrné aktualizace dat mapy.
2. Uživatel může zvolit letadlo v seznamu a použít trail/follow, ale bez rozsáhlého samostatného panelu R07.
3. R02 LOCAL/EXTENDED nesmí přidávat fiktivní letadla; oznámit nedostupnost externího zdroje. R12 má katalog z G-07.

## Testy a měření


- Mobil/tablet/desktop, mapa scroll/zoom/search, trail/follow/reconnect.
- Měřit React re-renders a MapLibre setData; test skutečného prázdného EXTENDED.

## Co neimplementovat

R05 barevné režimy, R07 detail, R10 multitrack, R11 trasa letišť, R15 HUD, R17 ATC, R18 univerzální overlay.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
