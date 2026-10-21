# G-00 — Přechod na Vite a volba PostgreSQL

**Stav:** implementace webu ověřena v prvním CI, finální připnuté npm ci čeká na kontrolu.

- Odstraněn Next.js webový skeleton, nahrazen českým Vite/React/TypeScript webem a statickým `dist/`.
- Vite dev proxy `/api` do Rust enginu, nová konfigurace a React SSR unit test pro obsah.
- [CI 39678151169](https://github.com/Boym323/AirRadarV2/actions/runs/39678151169): Rust, Vite lint, typecheck, Vitest, build, HTTP smoke a česká dokumentace úspěšné.
- PostgreSQL na existujícím separátním DB LXC je pevná volba; DB, role, SQLx migrace ani síť nebyly upraveny.
- Žádný V1 zásah, produkční nasazení, živý ingest ani skutečná mapa.

**Další krok:** finální `npm ci` CI a pak G-01.
