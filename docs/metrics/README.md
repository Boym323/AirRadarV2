# Metriky vývoje AirRadar V2

Tyto statistiky se týkají vývoje repozitáře, nikoli provozu ADS-B radaru.

- Zdrojový kód: fyzické řádky souborů Rust, TypeScript, JavaScript, CSS, SQL a Python sledovaných Gitem, včetně prázdných řádků; bez lockfilů a generovaných *.d.ts.
- Testovací soubory: soubory v test/tests/__tests__ nebo s názvem *.test.* a *.spec.*. Nejedná se o počet skutečných testů ani jejich úspěšnost.
- Stav etap: pouze označení HOTOVO v dokumentu docs/plan/ETAPY-A-STAV.md.
- Stav KEEP funkcí: pouze TESTED či ACCEPTED v contracts/feature-matrix.json s alespoň jedním důkazem v poli tests.
- Historie: uložené vzorky podle SHA commitu. Graf udává počet řádků zdrojového kódu, nikoli kvalitu nebo výkon.

## Automatizace

GitHub Actions workflow [Statistiky vývoje](../../.github/workflows/development-metrics.yml) se spouští **jednou denně ve 03:17 v časovém pásmu Europe/Prague** (automatické zohlednění letního a zimního času), případně ručně. Při push do main se nespouští.
Jestliže se změnila některá ze sledovaných hodnot, přidá nový vzorek do history.json, přepočítá README a SVG graf. Pokud se hodnoty nezměnily, nevznikne nový vzorek ani zbytečný commit.
Vyžaduje oprávnění contents: write a povolený zápis z GitHub Actions do main. Jestliže je větev chráněna před automatickými zápisy, workflow skončí chybou a je nutné upravit jeho režim (například na pull request).

Ručně lze generátor spustit příkazem: node tools/update-dev-stats.mjs (Node.js 22+ a git checkout).
Pak zkontrolujte git diff --check a výsledný diff README a docs/metrics.

CI badge v README ukazuje stav automatických kontrol Rust/Next.js, ale nenahrazuje důkazy v matici.
Provozní metriky (CPU, RAM, SSE p95/p99, fronty a databáze) se budou měřit až při odpovídajících implementačních etapách.
Automatizace nespouští produkční nasazení a nedotýká se AirRadar V1.
