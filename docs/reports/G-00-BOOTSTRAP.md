# G-00 — výsledný report založení AirRadar V2

**Stav kódu a CI: HOTOVO.**  
**Samostatný nový LXC: ZATÍM NEKONFIGUROVÁN.**  
**Živý radar a funkce G-01+: NEIMPLEMENTOVÁNY.**

## Ověřené výsledky

- Repozitář: https://github.com/Boym323/AirRadarV2
- Ověřený původní commit: `99df0456c4e57b8192db9b41020453abcd14c8f6`.
- [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202) — **obě úlohy úspěšné**, Rust i Next.js.
- Testováno: připnuté `Cargo.lock` a `package-lock.json`, Rust fmt/Clippy/testy, Next.js lint/typecheck/testy/build, reálný HTTP smoke obou spuštěných komponent.
- Předchozí běh `39657981782` selhal na Rust formátování a byl opraven. Následné běhy `39658242976` a `39658530749` prošly.
- Nové české dokumentační a UI změny budou mít vlastní kontrolní běh CI; historické PASS nesmí být vydáváno za důkaz jejich aktuálního SHA.

## Skutečně vytvořeno

Samostatný GitHub repo, Rust/Tokio/Axum HTTP základ (`health`, neimplementovaná letadla a testovací SSE s heartbeat), statický úvodní Next.js web, OpenAPI, funkční matice 79 KEEP / 99 REMOVE, připnuté závislosti a CI **bez deploye**.

## Co zatím neexistuje

Živý příjem readsb/BEAST/ADS-B, samostatné RAM tracks, skutečná mapa, PostgreSQL schema, letová historie, reálné notifikace, veřejná doména, systemd služby a produkční deployment.

Současné API záměrně vrací `NOT_IMPLEMENTED`. Výsledek G-00 **není funkční letecký radar**.

## Oddělení od V1

Původní kód, CI, PostgreSQL, reverse proxy a starý LXC nebyly změněny. V2 bude mít vlastní nový LXC, databázi, oprávnění a samostatné release postupy.

## Navazující rozhodnutí

- **PŘIPRAVENO** k realizaci českého zadání [G-01](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md).
- **NENÍ PŘIPRAVENO** k produkčnímu provozu či nahrazení V1.
- Příprava LXC čeká na zřízení a autorizovaný přístup; není nutná pro práci na G-01.
- Zbývá formální rozhodnutí o licenci před veřejným vydáním distribuovaných buildů.

