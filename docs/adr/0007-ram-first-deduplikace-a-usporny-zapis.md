# ADR 0007 — RAM-first deduplikace a adaptivní úsporný zápis

**Datum:** 22. 10. 2026. **Stav:** rozhodnutí pro implementaci J05/J09/J11, bez aktuálního reálného měření a bez spuštěné SQLx migrace. Nahrazuje pevný 15s zápis a napevno přidané sekundární indexy v původním návrhu G-04.

## Problém

Původní PK \`(bucket_at,aircraft_key)\` zajistil SQL unikátnost **jen v jednom časovém okně**, ale nevyloučil každý další 15s záznam nezměněné polohy. Fallback JSON/BEAST mohl navíc tvořit nadbytečné normalizované eventy, aniž by SQL unikátnost věděla, že se nezměnila skutečná poloha. Pouhý \`ON CONFLICT\` proto nešetří síť/CPU, WAL ani indexové I/O dostatečně.

## Rozhodnutí

- Jediný živý \`AircraftState\` v Rust RAM je zdrojem REST/SSE a záznamového watermarku; do DB se posílají **jen platné smysluplně nové polohy**, s nezávislým čerstvým \`position_at\`.
- Výchozí \`economy\` historické vzorkování: pro standardní letadlo **nejdříve 60 s** a při významné změně (2 km/500 ft/15°), nejpozději po 180 s jen pokud dorazila nová skutečná validní poloha; velký obrat/air-ground nejdříve 30 s; na zemi 120s při >100 m nebo kontrolní nový bod po 600 s. Vše musí projít replay a použitelností historie.
- Jednotný bounded coalescing RAM queue max 2 048 položek / 30 s, writer batch 128 řádků nebo 5 s, žádné prázdné SQL transakce a žádný per-message INSERT.
- PK \`(bucket_at,aircraft_key)\` nad 30s UTC bucketem jako **poslední idempotentní ochrana**; append-only, druhý B-tree a partitioning teprve podle měření, BRIN ne automaticky.
- SQLx max 2 pool connection na začátku; bezpečně nízká DB zátěž. Souhrny v RAM s maximálně 1 UPSERT každých 5 min. Výpadek SQL nesmí ovlivnit ingest ani SSE.
- Mapová stopa 20 min / max 64 bodů na cíl v RAM a SSE data z RAM. Žádné opakované DB čtení pro živou mapu, redis/broker či jiné služby.
- Na sdíleném PostgreSQL LXC nevypínat \`fsync\`, \`full_page_writes\`, měnit globální WAL ani vytvářet \`UNLOGGED\` historii. Durabilita se nenahrazuje RAM cache; počítat s vědomou ztrátou ještě nepotvrzených vzorků při crashi.

## Důkazy a zodpovědnost

[Úplná policy, RAM struktury, SQL writer a měřicí kritéria](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md) a [revidovaný datový model](../../database/POSTGRES-DATOVY-MODEL.md). Jde o **návrh**, nikoli dokončený G-04 nebo zjištěných −75 % fyzických zápisů. Návrh se skutečně přijme až po replay, EXPLAIN, měření SQL/WAL/SSD a schválené izolované DEV migraci.
