# G-11 — Závěrečná akceptace aktivních funkcí

**Stav:** NEZAHÁJENO  
**Závislosti:** G-01 až G-10  
**Vlastněná aktivní ID:** žádná; bezpečnostní nebo provozní brána

## Cíl

Uzavřít testy a bezpečnostní brány pro všech 45 aktivních ID, nikoli vytvářet další produkt.

## Konkrétní práce

1. Pro všechna ID doložit test, konkrétní chování a odkaz v JSON matici.
2. Spustit Rust/TS kontraktové a integrační testy, bezpečnostní audit a UI E2E.
3. Profilovat ingest 1×/2×/4×, SSE 1/10/100, obnovu DB a omezení paměti.

## Testy a měření

- Každé ID musí být TESTED/ACCEPTED s důkazem, jinak NO-GO.
- Žádný únik dat a žádné produkční nasazení bez approval.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-11-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
