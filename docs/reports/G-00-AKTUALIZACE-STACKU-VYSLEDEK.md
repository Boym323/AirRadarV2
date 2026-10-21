# G-00 — Výsledek aktualizace Node.js 26

**Stav: ČÁSTEČNĚ — čeká na finální CI.** Žádná nová funkce nebyla implementována.

## Základ a změny
- Výchozí SHA `e4b97d98e02bc76717e49479a9b184f7e0147628`.
- Výchozí [CI 39671307056](https://github.com/Boym323/AirRadarV2/actions/runs/39671307056): Rust a web úspěšné; kontrola dokumentace selhala kvůli odstraněným názvům etap.
- Vývojový runtime Node.js 26.12.0, npm 11.20.0; Next.js 16.4.2, React 19.3.1, TypeScript 6.0.3 (vynuceno kompatibilitou typescript-eslint), ESLint 10.13.0, Vitest 5.0.5.
- **Ověřeno:** automatický [běh regenerace 39672495568](https://github.com/Boym323/AirRadarV2/actions/runs/39672495568) na Node.js 26 dokončil `npm install --package-lock-only` a čisté `npm ci` úspěšně; lockfile byl zapsán do větve.
- Opravena CI kontrola dokumentace pro 45 aktivních funkcí a G-00 až G-13.

## Ověření a omezení
- [CI 39673071350](https://github.com/Boym323/AirRadarV2/actions/runs/39673071350) odhalila blokující nekompatibilitu typescript-eslint s TypeScript 7.0; proto návrat na podporovanou větev 6.0.3. Finální lint, typy, testy, build, Rust a HTTP: **zatím neověřeno**, vyžaduje nové CI.
- Žádné výkonové měření; live data flow není dosud implementován.
- Produkční nasazení ani změny AirRadar V1 nejsou součástí.
