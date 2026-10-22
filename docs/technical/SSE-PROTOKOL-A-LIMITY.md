# SSE full/delta — frekvence, kontrakt a ohraničená paměť

**Stav k 22. 10. 2026:** G-03 částečně implementováno a otestováno v izolované CI ([#39835102766](https://github.com/Boym323/AirRadarV2/actions/runs/39835102766)); cílový DEV LXC, skuteční pomalí klienti a browser streaming jsou nadále **NEOVĚŘENO**. Návrhové limity zůstávají nezměněné. Funkce J06/J07/J08, navazuje R01/R06/R08 a diagnostické O08.

## Zvolený protokol

- Jeden **Rust/Axum SSE producer**, jediný kanonický RAM state, žádný nový broker.
- Při připojení \`snapshot\` (kompletní aktuální stav) a následně **\`delta\` s \`upsert\` a \`removed\`**.
- Výchozí **1× za sekundu** posílat sloučené změny od posledního okna; při chybějící změně lze neodesílat delta. Zvýšení na 2 Hz jen s naměřeným přínosem a odpovídajícími testy.
- **Heartbeat každých 15 sekund** jako SSE comment \`: heartbeat\\n\\n\` (není to aktualizace provozu). Doporučený proxi idle timeout aspoň **75 sekund**, bez bufferingu.
- **Obnovení:** každá zpráva nese monotónní \`seq\` v JSON a event \`id\`. \`Last-Event-ID\` není příslib dostupného replay; server při chybějící sekvenci nebo restartu pošle nový snapshot. Použít \`epoch\`/generaci procesu a detekci gap ve webu.
- **Kontrakt validovat jako JSON** v G-01, neodvozovat automaticky z nynějšího bootstrap \`NOT_IMPLEMENTED\` OpenAPI.

~~~text
event: snapshot
id: epoch123:18001
data: {"epoch":"epoch123","seq":18001,"generatedAt":"...Z","aircraft":[...]}

event: delta
id: epoch123:18002
data: {"epoch":"epoch123","seq":18002,"generatedAt":"...Z",
       "upsert":[...],"removed":["icao:abc123"]}

: heartbeat
~~~

\`removed\` musí vzniknout z reálného zániku stavu/TTL a nesmí vymazat cíl jen kvůli krátkému výpadku jednoho zdroje. \`generatedAt\` označuje čas snímku, nikoli skutečný čas polohy. Neexistující souřadnice netransformovat na \`0,0\`.

## Výchozí limity pro první implementaci

| Omezení | Startovací hodnota | Kdy změnit |
|---|---:|---|
| Frekvence delta | **1 Hz**, sdílený batch | teprve po měření 2 Hz a React/MapLibre frame |
| Současní SSE odběratelé | **100** | load test 1/10/100/200 klientů |
| Počet živých identit v kanonickém RAM | **5 000** | skutečný receiver coverage a RSS |
| Načtený snapshot | cílový limit **1 MiB serializovaného obsahu** | pokud 5 000 letadel překročí budget, rozhodnout chunking/safe error, netruncovat „tiše“ |
| Per-client fronta pending dat | **max. 256 KiB** nebo **2 delta okna** | naměřená propustnost, rychlost nejpomalejšího klienta |
| Odhad hrubé rezervy pro 100 klientů | **≤ 25 MiB** při 256 KiB na klienta; mimo sdílené snapshoty, TCP buffery, serializéry a overhead | ověřit RSS měřením |
| Transport slow consumer | reconnect s novým snapshotem, případně odmítnout klienta; **nikdy nečekat v live ingest hot path** | chaos test klienta |
| Heartbeat | **15 s** | reálné HTTP proxy idle timeouty |
| Browser transport | **1 EventSource na stránku** | nevyvíjet per-panel zdroje |

**Výpočty velikosti jsou horní hranice plánovaných front, ne skutečně změřený RSS.** \`Arc<Bytes>\`/sdílená serializace pro stejný broadcast event omezuje kopírování; neznamená to, že každý uživatel drží vlastní kopií celého snapshotu. Uprostřed dlouhého streamu nestřádat všechny eventy kvůli hypotetickému replay.

## Správnost reconnectu

- První připojení: snapshot pro pevnou \`seq\` ze stejného kanonického stavu, na který navazuje delta, bez mezery mezi registrací odběru a snapshotem.
- Pokud se seq ztratí nebo \`epoch\` změní, klient označí stav za neaktuální, vyžádá nové připojení/snapshot, **nikoli aplikuje nekonzistentní delta**.
- Pokud fanout přetíží klienta, čítač \`slow_disconnects_total\`; uvolnit EventSource, přestat udržovat per-client stav, žádné nekonečné fronty.
- Cache-Control: \`no-cache, no-transform\`; Content-Type \`text/event-stream; charset=utf-8\`; proxy streaming bez bufferování.
- Preferovat HTTP/2 na veřejném TLS ingressu. HTTP/1.1 omezuje počet SSE spojení na doménu/prohlížeč; při jedné stránce držíme jeden EventSource. Web nepřipojuje diagnostická SSE zvlášť, pokud existující feed stačí.

## Přejímací testy

- 1/10/100 souběžných klientů s 25/250/1 000/5 000 letadly a 1/2 Hz; CPU, RSS, bytes/s, p50/p95/p99 z RAM do browseru.
- Výpadek klienta, neúplný event, heartbeat, změna epoch, HTTP proxy restart, přeskok sekvence, odstranění stale letadla.
- Pomalý klient čte po malých dávkách/nikdy: bounded queue je v limitu, neovlivňuje ostatní ani hlavní ingest.
- Restart Rust enginu: požadavek na zcela nový snapshot; stará delta se ignorují.
- Výpadek DB nevypne stream; payload neobsahuje citlivé údaje z receiveru.

## Dokumentace

- [MDN EventSource/SSE](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events) — formát, reconnect, limit HTTP/1.1 a heartbeat.
- [Axum](https://docs.rs/axum/latest/axum/response/sse/) — server-side SSE.

## P1: bounded SSE a explicitní přetížení (izolované PR #16)

- Jednou připravený sdílený SSE `Bytes` payload, max. 16KiB blok na jeden poll HTTP Body, žádná privátní aplikační fronta na klienta; Tokio `watch` drží jen poslední frame.
- 256KiB aplikační budget **nezahrnuje OS TCP socket buffer**; skutečný RSS/p95/p99 pro 1/10/100 klientů na DEV stále vyžaduje provozní přejímku.
- Více než 1MiB serializovaného full snapshotu nebo 5000 veřejných cílů: `health=degraded`, `GET aircraft/stream = 503 CAPACITY_EXCEEDED`, existující stream dostane `event: unavailable` a skončí. Starý validní stav se nikdy neposílá jako nové LIVE údaje.
- Po obnově pod limit server emituje nový plný snapshot, ne delta proti zahozenému stavu. Vlastní Rust RAM ingest se neodřezává. Testy jsou připraveny pro 5000 úsporných a 5000 plných objektů.

## G-03.07 návrhové provozní hranice a test po 22. 10. 2026

Bez vlastního TCP writeru nemůže aplikace spolehlivě vynutit uzavření nečteného socketu do pevného času; nová regrese proto testuje **nezávislé uvolnění aplikačního SSE permitu** při 30s nečinnosti. Každý z max. 100 klientů drží jen jednu sdílenou `Bytes` referenci, 16KiB blok na jeden poll, a jeden lehký dozorovací task přes `Weak`. Když hyper kvůli socket backpressure nepolluje Body, watchdog uvolní místo novému klientovi. Pokud klient později obnoví čtení, stream se uzavře. Vnější TCP send buffer a vývoj RSS vyžadují LXC/OS měření, takže parametr 256KiB per-client zatím není tvrzením o celkovém transportu.

## G-03.10 ověřený syntetický rozsah (CI #39845031595)

Při jediném readsb JSON souboru s 5000 validními řádky musí Rust ingest zpracovat celý snapshot atomicky jako jednu dávku. 2048-elementová BEAST FIFO se **nepřetěžuje 5000 jednotlivými JSON `try_send`**. Fronta může mít nanejvýš jednu čekající JSON dávku díky `AtomicBool`, další se v přetížení zahodí celá (s přesným počtem dropů); nikdy se nehromadí mnoho 4MiB payloadů. Důkaz `g03_10_json_batch_preserves_all_5000_tracks_without_fifo_loss` a CI scénáře 1/10/100 klientů × 5000 sparse tracků v GitHub Actions #39845031595. Naměřený Rust VmRSS peak 43396 KiB při 100 klientů; neznamená hranici paměti síťového jádra ani cílového LXC. Další brána: reálný pomalý TCP konzument a reálný Nginx/Browser na DEV.
