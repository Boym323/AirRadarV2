# Co z původního AirRadar V1 neimportovat

**Závazné pro každý nový modul a nové vlákno Codexu.** Zdrojová V1 slouží jako referenční systém, nikoliv jako výchozí implementační základ V2.

## Co je povolené použít

- Soubor `contracts/feature-matrix.json` v tomto repozitáři je autoritativní seznam **46 KEEP / 132 REMOVE** a aktuálních stavů.
- V1 dokumentace může vysvětlovat uživatelské chování, formáty a známé chyby. Použij jen relevantní části a nezaměňuj historický audit za aktuální realitu.
- Bezpečně vytvořené syntetické fixture, veřejné technické specifikace a ověřené provozní invarianty lze použít pro testy.
- Referenční data obnovuj z aktuálního dokumentovaného zdroje s licencí a původem; nikdy slepě nekopíruj stávající provozní DB.

## Výchozí zákaz převzetí

1. **132 funkcí označených REMOVE**, zejména staré statistické/recap a experimentální vrstvy, zaniklé dashboardy a nepoužívané API; i jejich závislosti jsou výchozím stavem mimo rozsah.
2. Kompletní složky `app/`, `components/`, `lib/`, `scripts/`, `prisma/`, `deploy/` z V1. Stejnojmenná nová složka ve V2 je v pořádku, ale nesmí jít o mechanickou kopii.
3. Starý Next.js `AircraftStateService`, druhého nezávislého příjemce, neomezený polling a provázání databáze se živým ADS-B.
4. Stará globální CSS pravidla, nepoužívané layouty, bývalé A/B varianty obrazovek, dřívější Airport Live Board experimenty a jednorázové script/benchmark soubory.
5. Prisma RC, produkční V1 migrace či její historické letové pozice, statistiky a osobní provozní údaje. **Starou historii nemigrovat ani v původní DB nemazat.**
6. Tajné klíče, tokeny, .env, privátní GPS pozici přijímače, logy s osobními údaji, nastavení proxy ani V1 LXC.
7. Knihovny, nové procesy, geospatial DB extensions, cache broker a diagnostické sondy „do zásoby“ bez přímého aktivního konzumenta.

## Gate proti zbytečnostem

| Otázka | Podmínka přijetí |
|---|---|
| Které KEEP ID danou část vyžaduje? | ID musí být přímo v dokumentu etapy a vlastník v katalogu modulů |
| Kdo ji volá? | konkrétní API/UI/jobs se scénářem použití |
| Jakou má provozní cenu? | rozpočet CPU/RAM/DB/I/O, bounded queue/timeout, měření |
| Je její implementace menší než přínos? | zvolená nejjednodušší testovatelná varianta |
| Jak se ověří a vypne? | replay/kontrakt/e2e, failure mode, rollback |
| Známe licenci dat a jejich provenance? | veřejný zdroj nebo zvlášť schválený přístup |

Pokud to nejde doložit, položku neimplementuj. Při nejasnostech v seznamu KEEP/REMOVE rozhoduje produktové zadání vlastníka, nikoliv automatická mechanická parita V1.

## Praktické hranice

- Jeden Rust procesní owner letadel; Next.js pouze render a server BFF podle kontraktů.
- Všechny historie začínají daty nové V2, se vzorkováním a přiměřenou retencí.
- Nová vrstva nesmí automaticky spouštět další externí dotaz při každém ADS-B snímku.
- Diagnostiky měří skutečné subsystémy, nesmí simulovat zdraví nebo zatěžovat live path.
- Specifikace a reporty jsou **v češtině**; technická terminologie/identifikátory zůstávají standardní.

Odkazy: [katalog modulů](MODULY-A-ROZSAH.md), [hlavní zadání](ZADANI-AIRRADAR-V2.md), [matice funkcí](MATICE-FUNKCI.md).

## Rozsah po revizi vlastníka 21. 10. 2026

G-03 (ADSBHub/ADSB.lol) a G-09 (OGN/RXW/externí metadata) se nemají implementovat. Neobnovovat ATC/ATS analytiku, mapový detail, 7/30 statistiky, METAR/TAF, coverage dashboard a automatický eAIP importer. Podrobnosti v [aktuálním katalogu modulů](MODULY-A-ROZSAH.md). Emergency squawk L06 zůstává jediným explicitním spouštěčem upozornění; neposílat skutečné zprávy při testech.
