# PostgreSQL V2 — fyzický model, indexy, retence a zálohy

**Stav:** závazný **návrh G-04** k implementaci a ověření v oddělené DEV databázi; **žádné SQL z tohoto dokumentu dosud nebylo spuštěno**. **Rozsah:** \`J09\`, \`S06\`, \`S09\`, související diagnostika \`O07\`. Historie V1 se nemigruje. Datum: 22. 10. 2026.

## 1. Datový model odpovídající schváleným funkcím

| Tabulka | Účel a konzument | Životní cyklus |
|---|---|---|
| \`airradar_v2.flight_samples\` | vzorkované *validní polohy* skutečných lokálně přijatých cílů; J09, případně statistické dotazy | **30 dní**, denní UTC partition |
| \`airradar_v2.receiver_daily\` | pravdivé agregované počty zpráv, letadel se zaznamenanou polohou, vzorků a zahozených vzorků; S09, O07 | **365 dní**, malá tabulka |
| \`airradar_v2.dimension_daily\` | pouze *doložené* skupiny typu letadla/operátora/trasy, mají-li skutečně zdroj; S06 | **365 dní**, pouze existující hodnoty, bez domýšlení |
| budoucí doručovací stav L06/L11/L12 | **není součást G-04**; model vznikne samostatným rozhodnutím v G-08 jen podle skutečné potřeby idempotentního doručení | případná krátká retence ve vlastní G-08 migraci |

Nevytvářet tabulky pro historický detail letadla, analytické profily, fiktivní síťové zdroje, ukládání všech raw BEAST rámců ani kopie schématu V1. Nepřidávat PostGIS, TimescaleDB, pg_partman či další rozšíření bez důkazu jejich potřeby.

**Identita** je text \`icao:abcdef\` nebo \`nonicao:abcdef\`; šestimístná hex část malými písmeny a **ne-ICAO se nikdy neslučuje s ICAO**. Neoznačovat callsign za primární identitu. \`bucket_at\` je počátek 15sekundového UTC časového okna; \`position_at\` je nejnovější *skutečně platný* čas polohy. Volbu vzorku v každém okně provádí jediný RAM owner; pozdě doručený horší nebo starší zdroj jej nesmí přepsat.

## 2. Návrh SQL schématu pro G-04 (neaplikovat bez revize)

~~~sql
-- Ukázka budoucí SQLx migrace; následující příkazy NESPOUŠTĚT na V1 ani
-- v existující databázi bez G-04 DEV schválení a backup/restore plánu.
CREATE SCHEMA IF NOT EXISTS airradar_v2;

CREATE TABLE airradar_v2.flight_samples (
  bucket_at       timestamptz NOT NULL,
  aircraft_key    varchar(14) NOT NULL
                    CHECK (aircraft_key ~ '^(icao|nonicao):[0-9a-f]{6}$'),
  position_at     timestamptz NOT NULL,
  latitude        double precision NOT NULL CHECK (latitude BETWEEN -90 AND 90),
  longitude       double precision NOT NULL CHECK (longitude BETWEEN -180 AND 180),
  altitude_ft     integer,
  ground_speed_kt real CHECK (ground_speed_kt >= 0 AND ground_speed_kt <= 1500),
  track_deg       real CHECK (track_deg >= 0 AND track_deg < 360),
  vertical_rate_fpm integer,
  squawk          varchar(4) CHECK (squawk ~ '^[0-7]{4}$'),
  on_ground       boolean,
  ingest_source   varchar(5) NOT NULL CHECK (ingest_source IN ('JSON','BEAST')),
  PRIMARY KEY (bucket_at, aircraft_key),
  CHECK (position_at <= bucket_at + interval '15 seconds')
) PARTITION BY RANGE (bucket_at);

-- Vytvářet vždy konkrétní UTC denní partition pouze migrační/údržbovou rolí.
-- Například pro demonstrační den 2026-10-22:
CREATE TABLE airradar_v2.flight_samples_20261022
  PARTITION OF airradar_v2.flight_samples
  FOR VALUES FROM ('2026-10-22 00:00:00+00')
            TO   ('2026-10-23 00:00:00+00');

-- Index pro dotazy podle letadla a intervalu; pod PK je už časový B-tree.
CREATE INDEX flight_samples_aircraft_time_idx
  ON airradar_v2.flight_samples (aircraft_key, bucket_at DESC);

CREATE TABLE airradar_v2.receiver_daily (
  day_utc             date PRIMARY KEY,
  received_messages   bigint NOT NULL DEFAULT 0 CHECK (received_messages >= 0),
  observed_aircraft   integer NOT NULL DEFAULT 0 CHECK (observed_aircraft >= 0),
  written_samples     bigint NOT NULL DEFAULT 0 CHECK (written_samples >= 0),
  dropped_samples     bigint NOT NULL DEFAULT 0 CHECK (dropped_samples >= 0),
  calculated_at       timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE airradar_v2.dimension_daily (
  day_utc         date NOT NULL,
  dimension       varchar(16) NOT NULL
                    CHECK (dimension IN ('aircraft_type','operator','route')),
  dimension_key   varchar(80) NOT NULL,
  aircraft_count  integer NOT NULL CHECK (aircraft_count >= 0),
  calculated_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (day_utc, dimension, dimension_key)
);
~~~

**Revize před migrací:** pokud se schválí záznam starých pozic po přechodu zdroje nebo jiný sampling interval, sjednotit výpočet \`bucket_at\` a \`CHECK(position_at)\`. Pozor na UTC časová razítka: \`position_at\` může být před začátkem bucketing okna u opožděného zdroje; před zápisem maximální stáří explicitně omezit. Sekvence událostí nesmí být předstírána jako SQL ID. V případném čase rolloveru používáme **UTC**, nikoli lokální DST. Návrh neobsahuje žádné DDL na sdílené role.

## 3. Zápisy a indexy

- **Vzorkování:** výchozí jeden vzorek za **15 s / validní aircraft identity**; pokud není čerstvá poloha, nevytvářet fiktivní záznam. Hodnoty mimo doménu odmítnout a metrikovat, ne „opravovat“ bez zdůvodnění.
- **Queue:** \`RAM → bounded async queue (výchozí 10 000 jednotlivých samples) → batch 250–1 000 řádků nebo nejvýše 1 s → SQLx pool\`. Dávka transakčně, \`INSERT ... ON CONFLICT (bucket_at, aircraft_key) DO NOTHING\`, nikoliv nové SQL připojení na letadlo.
- **Pool:** výchozí **max. 4 připojení** do DB LXC, connect timeout 3 s, statement timeout 5 s a krátké transakce. Při výpadku použít backoff a inkrementovat \`dropped_samples\`; nikdy nezablokovat live RAM/SSE a nikdy neukládat tajemství do logů.
- **Indexy:** základní PK \`(bucket_at, aircraft_key)\` umožňuje rozsah dne/partition, sekundární B-tree \`(aircraft_key, bucket_at DESC)\` slouží pro dotaz jednotlivé identity. **BRIN \`(bucket_at)\` nepřidávat automaticky** — při denních partitions může být redundantní. Přidat teprve pokud \`EXPLAIN (ANALYZE, BUFFERS)\` na reprezentativních datech prokáže snížení bloků/latence proti ceně zápisů.
- **Dotazy:** vždy používat omezené UTC intervaly, podporovat partition pruning; nevykonávat fullscan bez explicitního měření. Indexy pro typ/aerolinku nepřidávat, protože agregace jsou malé denní tabulky. Chybějící metadata S06 zobrazit jako nedostupná, **neodvozovat aerolinku/trasu jen z callsignu**.
- **Particie:** denní UTC, 2 dny zpět pro toleranci opožděných paketů, **7 dní dopředu připravené** maintenance jobem; žádná DEFAULT partition, která by bezděčně zadržela stará data. Neexistující aktuální partition = chyba persistence a zdravotní alarm, ne výpadek živého ingestu.
- **Retence:** denně řízeně \`DETACH PARTITION\`/po ověření \`DROP TABLE\` staré denní partition. Žádný masový \`DELETE\` celé velké tabulky. Admin příkazy provozovat odděleně s dokumentovanými zámky, transakcemi a možností rollbacku; \`DETACH ... CONCURRENTLY\` má specifická omezení. Automatický job musí nejdřív zobrazit návrh konkrétních partition, a mazat jen po otestování. Neprovádět mazání jiných schemat.
- **Idempotence agregace:** S09/S06 se počítá nad uzavřeným UTC dnem do \`receiver_daily\` a \`dimension_daily\`, opakovaná dávka používá UPSERT po dni a klíčích. \`received_messages\` používá delty skutečného counteru, se zohledněním restartu readsb; \`observed_aircraft\` musí mít jednoznačnou definici (např. unikátní cíle s *uloženou platnou polohou*), aby se nepletlo s celkovou detekcí zpráv.

## 4. Retence — startovací rozhodnutí a důsledky

| Kategorie | Výchozí návrh | Účel |
|---|---|---|
| Detailní platné polohy \`flight_samples\` | **30 dní** | úsporná historie, technické dotazy J09 |
| Denní souhrny \`receiver_daily\`, \`dimension_daily\` | **365 dní** | S06/S09 bez držení detailů |
| RAW BEAST/JSON rámce na disku | **neukládat** | cena a nepotřebná data |
| Krátká letová stopa pro mapu R08 | **20 min v RAM**, nedotazovat DB na každý SSE tick | nízká latence |
| Logy aplikace | **7–14 dní** bez citlivých paketů | diagnostika |
| Fronta budoucího doručování L11/L12 | až návrh G-08; bez implicitního retention | žádný nový produkt mimo rozsah |

30/365 dní jsou **projektová východiska**, nikoli ze zdroje či naměřeného LXC. Po dvou týdnech skutečného sběru upravit podle počtu řádků, prostoru, WAL a uživatelských dotazů. Před zkrácením retence ověřit, že se nemusí zachovat údaje kvůli jinému legitimnímu konzumentovi.

## 5. Kapacitní model (matematický příklad, NE MĚŘENÍ)

50 letadel nepřetržitě s platnou pozicí × 4 vzorky/min × 1 440 min/den = **288 000 vzorků/den**, tedy **8,64 milionu/30 dní**. Skutečný objem disku záleží na délce řádků, fillfactoru, indexech, WAL a kompresi; žádné číslo GB zde nevydáváme za naměřený výsledek. V G-04 měřit \`pg_total_relation_size\`, WAL bytes, checkpointy, latence insertů a \`EXPLAIN\`.

## 6. Zálohy — dvě nezávislé vrstvy

1. **Sdílený PostgreSQL cluster:** správce existujícího DB LXC musí potvrdit, že má funkční fyzické zálohy + případně WAL archivaci/PITR. To je ochrana celé instance, nelze bezpečně měnit jen pro V2. Zásadní: samotný \`pg_dump\` jedné databáze **není cluster PITR**.
2. **Samostatná logická záloha databáze V2:** výchozí **1× denně** do \`pg_dump -Fc\` zacíleného pouze na \`airradar_v2\`, uložit šifrovaně **mimo DB LXC** (jiné úložiště/prostředí). **14 denních + 8 týdenních** kopií, měsíční nezávislý restore test do \`airradar_v2_restore_test\` nebo do izolované testovací instance. Potvrdit, že záloha obsahuje schéma, závislosti a datové řádky. Privilegia obnovovaného vlastníka předem připravit bezpečně.
3. **Cílové RPO/RTO** navrhnout \`RPO <= 24 h\` při denním dumpu a \`RTO <= 2 h\`; jsou to cíle **k otestování**, ne garantované SLA. Pokud požadujeme \`RPO <= 15 min\`, pak je nutná skutečná průběžná WAL archivace na clusteru a odpovídající obnova.
4. Export vyzkoušet neprivilegovanou read-only rolí s potřebnými SELECT, ale \`pg_dump\` role musí mít přístup ke všem V2 objektům a sekvencím. V záloze nesmí být secrets; i letové údaje se ukládají mimo veřejný Git.
5. **Ověření obnovy**: \`pg_restore --list\` + vytvoření oddělené prázdné testovací DB + \`pg_restore --exit-on-error\` + počty řádků a relevantní API SELECT + zkušební SQLx připojení. Záloha bez obnovy není prověřená.
6. U clusterových fyzických záloh používat \`pg_verifybackup\` při podporovaném formátu, ale stále také dělat skutečné restore testy.

### Orientační příkazy bez tajemství (pouze po schválení prostředí)

~~~bash
# Nespouštět do produkce bez samostatné V2 DB a zabezpečené konfigurace.
pg_dump --format=custom --no-owner --no-acl \
  --file="$SAFE_BACKUP_DIR/airradar_v2_YYYYMMDD.dump" airradar_v2
pg_restore --list "$SAFE_BACKUP_DIR/airradar_v2_YYYYMMDD.dump"
# Obnovovat výhradně do nové prázdné, izolované testovací DB.
pg_restore --exit-on-error --no-owner --no-acl \
  --dbname=airradar_v2_restore_test "$SAFE_BACKUP_DIR/airradar_v2_YYYYMMDD.dump"
~~~

## 7. Přejímka G-04

- [ ] Ověřena přesná verze PostgreSQL na existujícím LXC, role, vlastní schéma a klientská TLS konfigurace.
- [ ] SQL DDL prošlo v nové **DEV V2** DB, partition maintenance precreate+expiry bezpečně otestovány.
- [ ] SQLx transakce a role mají jen potřebná oprávnění, žádné DDL z runtime účtu.
- [ ] Replay stejného datasetu při sampling 5/15/30 s, měřeny INSERT throughput, pool, p95/p99 a WAL.
- [ ] \`EXPLAIN (ANALYZE, BUFFERS)\` na skutečném objemu pro časový dotaz a dotaz \`aircraft_key\`, potvrzen počet potřebných indexů.
- [ ] Test výpadku DB, jeho restartu a dopadu na Rust RAM/SSE; bounded queue bez nekonečného růstu.
- [ ] Ověřený logický i existující fyzický backup a **provedený test restore**, RPO/RTO skutečně změřeny.
- [ ] Po schválení retence aktualizovat tuto specifikaci, SQLx migraci, český report, stav etap a changelog.

## Podklady

- [PostgreSQL 18 — Partitioning](https://www.postgresql.org/docs/18/ddl-partitioning.html): požadavek na partition key v PK, pruning a údržba.
- [PostgreSQL 18 — BRIN](https://www.postgresql.org/docs/18/brin.html): vhodnost korelovaných hodnot a indexování.
- [PostgreSQL — pg_dump](https://www.postgresql.org/docs/18/app-pgdump.html), [WAL/PITR](https://www.postgresql.org/docs/18/continuous-archiving.html), [pg_verifybackup](https://www.postgresql.org/docs/18/app-pgverifybackup.html).

**Všechny SQL bloky jsou výlučně návrh.** Žádná DB není při této rešerši vytvořena nebo změněna.
