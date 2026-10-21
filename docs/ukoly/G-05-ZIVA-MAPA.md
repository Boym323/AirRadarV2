# G-05 — Živá mapa a základní ovládání

**Stav:** NEZAHÁJENO  
**Závislosti:** G-03, pro letiště G-06  
**Vlastněná aktivní ID:** `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`

## Cíl

Zprovoznit rychlou MapLibre mapu s jedním lokálním proudem letadel a minimem ovládacích prvků.

## Konkrétní práce

1. MapLibre mapa, ikony podle typu/směru a čitelné popisky bez kolizí.
2. Aktuální seznam, filtrování a výběr letadla, živá stopa a sledování mapovou kamerou.
3. Samostatná vrstva okolních letišť z katalogu a přepínání existujících mapových podkladů.
4. Nevytvářet přepínač datového původu; radar je vždy lokální a jasně tak označený.

## Testy a měření

- Mobil/tablet/desktop, mapový lifecycle, reconnect SSE, filtrování, trail/follow.
- Ověřit, že UI neobsahuje inertní přepínače; změřit re-renders a aktualizace MapLibre.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-05-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
