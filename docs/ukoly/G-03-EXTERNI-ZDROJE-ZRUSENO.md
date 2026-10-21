# G-03 — Původní externí ADS-B zdroje — ZRUŠENO

**Stav:** VYŘAZENO (bez implementace)
**Závislosti:** žádná
**Primární KEEP ID:** žádná; interní brána nebo zrušený modul

## Cíl

Po schváleném vyřazení E01/E02/E03 se tato etapa **nerealizuje**. Zachovat číslo etapy pouze kvůli kontinuitě plánu.

## Postup

1. Nevytvářet síťové adaptéry, HTTP fallback, ADSBHub/ADSB.lol klienty ani nové credentials.
2. R02 zatím zůstává požadavkem UI, ale bez externího feedu se žádná další letadla nevykreslují. EXTENDED musí být označeno jako nedostupné.

## Testy a měření


- CI musí hlídat, že odkaz na vyřazené ID nepovažuje za KEEP.

## Co neimplementovat

Jakákoli implementace E01/E02/E03 bez dalšího výslovného zadání.

## Kritéria dokončení

Neprovádět žádnou novou implementaci. V plánu etapu evidovat jako zrušenou a zachovat pouze návaznost dalších G-XX.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
