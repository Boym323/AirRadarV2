# Plán etap a aktuální stav AirRadar V2

**Aktualizováno 21. 10. 2026.** Tento plán řeší **pouze 45 aktuálně vybraných funkcí**, bez vedlejších a nepoužívaných modulů. Výchozí jazyk: čeština. Zápis „NEZAHÁJENO“ není nedostatečný test — práce ještě nezačala.

| Etapa | Výstup a zadání | Stav |
|---|---|---|
| G-00 | Samostatný základ Rust + Next.js | **HOTOVO — pouze scaffold** |
| G-01 | [Kontrakty a deterministické fixture](../ukoly/G-01-KONTRAKTY-A-TESTOVACI-DATA.md) | NEZAHÁJENO |
| G-02 | [Lokální ADS-B ingest a RAM](../ukoly/G-02-LOKALNI-INGEST.md) | NEZAHÁJENO |
| G-03 | [Verzované HTTP API a SSE](../ukoly/G-03-API-A-SSE.md) | NEZAHÁJENO |
| G-04 | [Úsporná historie](../ukoly/G-04-POSTGRESQL-HISTORIE.md) | NEZAHÁJENO |
| G-05 | [Živá mapa a základní ovládání](../ukoly/G-05-ZIVA-MAPA.md) | NEZAHÁJENO |
| G-06 | [Letiště a NAVAID/FIX](../ukoly/G-06-LETISTE-NAVAID.md) | NEZAHÁJENO |
| G-07 | [Vybraná meteorologie](../ukoly/G-07-VYBRANE-POCASI.md) | NEZAHÁJENO |
| G-08 | [Nouzové squawk a doručení](../ukoly/G-08-NOUZOVE-KODY-A-NOTIFIKACE.md) | NEZAHÁJENO |
| G-09 | [Statistiky a provozní diagnostika](../ukoly/G-09-STATISTIKY-DIAGNOSTIKA.md) | NEZAHÁJENO |
| G-10 | [Webová platforma](../ukoly/G-10-PWA-LOKALIZACE.md) | NEZAHÁJENO |
| G-11 | [Akceptace všech aktivních funkcí](../ukoly/G-11-ZAVERECNE-TESTOVANI.md) | NEZAHÁJENO |
| G-12 | [Oddělený shadow provoz](../ukoly/G-12-ODDELENY-SHADOW.md) | NEZAHÁJENO |
| G-13 | [Schválené nasazení a rollback](../ukoly/G-13-BEZPECNY-RELEASE.md) | NEZAHÁJENO |

## Technologická údržba G-00 (bez změny funkčního rozsahu)

- Node.js 26 a aktualizace webu: [zadání](../ukoly/G-00-AKTUALIZACE-STACKU.md), [report](../reports/G-00-AKTUALIZACE-STACKU-VYSLEDEK.md).
- Stav: **ČÁSTEČNĚ**, do úspěšného CI na finálním commitu. Nejde o novou produktovou etapu.

## Ověřená výchozí základna

- G-00: Rust, Next.js, připnuté závislosti a skutečné HTTP smoke testy: [GitHub Actions 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202).
- Tato základna **nepřijímá živá letadla**; veřejné API správně vrací `NOT_IMPLEMENTED`.
- Nový LXC není v aktuálních podkladech potvrzen jako nakonfigurovaný, produkční V1 zůstává nedotčená.
- Každá nová etapa musí mít skutečné testy i český report podle [šablony](../sablony/REPORT-ETAPY.md), odkazy na přesný commit a aktualizovaný [CHANGELOG](../../CHANGELOG.md).

## Závislosti

G-01 → G-02 → G-03 → G-04/G-05; G-06 doplní mapový katalog; G-07 přidá počasí; G-08 přidá nouzové kódy; G-09 diagnostiku; G-10 dokončí web; G-11 je bezpečnostní a výkonnostní přejímka; G-12 odpojený shadow; G-13 odděleně schválený release. Při vývoji lze bezpečně paralelizovat nezávislé moduly, ale funkce označit `ACCEPTED` pouze po testu.

**Zdroj pravdy:** `contracts/feature-matrix.json`, [moduly](MODULY-A-ROZSAH.md), a [závazné zadání](ZADANI-AIRRADAR-V2.md). Rozšíření mimo tento rozsah se nepřidávají automaticky.
