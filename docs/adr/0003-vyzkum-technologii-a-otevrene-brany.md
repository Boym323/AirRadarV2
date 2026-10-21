# ADR 0003 — Technologické hodnocení a rozhodovací brány V2

**Datum:** 21. 10. 2026. **Stav:** doporučeno; dosud neimplementováno, bez vlastního benchmarku.

## Kontext

V2 má 45 aktivních funkcí, místní readsb/BEAST příjem, Rust engine a mapově orientovaný web. G-00 je pouze Rust/Axum a Next.js skeleton. Původní architektura předpokládala Next.js runtime a PostgreSQL bez porovnání se statickým webem nebo embedded databází.

## Rozhodnutí

1. **Rust + Tokio + Axum** zůstávají zvoleným jádrem. Go je věrohodná alternativa pro případ, že férové měření a produktivita změnu odůvodní.
2. **Vite + React + TypeScript + MapLibre** je preferovaným cílem webu místo běžícího Next.js serveru. Převod vyžaduje samostatně testovaný commit s PWA, i18n, bezpečným SSE a CI. Tento ADR sám web nemigruje.
3. **SQLite WAL** je preferovaný *kandidát* pro jediného lokálního zapisovače; **PostgreSQL** zůstává rovnocenně testovanou alternativou. V G-04 se nejprve porovná insert/čtení/retence/backup a teprve pak se vybere engine.
4. Preferovat jednosměrné **SSE**, readsb JSON jako referenční výchozí cestu a BEAST jako nízkolatenční cestu po prokázané paritě.
5. Nezavádět broker, Redis, Kubernetes, další běžící Node proces ani jiné runtime služby bez konkrétní aktivní funkce a měřeného přínosu.

## Důkazy a omezení

- [Průzkum se zdroji](../plan/TECHNOLOGICKY-PRUZKUM.md)
- [Konkrétní cílová architektura](../plan/ARCHITEKTURA-V2.md)
- [Reprodukovatelné benchmarky](../plan/BENCHMARKY-TECHNOLOGII.md)
- [Rozhodovací tabulka](../plan/TECHNOLOGICKA-ROZHODNUTI.md)

**Žádná čísla výkonu nebyla naměřena v AirRadar V2.** Vite a SQLite nejsou zatím nasazeny, Node 26 je k dnešku Current s plánovaným přechodem na LTS 28. října 2026. Jakákoli změna produkce V1 nebo V2 LXC vyžaduje zvláštní povolení.

## Důsledky

Přínos: méně provozních služeb, jednodušší statická distribuce, snazší izolace web/Rust a možnost jednodušší DB. Rizika: absence SSR, nutnost browser-side routing/auth/PWA a možné WAL checkpoint stalls SQLite. Tyto nevýhody musí být testovány na stejném zátěžovém profilu. Selže-li jednoduchá varianta, zaznamenat přesné důvody pro ponechání Next.js nebo PostgreSQL.
