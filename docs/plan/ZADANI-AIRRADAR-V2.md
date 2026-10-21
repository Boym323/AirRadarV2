# AirRadar V2 — závazné zadání projektu na zelené louce

**Vytvořeno:** 21. října 2026  
**Repozitář:** https://github.com/Boym323/AirRadarV2  
**Referenční V1:** https://github.com/Boym323/AirRadar  
**Stav:** založený projekt G-00; vývojové etapy G-01 až G-15 postupně realizovat, nikoliv předpokládat jejich dokončení.  
**Jazyk:** **čeština je výchozí a závazný jazyk všech zadání, dokumentace, reportů a viditelných textů aplikace**. Překlad UI do angličtiny zůstává budoucí samostatnou funkcí U04.

## 1. Poslání a rozsah

Vybudovat **skutečně nový AirRadar V2**, který přebírá poznatky z funkční V1, ale nezačne kopírováním jejího zdrojového stromu, architektonických kompromisů, rozsáhlých React komponent ani databázového modelu. Projekt musí mít vlastní GitHub repozitář, samostatné CI, nový LXC, oddělené služby a vlastní databázi. V1 zůstane běžet až do samostatně schváleného přechodu.

**Cílová funkčnost:** implementovat a individuálně ověřit **79 KEEP** funkcí ze [strojové matice](../../contracts/feature-matrix.json), přehledně popsaných v [české matici](MATICE-FUNKCI.md). Dalších **99 REMOVE** není součástí V2. Každá povinná funkce se stává hotovou až po skutečném behaviorálním testu, ne po vytvoření souboru nebo UI náhledu.

**Historie:** staré historické lety, pozice a statistiky z V1 se **nepřenášejí**. Nová V2 databáze začne prázdná a bude vytvářet vlastní záznamy. Neodstraňuj data z V1.

**Pracovní metoda:** jednotlivé etapy G-00 až G-15 v oddělených českých Markdown zadáních; stav a důkazy průběžně ve [stavovém dokumentu](ETAPY-A-STAV.md) a v `docs/reports/`. Dodrž [pravidla dokumentace](PRAVIDLA-DOKUMENTACE.md) a [AGENTS.md](../../AGENTS.md).

## 2. Povinná architektura a volba jazyků

| Součást | Výchozí technologie | Provozní hranice |
|---|---|---|
| Živý backend a autoritativní stav letadel | **Rust, Tokio, Axum** | jediný vlastník stavu a jediná orchestrace přijímačů |
| BEAST, Mode-S, CRC24, normalizace, deduplikace, geomet­rie | **Rust** | omezená paměť, validované rámce, profilované hot paths |
| Web a responzivní mapa | **TypeScript, Next.js, React, MapLibre** | bez přímého čtení BEAST nebo paralelního stavu letadel |
| Historie, agregace, indexy a retence | **PostgreSQL + SQLx / SQL** | asynchronní vzorkované zápisy, vhodné indexy, žádná závislost živého toku na DB |
| Datové kontrakty | **OpenAPI / JSON Schema** | verzované API, generované typy klienta, integrační testy |
| Offline importy a testovací data | **Python, Rust či TypeScript** | jen s odůvodněním; ne na živé kritické cestě |
| Observabilita | strukturované logy, metriky, trasování | sledovat CPU, RAM, p95/p99, DB fronty a provider health |

Jiné programovací jazyky jsou **povolené**, když měření, testy, provozní náklady a české ADR doloží přínos. Nezavádět automaticky Go služby, Kafka, Redis, Kubernetes nebo další databázi. Nepřidávat mikroservisy jen kvůli „modernosti“.

### Základní datový tok

`readsb JSON / lokální BEAST (read-only) → Rust ingest/normalizace → autoritativní RAM → Rust SSE/HTTP → Next.js/MapLibre`

`RAM → omezená asynchronní fronta vzorků → PostgreSQL`

Síťové providery (ADSBHub, ADSB.lol) dodávají samostatně označené záznamy. Počasí, identifikace letadel, ATC a OGN jsou izolované volitelné subsystémy. Výpadek databáze, počasí nebo doplňkových zdrojů nesmí přerušit živý radar.

### Samostatné provozní části

1. `airradarv2-engine`: Rust služba pro příjem, canonical state, veřejné API a SSE; později také dávkovou historii, navigaci, metriky a notifikace.
2. `airradarv2-web`: Next.js aplikace pro mapu, ovládání, panely a lokalizaci. Nevytváří vlastní ADS-B ingest ani samostatné zápisy historie.
3. `airradar_v2`: samostatná PostgreSQL databáze a oprávnění. Sdílení fyzického DB serveru s V1 lze zvážit jen při doložené izolaci a kapacitě.

## 3. Správnost živého radaru

