# RAM-first AirRadar V2 — deduplikace, cache a minimální počet zápisů

**Datum:** 22. 10. 2026. **Stav:** revidované a schválené výchozí návrhové pravidlo pro G-01/G-02/G-04/G-09, dosud **bez implementace a měření v LXC**. Cíl: co nejméně CPU, SQL transakcí, WAL, zápisů na disk a zatížení existující sdílené PostgreSQL instance. Žádný zásah do V1. **KEEP:** J01/J02/J03/J04/J05/J09/J11, S06/S09, O07/O08.

## Odpověď na otázku deduplikace

Původní SQL PK \`(bucket_at, aircraft_key)\` **zabraňoval pouze dvěma řádkům téže identity v jednom 15s bucketu**. Nezabraňoval dalšímu řádku ve *vedlejším bucketu*, pokud readsb opakoval nezměněnou polohu. Samotný \`INSERT ON CONFLICT DO NOTHING\` navíc stále používá CPU, síť a část práce PostgreSQL; nesmí být hlavní deduplikační algoritmus.

**Nové rozhodnutí:** primární deduplikace **před databází v Rust RAM**, SQL PK zůstává až poslední idempotentní ochranou pro reconnect, retry a restart.

## 1. Jediný stav v RAM; PostgreSQL není živý cache backend

~~~text
lokální readsb JSON / BEAST
          │ validace rámců, časů, CPR
          ▼
    normalizované pozorování
          ▼
    Rust aircraft HashMap (bounded)
      ├─ aktuální pravdivý stav → REST/SSE 1 Hz → MapLibre
      ├─ RAM trail (posledních 20 min, max. 64 bodů / cíl)
      ├─ RAM statistik/čítače → web / 5min agregace
      └─ sampler + per-aircraft write watermark
                    │ jen NOVÉ informačně hodnotné body
                    ▼
         coalescing RAM map, bounded
                    │ batch 128 řádků nebo 5 s
                    ▼
          SQLx → PostgreSQL V2
                    │ append-only, idempotentní SQL
                    ▼
          30denní historie, 365denní souhrny
~~~

**RAM obsahuje:** pro každé letadlo poslední zprávu, poslední *validní* polohu a její čas, callsign/squawk, souřadnici, letovou stopu, poslední zápis do SQL potvrzený COMMITem, poslední vzorek pending a TTL. Zprávy readsb a BEAST **nesmějí vytvářet nezávislé kopie**. HashMap má startovací strop 5 000 identit, TTL poslední zprávy 60 s a omezené stopy; jde o konfigurační limity do měření.

Web a SSE čtou RAM bez SQL \`SELECT\` pro každý update, výběr či pan mapy. Krátké diagnostické statistiky vycházejí z RAM. SQL čte jen **skutečnou historickou otázku**, s časovým omezením; případnou pozdější query-cache s TTL 30–60 s a malým LRU zřídit jen po prokázaném opakovaném dotazu. Neustále nezrcadlit celou PostgreSQL databázi do RAM.

## 2. Třívrstvá deduplikace — s různými typy duplicit

1. **Ingest:** stejné BEAST frame může přijít opakovaně (krátké okno fingerprintu + přijímač/timestamp, viz kontrakt BEAST); opakované JSON snapshoty nejsou nové pozorování, pokud \`seen_pos\` ukazuje stále *tentýž okamžik polohy*.
2. **Kanonická pole:** slučovat per-identity/per-field s časem a validitou. \`icao:abcdef\` a \`nonicao:abcdef\` jsou různé identity. Nová zpráva bez polohy **neaktualizuje poslední čas polohy**. Stejný callsign ani identická GPS souřadnice neprokazuje novou polohu. Novější zdroj nesmí být přepsán opožděným BEAST/JSON.
3. **Persistence RAM watermark:** pro uložení porovnat jen novou validní polohu se **skutečně potvrzeným posledním SQL vzorkem a s novějším čekajícím vzorkem**. Pokud je pozorování identické nebo nedochází k významné změně, neenqueue ani nic neposílat na DB. SQL PK + \`ON CONFLICT DO NOTHING\` kryjí opakované pokusy, nikoli časová okna obecně.

Pozor: stejná GPS souřadnice může být platným novým pozorováním stojícího letadla, takže celou identitu ani dostupnost z RAM **nesmíme vyřadit jen podle stejných souřadnic**. Deduplikace živé existence, časové identity a historického vzorku jsou odlišné operace.

## 3. Adaptivní historický sampling — výchozí ECONOMY režim

Všechna časová pravidla se řídí **validním \`position_at\`**, ne pouhým \`receivedAt\`, teoretickým BEAST MLAT čítačem ani SSE tickem. Sampling je *max. frekvence přijetí*, ne pravidelný timer generující umělé pozice.

| Situace | Politika historie | Proč |
|---|---|---|
| První platná poloha nového tracku | zapsat **jednou**, po konci aktuálního 30s UTC bucketu | počáteční bod bez potřeby druhé existence |
| Normální pohybující se letadlo | **nejdříve po 60 s** a jen při změně **≥ 2 km / ≥ 500 ft výšky / ≥ 15° směru**, případně po **180 s** s novou validní polohou | přibližně ≥ 75 % méně bodů než 15 s při trvale čerstvé poloze |
| Významný obrat nebo změna fáze letu | nejdříve **30 s** od minulého zápisu, pokud změna směru **≥ 30°**, výšky **≥ 1 500 ft**, nebo skutečný přechod země/vzduch | zachovat použitelné trajektorie bez 1Hz historie |
| Stojící letadlo | změna polohy **≥ 100 m**, pak nejdříve po **120 s**; jinak nejvýš **jedna čerstvá poloha za 600 s** | nevytvářet desítky téměř stejných záznamů |
| Nezměněná nebo chybějící stará poloha | **0 zápisů** | zastaralá poloha není nový důkaz |
| Změna squawk/volání bez platné nové polohy | aktualizovat pouze RAM a případný bezpečný notifikační event, **nepsat falešný GPS vzorek** | bezpečnost a úspora diskového I/O |
| Ztráta polohy / konec tracku | skončit v RAM a SSE; **nevytvářet nulovou souřadnici ani SQL heartbeat** | žádné syntetické události |

**Důležité:** hodnoty 60s/180s/600s a 2km/500ft/15° jsou **konfigurační návrh k testu**, ne zjištěná ideální frekvence. Záznam po 180 s/600 s se provádí pouze pokud existuje **nová opravdu validní poloha**; neukládat kopii původního bodu jen proto, že uběhl čas. V historii může být podrobnost záměrně nižší než na živé mapě. Pokud akceptační test R08/J09 potřebuje detailnější časový interval, přepnout pouze sampling profil; žádné druhé DB schéma.

**UTC bucket:** 30 s, \`bucket_at = floor(position_at / 30 s)\`. Do uzavřeného bucketu smí jít **nejvýše jedna** pozice \`(bucket_at,aircraft_key)\`. Pokud do téhož bucketu dorazí lepší/novější validní poloha, přepsat ji **jen v pending RAM map**, ne SQL \`UPDATE\`. Před odesláním vzorek zafixovat; po úspěšném commit aktualizovat \`lastPersisted\`, jinak jej ponechat jako nepotvrzený.

## 4. DB writer a odpojení PostgreSQL

- **Jedna Tokio persistence task**, nemá vlastní kopii živého leteckého state; z kanonického actoru přebírá pouze krátké vzorky. Žádné 1 INSERT na zprávu a žádný per-aircraft DB connection.
- Pending coalescing \`HashMap<(aircraft_key,bucket_at),sample>\`, výchozí **max. 2 048 položek**, max. stáří **30 s**, limit celkové RAM v reálném kódu; žádná neomezená FIFO 10 000 historických vzorků.
- Dávka: **128 řádků nebo nejvýše 5 sekund** pro ne-prázdnou frontu. Prázdná dávka se **neodesílá**. Flush při korektním shutdownu (bounded deadline) a UTC rolloveru. SQLx pool zpravidla **1 zapisující connection**, maximum 2 včetně read klienta; zvýšit jen po měření.
- SQL: vícerow \`INSERT ... ON CONFLICT (bucket_at,aircraft_key) DO NOTHING\` v jedné transakci. \`UPDATE\` historie a per-row \`SELECT\` jsou výjimky vyžadující zdůvodnění. \`COPY\` porovnat až při skutečně velkých dávkách, protože idempotentní \`ON CONFLICT\` nemá stejnou jednoduchou \`COPY\` ekvivalenci.
- PostgreSQL transakce při chybě: krátký exponential backoff s jitterem, **žádný nekonečný retry backlog**. Vzorky starší než 30 s ve frontě jsou explicitně zahazovány (čítadlo \`samples_dropped\` a poslední příčina); nový RAM stav/SSE je stále živý. Nikdy nezačít po obnovení DB přehrávat hodiny starých poloh a způsobit write storm.
- Periodické agregace statistik \`receiver_daily\` dělat v RAM a nanejvýš **1 UPSERT / 5 min nebo při přechodu UTC dne**. Hodnoty \`received_messages\` musí zachytit restart readsb, ztráty při crashi zohlednit a viditelně uvádět přesnost. \`dimension_daily\` vytvářet teprve při implementaci S06 s důvěryhodnými zdrojovými údaji; **nepořizovat tabulku pro prázdná metadata**.
- Není výchozí provozní potřeba Redis, Kafka, externí cache procesu, druhé DB nebo zvláštního „event store“.

## 5. Indexy, partitioning a WAL — minimalizovat write amplification

- DB dedup: jediný **PK \`(bucket_at,aircraft_key)\`**. Sekundární B-tree \`(aircraft_key,bucket_at DESC)\` **až když reálné dotazy historie vyžadují lepší index a EXPLAIN prokáže přínos**. BRIN nepřidávat automaticky. Každý index znamená další zápisy a zabrané místo.
- **Partitioning není automatický výkonový bonus**: porovnat bez partitions a s denní UTC partition na stejných datech. Při nízkém objemu může režie partition převýšit úspory; při velkých objemech umožňuje levnější smazání celého dne bez masivního \`DELETE\` a následného VACUUM.
- **Append-only** živých poloh. Vyhnout se dotazům „najdi poslední řádek“ při každém příjmu: poslední potvrzenou polohu držet v Rust RAM; po restartu může existovat starší DB záznam, idempotentní buckety ochrání replay.
- **Na sdíleném PostgreSQL LXC neměnit** \`fsync=off\`, \`full_page_writes=off\`, \`wal_level\`, checkpoint interval, globální \`synchronous_commit\` ani \`shared_buffers\` bez samostatného DBA schválení a měření. Nevytvářet persistentní historie jako \`UNLOGGED\`: nebyly by crash-safe.
- \`wal_compression\` může ušetřit WAL full-page images výměnou za CPU; u sdíleného serveru pouze zvážit po dohodě se správcem a srovnání reálných WAL/CPU. **Větší redukci je třeba získat eliminací INSERT ještě v RAM**, ne vypnutím ochrany dat.
- \`synchronous_commit=off\` lze pouze pro explicitně ztrátově tolerované *historické* transakce po změřeném přínosu a schválení RPO; nejde o základní nastavení, protože změna durability má cenu. Nikdy \`fsync=off\` na sdíleném serveru.

## 6. Cache strategie pro CPU a mapu

- readsb JSON parser přebírá jen změny (per-identity/per-field) a nepřepočítává celé trajektorie pokaždé, když se změní jedno pole. Sledovat časové razítko pozice. Hot RAM state se uklízí TTL.
- Drahé statické katalogy (letiště/navaid) jsou read-only data načtená jednou s explicitní verzí a bounded cache. Počasí TTL cache s ohraničenými obnovami a správně zobrazovaným stářím.
- SSE delta se serializuje **jednou pro všechny klienty** (např. sdílené \`Arc<Bytes>\`), webový seznam i mapa jej použijí bez SQL round-trip.
- MapLibre aktualizovat maximálně 1Hz; žádné přeposílání každé BEAST zprávy jako GeoJSON.
- Všechny cache mají **maximální počet položek a TTL**, při plné paměti předvídatelná evikce a metriky; vypnutí DB nezpůsobí OOM.

## 7. Očekávaná matematická úspora (NENÍ reálný benchmark)

Při hypotetických **50** nepřetržitě přítomných letadlech, z nichž každé poskytuje čerstvou platnou polohu:

| Pravidlo | Nejvýše řádků / den pro takto trvale obnovované letadlo | Všech 50 / den | 30 dní | Snížení řádků proti 15s intervalu |
|---|---:|---:|---:|---:|
| 15 s bez adaptace | 5 760 | 288 000 | 8 640 000 | 0 % |
| 30 s minimální interval | 2 880 | 144 000 | 4 320 000 | až 50 % |
| **60 s adaptivní běžné letadlo** | **1 440** | **72 000** | **2 160 000** | **až 75 %** |
| Stojící letadlo, 10min kontrolní max. | 144 | 7 200 | 216 000 | až 97,5 % |

Tabulka je pouze **model maximální frekvence** ve zjednodušeném scénáři bez zrychlených manévrů a eventových výjimek. Pravidlo „zapsat pouze významně změněnou pozici“ může počty ještě snížit; výjimky pro obraty/start/přistání mohou některé zápisy navýšit. **Pokles SQL řádků není totéž jako stejný procentní pokles fyzických SSD zápisů** kvůli WAL, indexům, checkpointům a zálohám.

## 8. Měřicí brána G-04/G-11

Porovnat **baseline 15 s** a adaptivní profily **30 s/60 s** nad stejnou anonymizovanou **golden replay sekvencí** na izolované V2 DEV DB. Měřit:
- \`observations_received\`, \`valid_positions\`, \`duplicate_frames_ignored\`, \`unchanged_positions_skipped\`, \`samples_eligible\`, \`samples_enqueued\`, \`samples_written\`, \`samples_conflicted\`, \`samples_dropped\`, \`batch_commits\`;
- CPU/RSS Rust enginu; avg/p95/p99 DB transakce, SQLx pool queue; PostgreSQL \`pg_stat_user_tables\` a \`pg_total_relation_size\`;
- WAL \`pg_stat_wal.wal_bytes\` a disk I/O **pouze v izolovaném kontrolovaném měření** — na sdíleném LXC tato metrika zahrnuje i ostatní databáze; normalizovat pouze proti známé zátěži a nevyvozovat přímé V2 IOPS;
- \`EXPLAIN (ANALYZE, BUFFERS)\` pro historii podle času a letadla před/po přidání indexu; \`DELETE\` versus partition DROP/DETACH v oddělené testovací DB;
- sémantickou kvalitu historické stopy (chyba vzdálenosti/časových událostí), CPU přínos versus ztráta detailu;
- restart DB, výpadek sítě, backlog limit, obnova a nulový dopad na 1Hz RAM→SSE.

**Akceptační zásada:** žádné body v SQL bez nové skutečné validní polohy; žádné neomezené fronty, nulové duplicate-key SQL řádky a prokazatelný objem skutečně ušetřených INSERTů. Cílové procento úspory vyhodnotit až po replay; neslibovat fiktivní 75 % při reálném provozu.

## Oficiální PostgreSQL podklady

- [Partitioning a náklady/retence](https://www.postgresql.org/docs/current/ddl-partitioning.html)
- [Populating database, batch/COPY](https://www.postgresql.org/docs/current/populate.html)
- [WAL a full_page_writes](https://www.postgresql.org/docs/current/runtime-config-wal.html)
- [Asynchronous commit — durabilita](https://www.postgresql.org/docs/current/wal-async-commit.html)
- [Statistiky PostgreSQL](https://www.postgresql.org/docs/current/monitoring-stats.html)
