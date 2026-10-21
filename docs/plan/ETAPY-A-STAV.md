# Etapy AirRadar V2 a aktuální stav

**Datum založení:** 21. 10. 2026. **Projektový jazyk: čeština.**  
**Zadání:** [ZADANI-AIRRADAR-V2.md](ZADANI-AIRRADAR-V2.md) · **matice:** [MATICE-FUNKCI.md](MATICE-FUNKCI.md)  
**Pravidlo:** stav měň až po doložení testů; průběžná práce a dílčí úspěch nejsou dokončená etapa.

| Etapa | Samostatné zadání a přejímka | Stav |
|---|---|---|
| G-00 | Založení samostatného repozitáře, Rust/Next.js, kontrakty, lockfily, CI, HTTP smoke | **HOTOVO (základ kódu)** |
| G-01 | [Datové kontrakty a deterministické testovací vstupy](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) | **NEZAHÁJENO** |
| G-02 | Rust ingest readsb/BEAST, CRC, normalizace a autoritativní RAM | NEZAHÁJENO |
| G-03 | ADSBHub, ADSB.lol, LOCAL/EXTENDED, merge a kontinuita | NEZAHÁJENO |
| G-04 | Verzované API a SSE full/delta, backpressure, heartbeat a cleanup | NEZAHÁJENO |
| G-05 | Nová PostgreSQL, SQLx, dávkové vzorkování, retence a statistiky | NEZAHÁJENO |
| G-06 | Next.js / MapLibre interaktivní mapa, detail letadla, mobilní UX | NEZAHÁJENO |
| G-07 | Letiště, ATC/ATS/NAVAID, prostorová geometrie, Navigation Integrity | NEZAHÁJENO |
| G-08 | METAR/TAF, ČHMÚ radar, SIGMET a palubní počasí | NEZAHÁJENO |
| G-09 | OGN/FLARM, DDB/SoftRF, RXW a obohacující providery | NEZAHÁJENO |
| G-10 | Admin, zabezpečení, Pushover a Web Push | NEZAHÁJENO |
| G-11 | Denní a časové statistiky, coverage, monitoring a T58 | NEZAHÁJENO |
| G-12 | UI dokončení, PWA, offline cache, klávesové zkratky, volitelná EN lokalizace | NEZAHÁJENO |
| G-13 | Závěrečná akceptace všech 79 KEEP a bezpečnostní i výkonnostní testy | NEZAHÁJENO |
| G-14 | Paralelní, neodesílající shadow provoz ve vlastním LXC | NEZAHÁJENO |
| G-15 | Výslovně schválený produkční přechod V2 a rollback | NEZAHÁJENO |

## G-00 — existující ověřené důkazy

- Základ Rust a Next.js, lockfily a CI byly ověřeny na commitu `99df0456`.
- [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202): úspěšné joby Rust a Next.js, včetně HTTP smoke.
- Detail: [report G-00](../reports/G-00-BOOTSTRAP.md).
- Nový LXC ještě **nebyl z tohoto repozitáře vytvořen ani nakonfigurován**; živé ADS-B a databáze neexistují.
- Změny českých textů a dokumentace vyžadují nový běh CI na konkrétním commitu.

## Jak udržovat stav

1. Před etapou otevři její Markdown soubor v `docs/ukoly/`; není-li založen, nejdříve jej vytvoř z [šablony](../sablony/ZADANI-ETAPY.md).
2. Po každé ucelené změně doplň výsledky a odkazy na testy. Konečný report vytvoř podle [šablony](../sablony/REPORT-ETAPY.md).
3. Funkční změny aktualizuj v `contracts/feature-matrix.json` a vygeneruj [MATICE-FUNKCI.md](MATICE-FUNKCI.md) pomocí `node tools/generate-feature-matrix.mjs`.
4. Stav **HOTOVO** stanov pouze po splnění akceptačních kritérií, testech a kontrole skutečného SHA.
5. **BLOKOVÁNO** napiš otevřeně s důvodem a dopadem; nic nepovoluj jen kvůli zelenému CI.

**Produkční V1:** beze změn. **V2:** stále bez produkčního nasazení.

