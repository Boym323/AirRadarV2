# ADR 0016 — Živý dohled nad G-03/G-04 a pravdivý DB health

**Datum:** 24. 10. 2026. **Stav:** implementováno v PR, provozní přejímka PŘED NASAZENÍM NEOVĚŘENA. **Etapy a konzumenti:** G-03/J06–J08, G-04/J09, příprava diagnostiky G-09/O02/O07.

## Problém

Ve výchozím G-04 runtime byl příznak `database_connected=true` odvozen pouze ze startovního spojení. Pozdější SQL výpadek nebyl v HTTP health vidět. Navíc živé G-03/G-04 větve spouštěly ingest ve vedlejším tasku, jehož neočekávaný konec pouze zalogovaly. HTTP poté mohlo dál obsluhovat již neobnovovaný stav.

## Rozhodnutí

- G-04 používá každých 15 sekund samostatný read-only `SELECT 1` přes existující SQLx pool, s třísekundovým timeoutem. Žádná nová DB, role, síť ani dotaz ve vnitřní smyčce ingestu. Stav se do HTTP health předává přes `Arc<AtomicBool>`.
- Je-li DB očekávána, ale read-only sonda selže, `/api/v1/health` vrací `database_connected=false` a `status=degraded`. Sonda má až přibližně 15s zpoždění a není náhradou za hostitelský monitoring WAL/disku.
- G-03/G-04 společně sledují běh HTTP a ingestu. Pád ingestu ukončí i HTTP chybovým exitem, což předá zotavení `Restart=on-failure` systemd. Při SIGTERM i SIGINT požádá supervisor o kontrolované ukončení a vyčká maximálně 65 s na SQL drain.
- Časovaný G-04 canary, G-00, AirRadar V1 a sdílená DB infrastruktura se nemění. Živý stav zůstává pouze v Rust RAM a SQL nemůže blokovat jeho zpracování.

## Testy a provozní brána

Test `g04_health_reflects_database_up_and_down` ověřuje přechody `up → down` a správné HTTP indikátory. Test `unexpected_ingest_end_closes_http` ověřuje, že pád ingestu nenechá otevřený listener. Povinné: Rust fmt/clippy/test, Nginx/SSE/CI, DEV deploy s ověřeným SHA. Po DEV deploymentu provést izolovanou simulaci výpadku DB a řízený SIGTERM, potvrdit `pending=0`/`inflight=0` a současně nepřerušené SSE při SQL výpadku. Do té doby jsou end-to-end chování a bezpečnostní přejímka **NEOVĚŘENO**.

## Retence a práva

Historie 30 dní se nemaže novým Rust timerem: současná aplikační role má pouze `SELECT/INSERT`, nikoli `DELETE`, a SDÍLENÝ DB LXC je chráněný. Pravidelné bezpečné čištění vyžaduje samostatný DBA schválený mechanismus, jeho kapacitní test a izolovaný oprávněný účet. Bez nich neprohlašovat retenci za automatickou ani neměnit `pg_hba.conf`, firewall či ostatní databáze.
