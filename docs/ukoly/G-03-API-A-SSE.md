# G-03 — Verzované HTTP API a SSE

**Stav:** NEZAHÁJENO  
**Závislosti:** G-01 a G-02  
**Vlastněná aktivní ID:** `J06`, `J07`, `J08`

## Cíl

Zpřístupnit Rust state přes bezpečné verzované HTTP a SSE full/delta.

## Konkrétní práce

1. Implementovat veřejné DTO s minimem údajů a jednotným datovým modelem pro web.
2. SSE initial snapshot, delta update/removed, sekvenční čísla, reconnect, heartbeat a cleanup.
3. Zajistit bounded fanout, coalescing a odmítnutí nového klienta při vyčerpání kapacity.

## Testy a měření

- Kontrakty a 1/10/100 SSE klientů včetně pomalého odběratele.
- Měřit P95/P99, CPU/RSS a sanitizaci API dat.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-03-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
