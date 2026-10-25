# G-02 — bezpečné rozšíření DF18 pro V1/V2 srovnávací testy

**Stav 25. 10. 2026: DF18 CF0/CF1 CI a DEV nasazení PASS; příspěvek ke snížení rozdílu nebyl izolovaně prokázán.**

## Proč

Původní AirRadar V1 zpracovává readsb JSON i více druhů Mode-S rámců,
zatímco V2 Rust přijímal z BEAST TCP pouze zprávy DF17. Veřejný read-only
shadow test #40228147552 našel v posledních třech vzorcích **15, 15 a 16**
letadel navíc ve V1; všechna měla v jeho veřejném snapshotu zdroj ADS-B.
Počet sám o sobě neprokazuje, že DF18 je jejich příčina.

## Rozsah implementace

- CRC24-validní 112bit DF17: dosavadní dekódování beze změny.
- **DF18 CF=0:** non-transponder ADS-B s ICAO adresou — typy TC/CPR
  rekonstruovány dosavadním validátorem.
- **DF18 CF=1:** stejný ADS-B ME dekodér s adresou označenou jako `nonicao:`;
  stejná hex hodnota se nikdy nesloučí s ICAO a pár even/odd z cizí identity
  nikdy nezaloží polohu.
- **DF18 CF=2–7 a Mode-S AP parity:** nadále se zahazují — TIS-B,
  rebroadcast a odpovědi radaru bez validace IMF/původu není bezpečné
  vydávat za přímou ADS-B ICAO identitu.

Dekodér čte pouze již existující BEAST kanál a přijímané zprávy předává do
původního jediného Rust RAM state ownera. Žádný nový API endpoint, přepínač
příjmu, administrátorská operace, nový zdroj dat nebo změna konfigurace LXC.
V2 DEV s G-04 může ukládat do vlastní oddělené DB pouze přípustné validní
polohy podle již existující SQL eligiblity logiky; V1 PostgreSQL nedotčeno.

## Přejímka

1. `cargo fmt --all -- --check`, Clippy, Rust testy, DEV CI (včetně integrace
   BEAST a DB) PASS na stejném SHA.
2. DEV nasazení immutable release SHA + read-only API test PASS.
3. Více časově srovnatelných vzorků V1/V2 a rozpad chybějících cílů, bez
   expozice ICAO, callsign a souřadnic. Výsledky musí rozlišit zvýšené
   pokrytí DF18 od nepřímých rozdílů readsb JSON vs BEAST.
4. Pokud scházejí běžné DF17/Mode-S cíle, další etapou je samostatný
   validátor AP reply vůči již známé identitě, ne slepé přijetí jejich CRC.

Dokončení tohoto PR není souhlas s Rust V1 produkčním cutoverem.

## Skutečný DF18 výsledek

[PR #58](https://github.com/Boym323/AirRadarV2/pull/58) sloučen jako
`de282bf5c64ad48778e553cb0caf8c5c0f66ae2d` a [DEV release
#40233582640](https://github.com/Boym323/AirRadarV2/actions/runs/40233582640)
PASS. První porovnání po DF18 však ještě mylně ignorovalo Rust identitu s
`stale=true`, které znamená pouze zastaralou polohu. Po opravě
porovnávacího nástroje byl rozdíl jen 3–5 cílů, převážně bezpolohové DF11.
Viz navazující DF11 report.
