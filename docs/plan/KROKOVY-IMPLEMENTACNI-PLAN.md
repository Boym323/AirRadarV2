# Podrobný krokový plán implementace AirRadar V2

**Datum:** 22. 10. 2026 · **Aktuální branch:** `main`  
**Rozsah:** přesně **45 aktivních produktových funkcí** v 9 logických modulech, bez automatického rozšiřování.  
**Technologie:** Rust/Tokio/Axum, Vite/React/TypeScript/MapLibre, PostgreSQL/SQLx na již provozovaném samostatném DB LXC.  
**Stav k 22. 10. 2026:** G-00 scaffold, G-01 HOTOVO 10/10, G-02 HOTOVO 11/11, G-03 ROZPRACOVÁNO 7/10; G-04 až G-13 NEZAHÁJENO. Tento plán jsou instrukce a odkazy na důkazy v etapových reportech.

## Implementační seznam

Každá etapa má vlastní detailní Markdown zadání s individuálními úkoly **G-XX.YY**, očekávaným výsledkem, důkazy PASS/FAIL a podmínkami převzetí.

| Etapa | Odkaz na konkrétní kroky | Kroků | Hlavní závislost | Výstup |
|---|---|---:|---|---|
| G-00 | [Ověřený základ](../reports/G-00-PRECHOD-VITE-POSTGRES-VYSLEDEK.md) | — | — | Rust + Vite scaffold, bez živých letadel |
| G-01 | [Kontrakty a fixtures](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) | **10** | G-00 | OpenAPI, normalizované typy, readsb/BEAST golden datasety |
| G-02 | [Rust readsb / BEAST](../ukoly/G-02-LOKALNI-INGEST.md) | **11** | G-01 | Jediný autoritativní RAM stav s přepínáním lokálních zdrojů |
| G-03 | [HTTP API a SSE](../ukoly/G-03-API-A-SSE.md) | **10** | G-01, G-02 | Verzované DTO, snapshot/delta, sekvence a backpressure |
| G-04 | [PostgreSQL historie](../ukoly/G-04-POSTGRESQL-HISTORIE.md) | **12** | G-02, G-03, schválený DB DEV přístup | Oddělená V2 DB, SQLx, vzorkování, retence a restore |
| G-05 | [Živá Vite / MapLibre mapa](../ukoly/G-05-ZIVA-MAPA.md) | **11** | G-03; letiště po G-06 | Symboly, seznam, výběr, trail, follow, podklady |
| G-06 | [Letiště a NAVAID/FIX](../ukoly/G-06-LETISTE-NAVAID.md) | **9** | G-01, G-03 | Legální zdroje, publikované frekvence a navigační katalog |
| G-07 | [Počasí](../ukoly/G-07-VYBRANE-POCASI.md) | **10** | G-02, G-03, G-05 | ČHMÚ radar, SIGMET a skutečná Mode-S meteorologie |
| G-08 | [Nouzové kódy a doručení](../ukoly/G-08-NOUZOVE-KODY-A-NOTIFIKACE.md) | **10** | G-02, G-03; SW vazba na G-10 | Validní emergency squawk, Web Push/Pushover pouze opt-in |
| G-09 | [Statistiky a zdraví systému](../ukoly/G-09-STATISTIKY-DIAGNOSTIKA.md) | **12** | Aktivní G-02 až G-08 | Lehké počítadla, logbook, runtime/DB metriky a SSE |
| G-10 | [PWA a české UI](../ukoly/G-10-PWA-LOKALIZACE.md) | **11** | G-05, API; Web Push vazba na G-08 | Mobilní rozhraní, instalace, offline stale, cs/en, hledání |
| G-11 | [Kompletní testovací přejímka](../ukoly/G-11-ZAVERECNE-TESTOVANI.md) | **12** | G-01 až G-10 | Důkazy všech 45 ID a výkon/bezpečnost GO/NO-GO |
| G-12 | [Oddělený shadow](../ukoly/G-12-ODDELENY-SHADOW.md) | **10** | G-11 GO a zvláštní souhlas | Izolovaný LXC, read-only replay/mirror, porovnání s V1 |
| G-13 | [Schválený release](../ukoly/G-13-BEZPECNY-RELEASE.md) | **12** | G-11/G-12 GO a explicitní schválení | Připravený V2 release, ověřený rollback, V1 beze změny |

**Celkem 140 implementačních a ověřovacích kroků** v etapách G-01 až G-13. Nejde o počet již dokončených úkolů ani odhad člověkodnů.

## Doporučené pořadí práce a sdílené kontrakty

