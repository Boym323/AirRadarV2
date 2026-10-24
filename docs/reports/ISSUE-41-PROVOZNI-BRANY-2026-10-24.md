# Issue #41 — audit zbývajících provozních bran

**Datum:** 24. 10. 2026
**Výchozí SHA:** `fecc55b` (`main`)
**Issue:** [#41 — Audit 24. 10. — zbývající provozní brány AirRadar V2](https://github.com/Boym323/AirRadarV2/issues/41)

## Výsledek

Issue neobsahuje jednu opravu v aplikačním kódu, ale šest provozních bran. Kódová část je na aktuálním `main` přítomná a testovatelná. Ochrana `main` byla po auditu nastavena přes GitHub Branch Protection API; DB retence a fyzický desktop/telefon stále vyžadují externí provozní podmínky.

| Brána | Stav | Důkaz nebo další vlastník |
|---|---|---|
| Ochrana `main` | **DOKONČENO** | Branch Protection API nyní vrací `protected=true`: povinný PR review, `rust-engine`, `vite-web`, `Česká dokumentace`, aktuální větev, zákaz force-push/mazání, ochrana administrátorů a povinné vyřešení konverzací. |
| DB retence | **BLOKOVÁNO — DBA** | Na odděleném DB LXC ověřit kapacitu, schválit dedikovanou maintenance roli, provést jednorázový test `database/maintenance/g04_retention_once.sql` podle [runbooku G-04](../runbooks/G-04-POSTGRES-DEV-PRIJEMKA.md) a až poté ručně zapnout timer. Žádný timer ani zásah do V1 se nepřidává do tohoto commitu. |
| GPU/FPS | **NEOVĚŘENO — fyzický hardware** | CI Chromium je funkční regrese, nikoli GPU benchmark. Potřeba je měření 25/100/500/2 000 cílů × 1/2 Hz na desktopu a telefonu s přesným SHA, zařízením a metodikou. |
| Snapshot 1 MiB | **KÓDOVĚ OŠETŘENO, PROVOZNĚ NEOVĚŘENO** | Rust testy ověřují 5 000 plných DTO, překročení rozpočtu, `503 CAPACITY_EXCEEDED`, ukončení přetíženého streamu a obnovu novým snapshotem. Neprobíhá tiché ořezání. Skutečné 1/10/100-klientské měření zůstává samostatnou branou. |
| Release security | **NEZAHÁJENO / NO-GO** | Před G-13 chybí schválená licence, SECURITY kontakt, dependency audit/SBOM a GitHub Actions supply-chain evidence. |
| G-06 až G-13 | **NO-GO** | Etapy zůstávají podle `ETAPY-A-STAV.md` nezahájené nebo bez cílových přejímacích důkazů. |

## Ověření retenčního SQL

Na 24. 10. 2026 proběhl izolovaný test z aktuálního pracovního stromu (`main` SHA `fecc55b`) v dočasném PostgreSQL 17.11 clusteru na loopbacku. Nebyla použita lokální běžící databáze ani sdílené DB LXC.

- migrace G-04 vytvořila `airradar_v2.flight_samples`;
- vloženo 2 501 starých řádků a 1 nový řádek;
- tři běhy `database/maintenance/g04_retention_once.sql` provedly `2501 → 1501 → 501 → 0`;
- každý běh byl omezen dávkou nejvýše 1 000 řádků;
- nový řádek zůstal a tabulka zachovala jediný primární index;
- dočasný PostgreSQL cluster byl po testu zastaven.

Výsledek: **retenční SQL je v izolaci funkčně ověřené**. Skutečné DB LXC stále není provozně potvrzené: chybí DBA schválení, kapacitní baseline a oprávnění k jednorázovému spuštění proti `airradar_v2_dev`. Timer se proto neinstaluje ani nezapíná.

### Read-only audit skutečné DEV databáze

Přes již existující chráněné DEV připojení proběhl pouze read-only audit. Dotaz potvrdil databázi `airradar_v2_dev`, PostgreSQL 17.10, schéma `airradar_v2` a tabulku `flight_samples`. Stav byl 44 990 řádků, 2 350 různých identit, **0 řádků starších než 30 dní**, nejstarší bucket `2026-10-22 12:00:00+00`, nejnovější `2026-10-24 16:00:30+00`, databáze přibližně 13 MB a jediný index.

Runtime role `airradar_v2_dev_app` má právo `SELECT`, ale nemá `DELETE`; pokus o plánování mazací operace skončil očekávaným `permission denied`. To potvrzuje, že běžný Rust runtime nemůže retenci spustit. `airradarv2-g04-canary.service` je `inactive` a retenční timer není nainstalovaný (`not-found`). Skutečné mazání proto nebylo a nemá být provedeno bez schválené maintenance role DBA.

## Ověření kódové části snapshotu

Kontrakt [SSE protokolu](../technical/SSE-PROTOKOL-A-LIMITY.md) stanoví limit 1 MiB serializovaného snapshotu, maximálně 5 000 veřejných cílů a zákaz tichého ořezání. Implementace v `apps/engine/src/api.rs` při překročení:

1. nevydá starý snapshot jako nový živý stav;
2. nastaví stav API na `degraded`;
3. vrátí `503 CAPACITY_EXCEEDED`;
4. ukončí existující stream událostí `unavailable`;
5. po návratu pod limit vydá nový plný snapshot.

To je bezpečná implementace kontraktu, nikoli důkaz výkonu fyzického klientského hardwaru. Stávající syntetický SSE benchmark pokrývá 1/10/100 klientů a 5 000 kompaktních cílů; plný DTO případ záměrně ověřuje ochrannou větev, protože může překročit 1 MiB.

## Bezpečnostní hranice

V tomto auditu nebyl měněn GitHub Ruleset, žádný DB LXC, žádná PostgreSQL konfigurace, timer, firewall, V1, PP Studio ani produkční deploy. Ověření, které vyžaduje správce nebo fyzické zařízení, zůstává označeno `BLOKOVÁNO`/`NEOVĚŘENO`, nikoli `HOTOVO`.

## Převzetí

Issue #41 je částečně uzavřen: ochrana `main` a kódová část snapshotu jsou doložené. DB retence a fyzický GPU/FPS test stále vyžadují DBA/fyzické zařízení; release security a G-06 až G-13 zůstávají záměrně NO-GO.
