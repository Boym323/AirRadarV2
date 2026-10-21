# G-00 — Aktualizace technologického stacku (údržba)

## Rozsah a hranice
Aktualizovat existující G-00 scaffold na Node.js 26, npm 11, současný Next.js/React, TypeScript, ESLint a Vitest. Neměnit Rust API, databázi, ADS-B přijímač ani produkční AirRadar V1.

## Závislosti
Aktuální matice má 45 schválených funkcí; stav žádné funkce se nemění. GitHub CI používá stejný Node jako vývojář. Jednorázový workflow pro lockfile se musí před dokončením odstranit.

## Scénáře ověření
1. Na Node 26/npm 11 generovat aktualizovaný `package-lock.json` a ověřit `npm ci`.
2. `npm run lint`, `npm run typecheck`, `npm test`, `npm run build`, HTTP smoke.
3. `node tools/check-docs.mjs` a kontrola matice 45 ID.
4. `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`, Rust API smoke.
5. Aktualizovat changelog a český report s přesným SHA a odkazem na CI.

## Výsledek
**ČÁSTEČNĚ** do finální úspěšné CI. Report: [G-00-AKTUALIZACE-STACKU-VYSLEDEK.md](../reports/G-00-AKTUALIZACE-STACKU-VYSLEDEK.md).
