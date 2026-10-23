# Propojení aplikačního LXC s existujícím PostgreSQL LXC

**Stav:** postup G-04 bez změn síťových a autentizačních pravidel databázového serveru, nikoli provedená konfigurace. Read-only audit DEV 23. 10. 2026 prokázal PostgreSQL 17.10, konektivitu a omezená runtime práva; **síťová izolace, firewall a kapacita sdíleného LXC nejsou z runtime účtu plně ověřitelné**. Zálohy postradatelné historie V2 nejsou dle [ADR 0010](../adr/0010-g04-postradatelna-historie-bez-zaloh.md) vyžadovány. Vlastník zvolil 23. 10. 2026 pro V2 důvěryhodnou LAN bez TLS, viz [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md).

## 1. Topologie a oddělení práv

~~~text
klientský prohlížeč → HTTPS → [V2 aplikační LXC] → SQLx/TCP LAN bez TLS → [stávající PostgreSQL LXC]
                         │ statický Vite + Rust                │
                         │ soukromý runtime účet               └── pouze DB airradar_v2
                         └── žádný PostgreSQL port veřejně
~~~

- Stávající databáze jiných aplikací/V1 nesmí být změněny. Neprovádět globální restart, změnu \`listen_addresses\`, \`ssl\`, \`pg_hba.conf\` ani pravidel firewallu bez autorizovaného správce a nezávislého impact plánu.
- V DEV vytvořit \`airradar_v2_dev\` a vlastní migrační/runtime role; teprve po ověření budoucí \`airradar_v2\`. Všechny DDL migrační účet pouze v cílové DB a schématu; runtime účet bez \`SUPERUSER\`, \`CREATEDB\`, \`CREATEROLE\` a bez práva vytvářet libovolná schemata.
- SQLx \`max_connections=2\` je návrh a musí respektovat kapacitu sdílené PG instance. \`connect_timeout=2 s\` + \`statement_timeout=5 s\`, transakce krátké.
- V2 web se **nikdy** nepřipojuje přímo na PostgreSQL. \`DATABASE_URL\`, CA a tajemství jsou pouze v chráněném runtime nastavení Rust systemd, nikoli v \`VITE_*\`, git ani logu CI.

## 2. Síť, firewall, `pg_hba.conf` a LAN bez TLS

1. Spojení obou LXC má probíhat jen přes existující důvěryhodnou privátní LAN. **Žádné nové firewall pravidlo není součástí G-04**. Read-only ověřit dostupnost a to, že TCP 5432 není veřejně vystaven. Pokud přístup neexistuje, nevytvářet jej automaticky.
2. Podle [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md) používá G-04 výslovně `sslmode=disable`. Nejde o šifrovaný přenos. Na PostgreSQL serveru **nevypínat SSL globálně**: ostatní databáze a aplikace ho mohou dál potřebovat.
3. **`pg_hba.conf` se nemění.** Běžné existující pravidlo typu `host` připouští šifrované i nešifrované TCP spojení; `hostssl` nešifrované připojení nepřijme. Nejprve read-only zkontrolovat existující pravidla (`pg_hba_file_rules`) a otestovat spojení z aplikace. Pokud se nelze přihlásit, evidovat blokaci; nevyžadovat speciální `hostnossl` ani automatické otevření přístupu.
4. Jméno a unikátní silné heslo poskytuje správce mimo GitHub a logy; ukládají se do chráněné systemd konfigurace. Způsob heslového ověření určuje existující pravidlo serveru (SCRAM je vhodnější než MD5). Nešifrované SQL dotazy a výsledky na LAN zůstávají rizikem.
5. Ověřit odmítnutí nesprávného hesla a práv do cizí DB. Skutečný rozsah síťového přístupu zjistit, nikoli předpokládat. Na žádost vlastníka se pravidla sdíleného PostgreSQL a firewallu **neupravují**.

**Žádná šablona nového pravidla se nenasazuje.** Pro přístup bez TLS se spoléháme výhradně na již existující odpovídající pravidlo. Není-li použitelné, přejímka G-04.02 je BLOKOVÁNO.

Toto rozhodnutí není přenositelné na WAN, nedůvěryhodnou VPN či sdílené sítě. V takovém případě znovu vyhodnotit ověřené šifrování.

## 3. Oddělení účtů, logů a migrací

- Migrační účet je používán při řízeném G-04 migracím, nikoli v běžném systemd env. Aplikační účet nemá DDL/role management práva.
- Schéma \`airradar_v2\`: \`REVOKE ALL ON SCHEMA public FROM PUBLIC\` v nové V2 DB pouze po schválení a kontrole rozdílů, žádné změny práv ve stávajících DB jiných služeb.
- Preferovat explicitní SQLx \`search_path\`, nikdy nevěřit libovolnému schema z \`public\`. SQLx runtime pool má ohraničenou velikost; čekání na DB není v živé ingest hot path.
- Konfigurace app service uživatele s \`chmod 0600\` pro tajemství, oddělený neprivilegovaný systemd účet. Při selhání spojení/role se engine může spustit ve **degraded persistence** stavu, ale nesmí zveřejnit detailní credentials ani zastavit lokální RAM/SSE.
- Spojení Rust → PG jen se známým zdrojem; reverse proxy ven má pouze web/HTTP read-only endpointy.

## 4. Ověření před první SQLx migrací

| Kontrola | PASS musí doložit |
|---|---|
| `SELECT version(), current_database(), current_user` | Skutečná verze a pouze schválená V2 DEV DB a role |
| `SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()` | `false`, v souladu s výslovně zvoleným LAN režimem |
| Existující firewall a `pg_hba_file_rules` | Skutečný povolený rozsah zdrojů a typ autentizace; bez požadavku na změnu konfigurace |
| Negativní test hesla | Odmítnuto, bez úniku údajů do logu; rozsah IP přístupu se auditně vyhodnotí |
| Negativní DDL, UPDATE/DELETE a přístup k jiným DB | Runtime role nemá potřebná oprávnění |
| Restart klienta/DB a síťový výpadek | Zachována živá RAM/SSE, žádný neomezený SQL backlog |
| Izolovaný export/restore G-04.11 | Technický test již proveden; pravidelný backup V2 není přejímací brána dle ADR 0010 |

## 5. Postradatelná V2 historie — ochrana sdílené DB

- Dle [ADR 0010](../adr/0010-g04-postradatelna-historie-bez-zaloh.md) zálohy, PITR ani provozní RPO/RTO pro V2 historii nepožadujeme. Při havárii může nenávratně zmizet celá stará historie; v izolaci otestovaný G-04.11 pg_dump/restore zůstává historickým důkazem.
- **Neměnit** zálohování PP Studio, V1 či jiných databází ani globální WAL nastavení sdíleného PostgreSQL LXC. Bez automatického backup timeru pro G-04.
- Povinné jsou volné místo dat/WAL, CPU/RAM, omezení zápisů, 30denní retence, bezpečný přístup, oddělený read-only canary feed a rollback bez mazání dat. SQL nesmí blokovat Rust RAM/SSE.

## Zdroje

- [PostgreSQL 18 — `pg_hba.conf`, hostnossl a SCRAM](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html)
- [PostgreSQL 18 — SASL/SCRAM](https://www.postgresql.org/docs/18/sasl-authentication.html)
- [PostgreSQL 18 — PITR/WAL](https://www.postgresql.org/docs/18/continuous-archiving.html)

**Nebyla provedena žádná změna na skutečném DB LXC.**
