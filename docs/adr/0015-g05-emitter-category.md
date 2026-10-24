# ADR 0015 — Kategorie letadel v jediném veřejném streamu G-03

**Datum:** 24. 10. 2026  
**Stav:** přijato jako **zpětně kompatibilní rozšíření existujících kontraktů** G-02/G-03 pro G-05.03 / R03.  
**Výchozí implementace:** PR #30, větev `feat/g05-category-contract-20261024`. **Nejde o novou produktovou funkci ani nové ID.**

## Kontext

G-05 má vlastní mapové siluety, ale před tímto krokem měl veřejný `PublicAircraft` jen polohu, callsign, squawk a telemetrii. Nešlo bezpečně vybrat vrtulník či kluzák. Readsb JSON obsahuje volitelné `category` a platné DF17 identifikační zprávy (Type Code 1–4) nesou identifikaci kategorií. Je zakázáno odvozovat kategorii ze známého typu přenosu, rychlosti, callsignu, výšky nebo externího katalogu.

## Rozhodnutí

1. `emitterCategory` je **volitelný** přesně whitelistovaný údaj v `Observation` a `PublicAircraft`, veřejně serializovaný v camelCase. Pokud není ověřen, pole **zcela chybí**; ne `null`, ne smyšlený default. Přijímá se přesně `A1–A7`, `B1/B2/B4/B6`, `C1/C2`; ostatní a rezervované kódy se zahazují.
2. **JSON readsb** parsuje `category` jen ze stejného místního snapshotu a bez jiného API/SQL dotazu. **BEAST DF17** parsuje přímo zprávy s validním CRC a Type Code 2, 3, 4 (odpovídající C/B/A), s konzervativním whitelistem. Type Code 1 (D) je nyní nevyužito, protože nemáme povolenou ikonografii.
3. **Jediná RAM autorita** a stávající G-02 quality precedence platí dále. Neplatné nebo starší zprávy nesmějí přepsat známý typ; při chybějící nové kategorii se poslední ověřená hodnota může zachovat jen do existujícího TTL tracku (60 s). Nová BEAST metadata se při čerstvém nadřazeném JSON stejně maskují jako ostatní metadata.
4. G-03 `/api/v1/aircraft` a `/api/v1/stream` nadále sdílejí stejné `PublicAircraft`. Pole je **aditivní a volitelné**; staré klientské fixture a SSE snapshot/delta zůstávají platné. Vstupní JSON a output DTO validují `contracts/*.schema.json`, `contracts/openapi.yaml`, Rust i TypeScript G-01 a G-05 adapter.
5. Render G-05 vybere již registrované originální siluety. Chybí-li typ, používá `generic-fixedwing` se skutečným trackem; chybí-li track, kruh. Kategorie **nepřepisuje polohu** a nepřidává domnělý letový plán.
6. Nedotýkat se V1, PostgreSQL G-04, veřejné adresy přijímače, notifikací ani produkčního nasazení. Automatický DEV CI/deploy je samostatná provozní brána.

## Proč takto

- Jediný datový tok a kapacitně omezený Rust RAM / SSE zůstává zachován.
- Existující readsb i BEAST mají dostatek důvěryhodných vlastních dat, není potřeba provozovat externí databázi typů.
- Whitelist + nepovinné pole jsou bezpečnější než odhad typů z telemetrie nebo nezaručené JSON vlastnosti.
- Nepřidávají se další API endpointy ani zvláštní tabulka historie; G-04 vzorkování zůstává stejné.

## Ověření

- Rust test CRC DF17, rezervovaných subtype, starého metadata a JSON/BEAST precedence.
- TypeScript kontraktové testy, G-05 atomický SSE store, 2 000 stabilních GeoJSON features.
- Stávající CI/Chromium map-first/stream test v PR #30. Vykázat PASS pouze s konkrétním Actions během nebo po sloučení.
- Reálné před/po FPS na hardwaru a letištní R12 stále vyžadují G-05.11/G-06, samotný ADR je netestuje.

## Dopady a rollback

API přijme chybějící pole ze starších verzí. Při nemožnosti přijímat kategoriová metadata je bezpečný fallback univerzální symbol/bod. Rollback odstraní jen volitelná metadata, nikoli identitu/polohu/stream. Dokumentace [G-05](../ukoly/G-05-ZIVA-MAPA.md), [mapové ikony](../plan/MAPOVE-IKONY-A-POPISKY.md) a [SSE kontrakt](../technical/SSE-PROTOKOL-A-LIMITY.md).
