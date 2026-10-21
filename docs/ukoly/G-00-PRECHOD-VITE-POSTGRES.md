# G-00 — Přechod na Vite a definitivní PostgreSQL

**Stav:** HOTOVO pro vývojový skeleton (web a dokumentace), bez deploye.
**Rozsah:** Převést Next.js na Vite + React/TypeScript, zachovat české `lang=cs`, všechny dosavadní G-00 testy a oddělení od Rust enginu. PostgreSQL na existujícím separátním DB LXC definitivně stanovit jako cílové úložiště bez změn serveru.

## Přejímka

- Next.js runtime a konfigurace jsou odstraněné, `npm ci` používá připnutý `package-lock.json`.
- `npm run lint`, `npm run typecheck`, `npm test`, `npm run build` a HTTP smoke na `vite preview` jsou zelené.
- `apps/web/dist/` je statický výstup, nikoliv produkční Vite preview server.
- `/api` v dev serveru má proxy na lokální Rust API; na produkci proxy patří do Nginx/Caddy.
- PostgreSQL DB a účet V2 budou založeny v G-04 s minimálními právy, TLS, síťovou izolací a backup/restore testem.
- Dokumenty, ADR, changelog a CI odpovídají kódu; 45 aktivních funkcí beze změny.
- Nesmí se měnit produkční V1, její LXC a historie ani existující DB jiné aplikace.

**Důkazy:** [první CI pro Vite](https://github.com/Boym323/AirRadarV2/actions/runs/39678151169), [finální lockfile/npm ci CI](https://github.com/Boym323/AirRadarV2/actions/runs/39679601910), [závěrečný report](../reports/G-00-PRECHOD-VITE-POSTGRES-VYSLEDEK.md).
