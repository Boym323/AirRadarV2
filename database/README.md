# V2 database contract (G-00 placeholder)

**No database has been created or migrated in this stage.**

G-05 will introduce its own PostgreSQL `airradar_v2` database and a least-privilege V2 role, schema migrations (SQLx) and sampled storage. It will **not migrate or delete** historical flights from AirRadar V1. Never point a V2 writer at V1's `DATABASE_URL`.

Before introducing migrations, define the actual queries and TTL/retention for reference catalogs, flight samples, daily rollups, navigation anomalies, alert delivery and operational telemetry. No raw BEAST stream persistence. Benchmark against isolated DEV fixtures.
