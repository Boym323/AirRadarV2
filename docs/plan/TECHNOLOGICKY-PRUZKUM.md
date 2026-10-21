# Technologický průzkum AirRadar V2

**Datum rešerše:** 21. 10. 2026  
**Rozsah:** 45 schválených funkcí, jediný místní přijímač readsb, nový oddělený LXC, žádný přenos databáze V1.  
**Typ důkazu:** ověřené oficiální dokumentace a aktuální stav repozitáře; **nejde o naměřený benchmark AirRadar V2**.  
**Výsledek:** viz [technologická rozhodnutí](TECHNOLOGICKA-ROZHODNUTI.md) a [benchmarkový plán](BENCHMARKY-TECHNOLOGII.md).

## Shrnutí závěrů

| Otázka | Odpověď pro tento projekt | Jistota | Podmínka |
|---|---|---|---|
| Rust/Axum, Go, nebo Node.js backend? | Ponechat **Rust + Tokio + Axum**. | Středně vysoká architektonicky, nikoli výkonově | Průchod golden BEAST/readsb replay; Go je věrohodný kontrolní kandidát |
| Next.js, nebo Vite? | Upřednostnit **Vite + React + TypeScript** a staticky servírovaný web. | Vysoká z hlediska potřeby funkcí; skutečnou rychlost změřit | Ověřit PWA/Web Push, české UI, mapu, routing a externí SSE |
| PostgreSQL, nebo SQLite? | **SQLite WAL** je legitimní a pro jediný lokální zapisovač pravděpodobně nejjednodušší volba; PostgreSQL ponechat jako alternativu. | Střední | Uzavřít až reprodukovatelným G-04 testem zátěže, retence a obnovy |
| readsb JSON, nebo BEAST? | **JSON pro rychlou první správnou implementaci, BEAST pro nižší latenci/podrobnosti**, až parser prokáže paritu. | Vysoká funkčně, neověřený výkon | Nespouštět dvě nezávislé autority tracků |
| SSE, nebo WebSocket? | **SSE** pro jednostranné živé změny a HTTP pro příkazy; full snapshot + delta. | Vysoká | Backpressure, resume a proxy timeouty testovat |
| MapLibre, nebo další mapový engine? | **MapLibre GL JS** zachovat; zkoušet GeoJSON layer a diff update, ne mnoho DOM markerů. | Vysoká pro vhodnost knihovny | Ověřit výkon na běžném mobilu |
| LXC: jaký runtime? | Jeden **Rust engine + statický web přes reverse proxy**; databázi lokalizovat dle benchmarku. | Střední | Oddělit od V1; bez produkční změny |
| Node 26? | V říjnu 2026 ještě **Current**, LTS plánováno na 28. 10. 2026. Na sestavovací CI může být; na běžícím LXC s Vite není Node nezbytný. | Vysoká | Pravidelně aktualizovat bezpečnostní vydání |

## A. Rust, Go a Node.js/TypeScript jako engine

### Specifické požadavky AirRadaru

Engine musí dlouho běžet, spolehlivě číst BEAST TCP a readsb JSON, validovat identitu letadla, udržovat jednotný RAM stav, fanout SSE klientů, řídit omezené fronty a nevázat živá data na disk. Měřicí kritéria: správnost BEAST/Mode-S/CRC24, absence race condition/duplicit, CPU%, RSS, rychlost parseru, end-to-end p50/p95/p99, reconnect a implementační/údržbová složitost.

| Vlastnost | Rust/Tokio/Axum | Go/net/http | Node.js/TypeScript |
|---|---|---|---|
| Správa paměti | Bez tracing GC, explicitní vlastnictví; vyšší nároky na programátora | Automatický současný GC, často jednodušší síťový kód | GC a běh JS; výkon záleží na alokacích a implementaci |
| Zpracování BEAST | Přirozené pro práci s binárními daty a protokolovými stavovými automaty | Také velmi vhodné | Vhodné pro menší zátěž, ale CPU-bound dekódování vyžaduje profilování |
| SSE + HTTP | Axum + Tokio + tower middleware | Stabilní standardní knihovna, jednoduché goroutines | Jednoduchý streaming, ale stejný event loop sdílí práci |
| Typová správnost | Compile-time typy, vlastnictví a bezpečnost | Compile-time typy s GC | TypeScript staticky při buildu, typy nejsou runtime validace |
| Databáze | SQLx pro SQLite/PostgreSQL | database/sql + driver | Různí drivery a ORM; dává smysl jen je-li Node backend potřebný |
| Riziko vývoje | Složitější ownership, async debugging a compile time | Obvykle rychlejší implementace I/O | Nejrychlejší přenos webových dovedností, riziko sdílené event-loop zátěže |
| Počet procesů | 1 engine | 1 engine | 1 backend; při Next serveru navíc druhý web runtime |

