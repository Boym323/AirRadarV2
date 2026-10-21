# G-07 — Vybraná meteorologie

**Stav:** NEZAHÁJENO  
**Závislosti:** G-02, G-03 a G-05  
**Vlastněná aktivní ID:** `W03`, `W06`, `W08`, `W09`

## Cíl

Implementovat přesně vybrané meteorologické informace mimo kritickou cestu přímého příjmu letadel.

## Konkrétní práce

1. ČHMÚ radar s časovou osou a cache/TTL a přesnou platností snímků.
2. SIGMET/AirSIGMET s časově/prostorově validní geometrií a odpovídajícím zobrazením.
3. BDS 4,4 dekódování měření letadel s kvalitou, nejistotou a poctivým stavem při chybějících údajích.

## Testy a měření

- Provider timeout/429/5xx, překryv platnosti, test BDS s chybnými hodnotami.
- CPU/RAM a provázání meteorologických dotazů s lokální ingest latencí.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-07-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
