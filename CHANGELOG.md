# Changelog — AirRadar V2

Tento soubor eviduje **skutečně dokončené a ověřené změny**, nikoli jen plán etap. Píše se česky. Vzor: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) přizpůsobený projektu. Zápis „Přidáno“ neznamená, že je produktová funkce KEEP dokončená; skutečný stav dokládá [matice](docs/plan/MATICE-FUNKCI.md).

## [Nepublikováno]

### Změněno
- Rozsah na základě výslovného rozhodnutí vlastníka: **46 KEEP / 132 REMOVE**; odstraněno 34 dříve plánovaných funkcí, vrácena **L06 (squawk 7500/7600/7700)**.
- Matice JSON, český katalog všech 178 funkcí, 9 logických modulů, etapový plán a implementační úkoly byly sladěny. Nevyřazené O11/O12 infrastruktury se drží jako technická povinnost, nikoli jako samostatný produkt.
- Automatické statistiky repozitáře budou od změny zobrazovat jmenovatel 46, staré historické vzorky 79 zůstávají zachovány.

### Přidáno
- České závazné zadání, původní katalog modulů (nahrazen devíti logickými moduly s 46 KEEP ID), anti-legacy pravidla a předávací instrukce pro nové vlákno.
- Samostatná česká Markdown zadání etap G-02 až G-15, včetně výslovného zrušení G-03 a G-09.

### Ověření
- Aktuální dokumentační změny vyžadují CI na přesném commitu; nelze je automaticky prohlašovat za ověřenou produkční funkčnost.

## 2026-10-21 — Založení V2 a český projektový základ (bez produkčního vydání)

### Přidáno
- Samostatné veřejné GitHub repo `Boym323/AirRadarV2`, počáteční Rust/Tokio/Axum HTTP kostra, Next.js úvodní stránka, OpenAPI a matice 79 KEEP/99 REMOVE.
- Pinned `Cargo.lock` a `apps/web/package-lock.json`, CI bez nasazovacího jobu a HTTP smoke pro Rust API a Next.js.
- České `README.md`, `AGENTS.md`, hlavní `.md` zadání, plán G-00 až G-15, ADR a runbook pro samostatný budoucí LXC.
- Výchozí `cs` jazyk úvodního webu a české popisy API.
- Automaticky generovaná česká matice funkcí a validace `node tools/check-docs.mjs`.

### Opraveno
- Po prvním neúspěšném Rust fmt checku následovala oprava a úspěšné opakované CI.

### Ověřeno
- [CI 39659205202](https://github.com/Boym323/AirRadarV2/actions/runs/39659205202) — Rust a web včetně spuštění HTTP, commit `99df0456`.
- [CI 39662365349](https://github.com/Boym323/AirRadarV2/actions/runs/39662365349) — Rust, Next.js a česká dokumentace, commit `d72fad1`.
- **Neověřeno / nedodáno:** živá ADS-B data, MapLibre aplikace, PostgreSQL instance, produkční LXC, nasazení a všech původně vybraných 79 KEEP funkcí (později rozsah zúžen na 46).

## Pravidla zápisu

- Každá změna v `apps/`, `crates/`, `database/`, `contracts/`, provozní konfiguraci `deploy/` a přímých závislostech musí mít položku v sekci **[Nepublikováno]** v témže uceleném změnovém balíčku. Úpravy čistě dokumentačních překlepů položku vyžadovat nemusí.
- Zapisuj **co se změnilo a proč**, ne seznam všech commitů. Pro důležité dokončené etapy uveď testy/CI/commit v reportu etapy.
- Při skutečně vydané verzi přesunout nové položky z `[Nepublikováno]` do datované sekce verze a vytvořit tag; dokud V2 vydaná není, nepoužívat falešný tag `v1.0.0` ani „produkční PASS“.
- Nedokončené plány patří do `docs/ukoly/` a `docs/plan/`, nikoliv mezi již hotové změny.
