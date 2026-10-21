# G-02 — Rust příjem readsb/BEAST a stav letadel

**Stav:** NEZAHÁJENO
**Závislosti:** G-01
**Primární KEEP ID:** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`

## Cíl

Funkční místní radar se single-owner stavem v Rust RAM, bez DB a externích providerů.

## Postup

1. Napsat bounded JSON snapshot adapter readsb a BEAST TCP dekodér se správným CRC24, DF, escape a časovým razítkem.
2. Přepínat BEAST/JSON pouze po výpadku, bez duplikace či ztráty identity; validovat ICAO/ne-ICAO, poloha, rychlost a stáří.
3. Udržet jediný kanonický RAM state, deduplikaci, ochranu před hromadným výpadkem a explicitní health/failure stav.

## Testy a měření


- Replay se syntetickými validními i poškozenými rámci; ztráta a návrat zdroje; continuity po změně callsign.
- 1×/2×/4× replay měření CPU/RSS, bounded paměti, latence; nulový přístup k DB.

## Co neimplementovat

Žádné externí ADSBHub/ADSB.lol/OGN, žádný Node polling, žádné surové zápisy všech zpráv.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
