# G-01 — Verzované kontrakty a testovací data

**Stav:** NEZAHÁJENO  
**Závislosti:** G-00  
**Vlastněná aktivní ID:** `J01`, `J02`, `J03`, `J04`, `J05`, `J11`, `J06`, `J07`, `J08`, `J09`, `R01`, `R03`, `R04`, `R06`, `R08`, `R09`, `R12`, `R14`, `A01`, `A02`, `T10`, `W03`, `W06`, `W08`, `W09`, `L06`, `L11`, `L12`, `S01`, `S06`, `S09`, `O01`, `O02`, `O03`, `O04`, `O05`, `O06`, `O07`, `O08`, `U01`, `U02`, `U03`, `U04`, `U05`, `U08`

## Cíl

Vytvořit jednoznačné smlouvy a reprodukovatelné testovací podklady pro všech 45 aktivních požadavků.

## Konkrétní práce

1. Definovat veřejné DTO a OpenAPI/JSON Schema pro health, letadla, chybové odpovědi a SSE snapshot/delta; strojové názvy stabilní, popisy české.
2. Vytvořit syntetické readsb JSON a BEAST fixtures (běžný let, duplicitní rámec, změna callsign, chybná poloha/čas, CRC a výpadek zdroje).
3. Pro každé aktivní ID doplnit akceptační scénář s vstupem, výstupem, mezními případy, testem a vazbou na modul.
4. Připravit Rust/TypeScript kontraktové testy bez živého přijímače a bez síťové služby.

## Testy a měření

- Ověřit přesnost 45 ID, bez neaktivních kontraktů.
- Spustit JSON/OpenAPI validaci, Rust/Next.js testy a aktualizovat českou dokumentaci.

## Kritéria převzetí

Všechny požadavky této etapy jsou doloženy reálnými testy a případnými relevantními měřeními. `contracts/feature-matrix.json`, český report `docs/reports/G-01-VYSLEDEK.md`, etapový stav i `CHANGELOG.md` odpovídají kódu a CI přesného commitu. Nesmí se zasáhnout do V1 ani provést produkční release bez zvláštního souhlasu.

## Co do etapy nepatří

Nedoplňovat rozšíření mimo [aktivní matice](../../contracts/feature-matrix.json); případné budoucí funkce jsou evidované samostatně v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).
