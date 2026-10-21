# ADR 0001 — Samostatná architektura AirRadar V2

**Stav:** historické výchozí rozhodnutí G-00. **Revidováno 21. 10. 2026**: web už byl převeden na Vite/React a PostgreSQL je definitivně zvolená pro existující samostatný DB LXC. Viz [ADR 0004](0004-vyzkum-technologii-a-otevrene-brany.md), [ADR 0005](0005-vite-a-postgresql-v2.md) a [rozhodnutí](../plan/TECHNOLOGICKA-ROZHODNUTI.md).

## Rozhodnutí

AirRadar V2 má vlastní GitHub repozitář, budoucí samostatný LXC, služby a databázi. Rust/Tokio/Axum bude vlastníkem živého stavu a přijímačů; Vite/React/MapLibre zajišťuje výhradně web a ovládání; PostgreSQL/SQLx na separátním databázovém LXC obstará vzorkovanou historii, indexované dotazy a agregace. Historie z V1 se nepřenáší.

## Odůvodnění

V1 obsahovala příliš těsné vazby mezi webem, příjmem, historií, diagnostikou a notifikacemi. V2 musí mít jeden jednoznačný vlastník stavu a oddělit práci s externí databází a providery od kritické cesty `readsb → RAM → SSE`.

## Hranice

Rust poskytuje kontrakty HTTP/SSE. Web nečte readsb ani nezapisuje letovou historii. API je verzované, přenos sanitizuje veřejné DTO a stav letadel pochází výhradně z místního přijímače. Výpadky podpůrných služeb nesmějí zastavit živý radar.

## Zvažované alternativy

Výhradně TypeScript byl možný, ale Rust se předběžně jeví vhodnější pro low-level BEAST a profily CPU. Neznamená to, že je TypeScript obecně nevýkonný. Mnoho mikroservis a nových front by navíc bez měření přineslo zbytečnou provozní složitost.

## Rizika a ověření

Rust přidává nároky na build a dva běžící procesy vyžadují smlouvu API. Před nasazením ověřit graceful shutdown, SSE backpressure, maximální paměť, restart, databázové výpadky, kompatibilitu LXC a rollback. Je-li jiná technologie prokazatelně lepší, zdůvodni změnu novým ADR.
