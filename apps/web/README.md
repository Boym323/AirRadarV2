# Web AirRadar V2 — Vite + React

Statický React/TypeScript klient pro **místní** radar. Výchozí jazyk `cs`. V G-00 jde jen o uvítací stránku, nikoliv o živou mapu.

## Vývoj

Na Node.js 26 a npm 11:

~~~bash
cd apps/web
npm ci
npm run dev
~~~

Web: `http://127.0.0.1:4100`. Cesty `/api/*` jsou v **dev serveru** předávány na Rust `http://127.0.0.1:4101`. Tajemství nikdy nedávat do `VITE_*`, protože se zabudují do veřejného JavaScriptu.

~~~bash
npm run lint
npm run typecheck
npm test
npm run build
npm run preview
~~~

`npm run build` vytváří **`apps/web/dist/`**, který může v produkci poskytovat Nginx/Caddy. `vite preview` je pouze vývojové ověření výsledku, **není produkční webový server**.

Provoz přes samostatný V2 LXC musí mít HTTPS reverse proxy se `/api/` a SSE bez bufferování. Není potřeba dlouhodobý Node proces. Připojení k databázi patří výhradně do Rust backendu, nikoliv do prohlížeče.

Související [technologické rozhodnutí](../../docs/plan/TECHNOLOGICKA-ROZHODNUTI.md).
