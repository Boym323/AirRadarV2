# Propojení aplikačního LXC s existujícím PostgreSQL LXC

**Stav:** konkrétní bezpečnostní projekt a postup G-04, nikoliv provedená konfigurace. Uživatel potvrdil, že PostgreSQL existuje v odděleném LXC; **verze serveru, síťová izolace, firewall, zálohy ani skutečná konektivita nejsou ověřené**. Vlastník zvolil 23. 10. 2026 pro V2 důvěryhodnou LAN bez TLS, viz [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md).

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

1. Spojení obou LXC musí probíhat jen přes důvěryhodný privátní segment. TCP 5432 nesmí být publikován do internetu; Proxmox/firewall povolí pouze přesnou IP V2 aplikačního LXC (/32).
2. Podle [ADR 0009](../adr/0009-g04-postgresql-lan-bez-tls.md) používá G-04 výslovně `sslmode=disable`. Nejde o šifrovaný přenos. Na PostgreSQL serveru **nevypínat SSL globálně**: ostatní databáze a aplikace ho mohou dál potřebovat.
3. Správce umístí do `pg_hba.conf` úzce cílené `hostnossl` s `scram-sha-256` pouze pro DB `airradar_v2_dev`, runtime roli a /32 IP aplikačního LXC. Zkontrolovat první odpovídající pravidlo pomocí `pg_hba_file_rules`, nepoužívat `trust`, `password` ani `host all all`.
4. Jméno a unikátní silné heslo poskytuje správce mimo GitHub a logy; ukládají se do chráněné systemd konfigurace. SCRAM autentizuje klienta, ale SQL dotazy a výsledky na LAN nejsou šifrovány. Nepovolovat přístup z cizích zařízení, VLAN nebo internetu.
5. Přístup do jiné databáze, přihlášení nesprávným heslem i z nepovolené IP musí být odmítnut. Změny sdíleného PostgreSQL/firewallu jsou možné jen po odsouhlasení správce a bez dopadu na stávající systémy.

**Pouze šablona pro DBA, nikoli příkaz ke spuštění:**

~~~conf
hostnossl airradar_v2_dev airradar_v2_dev_app <V2_APP_IP>/32 scram-sha-256
~~~

Toto rozhodnutí není přenositelné na WAN, nedůvěryhodnou VPN či sdílené sítě. V takovém případě znovu vyhodnotit ověřené šifrování.

## 3. Oddělení účtů, logů a migrací

- Migrační účet je používán při řízeném G-04 migracím, nikoli v běžném systemd env. Aplikační účet nemá DDL/role management práva.
- Schéma \`airradar_v2\`: \`REVOKE ALL ON SCHEMA public FROM PUBLIC\` v nové V2 DB pouze po schválení a kontrole rozdílů, žádné změny práv ve stávajících DB jiných služeb.
- Preferovat explicitní SQLx \`search_path\`, nikdy nevěřit libovolnému schema z \`public\`. SQLx runtime pool má ohraničenou velikost; čekání na DB není v živé ingest hot path.
- Konfigurace app service uživatele s \`chmod 0600\` pro tajemství, oddělený neprivilegovaný systemd účet. Při selhání TLS/role se engine může spustit ve **degraded persistence** stavu, ale nesmí zveřejnit detailní credentials ani zastavit lokální RAM/SSE.
- Spojení Rust → PG jen se známým zdrojem; reverse proxy ven má pouze web/HTTP read-only endpointy.

## 4. Ověření před první SQLx migrací

| Kontrola | PASS musí doložit |
|---|---|
| `SELECT version(), current_database(), current_user` | Skutečná verze a pouze schválená V2 DEV DB a role |
| `SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()` | `false`, v souladu s výslovně zvoleným LAN režimem |
| Firewall a `pg_hba_file_rules` | Jen konkrétní /32 IP klienta, přesná DB/role a SCRAM |
| Negativní test hesla a zdrojové IP | Odmítnuto, bez úniku údajů do logu |
| Negativní DDL, UPDATE/DELETE a přístup k jiným DB | Runtime role nemá potřebná oprávnění |
| Restart klienta/DB a síťový výpadek | Zachována živá RAM/SSE, žádný neomezený SQL backlog |
| Backup/restore v izolovaném cíli | Obnovená V2 historie bez zásahu do jiných databází |

## 5. Zálohy a provoz

- Výchozí **denní \`pg_dump -Fc\` V2** off-LXC, 14 denních + 8 týdenních kopií, měsíční restore test. Nepoužívat úložiště uvnitř stejných LXC jako jedinou zálohu.
- **Cluster PITR** se řeší na úrovni existujícího PostgreSQL serveru; ověřit \`pg_basebackup\`, WAL archivační status, retenci a RPO/RTO, ale nemačkat \`archive_mode\` naslepo kvůli V2.
- Při nesouladu s existující backup politikou se koordinuje s DBA. V2 logický dump je doplňková obnova jediné aplikace, ne náhrada fyzické cluster zálohy.
- Zálohy šifrovat, omezit přístup, ověřit možné GDPR/retention dopady. Provozní logy nesmějí obsahovat hesla ani raw BEAST.

## Zdroje

- [PostgreSQL 18 — `pg_hba.conf`, hostnossl a SCRAM](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html)
- [PostgreSQL 18 — SASL/SCRAM](https://www.postgresql.org/docs/18/sasl-authentication.html)
- [PostgreSQL 18 — PITR/WAL](https://www.postgresql.org/docs/18/continuous-archiving.html)

**Nebyla provedena žádná změna na skutečném DB LXC.**
