# G-00 — Přechod na Vite a volba PostgreSQL

**Stav:** HOTOVO pro migraci G-00 webu a volbu DB technologie. G-04 vytvoření V2 databáze zatím NEZAHÁJENO.

- Odstraněn Next.js webový skeleton, nahrazen českým Vite/React/TypeScript webem a statickým `dist/`.
- Vite dev proxy `/api` do Rust enginu, nová konfigurace a React SSR unit test pro obsah.
- [CI 39678151169](https://github.com/Boym323/AirRadarV2/actions/runs/39678151169): Rust, Vite lint, typecheck, Vitest, build, HTTP smoke a česká dokumentace úspěšné.
- PostgreSQL na existujícím separátním DB LXC je pevná volba; DB, role, SQLx migrace ani síť nebyly upraveny.
- Žádný V1 zásah, produkční nasazení, živý ingest ani skutečná mapa.

**Další krok:** finální `npm ci` CI a pak G-01.

## Reprodukovatelný finální build

- [GitHub Actions 39679601910](https://github.com/Boym323/AirRadarV2/actions/runs/39679601910) na commitu `8c68508c811fbd2fa0a6c45dee4f2b4d8fcc3910`: úspěšné **npm ci**, ESLint, TypeScript, Vitest, statický Vite build, HTTP smoke, Rust testy a dokumentace.
- Uzamčená verze Vite 8.3.5, React plugin 6.1.2; Next.js a eslint-config-next nejsou v uzamčených závislostech.
- Živý readsb, mapa, PostgreSQL DB připojení ani release se tímto netestovaly.
