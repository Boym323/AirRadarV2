# G-10 — Emergency squawk 7500/7600/7700 a notifikace

**Stav:** NEZAHÁJENO
**Závislosti:** G-02, G-04, G-05 pokud se použije durable doručení
**Primární KEEP ID:** `L06`, `L11`, `L12`

## Cíl

Detekce L06 v místně přijatých squawk datech a přesně vymezené volitelné doručení L11/L12.

## Postup

1. Spolehlivě rozpoznat squawk 7500/7600/7700 a změny/ukončení se zamezením duplicit stejné události.
2. Web Push/Pushover transport s bounded retry/idempotentním claimem, detekcí chybové odpovědi a explicitní konfigurací.
3. Ve výchozím stavu i ve stínovém provozu zakázat skutečné odesílání; žádná administrace jako produkt O15.

## Testy a měření


- Squawk sekvence v replay, opakovaná zpráva, více letadel, restart, falešné či chybné squawk.
- Provider 2xx s chybou, 4xx/429/5xx, souběh workerů, nulová skutečná notifikace v shadow.

## Co neimplementovat

L01–L05, L07–L10, L13–L15 jako samostatné produkty; žádný univerzální watchlist.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
