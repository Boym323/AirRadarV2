# ADR 0010 — Postradatelná historie AirRadar V2 bez povinných záloh

**Datum:** 23. 10. 2026. **Stav:** schváleno vlastníkem, provozní canary dosud NO-GO.
**Rozsah:** pouze historická data AirRadar V2, databáze `airradar_v2_dev` a tabulka `airradar_v2.flight_samples`; nikoliv ostatní databáze ani služby.

## Rozhodnutí

Vlastník rozhodl, že historické polohy letadel nejsou kritická data jako např. PP Studio a **nechce pro ně řešit pravidelné zálohy**.

- Pro G-04 DEV **nepožadujeme** pravidelný externí `pg_dump`, PITR, automatický backup timer ani potvrzené RPO/RTO jako podmínku canary nebo J09.
- **Vědomě přijímáme úplnou a nevratnou ztrátu dosavadní historie** při havárii databáze, nikoliv jen posledních 24 hodin. Staré polohy nelze spolehlivě zrekonstruovat z živé RAM/readsb. Žádná garance RPO/RTO není.
- G-04.11 již úspěšně otestoval izolovaný `pg_dump -Fc`/`pg_restore`; důkaz testu se zachová, ale **neznamená povinnost zavádět provozní zálohy**.
- Při poškození nebo ztrátě V2 databáze zastavit jen writer historie, zachovat G-03 RAM/SSE a po schválení DBA obnovit **pouze V2 schéma** ze schválených migrací. Historie se začne ukládat znovu od nuly. Není povoleno destruktivní automatické mazání ani povolit zakázanou down migraci.
- Nedestruktivní rollback canary (zastavit opt-in službu, zachovat G-03) zůstává povinný; neslibovat obnovu ztracené historie.
- **Kapacita a bezpečnost sdíleného PostgreSQL LXC zůstávají povinné:** volný prostor dat/WAL, CPU/RAM, retence 30 dní, bounded writer, časové limity SQL, provozní sledování, práva a vlastníci, oddělený read-only feed, izolovaný canary a ztráty do 1 %. Nesmí se zaplnit úložiště nebo omezit další aplikace jen proto, že historie je postradatelná.
- Neprovádět změny `pg_hba.conf`, firewallu, TLS, globálního WAL, V1, PP Studio ani zálohování ostatních databází. Tato výjimka platí **výhradně** pro vzorkovanou historii V2.

## Přednost a budoucnost

Toto aktuální rozhodnutí nahrazuje starší požadavky na **povinné zálohování historie V2** v G-04 zadání, technickém datovém modelu, runboocích a obecné části AGENTS.md. Ostatní architektonická a provozní pravidla zůstávají platná. Pokud se v budoucnu historie stane nenahraditelným podkladem pro další funkce, je nutné znovu rozhodnout o backup politice.

**G-04 i J09 zůstávají do skutečné provozní přejímky ROZPRACOVÁNO / IN_PROGRESS.**
