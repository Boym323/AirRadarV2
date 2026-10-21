# G-08 — Nouzové squawk a doručení

**Stav:** NEZAHÁJENO  
**Závislosti:** G-02, G-03 a podle potřeby G-04  
**Vlastněná aktivní ID:** `L06`, `L11`, `L12`

## Cíl

Rozpoznat nouzové kódy squawk a povolit volitelné bezpečné doručení.

## Konkrétní práce

1. Detekce změny squawk 7500/7600/7700, evidence události a idempotentní zpracování.
2. Web Push a Pushover jako samostatné volitelné transporty s bounded retry a skutečným error stavem.
3. Testovací a shadow konfigurace musí absolutně vypnout reálná oznámení.

## Testy a měření

- Syntetické squawk sekvence, duplicitní zprávy a souběh workera.
- Neplatná API odpověď, 429/5xx, souběh claim/retry a nulové produkční odesílání během testů.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-08-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
