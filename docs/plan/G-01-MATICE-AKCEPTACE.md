# G-01 — akceptační matice všech 45 funkcí

**Stav:** pouze scénáře budoucích etap. Převzetí G-01 **neznamená** implementaci následujících funkcí. Jediný zdroj pravdy o skutečném statusu: [feature-matrix.json](../../contracts/feature-matrix.json); tato tabulka eviduje požadované testy pro G-02 až G-10. Vlastnictví: [moduly](MODULY-A-ROZSAH.md).

| ID | Modul | Vlastní etapa | Budoucí PASS/FAIL scénář | Implementace |
|---|---|---|---|---|
| **J01** | M01 | G-02 | Validovat snapshot, chybějící pole, hodiny a restart readsb | Plán — neověřeno |
| **J02** | M01 | G-02 | Validovat úplné/rozseknuté BEAST rámce, escape a CRC | Plán — neověřeno |
| **J03** | M01 | G-02 | Zlaté CRC24 DF17 proti chybnému DF/paritě | Plán — neověřeno |
| **J04** | M01 | G-02 | BEAST→JSON→BEAST bez duplicit a teleportů | Plán — neověřeno |
| **J05** | M01 | G-02 | Jedna bounded RAM identita přes dvě lokální cesty | Plán — neověřeno |
| **J06** | M02 | G-03 | Veřejný Aircraft DTO bez receiverId/secrets | Plán — neověřeno |
| **J07** | M02 | G-03 | Po připojení přichází atomický snapshot a heartbeat | Plán — neověřeno |
| **J08** | M02 | G-03 | Delta seq/epoch, gap a restart vyžaduje resync | Plán — neověřeno |
| **J09** | M03 | G-04 | Jen nová časově validní poloha do SQL, idempotentní bucket | Plán — neověřeno |
| **J11** | M01 | G-02 | ICAO a non-ICAO oddělené, kolize, TTL a staleness | Plán — neověřeno |
| **R01** | M04 | G-05 | MapLibre zobrazí validní symbol a safe fallbacks | Plán — neověřeno |
| **R03** | M04 | G-05 | Rotace a typ ikony podle známých dat, unknown fallback | Plán — neověřeno |
| **R04** | M04 | G-05 | Čitelný popisek a potlačení kolizí při 500+ cílech | Plán — neověřeno |
| **R06** | M04 | G-05 | Aktualizace seznamu a filtrování po delta/removed | Plán — neověřeno |
| **R08** | M04 | G-05 | RAM stopa respektuje 20min/64 bodů bez DB v ticku | Plán — neověřeno |
| **R09** | M04 | G-05 | Follow a přerušení ruční interakcí, bez nuceného zoomu | Plán — neověřeno |
| **R12** | M04 | G-05 | Vrstva letišť samostatná od refresh aircraft | Plán — neověřeno |
| **R14** | M04 | G-05 | Styl mapy a přepnutí podkladu bez remountu tracks | Plán — neověřeno |
| **A01** | M05 | G-06 | Vyhledání ICAO ze statického platného katalogu, invalid input | Plán — neověřeno |
| **A02** | M05 | G-06 | Detail dráhy/frekvence se zdrojem a timestampem | Plán — neověřeno |
| **T10** | M05 | G-06 | NAVAID/FIX platná identita, verze a chybějící geometrie | Plán — neověřeno |
| **W03** | M06 | G-07 | Radar ČHMÚ časové snímky, stale/missing tile | Plán — neověřeno |
| **W06** | M06 | G-07 | SIGMET polygon, časová platnost a invalid geometry | Plán — neověřeno |
| **W08** | M06 | G-07 | BDS4,4 golden Mode-S zprávy, neznámý BDS odmítnout | Plán — neověřeno |
| **W09** | M06 | G-07 | Profil BDS jen skutečné hodnoty a doložená kvalita | Plán — neověřeno |
| **L11** | M07 | G-08 | Web Push s opt-in, bezpečné odvolání a mock delivery | Plán — neověřeno |
| **L12** | M07 | G-08 | Pushover jediný povolený příjemce a mock retry bez spamu | Plán — neověřeno |
| **S01** | M08 | G-09 | Counter rate z delta, restart readsb nepřidává negativní zprávy | Plán — neověřeno |
| **S06** | M08 | G-09 | Agregace typů a tras jen z důvěryhodných metadat | Plán — neověřeno |
| **S09** | M08 | G-09 | Denní logbook receiveru, výpadek/restart a 30d retence | Plán — neověřeno |
| **O01** | M08 | G-09 | Dashboard degradace DB/provideru bez zastavení radar RAM | Plán — neověřeno |
| **O02** | M08 | G-09 | Health vrací sanitizované verze a status bez adres | Plán — neověřeno |
| **O03** | M08 | G-09 | Watchdog readsb, ztráta zdroje a znovupřipojení | Plán — neověřeno |
| **O04** | M08 | G-09 | Monitoring jen používaných providerů a jejich timeoutů | Plán — neověřeno |
| **O05** | M08 | G-09 | Rust CPU/RSS, žádná imaginární Node GC telemetrie | Plán — neověřeno |
| **O06** | M08 | G-09 | Ohraničená RAM historie výkonu a čištění při retenci | Plán — neověřeno |
| **O07** | M08 | G-09 | SQLx pool/writer queue a měření drop/retry | Plán — neověřeno |
| **O08** | M08 | G-09 | Jednotný nízkofrekvenční diagnostický SSE stream | Plán — neověřeno |
| **U01** | M09 | G-10 | Mobilní velikosti/klávesnice a čitelnost mapy | Plán — neověřeno |
| **U02** | M09 | G-10 | PWA manifest a installability podle browseru | Plán — neověřeno |
| **U03** | M09 | G-10 | Offline stav označen stale; SSE nikdy cached-live | Plán — neověřeno |
| **U04** | M09 | G-10 | Čeština výchozí, angličtina alternativní a fallback | Plán — neověřeno |
| **U05** | M09 | G-10 | Command search přes aktivní letiště/letadla | Plán — neověřeno |
| **U08** | M09 | G-10 | Klávesové zkratky s vypnutím v textových polích | Plán — neověřeno |
| **L06** | M07 | G-08 | 7500/7600/7700 přechod a idempotentní alarm | Plán — neověřeno |

## Pravidla evidence

- Každé ID musí mít přesně jednoho vlastníka; scénáře se přidávají k CI a k výslednému reportu skutečné etapy, **nikoli k G-01 jako falešný PASS**.
- Pro ingest/API jsou společné kontraktové golden vzorky již ověřované v Rust a TypeScript, avšak implementace živých J01–J11 zůstává NOT_STARTED.
- U weather/letišť a notifikací se musí nejdřív ověřit aktuální licence, autentičnost zdroje a oprávnění; nelze vytvořit falešná data.
