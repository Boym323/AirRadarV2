# Architektonický audit AirRadar V2 — připravenost před implementací

**Datum:** 22. 10. 2026  
**Auditovaný commit:** `f97ba6e05d92903a7e840c2dc2258aa2a9aab0e2` (`main`, před tímto auditním commitem)  
**Metoda:** kontrola skutečného stromu, aktivní matice 45 funkcí, 9 modulů, 140 kroků G-01 až G-13, současných kontraktů, ADR, zdrojového kódu, OpenAPI a posledního CI. **Bez nových zátěžových testů, DB připojení a nasazení.**  
**Výsledek:** architektonický stack schválen, **implementace G-01 může začít**, ale celý návrh nelze označit za plně vyjasněný ani produkčně připravený. Prioritní otevřené body jsou uvedeny níže.

## 1. Co je skutečně ověřeno

- Rust + Tokio + Axum jako **jeden** RAM state owner, readsb JSON/BEAST lokální ingest, Vite + React + TypeScript + MapLibre, REST/SSE a SQLx/PostgreSQL na existujícím samostatném DB LXC. Není rozhodnuto měnit stack.
- Rozhodnuto: **žádný UI přepínač zdrojů**, žádná další služba Redis/Kafka/Node runtime nebo další produkční DB. Externí feedy a více vlastních RF přijímačů **nejsou schválenými aktivními funkcemi**; případné rozšíření je pouze v jediném archivu budoucích návrhů.
- Deduplikace v RAM, *adaptive economy* vzorkování, bounded SQL batch, retence a obecná bezpečnostní pravidla jsou **podrobně navržené**, ale nikoli běžící a naměřené.
- Poslední ověřená CI základna: [39738710700](https://github.com/Boym323/AirRadarV2/actions/runs/39738710700), 3 úspěšné joby na auditovaném head. CI testuje skutečný G-00 skeleton, nikoli žádnou z 45 funkcí.
- `apps/engine/src/main.rs` má stále jen bootstrap HTTP; `contracts/openapi.yaml` je **výslovně G-00 stub**. Skutečné G-01 DTO a protokol se teprve vytvoří.
- V repozitáři nejsou schválené SQLx migrační soubory, Playwright E2E, skutečný ingest, LXC provisioning, DB credentials ani potvrzená produkční verze PostgreSQL.

## 2. Otevřené body — blokace přijetí G-01

### P0-01 — Semantika času polohy a původ validního pozorování

**Doklad:** [readsb/BEAST kontrakty](../technical/READSB-BEAST-KONTRAKTY.md), [datový model](../../database/POSTGRES-DATOVY-MODEL.md), [ADR 0008](../adr/0008-identita-puvodu-lokalniho-prijmu.md).

readsb JSON poskytuje `now` a `seen_pos`; BEAST 12MHz clock není univerzální UTC. Současný DB návrh odmítá uložení přímé BEAST polohy bez bezpečně odvozeného UTC position time. **Není rozhodnuto, co přesně se děje při dlouhém BEAST-only provozu bez porovnatelného readsb JSON/receiver UTC**. To může vést k nulové historii, přestože živý radar funguje. readsb také nabízí vlastní informace o typu pozorování, například ADS-B, MLAT nebo TIS-B — **lokální readsb snapshot neznamená automaticky čistě vlastní RF pozici**.

**Návrh uzavření:** v G-01 formalizovat `receivedAt`, `positionAt`, `timeBasis` (`receiver_utc`, `derived_readsb`, `server_arrival`, `unknown`), `positionOrigin` (`adsb`, `mlat`, `tisb`, `estimated`, `unknown`) a přesná pravidla, co smí do RAM, SSE a SQL. Konzervativně **nesměšovat nekalibrované receiver ticks s UTC** a neukládat odhadnutou pozici jako ADS-B. Volba zda výjimečně ukládat server-arrival čas s explicitně nižší přesností musí být jednoznačně uvedena před G-04, jinak hrozí nesplnitelný J09 kontrakt.

**Brána:** golden JSON/BEAST fixture včetně absence UTC mapování a chybných/prošlých pozic; scénář BEAST-only má deterministický výsledek.

### P0-02 — Slučování zdrojů při konfliktních identitách

**Doklad:** [ADR 0008](../adr/0008-identita-puvodu-lokalniho-prijmu.md), [RAM-first](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md).

`receiverId` versus `transport` už je správně oddělené a současná jediná stanice nepotřebuje nový modul. Dosavadní `nonicao:<hex>` ale **není obecná globálně jednoznačná identita** napříč budoucími providery nebo vzdálenými přijímači; také ICAO adresu lze při vadném dekódování nebo spoofingu kolizně pozorovat. Pravidlo „stejný hex = jedno letadlo“ bez geografické/časové plausibility je příliš silné pro případné rozšíření.

**Návrh uzavření:** G-01 definovat `aircraftKey` pouze pro *důvěryhodná současná data*, `observationId` a kontrolu časového pořadí; při rozporných pozicích nepoužívat průměr ani last-write-wins. Řetězec non-ICAO mimo ověřený scope označit `ambiguous` a neprovést destruktivní merge. Dnes jen jedna stanice; nepřidávat multi-source runtime, DB raw tabulky ani nový UI přepínač.

**Brána:** syntetický konflikt totožné adresy na nemožných pozicích; duplicita JSON/BEAST jedné stanice nemá dva tracky ani dva SQL zápisy.

### P0-03 — Verzované DTO a atomické zahájení SSE

**Doklad:** [SSE kontrakt](../technical/SSE-PROTOKOL-A-LIMITY.md), současný [OpenAPI](../../contracts/openapi.yaml).

Odhad frekvence 1 Hz, 15s heartbeat a bounded fanout je dobrý. Není ale fyzický jednoznačný `AircraftDTO` ani formální schema `snapshot`, `delta`, `removed`, jejich pole s null/unknown/stale a garance transakční hranice mezi registrací do broadcastu a pořízením snapshotu. Kdyby mezi nimi vznikla mezera, nový browser může **navždy postrádat aktualizaci**.

**Návrh uzavření:** G-01 společná JSON Schema/OpenAPI + Rust/TS contract fixtures; snapshot musí pokrývat známou `epoch:seq` a na jeho navázání nesmí existovat drop race. Při mezeře/restartu povinná resynchronizace; při snapshotu větším než plánovaný 1MiB limit musí být definované bezpečné chování (odmítnutí / chunking, ne tiché zkrácení).

**Brána:** race test při současném připojení a burstu událostí, seq gap, timeout, nevalidní a chybějící hodnoty.

## 3. Otevřené body — rozhodnutí před cílovou etapou

### P1-04 — Historie SQL: ack, časový bucket a write-back při pádu

**Etapa:** G-04. **Doklad:** [fyzický DB model](../../database/POSTGRES-DATOVY-MODEL.md), [economy sampler](../technical/RAM-CACHE-A-DEDUP-ZAPISU.md).

30s idempotentní SQL bucket a typická 60s adaptace jsou rozumné **konfigurační hypotézy**. Chybí ale závazné chování pozdějšího lepšího vzorku ze stejného bucketu, uzavření bucketu, atomické potvrzení `lastPersisted` až po COMMIT, havárie mezi odesláním SQL a jeho ack a deterministické pořadí výběru bodů. SQL `ON CONFLICT DO NOTHING` může definitivně zahodit pozdější kvalitnější vzorek. Výstup 60s sampleru může také ztratit krátké manévry a změnit podobu logbooku.

**Návrh uzavření:** jedna pending mapa, uzavření bucketu + definovaný grace interval na opožděné pakety, deterministické pořadí kvalita/čas, explicitní volba mezi *append-only finalizací* a komplikovanějším replace režimem. Doporučená je append-only finalizace s metrikou pozdních pozorování. Commit ACK rozhoduje o watermarku; crash/retry testem ověřit SQL idempotenci. Změřit kvalitu rekonstruované trasy.

### P1-05 — Definice logbooku a dlouhodobých agregací

**Etapa:** G-04/G-09. **Doklad:** [M08 a funkce S06/S09](../plan/MODULY-A-ROZSAH.md).

J09 ukládá jen bodové vzorky a `receiver_daily`. **Není definováno, co je jeden let**, jak určit začátek/konec přerušované stopy, jak zobrazit logbook po 30denní retenci detailu a jak pravdivě odlišit *unikátní letadla*, *unikátní lety*, *zprávy přijímače* a *uložené vzorky*. Čítače pouze v RAM se při crashi resetují; denní počet unikátních identit nelze prostě sečíst z intervalových checkpointů.

**Návrh uzavření:** pro první verzi **receiver logbook = denní souhrn přijímače**, nikoliv garantovaný kompletní katalog jednotlivých letů. Pokud chce produkt skutečný seznam letů, definovat boundary/sessionization/stopu a případně nejmenší nový odvozený objekt podle aktivního konzumenta, až s měřením. Denní `distinct` počítat buď ze schválené per-day RAM množiny v malém limitu + korektní resume, nebo z uložených vzorků s explicitní definicí a známými ztrátami. Po 30 dnech uvádět jen agregáty.

### P1-06 — Statistiky typů, aerolinek a tras bez zaručených metadat

**Etapa:** G-06/G-09. **Doklad:** [S06 zadání](../ukoly/G-09-STATISTIKY-DIAGNOSTIKA.md).

readsb může volitelně vrátit `t` (typ) a `r` (registraci) při zapnutém místním katalogu, **neobsahuje však spolehlivý úplný zdroj tras letů a aerolinek pro všechny cíle**. Jednoduchá identifikace operátora z callsignu je nejistá, linka a skutečně vykonaná trasa jsou ještě jiná otázka. Bez zdroje je přísná formulace celé statistiky neprokazatelná.

**Návrh uzavření:** přesně vymezit, které dimenze S06 mají dovolený a auditovatelný původ; zbylé pole `unknown/unavailable`, nikoli odhadnutá trasa. Nevytvářet samostatné SQL katalogy ani placené feedy bez schváleného aktivního konzumenta. V G-09 definovat přejímku i pro instalační profil bez readsb katalogu.

### P1-07 — Web Push / Pushover bez administrační stránky

**Etapa:** G-08/G-10. **Doklad:** [notifikační zadání](../ukoly/G-08-NOUZOVE-KODY-A-NOTIFIKACE.md).

Vybrané notifikace mají opt-in a dry-run, ale nemají rozhodnuto **kdo smí přidat/odebrat subscription**, zda veřejný neautorizovaný endpoint dovoluje cizí registrace, jak omezit rate-limit a kvóty, chránit VAPID/příjemce, provést revokaci a smazání zastaralých tokenů. Schválená architektura nepotřebuje administrační UI, ale **bezpečný subscription lifecycle ano**.

**Návrh uzavření:** subscribe/unsubscribe s jednoznačným ověřením vlastnictví endpointu a ochranou proti zneužití, rate-limity, expirace a TTL, audit rozhodnutí a bezpečné persistence minimum. Pushover pouze pevně autorizovaný interní cíl z neveřejné konfigurace. Veřejný klient nesmí libovolně zvolit Pushover token/recipient. Živé odesílání až po samostatném povolení.

### P1-08 — Meteorologie a katalog: zdroje, kvalita, licence

**Etapa:** G-06/G-07. **Doklad:** [počasí](../ukoly/G-07-VYBRANE-POCASI.md), [letiště](../ukoly/G-06-LETISTE-NAVAID.md).

ČHMÚ radar, SIGMET, letiště/NAVAID a BDS 4,4 jsou aktivně plánované, ale konkrétní licencované výstupy a aktualizační protokoly dosud nebyly ověřeny pro aktuální nasazení. U Comm-B **nemusí být BDS kód přímo v downlinku**, takže je nutné rozpoznání/validace; nepodporované/ambivalentní BDS nesmějí vyrobit fiktivní meteorologii. Samotný readsb JSON může obsahovat vypočtené počasí, které není totéž co surové BDS 4,4 měření.

**Návrh uzavření:** u každého schváleného zdroje určit URI, verzi/formát, caching a TTL, atribuci, územní pokrytí, provozní limity a fallback. BDS 4,4 pouze jako dostupné a validované pozorování, nikoliv garance pro každé letadlo. Nezávislost počasí na hot path testovat.

### P1-09 — Skutečné parametry existujícího databázového LXC

**Etapa:** G-04 a release. **Doklad:** [bezpečnost/backup](../runbooks/POSTGRES-LXC-SECURITY-BACKUP.md).

Uvažujeme PostgreSQL, ale v GitHubu **není ověřená verze, CA/certifikát, firewall, kompatibilní TLS driver, síťová latence, účty/schéma, backup ani kapacita**. Zvolené `max_connections=2` a 5s flush mohou po síti fungovat, ale bez měření nejde potvrdit p95 či ztráty. Sdílená instance znamená společný WAL a disk, takže naměřený `pg_stat_wal` na produkci neříká, kolik zapisuje pouze V2.

**Návrh uzavření:** schválený test izolované V2 DEV DB a minimálních oprávnění, TLS `verify-full` s negativními testy, odpojení/spojení a skutečný `pg_restore`. Počítat i se zátěží **denního dumpu 30denní historie**, která může tvořit významnou diskovou/CPU/režii; backup zavést s limity I/O a vhodným časem.

### P1-10 — RAM/SSE/mobilní mapa: kapacita a přetížení

**Etapa:** G-02/G-03/G-05/G-11. **Doklad:** [SSE limity](../technical/SSE-PROTOKOL-A-LIMITY.md), [MapLibre](../technical/MAPLIBRE-VYKON-A-ZATEZ.md).

Limity `5 000` cílů × `64` trail bodů, `100` SSE klientů × `256 KiB` fronty, 1MiB snapshot a 1Hz refresh nejsou provázané **jednou ucelenou RAM/přenosovou bilancí**. Vzniká riziko simultánního 100× snapshotu, dalších TCP/proxy bufferů a GC/JS zátěže browseru. Rust actor nesmí čekat na serializaci do pomalých klientů.

**Návrh uzavření:** konkrétní `max serialized event bytes`, bounded broadcast queue, shared serialized event, politiky odmítnutí/reconnectu, vyhození neaktuálních trailů a klientských limitů; test 1/10/100 současných reconnectů, JSON payloadu u 5 000 cílů, pan/zoom a offline/stale. RAM a CPU profily až v G-11, počáteční limity jsou pouze brány.

### P1-11 — Externí pokrytí: rozsah a budoucí implementovatelnost

**Etapa:** pouze minimální G-01 kontrakt; případné rozšíření až po samostatném souhlasu. **Doklad:** [ADR 0008](../adr/0008-identita-puvodu-lokalniho-prijmu.md).

Připravené `receiverId`/`transport` je správný základ. Není ovšem definovaná budoucí síťová autentizace vzdálené stanice, rozdíl *vlastní RF* versus *provider-computed position*, práva na ukládání/retransmisi externích dat ani izolace chybných/spoofovaných zpráv. **Pro současných 45 funkcí to neblokuje G-01**, protože druhý zdroj ani externí feedy nejsou schválené.

**Rozhodnutí:** ponechat pouze důvěryhodné `receiverId`, `transport`, `positionOrigin` a `timeBasis` v interním kontraktu, bez runtime adapterů navíc. Před budoucím rozšířením udělat nové ADR a akceptační testy; nevracet přepínač zdrojů ani nové DB raw streamy do nynějšího projektu.

## 4. Drobné rozpory a dokumentační dluh

### P2-12 — Stale výčet G-00 a historické supersedované ADR

**Doklad:** [ETAPY-A-STAV](../plan/ETAPY-A-STAV.md) stále má v úvodní tabulce `G-00 Rust + Next.js`, přestože nyní běží Vite. Staré ADR 0006 výslovně mluví o původním 15s sampling, teprve dodatkem jej superseduje ADR 0007. Technologický průzkum obsahuje předchozí alternativy. Čtenář bez čtení dodatků může omylem zavést zastaralé parametry.

**Doporučení:** opravit aktuální tabulku G-00 na Vite, ke starému ADR dát na první obrazovku explicitní precedence 0007/0008 a držet aktuální rozcestník jako jediný vstup. Historické poznámky ponechat, ale nemíchat je s dnešním stavem. CI dnes zjevně nekontroluje věcnou konzistenci takových vět.

### P2-13 — Ověření neodpovídá připravenosti produktu

**Doklad:** zelené CI na současném `main` (Rust stub, statický web, MD kontrola a modelový sampling test). Chybí naopak Rust reálný parser, SQLx test DB, MapLibre vizualizace a skutečný Playwright E2E. Není důvod označovat některou z 45 funkcí za dokončenou.

**Doporučení:** definice DONE v etapách už je správná. Každý budoucí PR musí mít test, screenshot/trace tam kde relevantní, případný benchmark a aktualizované kontrakty bez přehnaného tvrzení.

## 5. Navržené pořadí uzavírání

| Milník | Co musí být uzavřeno | GO/NO-GO |
|---|---|---|
| **Start G-01** | Schválené jazyky, moduly, repo, 45 aktivních ID, izolace V1 | **GO** |
| **Převzetí G-01** | P0-01, P0-02, P0-03: čas/původ, identita, schema a atomický snapshot/delta; golden fixtures | **NO-GO, dokud nejsou testy** |
| **Převzetí G-02/G-03** | Správný parser, dedup, restart/fallback, bounded RAM a SSE | **NO-GO, dokud není replay a backpressure** |
| **Převzetí G-04** | P1-04, P1-09: uzavření bucketu/ack, SQLx schema, TLS, retence, výkon, test obnovy | **NO-GO bez bezpečné DEV DB a restore** |
| **Převzetí G-05/G-07** | Browser/mapa a P1-08 dostupnost dat/licencí | **NO-GO bez browser testů a skutečných podkladů** |
| **Převzetí G-08/G-09** | P1-05/P1-06/P1-07: logbook a dimenze, oprávnění subscription, provider kvalita | **NO-GO při neověřeném produktu** |
| **G-11/G-12/G-13** | Zátěž, crash recovery, security, E2E, izolovaný shadow a zvlášť povolený release | **NO-GO do všech bezpečnostních bran** |

**Pořadí je záměrně bez nových produktových modulů, služeb či rozšířených providerů.** Existující 140 kroků zůstává zdrojem pravdy; audit určuje akceptační otázky, ne automaticky novou implementační etapu.

## 6. Konečný závěr

**Stack/hranice: ANO, vyjasněné.** **Zdrojová identita, UTC/provenance, logbook/metadata, subscriber auth a provozní parametry: ZATÍM NE zcela uzavřené.** Není potřeba znovu navrhovat systém: musí se **převést podrobné návrhy na přesná G-01 testovatelná DTO, fixtures a kritéria** a postupně doložit další rozhodnutí při konkrétních etapách.

Nezasahovat do V1 ani existující DB LXC a nic nenasazovat bez autorizace. Tento dokument nepřeklasifikuje žádnou aktivní funkci, nemění 45 ID, nepřidává externí feed a nevydává archivované možnosti za schválené.

## Referenční podklady

- readsb JSON včetně `type`, `mlat`, `tisb`, `seen_pos` a `receiver.json`: https://github.com/wiedehopf/readsb/blob/dev/README-json.md
- PostgreSQL partition constraints: https://www.postgresql.org/docs/current/ddl-partitioning.html
- PostgreSQL backup/PITR: https://www.postgresql.org/docs/current/backup.html
- BDS kódování a inference Comm-B: https://mode-s.org/1090mhz/content/mode-s/5-commb.html
- SSE: https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
- Playwright: https://playwright.dev/docs/ci
