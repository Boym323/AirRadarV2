# AirRadar V2 — závazné technické kontrakty a limity

**Datum:** 22. 10. 2026. **Stav:** návrh G-04+ stále nepřevzat; G-03 HTTP/SSE a Chromium browser byly otestovány izolovaně v CI #39849652858, bez cílového DEV LXC měření či DB tabulek. Rozsah nadále obsahuje pouze 45 aktivních funkcí.

| Oblast | Startovací rozhodnutí | Detailní české zadání | Etapa |
|---|---|---|---|
| PostgreSQL | SQLx; primární RAM deduplikace, adaptivní vzorkování běžně ≥ 60 s, 30s dedup bucket; pouze PK, druhý index a partitioning jen podle měření, 30 dní detail a 365 dní agregace | [Datový model, SQL a zálohy](../../database/POSTGRES-DATOVY-MODEL.md) | G-04 |
| readsb/BEAST | JSON jako referenční zdroj, BEAST s validací DF/CRC/CPR, jedna RAM autorita; ICAO a ne-ICAO odděleně | [Kontrakty a deduplikace](../technical/READSB-BEAST-KONTRAKTY.md) | G-01/G-02 |
| SSE | initial snapshot + delta 1 Hz, 15 s heartbeat, 100 klientů, max. 256 KiB pending fronta na klienta, seq/epoch resync | [SSE a paměť](../technical/SSE-PROTOKOL-A-LIMITY.md) | G-01/G-03 |
| MapLibre | jedna Map instance, GeoJSON source + symbol layers, stabilní ID; `setData` baseline, `updateData` po měření | [Výkon mapy](../technical/MAPLIBRE-VYKON-A-ZATEZ.md) | G-05 |
| Browser E2E | Microsoft Playwright, izolované fixture, Chromium/Firefox/WebKit/mobil, trace na chyby | [Automatické testy](../technical/PLAYWRIGHT-BROWSEROVE-TESTY.md) | G-01/G-05/G-10/G-11 |
| Síť obou LXC | samostatné V2 DB/role, hostssl+SCRAM, TLS verify-full, firewall povolí jen V2 app | [Bezpečnost a obnova](../runbooks/POSTGRES-LXC-SECURITY-BACKUP.md) | G-04/G-11 |

## Výslovné omezení

Tato čísla jsou **konfigurační výchozí hodnoty**, nikoli garantovaný výkon existujícího LXC. Zálohovací cíle: denní logický dump V2 mimo DB LXC, 14 denních a 8 týdenních kopií, měsíční nezávislá testovací obnova; RPO 24 h a RTO 2 h jsou **cíle k ověření**, cluster WAL/PITR ověřuje správce. Testovat replay a obnovení před definitivní akceptací.

K implementaci nejprve G-01 fixture a kontrakty, potom G-02 ingest, G-03 SSE, G-04 DB, G-05 mapa. Playwright a bezpečnostní testy běží v izolovaném DEV/CI bez skutečných notifikací a bez přístupu k V1. Každou změnu rozhodnutí je třeba podložit měřením a českým reportem.

**Neprováděno:** žádná databázová migrace, instalace Playwright, nasazení, změna firewallu, TLS či LXC. Závěry shrnuje [ADR 0006](../adr/0006-kontrakty-dat-streamu-mapy-a-lxc.md).

## Pozdější revize — ochrana CPU a SSD (ADR 0007)

**Výchozí politika byla změněna** ze starých pevných 15 s na [RAM-first deduplikaci a adaptivní zápisy](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md). Samotný SQL PK nikdy neodstranil zbytečné řádky napříč časovými buckety. Nový [fyzický PostgreSQL návrh](../../database/POSTGRES-DATOVY-MODEL.md) má 30s SQL bucket a běžný historií interval ≥ 60 s, RAM coalescing, batch zápis, žádný per-event DB access. Indexy a partitioning musí projít DEV benchmarkem. [ADR 0007](../adr/0007-ram-first-deduplikace-a-usporny-zapis.md) má v otázce diskových zápisů přednost před ADR 0006.
