# G-04 — výsledek auditu a průběžná přejímka PostgreSQL historie

**Aktualizace:** 23. 10. 2026. **Etapa:** ROZPRACOVÁNO. **Primární KEEP:** J09 (`IN_PROGRESS`).
**Větev/PR:** `codex/g04-history-20261023`, draft PR #21. **Ověřený implementační SHA:** `e691159e600f80519c3e3e60d5bd51f41d458e59`.

## Audit G-04.01–G-04.12

| Krok | Stav | Důkaz / zbývá |
|---|---|---|
| G-04.01 | ČÁSTEČNĚ | Read-only připojení k `airradar_v2_dev` je dostupné z aplikačního hostitele; verze, kapacita a backup politika sdíleného LXC nejsou doložené. |
| G-04.02 | TESTED pro DEV | Připojení přes `PgSslMode::Disable`, `pg_stat_ssl.ssl=false`, bez změny `pg_hba.conf`/firewallu; důvěryhodnost LAN je provozní riziko. |
| G-04.03 | ČÁSTEČNĚ | `current_database()`, `current_user`, existence tabulky a omezená práva ověřeny; migrátor a izolace ostatních databází nejsou ověřeny. |
| G-04.04 | TESTED v CI/DEV kompatibilitě | Jedna tabulka, 30s bucket, PK deduplikace, opakované `up` a odmítnutý destruktivní `down`; migrace na skutečné DB nebyla spuštěna. |
| G-04.05 | TESTED v kódu | SQLx pool max. 2, acquire 2 s, SQL timeout 5 s, opt-in `--g04-dev-history`; reconnect/chaos provozní test chybí. |
| G-04.06 | TESTED v unit testech | RAM-first sampler, 30/60/120/180/600 s pravidla, 30s bucket a validace polohy. |
| G-04.07 | TESTED v unit testech/kódu | Pending + inflight nejvýše 2048, dávka nejvýše 128, coalescing, ACK po COMMIT, retry 3 pokusy s backoffem; reálný výpadek DB chybí. |
| G-04.08 | TESTED v SQLx testu | Časové čtení jedné identity, limit 1000, bez query v živé RAM/SSE cestě. |
| G-04.09 | ČÁSTEČNĚ | Retenční SQL je omezené na `airradar_v2_dev` a 1000 řádků; 30denní provozní retenční měření a index/partition benchmark chybí. |
| G-04.10 | NEOVĚŘENO | Chybí řízený timeout/výpadek/obnovení DB s důkazem kontinuity G-02/G-03. |
| G-04.11 | TESTED pouze izolovaně | CI restore test existuje; obnova skutečného DEV dumpu do nezávislé testovací DB nebyla provedena. |
| G-04.12 | NEOVĚŘENO | Chybí reprodukovatelné 1×/2×/4× p95/p99, RAM/CPU, WAL/I/O měření se SHA a metodikou. |

## Skutečné DEV ověření

Read-only/transaction test z aplikačního hostitele (`192.168.1.126 → 192.168.1.55:5432`) dne 23. 10. 2026:

- `current_database() = airradar_v2_dev`, `current_user = airradar_v2_dev_app`, `pg_stat_ssl.ssl = false`;
- `airradar_v2.flight_samples` existuje;
- runtime má `USAGE` schématu a `SELECT, INSERT`, ale nemá `CREATE, UPDATE, DELETE, TRUNCATE`;
- SQLx integrační test deduplikace a čtení prošel proti této DB; zápis byl idempotentní přes PK `(bucket_at, aircraft_key)` a zanechal jeden testovací vzorek `icao:fed123`;
- transakční zápis/čtení vzorku proběhl a byl vrácen `ROLLBACK`, takže test nezanechal fixture;
- špatné heslo a DDL runtime účtem byly odmítnuty.

Heslo ani interní tajemství nejsou součástí reportu, Git historie ani CI logu. Testovací vzorek byl vložen pouze do již připravené tabulky; migrace, role ani nastavení sdíleného DB LXC nebyly tímto testem měněny. Runtime účet nemá `DELETE`, proto se vzorek neodstraňoval destruktivním obcházením oprávnění.

## Co je implementováno

PR obsahuje `history.rs` (RAM sampler), `history_store.rs` (SQLx pool, batch INSERT, query a omezený retry), napojení do jediného ingest actoru, retenční SQL a izolované CI/restore testy. G-02 RAM/readsb/BEAST a G-03 HTTP/SSE zůstávají opt-in a databáze není aktivní v běžném režimu.

Destruktivní `.down.sql` migrace je nyní fail-closed; rollback historie se provádí pouze `pg_dump`/`pg_restore` do nové izolované databáze. `up.sql` je idempotentní vůči existující tabulce, ale před ostrým spuštěním musí migrační účet ověřit kompatibilitu sloupců a omezení.

## Neuzavřené brány

Neexistuje důkaz restartu/výpadku DB, skutečného `pg_dump`/`pg_restore` z DEV, retence po 30 dnech ani 1×/2×/4× benchmarku. Proto **G-04 není HOTOVO a J09 není TESTED/ACCEPTED**. PR #21 nesmí aktivovat writer v DEV bez samostatného GO, návratu na běžný G-03 režim a ověřeného restore.

### Bezpečný postup další aktivace

1. Správce potvrdí migrátor, backup/RPO/RTO a izolaci databáze bez změny V1, `pg_hba.conf` nebo firewallu.
2. Na nové/ověřené DEV DB proběhne pouze kompatibilitní migrace a negativní test práv.
3. V izolaci se změří restart/timeout/reconnect, restore a benchmarky; živý příjem/SSE musí pokračovat při SQL degradaci.
4. Teprve po písemném schválení se spustí `--g04-dev-history`; návrat je zastavení tohoto opt-in procesu a návrat na `--g03-live-api`.
