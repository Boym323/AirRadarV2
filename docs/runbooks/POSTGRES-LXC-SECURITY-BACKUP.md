# Propojení aplikačního LXC s existujícím PostgreSQL LXC

**Stav:** konkrétní bezpečnostní projekt a postup G-04, nikoliv provedená konfigurace. Uživatel potvrdil, že PostgreSQL existuje v odděleném LXC; **verze serveru, síťové rozhraní, certifikát, zálohovací režim ani skutečná konektivita nejsou ověřené**.

## 1. Topologie a oddělení práv

~~~text
klientský prohlížeč → HTTPS → [V2 aplikační LXC] → SQLx/TLS → [stávající PostgreSQL LXC]
                         │ statický Vite + Rust                │
                         │ soukromý runtime účet               └── pouze DB airradar_v2
                         └── žádný PostgreSQL port veřejně
~~~

- Stávající databáze jiných aplikací/V1 nesmí být změněny. Neprovádět globální restart, změnu \`listen_addresses\`, \`ssl\`, \`pg_hba.conf\` ani pravidel firewallu bez autorizovaného správce a nezávislého impact plánu.
- V DEV vytvořit \`airradar_v2_dev\` a vlastní migrační/runtime role; teprve po ověření budoucí \`airradar_v2\`. Všechny DDL migrační účet pouze v cílové DB a schématu; runtime účet bez \`SUPERUSER\`, \`CREATEDB\`, \`CREATEROLE\` a bez práva vytvářet libovolná schemata.
- SQLx \`max_connections=4\` je návrh a musí respektovat kapacitu sdílené PG instance. \`connect_timeout=3 s\` + \`statement_timeout=5 s\`, transakce krátké.
- V2 web se **nikdy** nepřipojuje přímo na PostgreSQL. \`DATABASE_URL\`, CA a tajemství jsou pouze v chráněném runtime nastavení Rust systemd, nikoli v \`VITE_*\`, git ani logu CI.

## 2. Síť, firewall, \`pg_hba.conf\` a TLS

1. Ověřit síťovou trasu mezi oběma LXC na *privátním* interface. Na PostgreSQL LXC žádná publikace \`5432\` do internetu.
2. Firewall povolit příchozí TCP \`5432\` výhradně z konkrétního V2 aplikačního LXC; jiné aplikace neomezovat neověřenou změnou globálního pravidla.
3. V \`pg_hba.conf\` se používá první odpovídající záznam. Nové cílené pravidlo \`hostssl\` + \`scram-sha-256\` je třeba zasadit správně před případné širší povolení, ověřit syntaxi a pravidla pomocí \`pg_hba_file_rules\`. \`hostssl\` vyžaduje SSL zapnuté na serveru, samo jej nezapne.
4. TLS server certifikát musí mít platný SAN odpovídající cílovému hostname. U klienta používat **\`sslmode=verify-full\`** s důvěryhodnou CA (\`sslrootcert\` nebo odpovídající SQLx TLS konfigurace); prosté \`require\` šifruje, ale nemusí plně ověřovat identitu serveru. Test „neznámé CA / chybné jméno hostitele“ má skončit odmítnutím spojení.
5. Pokud server nemá TLS, **neobcházet bezpečnost pomocí \`sslmode=disable\` nebo \`trust\`**. Nejdřív schválit změnu sdíleného PostgreSQL nebo izolovaný chráněný transport a doložit jeho audit; do té doby DB G-04 BLOKOVÁNO.
6. Přihlášení uživatele přes SCRAM, dlouhé unikátní credentials. Při úniku role heslo otočit a otestovat, že žádné jiné DB přístupné nejsou.

**Šablona pravidla k revizi, neprovádět přímo:**

~~~conf
# pouze ilustrativní; skutečné DB_USER, IPv4/CIDR a pořadí pravidel určí správce
hostssl airradar_v2_dev airradar_v2_dev_app <V2_APP_IP>/32 scram-sha-256
~~~

Pro produkční \`airradar_v2\` použít **jinou** roli a pravidlo; nemíchat DEV a produkci. Ze serveru získat po ověřeném připojení \`pg_stat_ssl\` stav (a v jeho možnosti ověřit TLS cipher), nikoliv pouze zvolit \`sslmode\` v konfiguračním souboru.

## 3. Oddělení účtů, logů a migrací

- Migrační účet je používán při řízeném G-04 migracím, nikoli v běžném systemd env. Aplikační účet nemá DDL/role management práva.
- Schéma \`airradar_v2\`: \`REVOKE ALL ON SCHEMA public FROM PUBLIC\` v nové V2 DB pouze po schválení a kontrole rozdílů, žádné změny práv ve stávajících DB jiných služeb.
- Preferovat explicitní SQLx \`search_path\`, nikdy nevěřit libovolnému schema z \`public\`. SQLx runtime pool má ohraničenou velikost; čekání na DB není v živé ingest hot path.
- Konfigurace app service uživatele s \`chmod 0600\` pro tajemství, oddělený neprivilegovaný systemd účet. Při selhání TLS/role se engine může spustit ve **degraded persistence** stavu, ale nesmí zveřejnit detailní credentials ani zastavit lokální RAM/SSE.
- Spojení Rust → PG jen se známým zdrojem; reverse proxy ven má pouze web/HTTP read-only endpointy.

## 4. Ověření před první SQLx migrací

| Kontrola | PASS musí doložit |
|---|---|
| \`SELECT version()\` | skutečná verze serveru a její podpora |
| \`SELECT current_database(), current_user\` | V2 DEV DB a právě role, která měla být povolena |
| \`SELECT * FROM pg_stat_ssl WHERE pid = pg_backend_pid()\` | skutečné TLS u aktivního spojení |
| Negativní připojení do cizí DB | odepřeno, bez zjišťování cizích dat |
| Negativní TLS test | chybné DNS/cert/CA → spojení odmítnuto |
| Negativní DDL test runtime role | nelze vytvářet schéma/tabulky tam, kde nemá právo |
| Restart SQLx/Rust klienta | zvládnutý reconnect, žádné nekonečné retry ani blokování ingestu |
| Backup/restore | obnovená **samostatná DEV DB** na izolovaném cíli |
| Sdílená cluster záloha | její stav potvrdil správce; V2 ji nemění bez impact review |

## 5. Zálohy a provoz

- Výchozí **denní \`pg_dump -Fc\` V2** off-LXC, 14 denních + 8 týdenních kopií, měsíční restore test. Nepoužívat úložiště uvnitř stejných LXC jako jedinou zálohu.
- **Cluster PITR** se řeší na úrovni existujícího PostgreSQL serveru; ověřit \`pg_basebackup\`, WAL archivační status, retenci a RPO/RTO, ale nemačkat \`archive_mode\` naslepo kvůli V2.
- Při nesouladu s existující backup politikou se koordinuje s DBA. V2 logický dump je doplňková obnova jediné aplikace, ne náhrada fyzické cluster zálohy.
- Zálohy šifrovat, omezit přístup, ověřit možné GDPR/retention dopady. Provozní logy nesmějí obsahovat hesla ani raw BEAST.

## Zdroje

- [PostgreSQL 18 — \`pg_hba.conf\`, pořadí a hostssl](https://www.postgresql.org/docs/18/auth-pg-hba-conf.html)
- [PostgreSQL 18 — klientské SSL a verify-full](https://www.postgresql.org/docs/18/libpq-ssl.html)
- [PostgreSQL 18 — TLS serveru](https://www.postgresql.org/docs/18/ssl-tcp.html)
- [PostgreSQL 18 — PITR a WAL](https://www.postgresql.org/docs/18/continuous-archiving.html)
- [PostgreSQL 18 — pg_verifybackup](https://www.postgresql.org/docs/18/app-pgverifybackup.html)

**Nebylo provedeno žádné síťové připojení ani změna PostgreSQL LXC.**
