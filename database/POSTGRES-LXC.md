# PostgreSQL AirRadarV2 v existujícím samostatném LXC

**Stav:** read-only audit DEV připojení 23. 10. 2026 prokázal PostgreSQL 17.10, skutečnou konektivitu a omezená runtime práva. Zálohy postradatelné historie V2 nejsou požadovány (ADR 0010); kapacita sdíleného clusteru a globální síťová konfigurace zůstávají neověřené; nic nebylo měněno.

## Topologie

V2 aplikační LXC → důvěryhodná privátní síť bez TLS → PostgreSQL LXC → izolovaná databáze `airradar_v2_dev`. Browser nemá DB přístup, pouze Rust/SQLx.

## Kroky v G-04

1. Ověřit dostupnou kapacitu a verzi PostgreSQL bez zásahu do jiných aplikací.
2. Vytvořit jen novou databázi a oddělený DEV prostor, samostatné migrační a aplikační role bez SUPERUSER/CREATEDB/CREATEROLE.
3. Použít pouze existující síťové a PostgreSQL pravidlo pro konkrétní aplikační LXC; nepřidávat `hostnossl`, neměnit `pg_hba.conf` ani firewall a nevystavovat port 5432 veřejně. G-04 používá schválenou důvěryhodnou LAN bez TLS.
4. Heslo a `DATABASE_URL` držet pouze v zabezpečeném prostředí Rust; nikdy ve `VITE_*` a GitHubu.
5. Nejprve pouze read-only test `SELECT current_database(), current_user, version();`, poté samostatné testovací SQLx migrace, měření WAL/indexů/retence a izolovaný export/restore test G-04.11; provozní backup historie V2 se nevyžaduje (ADR 0010).
6. Žádný přístup k tabulkám, rolím nebo historickým datům V1; bezpečný rollback V2 je povinný.

Dokumentace: [PostgreSQL host authentication](https://www.postgresql.org/docs/current/auth-pg-hba-conf.html), [SCRAM](https://www.postgresql.org/docs/current/auth-password.html).
