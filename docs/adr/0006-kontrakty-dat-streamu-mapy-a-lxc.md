# ADR 0006 — Technické kontrakty dat, streamu, mapy a LXC

**Datum:** 22. 10. 2026. **Stav:** historický výchozí technický návrh. **Pozor:** jeho původní 15s DB sampling, předem vytvořený druhý index a automatický partitioning byly následně revidovány [ADR 0007](0007-ram-first-deduplikace-a-usporny-zapis.md). [Audit 22. 10. 2026](../audits/ARCHITEKTONICKY-AUDIT-AIRRADAR-V2-2026-10-22.md) uvádí otevřené otázky; benchmarky dosud neprovedeny. Navazuje na [ADR 0005](0005-vite-a-postgresql-v2.md).

## Kontext a rozhodnutí

AirRadar V2 je lokální přijímač readsb/BEAST s Rust/Tokio/Axum, statickým Vite/React/MapLibre webem a vlastní databází PostgreSQL v již existujícím separátním LXC. Zůstává přesně 45 aktivních funkcí. Rozhodujeme o jejich technických kontraktech, nikoli o nových produktech.

1. **DB:** platné polohy ukládat po 15s UTC bucketech, tabulka denně partitionovaná, PK (bucket_at, aircraft_key), B-tree podle letadla; počáteční retence 30 dní a 365 dní agregací. BRIN jen po EXPLAIN. Oddělená logická záloha V2 a správci potvrzený cluster backup.
2. **readsb/BEAST:** JSON `now/seen/seen_pos` a BEAST rámce s 12MHz MLAT čítačem mají různou časovou sémantiku; normalizovat a deduplikovat do jediného RAM vlastníka, oddělit ICAO/ne-ICAO, validovat CPR/DF a paritu.
3. **SSE:** snapshot+delta, výchozí 1Hz a heartbeat 15s; 100 připojení, 256KiB bounded fronty/klient a sekvenční resync.
4. **Mapa:** jedna MapLibre instance, společný GeoJSON source a symbol layer, stabilní ID. Diff aktualizace jen při doloženém výkonovém přínosu.
5. **Playwright:** browserové regresní testy s deterministickým mock/SSE backendem, Chromium/Firefox/WebKit a mobilní projekty, trace při chybě.
6. **LXC:** zcela separátní DB/role V2, cílený firewall, `hostssl`/SCRAM, TLS `verify-full`, žádné přihlašovací údaje v JavaScriptu nebo veřejném Git.

## Důvody a rizika

Minimalizovat dopad na existující PostgreSQL server, chybné deduplikace, náklady na SSE fanout a MapLibre re-render. Konkrétní čísla jsou **výchozí ohraničení k ověření**, nikoli naměřené provozní výsledky. Každé omezení a případná změna musí projít replay/SQL `EXPLAIN`/restore/browser E2E a aktualizací reportu.

## Implementační podklady

[Hlavní rozcestník šesti kontraktů](../plan/TECHNICKE-KONTRAKTY-A-LIMITY.md) a připojené podrobné české dokumenty. Žádný DDL, skutečné Playwright testy, živé připojení nebo deploy se tímto ADR nespouštějí. Produktová matice ani zrušené požadavky se nemění.

**Výsledný status:** architektura specifikována; funkce G-01 až G-11 nadále NEZAHÁJENO, dokud neexistují nezávislé důkazy implementace.

## Pozdější optimalizace

V otázce **deduplikace a frekvence historických zápisů** nahrazuje původní 15s interval [ADR 0007 — RAM-first adaptivní sampling](0007-ram-first-deduplikace-a-usporny-zapis.md). Zbývající protokoly a bezpečnostní brány zůstávají v platnosti. Současná maximální úspora SQL řádků je zatím pouze matematická, nikoli naměřené SSD IOPS.
