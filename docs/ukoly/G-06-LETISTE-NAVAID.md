# G-06 — Letiště a NAVAID/FIX

**Stav:** NEZAHÁJENO  
**Závislosti:** G-01, G-03 a podle ukládání G-04  
**Vlastněná aktivní ID:** `A01`, `A02`, `T10`

## Cíl

Zpřístupnit publikované letiště, dráhy, frekvence a navigační body ve správném rozsahu.

## Konkrétní práce

1. Navrhnout verzovaný lehký katalog podle aktuálních zdrojů a licence.
2. Vyhledávání ICAO, detail letiště A02 a NAVAID/FIX s chybějícími daty označenými jako neznámé.
3. Optimalizovat konkrétní indexy a mapovou projekci bez zbytečných pravidelných synchronizací.

## Testy a měření

- Golden katalogová data, vyhledávání, integrita drah a frekvencí.
- Výkon dotazů, velikost katalogu, skutečná dostupnost a licence dat.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-06-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
