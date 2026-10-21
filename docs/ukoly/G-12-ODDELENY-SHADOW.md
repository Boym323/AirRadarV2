# G-12 — Bezpečný oddělený shadow

**Stav:** NEZAHÁJENO  
**Závislosti:** G-11  
**Vlastněná aktivní ID:** žádná; bezpečnostní nebo provozní brána

## Cíl

Ověřit V2 ve vlastním LXC bez ovlivnění V1 a bez skutečného odesílání oznámení.

## Konkrétní práce

1. Použít výhradně izolované aplikace, databázi, porty a credentials.
2. Testovaný read-only mirror jen s výslovným souhlasem a monitoringem dopadu na V1.
3. Ověřit výpadky, restart, RSS a dlouhodobou odezvu při reprezentativních letech.

## Testy a měření

- Žádná změna V1 po zastavení V2.
- Záznam skutečného výkonu a návratového postupu.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-12-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
