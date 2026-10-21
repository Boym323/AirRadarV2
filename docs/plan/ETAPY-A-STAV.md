# Etapy AirRadar V2 a aktuální stav

**Aktualizace rozsahu: 21. 10. 2026.** **46 KEEP / 132 REMOVE**, z toho **L06 navráceno** a 34 původních KEEP vyřazeno. Žádné nové funkce zatím nebyly implementovány. Výchozí jazyk čeština.

| Etapa | Zadání | Stav |
|---|---|---|
| G-00 | Založení samostatného projektu Rust/Next.js, CI a smoke | **HOTOVO (pouze scaffold)** |
| G-01 | [Kontrakty a fixture](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) pro 46 KEEP | NEZAHÁJENO |
| G-02 | [Rust příjem readsb/BEAST a stav letadel](../ukoly/G-02-LOKALNI-INGEST.md) | NEZAHÁJENO |
| G-03 | [Původní externí ADS-B zdroje — ZRUŠENO](../ukoly/G-03-EXTERNI-ZDROJE-ZRUSENO.md) | **VYŘAZENO / NEREALIZOVAT** |
| G-04 | [Veřejné API a SSE snapshot/delta](../ukoly/G-04-API-A-SSE.md) | NEZAHÁJENO |
| G-05 | [Nová PostgreSQL a úsporné vzorkování](../ukoly/G-05-POSTGRESQL-HISTORIE.md) | NEZAHÁJENO |
| G-06 | [Interaktivní MapLibre mapa se základními prvky](../ukoly/G-06-ZIVA-MAPA.md) | NEZAHÁJENO |
| G-07 | [Katalog letišť a NAVAID/FIX](../ukoly/G-07-LETISTE-NAVAID.md) | NEZAHÁJENO |
| G-08 | [ČHMÚ radar, SIGMET a Mode-S meteorologie](../ukoly/G-08-VYBRANE-POCASI.md) | NEZAHÁJENO |
| G-09 | [Původní OGN a externí obohacení — ZRUŠENO](../ukoly/G-09-OGN-INTEGRACE-ZRUSENO.md) | **VYŘAZENO / NEREALIZOVAT** |
| G-10 | [Emergency squawk 7500/7600/7700 a notifikace](../ukoly/G-10-NOUZOVE-KODY-A-NOTIFIKACE.md) | NEZAHÁJENO |
| G-11 | [Základní statistiky, logbook a systémový dohled](../ukoly/G-11-STATISTIKY-DIAGNOSTIKA.md) | NEZAHÁJENO |
| G-12 | [Mobilní UX, PWA, offline cache a lokalizace](../ukoly/G-12-PWA-LOKALIZACE.md) | NEZAHÁJENO |
| G-13 | [Akceptace 46 KEEP a bezpečnostní brány](../ukoly/G-13-ZAVERECNE-TESTOVANI.md) | NEZAHÁJENO |
| G-14 | [Neovlivňující shadow test V2](../ukoly/G-14-ODDELENY-SHADOW.md) | NEZAHÁJENO |
| G-15 | [Manuálně schválené nasazení V2 s návratem](../ukoly/G-15-BEZPECNY-RELEASE.md) | NEZAHÁJENO |

## Co se změnilo vůči původnímu plánu

- **G-03** externí ADS-B a **G-09** OGN/integrace jsou nyní **bez realizace**; nejsou to chyby či blokéry CI.
- G-05 zůstává kvůli J09, ale bez profilů H07/H14 a S02/S03/S04.
- G-06 nemá R05/R07/R10/R11/R15/R17/R18; G-07 nemá ATS/ATC/Navigation Integrity; G-08 nemá METAR/TAF.
- G-10 nově obsahuje **L06**; L11/L12 zůstávají jako doručovací možnosti. Veškeré skutečné odesílání musí být odděleně povolené.
- G-13 (testy), G-14 (shadow) a G-15 (kontrolovaný release) jsou **interní bezpečnostní brány** i po vyřazení O09/O10/O11/O12 jako produktových funkcí.
- Veřejná část V2 nemá administrační login O15. Přístupová ochrana neveřejných změn je základní bezpečnostní požadavek, nikoli samostatná produktová funkce.

## Již doložené

- G-00 původní Rust/Next.js scaffold s připnutými závislostmi: [GitHub CI 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202).
- Pozdější CI validovalo také české dokumenty, ale **není důkazem provozu živých letadel**.
- Stav každého funkčního KEEP ID je v [matici](MATICE-FUNKCI.md); stav všech 46 je zatím `NOT_STARTED`.
- Nový LXC není v tomto repozitáři potvrzen jako nakonfigurovaný. Stará produkční V1 nedotčená.

## Postup údržby

Před novou etapou čti `AGENTS.md`, [hlavní zadání](ZADANI-AIRRADAR-V2.md), [moduly](MODULY-A-ROZSAH.md) a příslušný jediný `.md` úkol. Po realizaci uveď skutečné testy v českém reportu `docs/reports/G-XX-VYSLEDEK.md`, aktualizuj JSON matici a vygeneruj MD pomocí `node tools/generate-feature-matrix.mjs`, dále uprav `CHANGELOG.md`. Žádnou funkcionalitu bez důkazů nepřidávej do hotových.