**Není podložené tvrdit, že Rust bude vždy Xkrát rychlejší než Go či Node.** Výběr podporuje existující Rust kostra G-00, zájem o přesný binární parser a kontrolu paměti; výkonové rozhodnutí musí být ověřeno replayem. Go je nejsilnější záložní kandidát, pokud implementační čas nebo provozní stabilita vyjdou ve srovnatelném testu významně lépe. Node jako ingest engine nedoporučujeme bez nového důvodu — hlavní předností TypeScriptu je zde frontend, nikoliv nezbytná duplicita serverového kódu.

**Dokumentace:** [Axum](https://docs.rs/axum/latest/axum/), [Tokio – sdílený stav](https://tokio.rs/tokio/tutorial/shared-state), [Go GC guide](https://go.dev/doc/gc-guide).

## B. Next.js versus Vite + React

Projekt má převážně interaktivní živou mapu, filtr letadel, základní katalogy, české UI, PWA/offline a Web Push. Vlastní datový backend už bude poskytovat Rust. Nynější V2 nepožaduje SEO rozsáhlého veřejného katalogu, server components, SSR personalizovaných stránek, server actions ani Next route handlers.

| Kritérium | Next.js se serverem | Next.js static export | Vite + React SPA |
|---|---|---|---|
| Běžící Node.js v LXC | Ano | Ne | Ne |
| Vlastní REST/SSE backend | Rust (druhý server) | Rust | Rust |
| SEO / server render dynamických dat | Ano | Ne v plném rozsahu | Ne; není aktivním požadavkem |
| Offline/PWA | Vhodné s vhodným service workerem | Vhodné | Vhodné |
| Nasazení | Build + běžící Node a proxy | Statické HTML/JS/CSS | Statické HTML/JS/CSS |
| Knihovny React/MapLibre | Ano | Ano | Ano |
| Konfigurace SSR/server cache | Ano, další práce | Z části nevyužitá | Neexistuje |
| Přesná velikost JS a doba buildu | **Nezměřeno** | **Nezměřeno** | **Nezměřeno** |

**Doporučení:** Vite/React + React Router pouze pokud potřebujeme více obrazovek, jinak jednoduchý router. Vite staticky sestaví \`dist/\`. Nginx nebo Caddy může pod jednou originou servírovat web a proxy \`/api/\` do Rustu včetně SSE; vyhneme se CORS komplikacím a běžícímu Node procesu. Vite ale automaticky neřeší SEO, serverové funkce či push backend — push odesílá Rust a frontend jen registruje service worker.

**Nezaměňovat:** Vite není samo o sobě běžící webový server na produkci; je nástroj buildu a vývojového serveru. Neříkáme, že samotný Vite nezávisle „zrychlí MapLibre“. Při shodném React a JS kódu bude výkon mapy většinou určovat MapLibre a implementace aktualizací.

**Podklady:** [Vite 8 – návod](https://vite.dev/guide/), [Vite production build](https://vite.dev/guide/build), [Next.js static export a omezení](https://nextjs.org/docs/app/guides/static-exports), [Next.js deployment](https://nextjs.org/docs/app/getting-started/deploying).

## C. PostgreSQL versus SQLite WAL

Dnes požadujeme novou, od nuly budovanou historii J09, logbook a lehké agregace, bez importu V1 a bez těžkých historických analýz. Živý RAM state je nezávislý na úložišti. Základní cesta ukládání je **jeden Rust writer** s bounded queue a dávkovými zápisy.

| Kritérium | SQLite WAL | PostgreSQL |
|---|---|---|
| Provoz | Soubor lokálně v LXC; bez dalšího DB serverového procesu | Samostatná služba/instance, role, pooling, vacuum, WAL |
| Souběh | Mnoho čtenářů, jeden writer v okamžiku | Více souběžných zapisovačů a čtenářů |
| Síťový přístup | Neřešit sdílením DB souboru přes síť | Přirozený databázový TCP server |
| Rozsah dotazů | Dostatečný pro jednoduché intervaly/agregace | Výhoda při rozsáhlejších SQL analýzách a dalších konzumentech |
| Indexy/retence | B-tree indexy a explicitní retence/vacuum; WAL checkpoint | B-tree, BRIN, index management, VACUUM, partitioning dle měření |
| Backup | SQLite Online Backup API / VACUUM INTO; ověřit konzistenci WAL a restore | pg_dump nebo base-backup/WAL archiving/PITR dle požadavků |
| Možný bottleneck | Dlouhé čtečky brání checkpointu, serializovaný writer | Connection pool, VACUUM/WAL, další server a resource overhead |
| Prokázaný výkon v AirRadaru | **Neměřeno** | **Neměřeno** |

**Závěr:** pro jednoduchou V2 stojí za to benchmarkovat SQLite jako **první minimalistický kandidát**, ne rovnou zavést a provozovat PostgreSQL. SQLx dnes podporuje obě varianty. SQLite funguje nejlépe s DB souborem na lokálním stabilním filesystemu, nikoliv na síťovém mountu. Zálohuj prostřednictvím správných SQLite API, nikoliv nekonzistentním \`cp\` otevřeného souboru. Pokud se prokáže potřeba více writerů, oddělené DB služby, náročnější analytika či nereprodukovatelná latence WAL checkpointů, vyber PostgreSQL.

**Dokumentace:** [SQLite – kdy použít](https://www.sqlite.org/whentouse.html), [SQLite WAL](https://www.sqlite.org/wal.html), [SQLite online backup](https://sqlite.org/backup.html), [PostgreSQL BRIN](https://www.postgresql.org/docs/current/brin.html), [PostgreSQL PITR](https://www.postgresql.org/docs/current/continuous-archiving.html), [SQLx](https://docs.rs/sqlx/latest/sqlx/).

## D. readsb JSON versus lokální BEAST TCP

readsb už provádí dekódování ADS-B a nabízí \`aircraft.json\` (úplný aktuální snapshot) a BEAST TCP output (typicky port \`30005\`, je-li povolen). JSON obsahuje explicitní \`now\`, \`messages\` a \`aircraft\`; některá pole mohou chybět, \`hex\` může začínat \`~\` pro ne-ICAO identifikátory. BEAST poskytuje rámce s jemnějším časováním; vyžaduje validní decoder a kompletní skládání zpráv/pozic včetně CPR, není náhradou kvalitativních pravidel readsb.

**Doporučený vývojový postup:**
1. Implementovat readsb JSON jako referenční funkční cestu a golden fixture.
2. Rust BEAST jako druhou paralelní *ingest* cestu pro nižší latenci a dostupnost BDS/Mode-S obsahu, ne jako druhého vlastníka letadel.
3. Doplnit automatické přepnutí, explicitní priority polí, freshness, coverage a deduplikaci. Nespoléhat pouze na CRC jako potvrzení správné polohy.
4. BEAST zapnout jako preferovanou cestu až při **paritě identit, pozic a stability** oproti lokálnímu readsb snapshotu a při doloženém přínosu; přepnutí musí fungovat oběma směry.
5. Citlivé lokální informace o přijímači nesmějí být ve veřejných DTO.

Čísla latence a CPU **nejsou zjištěna**; readsb interval \`--write-json-every\` je konfigurovatelný. BEAST nemusí být automaticky vhodnější pro každou instalaci a chybné časové značky/filtrování mohou výsledek zhoršit.

**Zdroje:** [readsb a porty](https://github.com/wiedehopf/readsb), [formát readsb JSON](https://sources.debian.org/src/readsb/3.14.1630%2Bgit20240609.adc080d-1/README-json.md/).

## E. SSE versus WebSocket a HTTP polling

SSE přenáší jednosměrně server → prohlížeč, přes prohlížečový \`EventSource\` s vestavěným reconnectem. AirRadar živý radar nepotřebuje symetrické low-level real-time povely; ovládání mapy je lokální, ostatní mutace mohou používat REST.

- Posílat jeden full snapshot po připojení nebo nutnosti resynchronizace, potom změny (\`upsert\`/\`removed\`) s monotónní sekvencí.
- Lze implementovat \`Last-Event-ID\` / vlastní \`lastSequence\`, ale neslibovat replay, pokud server nedrží bezpečnou bounded historii. Při mezeře sekvencí vždy nový snapshot.
- Jeden canonical producer, sdílený serializační výstup a bounded per-client queue. Pomalé klienty nepřetěžovat: coalescing, reconnect nebo odmítnutí.
- Reverse proxy musí vypnout buffering SSE, vyhovět idle timeoutu a držet heartbeat; komprese a síťový overhead měřit.
- Při HTTP/1.1 má browser omezený počet souběžných SSE připojení na doménu; HTTP/2 více streamů vyjednává s peerem. Preferovat jediný \`EventSource\` na tab/stránku, a nepřidávat mnoho paralelních diagnostických EventSource.

**Kdy WebSocket:** až kdyby vznikla skutečná potřeba obousměrné vysokofrekvenční komunikace. V tomto rozsahu nepotřebujeme ani vlastní message broker.

**Zdroj:** [MDN – Server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events).

## F. MapLibre a React rendering

MapLibre GL JS vykresluje přes WebGL v prohlížeči. Dlouhodobá architektura by neměla vytvořit samostatný React DOM marker pro každý neustále se měnící bod.

- Jeden \`MapLibre.Map\` lifecycle s refs, ne remount při každém SSE ticku.
- Letadla ve **společném GeoJSON source + symbol layer**; všechny záznamy mají stabilní unikátní ID.
- Změny v modelu spojovat podle frekvence zobrazení (batch), nikdy zbytečně full-re-render React stromu kvůli každému aircraft update.
- Umožňuje-li nainstalovaná verze, využít \`GeoJSONSource.updateData\` diff; unikátní IDs jsou požadavkem API. Pro menší počet letadel může být prostý \`setData\` jednodušší a stejně rychlý; rozhodnout podle profileru.
- Oddělit zdroje letišť, trail a počasí od vysokofrekvenční vrstvy letadel; upřednostnit veškerá nepotřebná data mimo GeoJSON vlastnosti.
- Výkonnost ověřit na desktopu i mobilu, při interakci (pan/zoom/follow), WebGL lost-context a degradaci mapového podkladu.

**Zdroje:** [MapLibre performance guide](https://maplibre.org/maplibre-gl-js/docs/guides/large-data/), [GeoJSONSource / updateData](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/).

## G. Node, LXC, aktualizace a bezpečnost

**K 21. 10. 2026:** Node.js 26.12.1 je bezpečnostní vydání větve **Current**, plánované LTS začíná 28. 10. 2026; 24.x je LTS. Na dnešní produkci je rozumné upřednostnit aktuálně záplatovanou LTS 24.x, nebo vyzkoušet 26.x v CI a později nasadit jako LTS. Pokud se rozhodneme pro Vite jako static build, na LXC je potřeba Node jen při sestavování (ideálně v CI), **nikoliv jako samostatná web služba v runtime**.

- **Provoz V2:** dedikovaný neprivilegovaný LXC, lokální Rust systemd service, Nginx/Caddy pro statický \`dist/\` a stejnooriginové \`/api/\` proxy, DB dle G-04 rozhodnutí.
- **Přístup:** readsb výhradně read-only; neveřejná receiver konfigurace a přesné souřadnice se nesmějí objevit v browseru.
- **Role:** samostatný systémový uživatel, omezení práv k DB souboru a state dir; žádné API secret ve frontend buildu.
- **Povolení:** Web Push potřebuje HTTPS (v produkci), správnou service worker registraci a explicitní user consent. Backend podepisuje/doručuje, během shadow má pevný dry-run.
- **Monitoring:** CPU/RSS, počet stavových objektů, ingest lag, SSE subscriber count, DB queue, disk/WAL, chyby providers. Alerty při vypnutí DB nesmí zastavit engine.
- **Nasazení:** podepsané/identifikované artefakty podle SHA, rollback, otestovaná záloha/obnova, oddělené secrets; netahat V1 LXC do změn.

**Zdroje:** [Node release plan](https://github.com/nodejs/Release), [Node.js 26.12.1 security](https://nodejs.org/en/blog/release/v26.12.1/), [Vite guide](https://vite.dev/guide/), [PostgreSQL PITR](https://www.postgresql.org/docs/current/continuous-archiving.html).

## Co tato rešerše NEDOKAZUJE

1. Skutečné CPU/RSS či rychlost Rust vs Go vs Node při reálných letech.
2. Přesnou velikost a rychlost Next.js vs Vite pro budoucí MapLibre aplikaci.
3. Zda SQLite zvládne konkrétní dlouhodobý profil J09 a logbook za plné retence.
4. Kolik přinese BEAST oproti JSON na našem přijímači.
5. Kolik klientů a letadel zvládne zvolený LXC bez zpoždění.
6. Jestli Vite + React splní budoucí detailní požadavky PWA a push na všech cílových prohlížečích.

Tyto otázky mají reprodukovatelné scénáře a kritéria v [benchmarkovém plánu](BENCHMARKY-TECHNOLOGII.md). **Bez měření nejsou žádné číselné přednosti deklarovány jako výsledek projektu.**
