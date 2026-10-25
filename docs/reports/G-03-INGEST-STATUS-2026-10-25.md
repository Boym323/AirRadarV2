# G-03: veřejný ingest status pro šetření neaktivní V2

**Stav 25. 10. 2026: návrh, bez potvrzeného DEV nasazení.**

## Ověřený incident
Nezávislé GitHub Actions testování veřejných rozhraní dalo V1 HTTP 200,
49 místních letadel, V2 HTTP 200, `waiting_for_observations`, 0 letadel.
V2 `generatedAt` byl přibližně 9 hodin starý. Stejný prázdný snapshot se
záměrně nemusí znovu publikovat, proto čas sám nedokazuje výpadek procesu.

## Změna
`GET /api/v1/ingest-status` (jen G-03/G-04 Rust router; bez autentizace)
vrací pouze `mode`, `stage`, `jsonConfigured`, `beastConfigured`,
`jsonHealthy`, `jsonLastOkAgeSeconds`, `jsonLastErrorAgeSeconds`,
`beastConnected`, `beastLastFrameAgeSeconds`, `ingestTickAgeSeconds`
a `aircraftCount`. Hodnoty vycházejí z atomických agregátů v jediném Rust
procesu, nevyvolávají polling, SQL, další feed ani zápisy. Query parametry
jsou odmítnuty (400); cache vypnuta.

Veřejná odpověď nikdy nevypisuje identitu nebo umístění letadel či přijímače,
adresu BEAST serveru, cestu k JSON, DB konfiguraci ani tajemství. Nezavádíme
žádné nové přihlašování ani přístup k administrátorským operacím.

## Testy a brány
1. Rust unit + Clippy + rustfmt + celá V2 CI = musí být PASS.
2. V2 DEV `main` auto-deploy musí ověřit stejný SHA; ne produkční G-13.
3. Veřejně ověřit JSON odpověď, pravdivý `stage` a úspěšné 1Hz ingest tick.
4. Podle výsledků diagnostikovat chybějící BEAST/JSON vstup a reálně obnovit
   zdroj. Teprve pak provést porovnání V1/V2 živých poloh.
