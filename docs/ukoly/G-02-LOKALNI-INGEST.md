# G-02 — Lokální ADS-B ingest a RAM

**Stav:** NEZAHÁJENO  
**Závislosti:** G-01  
**Vlastněná aktivní ID:** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`

## Cíl

Rust engine dekóduje místní readsb JSON a BEAST a drží jednotný, ohraničený stav letadel v paměti.

## Konkrétní práce

1. Implementovat čistý Rust BEAST/Mode-S/CRC24 parser a readsb JSON adapter bez přístupu k produkci.
2. Řešit autentickou identitu ICAO/ne-ICAO, deduplikaci, stale TTL, validní polohy, callsign a časové přetečení.
3. Při výpadku přepnout BEAST/JSON bez duplikace a restartovat spojení s backoff; žádná závislost na DB.
4. Mít jediný vlastník RAM state s explicitními limity počtu tracků a příjmových front.

## Testy a měření

- Golden fixtures a fuzz/property testy chybně zformátovaných rámců.
- Zátěž 1×/2×/4× a opakované reconnect; evidence CPU, RSS a latence.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-02-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
