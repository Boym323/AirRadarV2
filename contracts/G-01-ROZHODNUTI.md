# G-01 — závazná rozhodnutí k P0 blokacím

**Datum:** 22. 10. 2026. **Stav:** první implementovaná sada sdílených kontraktů; plný BEAST parser a atomický SSE producer až G-02/G-03.

## P0-01 — čas a původ
- `receiverId` z neveřejné konfigurace znamená místo příjmu, `transport` JSON/BEAST znamená pouze transport.
- `timeBasis`: `derived_readsb`, `receiver_utc`, `server_arrival`, `unknown`.
- `positionOrigin`: `adsb`, `mlat`, `tisb`, `estimated`, `unknown`.
- **Konzervativní G-01 politika:** BEAST-only s nesynchronizovaným časem **nemá důvěryhodné `positionAtMs`**; jeho rámce mohou přispívat k RAM diagnostice, nikoli k nové potvrzené GPS historii. Do PostgreSQL smí pouze validní `adsb` poloha s doloženým UTC (`receiver_utc` či `derived_readsb`). Přehodnotit před G-02/G-04, pokud implementace umí spolehlivě odvodit čas.
- Zůstává možné zobrazit zdrojově označenou platnou polohu v RAM až po důvěryhodném CPR dekódování; nesměšovat „čas přijmu“ a „čas polohy“.

## P0-02 — identita a konflikt
- `icao:hex6` a `nonicao:hex6` se nikdy nesloučí. Callsign není identita; JSON/BEAST jedné stanice nepřidává druhé letadlo.
- Implausibilní skok polohy se **odmítne jako update**, nikoli průměruje nebo přepíše čerstvý validní stav. První konservativní filtr rychlosti 1500 km/h slouží jen pro fixture, není garancí fyzikální validace všech letadel a výškových profilů.
- Scope `nonicao` při případné vícestaniční architektuře znovu posoudit; žádná externí integrace se zde nezavádí.

## P0-03 — SSE
- Přijato `epoch` + monotónní `seq`, první `snapshot`, potom `delta` s `upsert/removed`, bezpečný resync při gap/restart.
- **Atomická registrace subscriberu/snapshot boundary se v G-03 musí realizovat a otestovat**, jinak etapa G-03 není převzata. G-01 dodává serializační typ a test sousední sekvence, nikoliv již živý stream.

## Omezení důkazu
Syntetická JSON fixture a čisté validace v Rust/TS potvrzují jen podmnožinu G-01. BEAST golden binární rámce/CRC/CPR, externí OpenAPI validátor a plná evidence 45 přejímacích scénářů nejsou ještě dokončeny.
