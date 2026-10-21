# Pravidla greenfield vývoje — nepřenášet nepotřebný kód

**Cíl:** nově napsat funkční, měřitelný a přehledný místní radar, ne mechanicky kopírovat V1.

## Dovolená reference

Původní V1 lze pouze číst při ověřování ADS-B/BEAST formátů, uživatelského chování a bezpečnostních invariantů. Třetí strany či referenční katalogy musí mít jasný původ a licenci. Historické záznamy V1 se nemigrují.

## Před vznikem každé nové části

| Otázka | Povinný výsledek |
|---|---|
| Jaké aktivní ID tento kód vyžaduje? | Odkaz na jednu z funkcí v aktuální matici |
| Kdo jej volá nebo zobrazuje? | Konkrétní konzument a test |
| Lze to provést bez dalšího procesu/DB tabulky? | Přednost má nejmenší dostatečné řešení |
| Co stojí CPU, RSS, DB I/O nebo síť? | Rozumný rozpočet a měření |
| Jak to selže a obnoví se? | Bounded fronty, timeouty, chybové stavy |
| Je správně chráněné soukromí? | Žádné uniklé secrets nebo soukromé údaje |

Nekopíruj celé složky `app/`, `components/`, `lib/`, `prisma/` ani jednorázové skripty V1. Bez důkazu nezaváděj více ingest ownerů, druhou databázi, broker, další runtime jazyk, provider cache nebo globální UI framework.

## Jediná volba rozsahu

Implementuj výhradně ID ze [strojové matice](../../contracts/feature-matrix.json). Další návrhy jsou izolované v [archivu budoucích rozšíření](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md); do vývoje vstupují jedině po výslovném schválení. Pravidla bezpečnosti, CI, dokumentace a obnovy jsou povinná i mimo konkrétní produktovou funkcionalitu.
