# G-13 — Schválený release a návrat

**Stav:** NEZAHÁJENO  
**Závislosti:** G-11 a G-12 + samostatné schválení  
**Vlastněná aktivní ID:** žádná; bezpečnostní nebo provozní brána

## Cíl

Připravit samostatné nasazení V2 a rollback bez automatických zásahů do V1.

## Konkrétní práce

1. Připravit release runbook, samostatné systemd služby, pinned SHA, vlastní secrets a zálohování V2.
2. Před změnou veřejné domény/redirectů si vyžádat další explicitní souhlas.
3. Doložit runtime health, smoke a vyzkoušené obnovení DB a aplikace.

## Testy a měření

- Ověřený restart, rollback, DB restore a monitoring.
- Produkční nasazení bez povolení se neprovádí.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-13-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
