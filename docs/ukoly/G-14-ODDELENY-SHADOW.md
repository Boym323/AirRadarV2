# G-14 — Neovlivňující shadow test V2

**Stav:** NEZAHÁJENO
**Závislosti:** G-13
**Primární KEEP ID:** žádná; interní brána nebo zrušený modul

## Cíl

Spustit V2 odděleně ve vlastním LXC na schváleném read-only mirroru nebo replay.

## Postup

1. Žádné sdílené pracovní adresáře/DB/ports/deploy runner s V1.
2. Read-only napojení zdroje až po doložení, že druhý konzument neovlivní V1.
3. Zakázat doručování L11/L12, porovnat místní provoz, reconnect, RAM/CPU/P95 a DB I/O.

## Testy a měření


- Doložené porovnatelné vstupy a dlouhodobé bounded zdroje.
- Po zastavení V2 ověřit, že V1 beze změny.

## Co neimplementovat

Žádný produkční cutover, DNS/proxy switch ani restart V1.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
