# Výsledná technologická rozhodnutí — AirRadar V2

**Posouzení:** 21. 10. 2026.  
**Stav:** architektonická rešerše **dokončena**, výkonové a provozní experimenty **neprovedeny**; technologie se nesmějí prohlásit za kvantitativně nejrychlejší bez benchmarku.  
**45 aktivních funkcí** neměníme; neobnovujeme žádné odložené moduly.

## Rozhodovací tabulka

| ID | Otázka | Doporučení | Stav závaznosti a důvod |
|---|---|---|---|
| D-01 | Jazyk živého enginu | **Rust** | Ponechat; G-00 skeleton a binární BEAST/Mode-S práce; výkonnost ověřit |
| D-02 | Asynchronní runtime a HTTP | **Tokio + Axum** | Ponechat; modulární middleware, podporované SSE/HTTP |
| D-03 | Frontend framework | **Vite + React + TypeScript** | **Doporučená změna**, G-00 dnes běží na Next.js, zatím se nic nepřepisovalo |
| D-04 | Mapa | **MapLibre GL JS** | Ponechat; vrstvy a GeoJSON, testovat diff vs full source |
| D-05 | Databáze | **SQLite WAL preferovaný kandidát; PostgreSQL fallback** | **NEUZAVŘENO** do férového G-04 DB benchmarku a restore |
| D-06 | SQL vrstva | **SQLx** | Ponechat, použít pouze driver zvoleného DB |
| D-07 | ADS-B ingest | **readsb JSON referenční, BEAST volitelně preferovaný po paritě** | Ponechat obě aktivní funkce J01/J02, nebudovat dvě RAM autority |
| D-08 | Živý přenos | **SSE full + delta** | Ponechat, jeden sdílený producer a bounded queue |
| D-09 | Provoz | **Vlastní LXC, Rust + statický web + proxy** | Cíl, ale dosud bez provozního benchmarku a schváleného deploye |
| D-10 | JavaScript runtime | **Node 24 LTS pro stabilní build, CI Node 26 jako kompatibilitní matice** | Vite runtime Node nepotřebuje; k 21. 10. je Node 26 Current |
| D-11 | Další jazyky | **Nezavádět další runtime bez potřeby** | Go pouze jako výkonnostní spike; Python případně jednorázové fixture/import skripty |
| D-12 | Cloudové platformy, broker, Redis | **Nenasazovat** | V tomto produktu chybí aktivní konzument a měřený důvod |

## Doporučený finální technologický stack — pokud benchmarky nic nevyvrátí

~~~text
readsb JSON / BEAST
        ↓
Rust 2024 + Tokio + Axum (Mode-S, RAM state, REST, SSE)
        ↓                             ↘
Vite + React + TypeScript           SQLx → SQLite WAL
      ↓                                     
MapLibre GL JS                     PostgreSQL pouze při prokázané potřebě
      ↓
Statický build přes Nginx/Caddy v samostatném LXC
~~~

**Jasné rozhodnutí ke změně:** nepřidávat další Node server pro čistě mapově orientovaný web. Vite stačí pro dnešní funkcionalitu; případné SSR funkce se mohou znovu posoudit při výslovném schválení rozsahu.

**Podmíněné rozhodnutí:** SQLite by pro jednoho zapisujícího Rust vlastníka mohl být výrazně jednodušší na správu, ale není prokázáno, že bude vyhovovat zápisům historie, retenci a obnově. G-04 musí porovnat obě databáze a před migrací rozhodnutí zaznamenat.

## V čem se liší návrh od aktuálního repozitáře

| Dnes v \`main\` | Doporučená změna | Kdy |
|---|---|---|
| \`apps/web\` obsahuje Next.js 16 a React 19 | Převést na Vite/React s zachováním českého UI a všech smoke testů | Oddělený přípravný PR/commit před G-05 |
| \`README.md\` a \`AGENTS.md\` používají Next.js test/build povely | Aktualizovat až současně se skutečnou migrací | Ve stejném commitu, nikdy pouze v dokumentaci |
| PostgreSQL je v závazném návrhu považována za pevnou volbu | Nezapisovat data ani neprovádět migrace, dokud B05 nerozhodne SQLite/PostgreSQL | Před G-04 implementací |
| Rust engine pouze G-00 stub | Protokol a normalizace podle golden dat G-01/G-02 | G-01 / G-02 |
| SSE je zatím pouze zkušební stream | Full/delta včetně backpressure a resync | G-03 |
| Statický web není v LXC nasazen | V2-only reverse proxy a origin \`/api\` | G-12/G-13 se schválením |

## Co výslovně NEDĚLAT nyní

- Neměnit žádný LXC, reverse proxy, služby ani databázi V1.
- Neoznačit SQLite nebo Vite za „rychlejší o X %“ bez skutečných dat.
- Nepřidávat experimentální benchmarky a alternativní backends jako budoucí runtime produktové moduly; studie/benchmark artefakty musí zůstat izolované.
- Neměnit aktivní výběr 45 KEEP ani jediný archiv budoucích rozšíření.
- Neměnit vydanou produkční konfiguraci nebo začít posílat push notifikace.

## Technická rozhodovací brána před další implementací

1. Potvrdit referenční JSON/BEAST fixture a očekávanou paritu.
2. U webu vytvořit minimalistický **funkčně shodný** Vite/React spike, ověřit české UI, PWA a SSE. Pokud uspěje, migrovat Next.js v samostatném PR/commitu s build/lint/test/smoke.
3. Navrhnout minimalistický fyzický DB model jen pro skutečné aktivní dotazy a provést B05 SQLite/PostgreSQL.
4. Zapsat do ADR definitivní výsledek databáze a skutečný naměřený přínos, nebo explicitně ponechat PostgreSQL pokud SQLite nedoloží přínos.
5. Před produkcí provést B01–B07 měření na izolovaném V2 prostředí.

## Ověřené nezávislé zdroje

- [Vite produkční build](https://vite.dev/guide/build), [Next.js static export](https://nextjs.org/docs/app/guides/static-exports).
- [SQLite WAL](https://www.sqlite.org/wal.html), [SQLite kdy použít](https://www.sqlite.org/whentouse.html), [PostgreSQL PITR](https://www.postgresql.org/docs/current/continuous-archiving.html).
- [Axum](https://docs.rs/axum/latest/axum/), [SQLx](https://docs.rs/sqlx/latest/sqlx/).
- [readsb](https://github.com/wiedehopf/readsb), [readsb JSON](https://sources.debian.org/src/readsb/3.14.1630%2Bgit20240609.adc080d-1/README-json.md/).
- [MapLibre performance](https://maplibre.org/maplibre-gl-js/docs/guides/large-data/), [MDN SSE](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events).
- [Node.js release schedule](https://github.com/nodejs/Release), [Node 26.12.1 bezpečnostní vydání dne 21. 10. 2026](https://nodejs.org/en/blog/release/v26.12.1/).

**Nezpochybnitelný aktuální fakt:** repo už ověřilo pouze G-00 stub; výzkum dnes přináší podloženou architektonickou volbu, ale *žádný nový naměřený výkon*.
