# Výsledná technologická rozhodnutí — AirRadar V2

**Posouzení:** 21. 10. 2026.  
**Stav:** rešerše dokončena; **Vite + React bylo následně skutečně nasazeno do G-00 skeletonu a PostgreSQL definitivně zvolena**. Výkonové benchmarky zůstávají neprovedeny; technologie se nesmějí prohlásit za kvantitativně nejrychlejší bez benchmarku.  
**45 aktivních funkcí** neměníme; neobnovujeme žádné odložené moduly.

## Rozhodovací tabulka

| ID | Otázka | Doporučení | Stav závaznosti a důvod |
|---|---|---|---|
| D-01 | Jazyk živého enginu | **Rust** | Ponechat; G-00 skeleton a binární BEAST/Mode-S práce; výkonnost ověřit |
| D-02 | Asynchronní runtime a HTTP | **Tokio + Axum** | Ponechat; modulární middleware, podporované SSE/HTTP |
| D-03 | Frontend framework | **Vite + React + TypeScript** | **PROVEDENO** v G-00; statický Vite + React build |
| D-04 | Mapa | **MapLibre GL JS** | Ponechat; vrstvy a GeoJSON, testovat diff vs full source |
| D-05 | Databáze | **PostgreSQL na existujícím DB LXC** | **SCHVÁLENO VLASTNÍKEM**; G-04 ověří role, výkon a zálohy |
| D-06 | SQL vrstva | **SQLx** | Ponechat pouze PostgreSQL driver |
| D-07 | ADS-B ingest | **readsb JSON referenční, BEAST volitelně preferovaný po paritě** | Ponechat obě aktivní funkce J01/J02, nebudovat dvě RAM autority |
| D-08 | Živý přenos | **SSE full + delta** | Ponechat, jeden sdílený producer a bounded queue |
| D-09 | Provoz | **Vlastní LXC, Rust + statický web + proxy** | Cíl, ale dosud bez provozního benchmarku a schváleného deploye |
| D-10 | JavaScript runtime | **Node 26/npm 11 pro reprodukovatelný Vite build** | Statický web nepotřebuje běžící Node na LXC |
| D-11 | Další jazyky | **Nezavádět další runtime bez potřeby** | Go pouze jako výkonnostní spike; Python případně jednorázové fixture/import skripty |
| D-12 | Cloudové platformy, broker, Redis | **Nenasazovat** | V tomto produktu chybí aktivní konzument a měřený důvod |

## Závazný stack po rozhodnutí vlastníka

~~~text
readsb JSON / BEAST → Rust/Tokio/Axum (RAM, REST, SSE)
                        ├──→ Vite + React + TypeScript + MapLibre (statický web)
                        └──→ SQLx → PostgreSQL (existující separátní DB LXC)
~~~

**G-00 web byl migrován a testován v CI.** PostgreSQL je volba vlastníka, nikoli výkonové tvrzení o SQLite. Samostatnou V2 DB, role, izolaci LAN a autentizaci SCRAM bez TLS (ADR 0009) a obnovu je potřeba ověřit při G-04; na existujícím databázovém LXC jsme neprováděli žádné zásahy.

Historické alternativy zůstávají jako [výzkumná evidence](TECHNOLOGICKY-PRUZKUM.md), ale již nejsou aktivními implementačními variantami.

## Co výslovně NEDĚLAT nyní

- Neměnit žádný LXC, reverse proxy, služby ani databázi V1.
- Neoznačit PostgreSQL nebo Vite za „rychlejší o X %“ bez skutečných dat.
- Nepřidávat experimentální benchmarky a alternativní backends jako budoucí runtime produktové moduly; studie/benchmark artefakty musí zůstat izolované.
- Neměnit aktivní výběr 45 KEEP ani jediný archiv budoucích rozšíření.
- Neměnit vydanou produkční konfiguraci nebo začít posílat push notifikace.

## Technická rozhodovací brána před další implementací

1. Potvrdit referenční JSON/BEAST fixture a očekávanou paritu.
2. G-00 Vite web je migrován; MapLibre, PWA a SSE integrační chování testovat v příslušných etapách, nikoliv vytvářet další Next.js implementaci.
3. Navrhnout PostgreSQL schéma pro aktivní dotazy a změřit na izolované DEV DB zápisy, indexy, síťovou izolaci, SCRAM bez TLS dle ADR 0009 a obnovu.
4. V G-04 bezpečně připravit novou databázi/role V2 a reportovat skutečné naměřené limity PostgreSQL.
5. Před produkcí provést B01–B07 měření na izolovaném V2 prostředí.

## Ověřené nezávislé zdroje

- [Vite produkční build](https://vite.dev/guide/build), [Next.js static export](https://nextjs.org/docs/app/guides/static-exports).
- [SQLite WAL](https://www.sqlite.org/wal.html), [SQLite kdy použít](https://www.sqlite.org/whentouse.html), [PostgreSQL PITR](https://www.postgresql.org/docs/current/continuous-archiving.html).
- [Axum](https://docs.rs/axum/latest/axum/), [SQLx](https://docs.rs/sqlx/latest/sqlx/).
- [readsb](https://github.com/wiedehopf/readsb), [readsb JSON](https://sources.debian.org/src/readsb/3.14.1630%2Bgit20240609.adc080d-1/README-json.md/).
- [MapLibre performance](https://maplibre.org/maplibre-gl-js/docs/guides/large-data/), [MDN SSE](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events).
- [Node.js release schedule](https://github.com/nodejs/Release), [Node 26.12.1 bezpečnostní vydání dne 21. 10. 2026](https://nodejs.org/en/blog/release/v26.12.1/).

**Nezpochybnitelný aktuální fakt:** repo už ověřilo pouze G-00 stub; výzkum dnes přináší podloženou architektonickou volbu, ale *žádný nový naměřený výkon*.