```text
G-00 (hotový skeleton)
  └─ G-01 (kontrakty, fixture, akceptační scénáře)
       └─ G-02 (readsb/BEAST + RAM)
            └─ G-03 (REST/SSE)
                 ├─ G-04 (PostgreSQL historie na odděleném DB LXC)
                 ├─ G-05 (mapa bez letištní vrstvy)
                 └─ G-06 (letiště/NAVAID)
                        └─ dokončení letištní vrstvy G-05
                              ├─ G-07 (počasí)
                              ├─ G-08 (squawk/transport)
                              ├─ G-09 (diagnostika a statistiky)
                              └─ G-10 (PWA, offline, cs/en)
                                    └─ společný service worker pro G-08/G-10
                                         └─ G-11 (kompletní GO/NO-GO)
                                              └─ G-12 (schválený izolovaný shadow)
                                                   └─ G-13 (samostatně schválený release)
```

**Poznámky k návaznostem:**
- V G-04 neinstalovat vlastní PostgreSQL do aplikačního LXC: použít **existující separátní databázový LXC**, vlastní V2 DB, odlišné role a šifrované připojení. Založení/DDL až po ověření správcem.
- G-05 může začít hned po G-03, ale bod mapové vrstvy letišť čeká na G-06. Nejde o chybu etap.
- G-07 čte meteorologické údaje ze skutečných podkladů. Nezíská-li licenci nebo validní data, označí dílčí bod BLOKOVÁNO, nikdy si data nevymyslí.
- G-08 lze začít před G-10 jako backend dry-run; Web Push service worker dokončit koordinovaně s G-10.
- G-09 měří skutečný Rust/SQLx runtime; Node.js se používá **jen při buildu statického webu**, nikoliv jako trvalá serverová služba.
- Testy v G-11, shadow v G-12 a nasazení G-13 vyžadují explicitní důkazy a provozní GO, nejde o automatický CI deploy.

## Pravidla pro jednotlivé implementační PR

1. **Začátek:** přečíst `AGENTS.md`, tuto stránku, aktivní matici a jediné konkrétní zadání `G-XX`; ověřit přesný `main` a aktuální CI.
2. **Rozsah:** uvést konkrétní krok `G-XX.YY`, primární aktivní ID a očekávané chování; nepropojovat nesouvisející moduly.
3. **Kód:** přidat nejmenší nutnou Rust/Vite/SQLx část. Každá nová DB tabulka/worker/source musí mít schválený aktivní konzument.
4. **Test:** současně s funkcí napsat golden, unit, integration nebo E2E dle etapy; zapsat příkaz, vstup, výsledek PASS/FAIL a výkonovou metodu u výkonových změn.
5. **Bezpečnost:** nepoužívat data V1 pro zápis, veřejné secrets, přístup do ostatních DB ani skutečné Push/Pushover v replay/shadow.
6. **Doklady:** u skutečně dokončeného kroku vyplnit checkbox až po spojení do schváleného kódu a odkazu na důkaz. Aktualizovat `docs/reports/G-XX-VYSLEDEK.md`, `CHANGELOG.md` a `ETAPY-A-STAV.md`; aktivní funkci označit jako TESTED/ACCEPTED pouze při dokončení jejího scénáře.
7. **Kontroly:** Rust `cargo fmt --all -- --check`, Clippy, `cargo test --locked --workspace`; web `npm ci`, lint, typecheck, Vitest, Vite build; `node tools/check-docs.mjs`; CI finálního SHA.
8. **Závěr:** uvedený commit, změněné soubory, konkrétní testy, výkon před/po pokud měřen, známá rizika a následující krok.

## Evidence stavů

| Úroveň | Stav | Kdy se smí změnit |
|---|---|---|
| Jednotlivý krok | `[ ]` nebo `[x]` | `[x]` pouze po skutečném kódu, testu a odkazu na důkaz |
| Etapa | NEZAHÁJENO → ROZPRACOVÁNO → TESTOVÁNO → HOTOVO | Po ověřeném PR a finálním reportu |
| Funkce v `contracts/feature-matrix.json` | NOT_STARTED → IN_PROGRESS → TESTED → ACCEPTED | Dle konkrétního funkčního testu a schválené akceptace |
| G-11 až G-13 | NO-GO/BLOKOVÁNO nebo GO | Bez skutečné evidence nesmí být GO |

**Pozor:** G-01 připraví kontrakty pro 45 funkcí, ale nesmí tím označit žádnou produktovou funkci jako implementovanou. Z G-11 nevzniká právo provést G-12 či G-13 bez provozního souhlasu.

## Nejbližší krok

**G-03.06–.10 — Dokončit lifecycle, backpressure, proxy a měření.** Navazuj v [G-03](../ukoly/G-03-API-A-SSE.md) a ověř CI finálního SHA. Celý projekt lze převzít v novém chatu z [průvodce](ZACNI-ZDE-NOVE-VLAKNO.md). Samotný zápis tohoto dokumentu nemění funkční stav aplikace.

## Detailní technické kontrakty před realizací

Podklad pro G-01 až G-11: [PostgreSQL, readsb/BEAST, SSE, MapLibre, Playwright a propojení LXC](TECHNICKE-KONTRAKTY-A-LIMITY.md). Jde výlučně o **návrhové parametry s měřicími kritérii**, nikoli o implementovaný produktový modul. Počet a pořadí kroků G-XX.YY se nemění.
