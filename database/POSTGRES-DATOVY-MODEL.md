# PostgreSQL V2 — deduplikovaný úsporný datový model

**Revize:** 22. 10. 2026 — RAM-first optimalizace; **nahrazuje původní pevné 15s vzorkování**.  
**Stav:** schválený model G-04; DEV schema/tabulka byly auditně ověřeny a izolovaný benchmark/restore proběhl, nikoli však proti sdílenému LXC.
**Primární KEEP:** J09, podpůrně S06/S09 a O07; lokální Rust RAM zůstává živou autoritou J05/J11.

**Nejdříve přečíst:** [RAM cache, deduplikace a podmínky zápisu](../docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md). Všechny níže uvedené časové a kapacitní limity se mohou upravit po reálném měření; bez dalších produktových funkcí.

## 1. Co se skutečně ukládá

| Fyzický objekt | Data a přímý konzument | Výchozí retence |
|---|---|---|
| \`airradar_v2.flight_samples\` | *nové a validní významné* vzorky poloh J09, po deduplikaci v RAM | **30 dní**, nejvýše 1 záznam na aircraft a 30s UTC bucket |
| \`airradar_v2.receiver_daily\` | pravdivé denní počty uložených a ztracených vzorků, reálné snapshot počítače readsb, S09/O07 | **365 dní** |
| \`airradar_v2.dimension_daily\` | **nezakládat v G-04**, dokud G-09 neprokáže důvěryhodný zdroj type/operator/route a reálný konzument S06 | případně **365 dní** |

Žádné tabulky pro raw BEAST rámce, plný JSON, duplicity, cache, SSE, seznam aktivních letadel či každé stisknutí mapy. PostgreSQL slouží **pouze k dlouhodobé historii**; lokální RAM store a MapLibre ji nevolají při každé aktualizaci.

## 2. Schéma V2 — ilustrativní SQLx migrace, NEPROVÁDĚT

~~~sql
-- Ukázka pro novou oddělenou DEV DB airradar_v2_dev.
-- Nikdy nepouštět bez G-04 schválení proti sdílené produkci.
CREATE SCHEMA IF NOT EXISTS airradar_v2;

CREATE TABLE airradar_v2.flight_samples (
  bucket_at          timestamptz NOT NULL,
  aircraft_key       varchar(14) NOT NULL
                       CHECK (aircraft_key ~ '^(icao|nonicao):[0-9a-f]{6}$'),
  position_at        timestamptz NOT NULL,
  latitude           double precision NOT NULL
                       CHECK (latitude BETWEEN -90 AND 90),
  longitude          double precision NOT NULL
                       CHECK (longitude BETWEEN -180 AND 180),
  altitude_ft        integer,
  ground_speed_kt    real CHECK (ground_speed_kt BETWEEN 0 AND 1500),
  track_deg          real CHECK (track_deg >= 0 AND track_deg < 360),
  squawk             varchar(4) CHECK (squawk ~ '^[0-7]{4}$'),
  on_ground          boolean,
  PRIMARY KEY (bucket_at, aircraft_key),
  CHECK (
    position_at >= bucket_at
    AND position_at < bucket_at + interval '30 seconds'
  )
) PARTITION BY RANGE (bucket_at);

-- Jen pokud měřená data opravňují partitions:
-- budoucí údržbová/migrační role připraví konkrétní denní UTC partition.
CREATE TABLE airradar_v2.flight_samples_20261022
  PARTITION OF airradar_v2.flight_samples
  FOR VALUES FROM ('2026-10-22 00:00:00+00')
            TO   ('2026-10-23 00:00:00+00');

-- ŽÁDNÝ další index v první verzi!
-- Konkrétním EXPLAIN ověřit, zda je později třeba:
-- CREATE INDEX ... ON ... (aircraft_key, bucket_at DESC);

CREATE TABLE airradar_v2.receiver_daily (
  day_utc             date PRIMARY KEY,
  received_messages   bigint NOT NULL DEFAULT 0
                        CHECK (received_messages >= 0),
  observed_aircraft   integer NOT NULL DEFAULT 0
                        CHECK (observed_aircraft >= 0),
  written_samples     bigint NOT NULL DEFAULT 0
                        CHECK (written_samples >= 0),
  dropped_samples     bigint NOT NULL DEFAULT 0
                        CHECK (dropped_samples >= 0),
  calculated_at       timestamptz NOT NULL DEFAULT now()
);
~~~

**Integrita:** \`bucket_at\` je \`position_at\` zaokrouhlené dolů na 30s UTC (např. v aplikaci \`date_bin('30 seconds', position_at, epoch)\`). \`nonicao:\` a \`icao:\` jsou oddělené. \`PRIMARY KEY(bucket_at, aircraft_key)\` zabraňuje druhému řádku v **tomtéž bucketu**, nikoli ve všech sousedních intervalech. **Hlavní deduplikační logika běží v RAM ještě před SQL**. \`ON CONFLICT DO NOTHING\` je až poslední bezpečnostní vrstva proti replays/retry.

Čas polohy musí být skutečně doložený: u readsb \`now - seen_pos\` po validaci, u BEAST jen pokud je bezpečně rekonstruovaný; jeho MLAT timestamp není automaticky UTC. Pokud lze spolehlivě určit jen received UTC a chybí validní čas polohy, takový bod nevkládat jako údajně historickou platnou polohu bez zvláštního explicitního kontraktu.

**Záměrná změna:** původní dokument definoval třetí agregační tabulku ještě před potřebou, dva indexy od začátku a fixní 15s INSERT. Tento návrh je přísnější vůči diskové zátěži.

## 3. RAM watermark, change-driven sampling a bezplatná mapa

- **Nejprve** normalizace J01/J02/J03/J04/J11; pokud stejná/opožděná poloha z JSON i BEAST, žádný zápis ani nové přepsání \`lastPositionAt\`.
- **V RAM** stav každého letadla nese \`lastPersisted\` a \`pendingSample\`. Jedna platná první poloha, poté:
  - běžný pohyb: nové skutečné pozorování **nejdříve po 60 s** a teprve při změně 2 km/500 ft/15°, případně po 180 s nového validního měření;
  - výjimečný velký obrat 30°/stoupání 1500 ft/přechod air/ground: **nejdříve po 30 s**;
  - stojící: 120 s + pohyb 100 m, jinak čerstvý 10min kontrolní bod;
  - bez nové platné polohy: **žádný další SQL řádek**.
- \`30s UTC bucket\` je pouze idempotentní hranice uloženého bodu, **nikoli povinný zápis každých 30s**.
- Všechny detaily a nuance potvrzení SQL transakcí jsou v [policy](../docs/technical/RAM-CACHE-A-DEDUP-ZAPISU.md). Výše uvedené prahy jsou **startovací hypotézy** pro replay 15/30/60 s, ne garantovaná konfigurace.
- Stav letadel a krátké mapové stopy (20 min, max 64 bodů na cíl) zůstává v Rust RAM. SSE 1Hz se nesmí dotazovat SQL.

## 4. SQLx writer: co nejméně transakcí a indexových zápisů

- Jeden hlavní writer nad **bounded coalescing pending map** max 2 048 položek, max 30 s stáří. **Nevzniká** 10 000 položková FIFO pro všechny zprávy.
- **Batch insert** maximálně 128 vzorků nebo flush po 5 sekundách neprázdného pending bufferu. Po prázdném ticku neodesílat SQL. Krátká transakce, připravené parametry, \`ON CONFLICT DO NOTHING\`. Pool start max **2** (writer + případné historické čtení).
- Výpadek DB: omezené retry, stálý RAM live stav a SSE, nadlimitní/stará pending položka se zahodí se statistikou. Žádný catch-up write storm po obnově. Při crashi lze ztratit nepotvrzený vzorek; to je vědomý kompromis \`write-back RAM cache\`, ne stoprocentní durable queue.
- Denní přijaté message counters a souhrny počítat **v RAM**, checkpoint nanejvýš **jednou za 5 min** nebo po UTC rolloveru, nikdy na každý přijatý rámec. Restart readsb i Rust může způsobit rozdíl proti přesnému počítadlu; explicitně dokumentovat přesnost. Počty zapsaných sample lze v DEV ověřit jednorázovým SQL agregátem.
- Nevytvářet \`UPDATE\` historických pozic. Po zavření RAM bucketu vložit konečný vybraný vzorek. Kompaktní \`receiver_daily\` UPSERT je jediné plánované pravidelné přepsání jednoho řádku.

## 5. Indexy a retence podle skutečné velikosti

1. Nejdříve **porovnat dvě fyzické varianty** na DEV: **jedna běžná tabulka** s pravidelnou retenční údržbou vs **denní UTC partitions**, vždy stejný SQLx reader/writer a stejný replay. Nad malou tabulkou může být partition overhead zbytečný; nad miliony řádků umožní \`DROP/DETACH PARTITION\` odstranit celé dny bez masových DELETE/VACUUM. PostgreSQL samo uvádí, že partitioning bývá užitečný až pro dostatečně velké tabulky.
2. PK je v obou variantách nutný pro poslední dedup. Dodatečný B-tree na \`aircraft_key, bucket_at DESC\` vytvořit **jen** když J09/S09 skutečně hledají detailní trasu letadla a \`EXPLAIN (ANALYZE, BUFFERS)\` prokáže výhodu.
3. BRIN je kandidát jen při velmi velké append-only chronologicky uložené tabulce; nevytvářet jej automaticky vedle denních partitions.
4. Retence detailu výchozí 30 dní, souhrny 365 dní; u partitions pravidelný **DROP/DETACH nejstarších celých dní**, u jedné tabulky jen ohraničený batch delete s dopadem na WAL/VACUUM. Retention job spouštět mimo Rust live hot path a izolovat práva.
5. Žádný \`UNLOGGED\` pro persistentní J09, \`fsync=off\`, \`full_page_writes=off\` ani globální kompromis durability na sdíleném DB LXC.
6. \`wal_compression\` lze zkoumat **jen se souhlasem správce**: omezuje velikost WAL full-page images výměnou za CPU a může zasáhnout ostatní databáze. Prioritou je **nedělat zbytečné INSERTy**.

## 6. Model předpokládané redukce (NE měření)

Hypotetických 50 letadel, neustále čerstvé polohy, žádné výjimečné velké manévry:

| Maximální běžná frekvence | Řádky za den | Za 30 dní | Proti 15 s |
|---|---:|---:|---:|
| původní fixní 15 s | 288 000 | 8 640 000 | 0 % |
| 30 s | 144 000 | 4 320 000 | až −50 % |
| **adaptivně nejdříve 60 s** | **72 000** | **2 160 000** | **až −75 %** |

To je **matematika počtu vzorků**, ne měřené zmenšení WAL, skutečných diskových zápisů ani CPU. Deduplikace stejné polohy může ušetřit více, záznamy prudkých manévrů naopak vedou k vyšší četnosti. Počet vzorků nesmí být optimalizován na úkor použitelnosti J09/R08/S09; potřebujeme skutečný replay a profilování.

## 7. Postradatelná historie a přijatá ztráta dat

Aktuální [ADR 0010](../docs/adr/0010-g04-postradatelna-historie-bez-zaloh.md) nahrazuje původní povinné zálohování V2. Historie letadel je postradatelná; její celá dosavadní část může být po havárii nenávratně ztracena. Nevyžadují se denní `pg_dump`, externí kopie, PITR ani provozní RPO/RTO. Izolovaný `pg_dump/pg_restore` test G-04.11 je již dokončen a zachovává se jako technický důkaz.

Při ztrátě V2 databáze se může jen po schválení DBA znovu vytvořit schéma ze schválených migrací a nová historie se začne sbírat od nuly. Nedotýká se to zálohování PP Studio, V1 a jiných databází. Nadále hlídat disk/WAL/CPU/RAM společného LXC, retenci a bounded SQL writer; globální nastavení serveru neměnit.

## 8. Podmínky přejímky a povinná měření G-04

- [ ] Stejný golden replay v baseline 15s, 30s a novém adaptivním 60s režimu; doložit přesné skutečné INSERTy, unikátní identity a trasovou přesnost.
- [ ] V RAM test duplicit JSON/BEAST, stejných/opožděných pozic, stale position time, přechodu air/ground a restartu readsb.
- [ ] Nulové SQL INSERTy při nezměněných/nenových pozicích, nulové prázdné transakce a SQL \`UPDATE\` na polohových řádcích.
- [ ] Buffer omezený počtem položek i stářím, žádné memory leak nebo catch-up write storm při výpadku SQL.
- [ ] Správnost PK při restartu/retry a \`ON CONFLICT DO NOTHING\`, nedvojí započtení potvrzeného sample.
- [ ] Porovnání nepartitionované vs denně partitionované tabulky a PK-only vs s přidaným indexem.
- [ ] CPU/RSS Rust, SQL transakce/s, \`pg_stat_user_tables\`, \`pg_total_relation_size\`, \`EXPLAIN/BUFFERS\`, WAL a I/O (jen izolovaně; \`pg_stat_wal\` agreguje *celý cluster*).
- [ ] Izolovaný pg_dump/restore test je doložen, provozní zálohování historie V2 není požadováno (ADR 0010); povinné jsou práva, volná kapacita a nulový dopad SQL výpadku na RAM/SSE.

Žádné označení \`ACCEPTED\` G-04/J09 nevzniká schválením tohoto dokumentu.

## Zdroje

- [PostgreSQL partitioning a jeho náklady](https://www.postgresql.org/docs/current/ddl-partitioning.html).
- [PostgreSQL WAL a bezpečná durabilita](https://www.postgresql.org/docs/current/runtime-config-wal.html).
- [PostgreSQL COPY vs INSERT](https://www.postgresql.org/docs/current/populate.html).
- [PostgreSQL monitoring](https://www.postgresql.org/docs/current/monitoring-stats.html).
- [PostgreSQL backup a recovery](https://www.postgresql.org/docs/current/backup.html).

**Žádný příkaz z této specifikace nebyl spuštěn proti existující databázi ani LXC.**
