# ADR 0001 — Samostatná architektura AirRadar V2

**Stav:** přijato jako výchozí návrh; pravidelně ověřovat měřením.

## Rozhodnutí

AirRadar V2 má vlastní GitHub repozitář, budoucí samostatný LXC, služby a databázi. Rust/Tokio/Axum bude vlastníkem živého stavu a přijímačů; Next.js/React/MapLibre zajišťuje výhradně web a ovládání; PostgreSQL/SQLx obstará vzorkovanou historii, indexované dotazy a agregace. Historie z V1 se nepřenáší.

## Odůvodnění

V1 obsahovala příliš těsné vazby mezi webem, příjmem, historií, diagnostikou a notifikacemi. V2 musí mít jeden jednoznačný vlastník stavu a oddělit práci s externí databází a providery od kritické cesty `readsb → RAM → SSE`.

## Hranice

Rust poskytuje kontrakty HTTP/SSE. Web nečte readsb ani nezapisuje letovou historii. API je verzované, OGN privacy fail-closed a data LOCAL/EXTENDED mají jednoznačný původ. Výpadky externích služeb nesmějí zastavit živý radar.

## Zvažované alternativy

Výhradně TypeScript byl možný, ale Rust se předběžně jeví vhodnější pro low-level BEAST a profily CPU. Neznamená to, že je TypeScript obecně nevýkonný. Mnoho mikroservis a nových front by navíc bez měření přineslo zbytečnou provozní složitost.

## Rizika a ověření

Rust přidává nároky na build a dva běžící procesy vyžadují smlouvu API. Před nasazením ověřit graceful shutdown, SSE backpressure, maximální paměť, restart, databázové výpadky, kompatibilitu LXC a rollback. Je-li jiná technologie prokazatelně lepší, zdůvodni změnu novým ADR.