- Základní identitou je ICAO hex; callsign a registrace jsou pozorované vlastnosti, ne stabilní klíč.
- Správně pracovat s ne-ICAO identifikátory readsb, validací zeměpisných pozic, časovými známkami, jejich přetečením a zastaráváním.
- Příjem readsb JSON a BEAST oddělit. BEAST parser pokrývá odpovídající DF varianty, CRC24 a časovou kontinuitu. Při výpadku jedné cesty automaticky přejít na druhou bez falešné změny identity.
- Rozlišovat **LOCAL** (místní příjem) a **EXTENDED** (externí síťové zdroje); poskytovat provenance a zachovat prioritu čerstvých lokálních údajů. Síťová data nesmějí být podvodně vydávána za lokální.
- Omezit počet objektů, front, retry, velikosti odpovědí a paměť; při chybách zachovat stav pouze podle definovaných freshness pravidel.
- Srovnávací testy použijí anonymizované fixní ADS-B záznamy; nezávisí na proměnlivých aktuálních letech.

## 4. API, streamování a bezpečnost

Navrhnout verzované Rust REST API a zdokumentovat jej v `contracts/openapi.yaml`. Cílové oblasti: `/api/v1/aircraft`, `/api/v1/aircraft/:id`, `/api/v1/stream`, historie, statistiky, navigace, počasí a system status. Konkrétní endpointy závazně schvalovat přes kontrakty, ne automaticky kopírovat V1.

SSE musí poskytovat výchozí celý snapshot a volitelný initial-snapshot + delta režim; zachovat pořadí a sekvence, heartbeat, reconnect, cleanup, bounded fanout/coalescing, ochranu před pomalým klientem a odmítnutí při kapacitním limitu. Nedovolit workload úměrný počtu klientů krát počet letadel při každém pollingu bez doložené nutnosti.

Veřejné DTO nesmí obsahovat credentials, interní chyby ani nechtěnou přesnou polohu přijímače. Sanitizované read-only diagnostiky mohou být veřejné; privilegovaná data a administrační mutace vyžadují autentizaci a autorizaci. Obslužné texty API dokumentovat česky, názvy polí a stavových kódů držet stabilní.

## 5. PostgreSQL V2 — nové schéma, bez přenosu staré historie

Databázi navrhnout **podle reálně požadovaných dotazů**, ne kopírováním V1 Prisma modelů. Logické oblasti (nemusí to být stejnojmenná schémata): reference (letadla, letiště, ATC/ATS/NAVAID, verze), lety a vzorkované pozice, denní agregace/coverage, Navigation Integrity, notifikační fronta, omezená provozní telemetrie.

Povinné optimalizace:

- vzorkování, coalescing a dávkové INSERT/UPSERT místo ukládání všech zpráv;
- bounded write queue, transakční dávky, odolnost vůči restartu a chybám DB;
- časové rozsahy, stránkování, správné composite indexy, úsporné agregace v SQL, žádné N+1 či neregulované úplné `SELECT *`;
- `EXPLAIN (ANALYZE, BUFFERS)` na DEV datech, měření p50/p95/p99, zápisů, WAL, dead tuples a paměti;
- retence podle typu dat a potřeby UI (pozice, letové instance, agregace, anomálie a provozní diagnostiky);
- partitioning/BRIN jen po benchmarku a zdůvodnění jejich složitosti;
- nová vlastní správa migrací v SQLx; záloha a skutečný test obnovy před nasazením.

Při výpadku DB zůstanou letadla v RAM dostupná. Neexistující historie a 7/30denní přehledy musí zobrazit korektní prázdný stav, dokud V2 nenasbírá nové údaje. **Nikdy neměnit, nepoužívat jako testovací cílovou DB a nemazat PostgreSQL V1.**

## 6. Letecké a doplňkové subsystémy

**ATC a navigace:** importovaný a verzovaný eAIP, prostorové indexy pro sektory, ATS tratě, NAVAID/FIX a Navigation Integrity. ATC assignment je pravděpodobné přiřazení podle polohy, výšky a času; neznamená skutečně naladěnou frekvenci letadla. Zachovat vybrané české a zahraniční ATS tratě dle matice.

**Počasí:** aktuální METAR/TAF, ČHMÚ radar s časovou osou, SIGMET/AirSIGMET, Mode-S/BDS měření. Samostatné TTL cache a platnost časových intervalů; chyby providera nezastaví ADS-B.

**OGN/FLARM:** oddělený ingest a public serializer. DDB privacy `noTracking` a nevyřešená identita musí být fail-closed; anonymní identifikátory nesmějí prozradit skutečné údaje. OGN nepatří do ADS-B statistiky ani běžné ADS-B historie.

