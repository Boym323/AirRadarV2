# G-09 — Statistiky a provozní diagnostika

**Stav:** NEZAHÁJENO  
**Závislosti:** G-02 až G-08 pouze aktivní moduly  
**Vlastněná aktivní ID:** `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`

## Cíl

Lehký přehled zdraví fungujícího radaru, přijímače, DB a používaných podpůrných služeb.

## Konkrétní práce

1. Aktuální počítadla letadel a zpráv/s, jednoduché agregace a logbook jen z opravdu dostupných dat.
2. Diagnostika přijímače, API, DB latencí, front, CPU/RSS a Node event-loop odděleně.
3. Omezená provozní historie a diagnostické SSE s bounded fanout, bez zveřejnění citlivých dat.

## Testy a měření

- Výpadek readsb/DB/podpůrné služby a bezpečné veřejné DTO.
- Nízký počet probe a SQL dotazů i při mnoha klientech, CPU/RSS a retence.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-09-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
