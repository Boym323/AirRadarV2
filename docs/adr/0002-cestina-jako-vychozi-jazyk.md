# ADR 0002 — Čeština jako výchozí jazyk projektu

**Stav:** přijato dne 21. 10. 2026.

## Rozhodnutí

Všechny hlavní Markdown dokumenty, zadání pro Codex, vývojové reporty, ADR, provozní návody, vysvětlení OpenAPI a uživatelské texty AirRadar V2 vznikají **v češtině**. HTML používá `lang="cs"` a výchozí locale je `cs-CZ`.

Názvy typů, proměnných, technologií, JSON klíčů, HTTP cest a strojových stavů zůstávají v běžném technickém formátu. Tyto identifikátory se nepřekládají, aby nevznikaly nekompatibility.

Funkce U04 vyžaduje budoucí volitelnou angličtinu v **UI**, nikoli anglickou projektovou dokumentaci. První verze tedy musí být použitelná česky a anglický překlad bude realizován až s testovanou lokalizací.

## Odůvodnění

Požadavek vlastníka projektu je dlouhodobě pracovat s přehlednou českou dokumentací a samostatnými MD zadáními. Jednotný jazyk snižuje nejasnosti v předávání etap mezi agenty a lidskou kontrolou.

## Ověření

Při každé etapě projít změněné MD a UI texty, zkontrolovat `tools/check-docs.mjs` a stav `docs/plan/ETAPY-A-STAV.md`. Anglická technická konstanta není sama o sobě porušení pravidla.

