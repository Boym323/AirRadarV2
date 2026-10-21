# G-15 — Manuálně schválené nasazení V2 s návratem

**Stav:** NEZAHÁJENO
**Závislosti:** G-13 a G-14, přístup do nového LXC
**Primární KEEP ID:** žádná; interní brána nebo zrušený modul

## Cíl

Bezpečně vypustit samostatnou V2, aniž by se obnovoval O12 jako komplexní automatický release engine.

## Postup

1. Připravit ruční verifikovaný release runbook, systemd jednotky, oddělené secrets a zmapované porty.
2. Záloha/restore V2 DB, exact SHA build, health/rollback a test nezávislosti na V1.
3. Jakákoliv změna původní domény, reverzní proxy nebo vypnutí V1 potřebuje samostatné výslovné schválení.

## Testy a měření


- HTTP/SSE/map smoke a skutečná postrelease pozorování.
- Záloha+obnova DB a bezpečný návrat k funkční verzi.

## Co neimplementovat

O12 produktové automatické nasazování; žádný automatický zásah do V1.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
