# ADR 0017 — Oddělená DBA-managed retence G-04 DEV historie

**Datum:** 24. 10. 2026. **Stav:** implementováno jako volitelná šablona, aktivace na DB LXC čeká na DBA GO. **Konzument:** G-04.09/J09, nikoliv nová aktivní funkce.

## Kontext

Výchozí 30denní retence je zapsána v dokumentaci, ale existuje jen jednorázový ruční SQL skript. Bez pravidelné údržby by `airradar_v2.flight_samples` mohla zaplnit disk a WAL sdíleného DB LXC. Současná runtime role AirRadar V2 má záměrně pouze `SELECT/INSERT`. Přidat jí `DELETE` nebo automaticky mazat z ingestu je nepřijatelné.

## Rozhodnutí

- DBA smí výhradně pro `airradar_v2_dev` vytvořit dedikovanou nejméně privilegovanou údržbovou roli s `SELECT/DELETE` pouze na `airradar_v2.flight_samples`. Bez superuser, dalších DB, přepsání `pg_hba.conf` nebo firewallu.
- Root-owned oddělený systemd timer v DB LXC může po DBA GO jednou denně spustit samostatný krátký skript. Do automatického release AirRadar V2 app LXC není údržba zařazená. Rozhodnutí se netýká V1 ani záloh ostatních databází.
- Jeden běh nejvýše 20 000 starých řádků, 20 × 1 000, maximálně 90 s, timeout SQL 5 s, zámek 1 s. Použít přesnou guard kontrolu DB/role v každé transakci.
- `AIRRADAR_G04_RETENTION_APPROVED=NO` je výchozí blokace. Zelená CI není oprávnění timer nainstalovat ani povolit. Nastavení `YES` je až samostatné provozní rozhodnutí vlastníka/DBA na místě.
- Zachovat ADR 0009 důvěryhodnou LAN a ADR 0010 postradatelnou historii. Runtime pool, SQLx batch writer, schéma a G-03 SSE zůstávají nedotčeny.

## Přejímka

Izolovaný PostgreSQL 18 CI musí prokázat explicitní odmítnutí jiné DB/role/chybějícího schválení, odstranění starého a zachování nového vzorku. Skutečný test na sdíleném DB LXC má navíc doložit WAL, disk, CPU, počet smazaných řádků, potvrdit případný no-op a neporušený provoz PP Studio/V1. Bez těchto měření je stav **PŘIPRAVENO, NEAKTIVOVÁNO**. Runbook: [G-04 retence 30 dnů](../runbooks/G-04-RETENCE-30-DNI.md).
