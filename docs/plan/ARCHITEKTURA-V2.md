# Architektura AirRadar V2 — optimalizovaný návrh po technologickém průzkumu

**Stav:** doporučený cílový návrh, nikoliv stav běžící implementace.  
**Rozsah:** 45 aktuálních aktivních funkcí, výhradně místní readsb/BEAST data.  
**Rozhodnutí k potvrzení:** web Vite/React a databáze SQLite vs PostgreSQL před implementací dotčených etap.  
**V1:** bez zásahu a bez migrace starých historických dat.

## 1. Systémové hranice

~~~text
                    MÍSTNÍ PŘIJÍMAČ
                  readsb JSON + BEAST
                          │ (read-only, omezené spojení)
                          ▼
┌─────────────────────────────────────────────────────────┐
│ Rust engine (jeden proces a jeden vlastník letadel)     │
│  ingest adapters → parser → normalizace → RAM tracks     │
│                                 │                       │
│               ┌─────────────────┼───────────────────┐   │
│               ▼                 ▼                   ▼   │
│          REST a SSE      sampling queue       squawk 7500/│
│          snapshot/delta    bounded             7600/7700  │
│               │                 │                   │     │
│               │                 ▼                   ▼     │
│               │              úložiště          push dry-run│
└───────────────┼─────────────────┼─────────────────────────┘
                │                 │
                ▼                 ▼
        HTTPS reverse proxy   SQLite WAL nebo PostgreSQL
           ┌──────┴──────┐        pouze V2 data
           ▼             ▼
    statický Vite/React  /api/ proxy Rust
      MapLibre, PWA,     /api/v1/stream SSE
      české UI
~~~

**Poznámka:** Současný G-00 kód stále obsahuje Next.js a nemá reálný ingest ani databázi. Toto schéma proto definuje *cílové členění*, nikoli implementovaný stav. Přechod webového buildu musí být samostatným testovaným commitem.

## 2. Proč bez samostatného Next.js serveru

Nemáme nyní server-side personalizované stránky, SEO obsah ani business logiku pro Next.js server actions. REST, SSE, meteorologická cache a notifikace běží či budou běžet v Rustu. Statický React obsah stačí. Vynechání produkčního Node runtime:
- odstraní jeden dlouhodobě běžící proces, další health-check a jednu sadu serverových závislostí;
- umožní obsluhu webu na stejném originu z reverzní proxy, které už předává Rust API;
- zjednoduší rollback na atomickou výměnu statického adresáře a Rust binárky.

Server-side rendering lze zvážit jen s novým aktivním požadavkem; nejde o nutnou součást současného produktu. Výkonnost browseru se musí prokázat na reálném UI.

## 3. Logické moduly (nikoliv mikroservisy)

| Modul | Vlastník | Komunikace | Charakter |
|---|---|---|---|
| M01 readsb/BEAST ingest a track state | Rust | TCP a JSON → kanonická RAM | kritická nízkolatenční cesta |
| M02 REST + SSE | Rust Axum | readonly DTO → web | sdílený producent, bounded klienti |
| M03 historie | Rust + SQLx | vzorky do zvolené DB | fail-soft, bounded queue |
| M04 mapa a ovládání | React + MapLibre | read API / EventSource | frontend, bez ingest |
| M05 letiště/NAVAID | Rust + katalog | cache/SQL → web | není součást live hot path |
| M06 vybrané počasí | Rust cache + MapLibre | HTTP provider → validní cache | oddělené TTL/failure |
| M07 squawk a doručení | Rust + web service worker | lokální event → opt-in push | default dry-run |
| M08 statistiky a diagnostika | Rust + SQL | metriky → API/SSE | levné a omezené dotazy |
| M09 PWA/i18n/search | React build | statické assety / lokalizace | pouze UI |

Konkrétní aktivní ID jsou v [katalogu modulů](MODULY-A-ROZSAH.md); tato architektonická stránka nezakládá nové funkce ani datové zdroje.

## 4. Datové kontrakty a ingest

- **Identita:** stabilní ICAO / rozlišení ne-ICAO identifikátorů, callsign jako pozorovaný atribut; nerecyklovat identitu bez freshness.
- **Pole:** každá zpracovaná zpráva má source, sourceTimestamp/receivedAt, validity, age, confidence, volitelnou souřadnici a explicitní absenci neznámých vlastností.
- **readsb JSON:** získat coherent snapshot, bezpečně přebrat polohy a rychlost, včetně chybějících polí a restartu receiveru.
- **BEAST:** validovat délku, escape, CRC, DF a čas, případně CPR; nesmí produkovat jiný druh identity než JSON.
- **Pravidla merge:** jeden RAM store přijímá normalizované pozorování, jedinečná identita; fallback z BEAST na JSON s explicitní kontrolou stáří.
- **Omezení:** maximální délky bufferů, bounded počet letadel, TTL, jitter/backoff; vynucené shazování nevalidních rámců.

