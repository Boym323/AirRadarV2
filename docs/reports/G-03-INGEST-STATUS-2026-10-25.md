# G-03: veřejný ingest status pro šetření neaktivní V2

**Stav 25. 10. 2026: CI a DEV nasazení PASS; dlouhodobá stabilita ještě NEOVĚŘENA.**

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

## Provedená přejímka (25. 10. 2026)

- [V2 PR #56](https://github.com/Boym323/AirRadarV2/pull/56): schválené testy,
  merge do V2 main `88be61c7d1d4036dd28c0589287cacfcb50404c9`.
- [DEV release #40227541079](https://github.com/Boym323/AirRadarV2/actions/runs/40227541079):
  sestavení, publikace a nasazení na izolovaný DEV runner PASS.
- [Živá read-only kontrola #40227633768](https://github.com/Boym323/AirRadar/actions/runs/40227633768):
  `mode=read_only`, `stage=G-04`, `beastConfigured=true`,
  `beastConnected=true`, stáří rámce a ingest ticku 0 s.
  `jsonConfigured=false` je při používaném BEAST zdroji přípustné.
- Tři párové HTTP snapshoty: V1 lokálních 52/54/54, Rust V2 42/41/39,
  shodných 42/41/39 a P95 rozdílu poloh 0,497/0,000/0,122 km.
  **To nedokazuje ekvivalenci všech funkcí, CPU ani vhodnost produkčního cutoveru.**
- Příčina předchozí absence pozorování zůstává bez historických provozních
  důkazů neznámá; nové nasazení obnovilo živý příjem, ale nedokazuje
  trvalé vyřešení původní příčiny.
