# G-04 — Veřejné API a SSE snapshot/delta

**Stav:** NEZAHÁJENO
**Závislosti:** G-01, G-02
**Primární KEEP ID:** `J06`, `J07`, `J08`

## Cíl

Čisté verzované API, veřejné DTO a full/delta SSE nad lokálním RAM state.

## Postup

1. Doplnit OpenAPI a typované Rust/TS kontrakty pro list, výběr, initial snapshot a changed/removed.
2. SSE heartbeat 15 s, správné pořadí, disconnect cleanup, bounded buffer, pomalý klient a capacity 503.
3. Nevydávat lokální interní údaje (privátní pozici přijímače, secrets, raw metadata) do veřejných odpovědí.

## Testy a měření


- SSE 1/10/100 klientů; reconnect, backpressure, zahozené/zmizelé letadlo, absence data leaks.
- Měřená P95/P99 latence a CPU/RSS pro stejné replay.

## Co neimplementovat

Žádný opakovaný readsb polling na každého klienta, žádný další message broker.

## Kritéria dokončení

Každá související KEEP funkce má konkrétní testovací důkaz, relevantní build/CI prošly na přesném SHA, byly aktualizovány český report, stav etap a changelog. Nesplněné položky označit ČÁSTEČNĚ/BLOKOVÁNO. Žádný deploy do V1.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
