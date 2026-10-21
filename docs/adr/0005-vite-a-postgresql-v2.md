# ADR 0005 — Vite a PostgreSQL v odděleném LXC

**Datum:** 21. 10. 2026. **Stav:** schváleno vlastníkem.

## Rozhodnutí

Web AirRadarV2 je statická aplikace **Vite + React + TypeScript**, bez produkčního Next.js/Node serveru. Rust/Tokio/Axum poskytuje REST/SSE a Vite dev server proxy na `/api`. Databáze je výhradně **PostgreSQL/SQLx** na již existujícím samostatném databázovém LXC. Nová V2 DB, účet a migrace teprve v G-04.

## Důvody

Mapová aplikace nepotřebuje SSR. Existující PostgreSQL LXC poskytuje preferované oddělené úložiště bez zavádění další DB technologie.

## Bezpečnost

Jen vlastní role/databáze V2, minimální práva, privátní firewall, TLS s ověřením serveru, žádné credentials ve frontend buildu, žádný přenos historie V1.

## Ověření

První migrační [CI 39678151169](https://github.com/Boym323/AirRadarV2/actions/runs/39678151169) prošel. Finální čisté `npm ci` na přesném SHA se doloží v reportu. Skutečné PostgreSQL připojení ani migrace dosud neproběhly.

[Runbook](../../database/POSTGRES-LXC.md).