**Integrace:** ADSBHub, ADSB.lol raw/HTTP, RXW Hub, FlightAware, ADSBDB, Planespotters. Respektovat skutečné limity, licence, provenance, TTL a ceny; neprovádět neomezené síťové volání na kritické cestě.

**Notifikace:** Web Push a Pushover s durable claim/retry automatem; nepodporovaný kanál nesmí skončit jako doručený. Shadow/test provoz nikdy nesmí vyvolat skutečné alerty uživatelům.

**UI:** funkční MapLibre, seznam, výběr, detail, stopy, follow, vrstvy, responzivita, PWA, offline cache a command search dle matice. **České texty výchozí**; U04 později doplní volitelnou angličtinu.

## 7. Samostatný LXC a nasazování

Nový GitHub repo a nový LXC jsou nezávislé na V1. Předběžné neověřené rozměrování je 4 vCPU, 8 GB RAM a 60 GB disku; teprve zátěžový test určí provozní potřebu. Podrobnosti v [runbooku](../runbooks/LXC-SETUP.md).

V2 engine používá předběžně `127.0.0.1:4101`, web `4100`, samostatné názvy systemd, samostatné secrets a účet. V CI jsou nejprve jen buildy/testy bez deploye. Pozdější release musí být vázaný na přesný SHA, mít rollback a **nesmí přepnout proxy, doménu, V1 LXC či databázi bez zvláštního schválení**.

Živý readsb/BEAST feed může V2 číst pouze read-only a až po ověření, že další klient neovlivní V1. Do té doby se používá generovaný a anonymizovaný replay.

## 8. Metriky a důkazní standard

| Oblast | Doklad | Přijímací podmínka |
|---|---|---|
| Live freshness | replay / případně schválený shadow | žádné nevysvětlené zpoždění |
| CPU a paměť | 1×, 2×, 4×, soak | bounded stav, žádný dlouhodobý růst |
| SSE | 1 / 10 / 100 klientů | reconnect, backpressure, P95/P99 |
| Databáze | query p95/p99, WAL, I/O, velikost indexů | měřená stabilita, bez vazby na hot path |
| BEAST/JSON | golden fixtures a property tests | správnost a stejné identity |
| Funkce | matice 79 ID + konkrétní test | žádné falešné „hotovo“ |
| Bezpečnost | CodeQL, auth, OGN privacy, secrets | žádné kritické úniky |
| Obnova | výpadek poskytovatele/DB, restart a DB restore | kontrolovaný návrat bez vlivu na V1 |

Cílové číselné limity stanovit až po baseline. Nedeklarovat zrychlení z jednoho kolísajícího vzorku. Výsledky ukládat **v češtině** do `docs/reports/` s přesným SHA a linky na CI či naměřené artefakty.

## 9. Etapy a definice dokončení

Plný seznam a stav je v [ETAPY-A-STAV.md](ETAPY-A-STAV.md). Souhrn: G-00 základ repozitáře; G-01 kontrakty a fixture; G-02 lokální Rust ADS-B; G-03 síťový provoz; G-04 SSE; G-05 PostgreSQL; G-06 frontend mapa; G-07 ATC/ATS/navigation; G-08 počasí; G-09 OGN/integrace; G-10 notifikace/admin; G-11 statistiky/T58; G-12 dokončení UI/PWA/i18n; G-13 akceptace 79 funkcí; G-14 oddělený shadow; G-15 samostatně schválený přechod.

Pro každou etapu požaduj: konkrétní změny, testy, výkonnostní měření (kde dává smysl), stabilní veřejný kontrakt, aktualizovaný stav funkcí, české dokumenty a report, případné blokéry a postup návratu. Etapy G-00 až G-14 **nesmí automaticky nasazovat do V1 ani měnit produkční provoz**.

**G-15 je jediná etapa pro finální přechod**, a ani ona nezakládá automatický souhlas se změnou DNS, reverse proxy, vypnutím V1 nebo smazáním historických dat. Tato jednání musí mít samostatné výslovné schválení.

## 10. Pokyn pro agenta / Codex

> Pracuj v repozitáři https://github.com/Boym323/AirRadarV2. Nejdříve přečti `AGENTS.md`, `docs/plan/ZADANI-AIRRADAR-V2.md`, `docs/plan/ETAPY-A-STAV.md` a zadání vybrané G-XX etapy. Všechny dokumenty, reporty a viditelné UI texty piš **česky**. Implementuj vždy jen jednu etapu, prováděj skutečné testy, měř výkon a aktualizuj české Markdown dokumenty i matici funkcí. Původní AirRadar V1 a jeho LXC/databázi neměň. Neprováděj produkční release bez zvláštního souhlasu. V závěru uveď commit, ověřené výsledky, chybějící důkazy a stav etapy.

