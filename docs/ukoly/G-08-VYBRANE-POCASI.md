# G-08 — ČHMÚ radar, SIGMET a Mode-S meteorologie

**Stav:** NEZAHÁJENO
**Závislosti:** G-02, G-06
**Primární KEEP ID:** `W03`, `W06`, `W08`, `W09`

## Cíl

Pouze W03/W06/W08/W09 bez METAR/TAF a bez obecného weather-fusion systému.

## Postup

1. Oddělené cache/TTL pro ČHMÚ radar a SIGMET s platnými časovými okny; mapové zobrazení jen těchto produktů.
2. Dekódování BDS 4,4 z živých zpráv letadla, s kvalitou/nejistotou a údajem o chybějícím měření.
3. Žádné provider volání v kritické cestě ingest; W09 jednoduchý přehled kvality skutečně dostupných měření, ne H07 historický profil.

## Testy a měření


- Weather provider 429/5xx/timeout, stale-if-error, intervaly platnosti, BDS s invalid údaji.
- Porovnat náklady mapového overlay a nulový dopad na readsb v nepřítomnosti uživatelů počasí.

## Co neimplementovat

W01/W02 METAR/TAF, W04 Echo Top, W05 vítr, W07 PIREP, W10–W14, R18 univerzální vrstvy.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
