# PostgreSQL AirRadarV2 v existujícím samostatném LXC

**Stav:** pouze příprava. Existující PostgreSQL server nebyl touto změnou kontaktován ani upravován.

## Topologie

V2 aplikační LXC → privátní síť/TLS → PostgreSQL LXC → izolovaná databáze `airradar_v2`. Browser nemá DB přístup, pouze Rust/SQLx.

## Kroky v G-04

1. Ověřit dostupnou kapacitu a verzi PostgreSQL bez zásahu do jiných aplikací.
2. Vytvořit jen novou databázi a oddělený DEV prostor, samostatné migrační a aplikační role bez SUPERUSER/CREATEDB/CREATEROLE.
3. Povolit síť pouze z konkrétní IP nového aplikačního LXC. Použít `hostssl`, `scram-sha-256` a ověření serverového TLS certifikátu; nevystavovat port 5432 veřejně.
4. Heslo a `DATABASE_URL` držet pouze v zabezpečeném prostředí Rust; nikdy ve `VITE_*` a GitHubu.
5. Nejprve pouze read-only test `SELECT current_database(), current_user, version();`, poté samostatné testovací SQLx migrace, měření WAL/indexů/retence a záloha+obnova.
6. Žádný přístup k tabulkám, rolím nebo historickým datům V1; bezpečný rollback V2 je povinný.

Dokumentace: [PostgreSQL host authentication](https://www.postgresql.org/docs/current/auth-pg-hba-conf.html), [SCRAM](https://www.postgresql.org/docs/current/auth-password.html).
