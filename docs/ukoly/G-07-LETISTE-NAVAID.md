# G-07 — Katalog letišť a NAVAID/FIX

**Stav:** NEZAHÁJENO
**Závislosti:** G-01, G-04; G-05 pokud je nutná trvalá DB
**Primární KEEP ID:** `A01`, `A02`, `T10`

## Cíl

Úsporný katalog letišť, drah, jejich skutečných frekvencí a navigačních bodů pro mapu a vyhledání.

## Postup

1. Převzít jen schválené veřejné datové zdroje s licencí, verzí a proveniencí; zvolit nejjednodušší import.
2. API pro A01/A02/T10, indexy pro ICAO/poziční dotazy, chránit proti nesprávným datům.
3. Stav katalogu po aktualizaci řízeně validovat, ale bez plnohodnotného pravidelného eAIP synchronizačního enginu O13.

## Testy a měření


- Vyhledávání ICAO, správnost drah/frekvencí/NAVAID, chybějící údaje, licence.
- Rychlost katalogových dotazů a objem dat.

## Co neimplementovat

Žádné ATS tratě ČR/SK/AT, sektory ATC, historická anomálie T13/T14 nebo pravidelný O13 pipeline.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
