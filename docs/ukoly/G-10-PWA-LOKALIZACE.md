# G-10 — Webová platforma

**Stav:** NEZAHÁJENO  
**Závislosti:** G-05 a existující API  
**Vlastněná aktivní ID:** `U01`, `U02`, `U03`, `U04`, `U05`, `U08`

## Cíl

Dokončit moderní český mobilní i desktopový web bez zbytečné složitosti.

## Konkrétní práce

1. Responsive rozvržení, PWA manifest, dostupnost ovládání a skutečné offline chování s výstrahou o stáří dat.
2. Výchozí cs-CZ a volitelný anglický překlad UI včetně formátování časů/jednotek.
3. Globální vyhledávání a zkratky jen pro opravdu existující funkce.

## Testy a měření

- Playwright na desktop/tablet/mobil, offline a PWA installability.
- Přístupnost, fokus, touch, i18n fallback a stabilita UI.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-10-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