## 5. SSE kontrakt

**Návrh, který se musí potvrdit v G-01:**

~~~text
GET /api/v1/aircraft           → aktuální sanitizovaný snapshot
GET /api/v1/stream             → text/event-stream
event: snapshot                {sequence, serverTime, aircraft:[...]}
event: delta                   {sequence, upsert:[...], removed:[...]}
event: heartbeat               komentář/heartbeat bez dat letadel
~~~

- Full snapshot při novém připojení, po výpadku sekvence nebo při nemožnosti doplnit stav.
- Delta až po dokončení počátečního snapshotu; monotónní sequence, stabilní ID.
- Bounded broadcast, low watermark reconnect pro pomalé klienty, žádné neomezené per-client fronty.
- Jeden EventSource na klienta, proxy \`X-Accel-Buffering: no\` / \`proxy_buffering off\` a pravidelná heartbeat. Za reverse proxy otestovat chování HTTP/1.1 a HTTP/2.
- Admin mutace (pokud nějaké interně vůbec existují) mimo veřejná read-only API.

## 6. Webová mapa

- Oddělený React state pro vybraný cíl, filtr a boční panel; vysokofrekvenční aircraft store nepřepisuje každou React komponentu.
- MapLibre instance persistuje po celou životnost stránky; stejné \`GeoJSONSource\` feature IDs odpovídají ICAO/normalizované identitě.
- SSE update se skládá v store. Render podle frame budgetu: \`setData\` pro jednoduchý případ, \`updateData\` při skutečném přínosu (nutná unikátní ID).
- Letiště, počasí, letové stopy a letadla jsou samostatné vrstvy s různými aktualizačními intervaly.
- PWA offline ukazuje pouze výslovně zastaralý snapshot; nesmí zobrazit zamrzlou polohu jako právě živé letadlo.

## 7. Úložiště a retence

**Finální DB engine není zatím vybrán na základě měření.** Preferované ověřování: SQLite WAL a PostgreSQL se stejným schématem a ekvivalentními SQL dotazy.

- Povinné logické tabulky jen podle aktivních konzumentů, zejména samples/flight identity, agregace pro S06/S09 a případné push queue; katalogy lze držet ve statické verzované podobě, pokud data nemusejí být měněna online.
- Zápisová cesta **RAM → bounded queue → coalescing → batch writer**; při plné frontě nikdy blokovat ingest, místo toho explicitně hlásit ztracené vzorky.
- SQLite: lokální file path v privátním V2 adresáři, WAL, \`busy_timeout\`, předvídatelný checkpoint, jediný sériový writer, verifikovaný online backup a restore.
- PostgreSQL: oddělený DB/role/schema, pool limits, WAL/autovacuum, \`pg_dump\` či base backup/PITR dle potřeb, obdobná retence.
- Obě cesty musí poskytovat shodné aplikační chování. Není nutné zavádět obecnou runtime pluginovou vrstvu DB, pokud bude jeden engine definitivně zvolen.

## 8. Procesy, konfigurace, deployment

**Cíl při variantě Vite + SQLite:** běžící Rust engine + HTTPS reverse proxy, která rovněž servíruje statický build, a jeden lokální SQLite datový soubor. Žádný dlouhodobý Node runtime, samostatný DB proces či Redis.

**Cíl při PostgreSQL:** Rust engine + statický web + oddělená PostgreSQL instance, vždy s vlastní V2 databází; žádná V1 DB.

- Vlastní unprivileged LXC, vlastní hostname, porty, uživatel a cgroup limity. Výchozí 4 vCPU / 8 GiB / 60 GiB je **nezměřený plán**, ne faktická potřeba.
- CI build Rust + web s připnutými lockfile, samostatné artefakty a SHA.
- \`systemd\` pro Rust s omezenými právy, watchdog/restart, health/smoke; proxy s TLS, SSE bez bufferování.
- Před jakýmkoliv prod release otestovat zálohu/restore dat, LXC restart a návrat na předešlý build.
- Nový LXC není zatím v repozitáři prokázaný jako připravený, takže žádný provisioning/deploy se z tohoto průzkumu neprovádí.

## 9. Nezbytné kvalitativní brány

Před G-02 musí G-01 dodat kontrakty a opakovatelné fixtures. Před G-04 musí být zavřeno DB rozhodnutí. Před G-05 musí Vite frontend prokázat PWA/SSR-nepotřebnost v prototypu. Před G-12 shadow musí být připraveny limity přijímače a prokázána neškodnost druhého read-only připojení. G-13 je samostatné povolení produkčního vydání, nikoliv automatický deploy.
