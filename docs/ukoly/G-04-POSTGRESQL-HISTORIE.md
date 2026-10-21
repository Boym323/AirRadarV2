# G-04 — Úsporná historie

**Stav:** NEZAHÁJENO  
**Závislosti:** G-02 a G-03  
**Vlastněná aktivní ID:** `J09`

## Cíl

Založit prázdnou PostgreSQL V2 a ukládat jen vybrané vzorky živých letů.

## Konkrétní práce

1. Query-first SQLx schéma pouze podle aktivních konzumentů, vlastní V2 credentials a migrace.
2. Asynchronní bounded queue, sampling, dávkové zápisy, indexy a retence.
3. Odpojení či restart DB nesmí zablokovat aktualizaci letadel v RAM; žádný import V1.

## Testy a měření

- DEV transakce, query EXPLAIN, WAL, latence a objem storage.
- Poruchy spojení, pool saturation, reconnect a jednoduchý restore.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-04-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
