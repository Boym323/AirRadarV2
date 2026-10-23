# ADR 0009 — PostgreSQL V2 v důvěryhodné LAN bez TLS

**Rozhodnuto vlastníkem:** 23. 10. 2026. **Stav:** schválená cílová konfigurace G-04; read-only konektivita a omezená práva DEV ověřeny, provozní přejímka databáze není uzavřena.
**Rozsah:** pouze G-04 / J09 a samostatná databáze `airradar_v2_dev`.

## Kontext

Původní návrh vyžadoval mezi aplikačním a databázovým LXC šifrování TLS (`verify-full`). Vlastník výslovně zvolil přímé připojení v důvěryhodné místní síti **bez TLS** a přihlášení samostatným uživatelským jménem a heslem. Dále výslovně odmítl požadavek přidávat pravidlo `hostnossl` nebo jinak měnit existující `pg_hba.conf`.

## Rozhodnutí

- SQLx v režimu `--g04-dev-history` používá `PgSslMode::Disable`; konfigurace vyžaduje `AIRRADAR_V2_DEV_DB_HOST`, `_PORT`, `_USER` a `_PASSWORD`, nikoli CA certifikát.
- Připojení proběhne pouze přes existující privátní LAN. Existující firewall/Proxmox pravidla se nemění; jejich skutečný stav je třeba nejprve read-only ověřit. Není povoleno automaticky otevřít port 5432 do internetu ani dalších sítí.
- **Žádné nové pravidlo `hostnossl` ani změna `pg_hba.conf`.** Použije se stávající autorizace PostgreSQL. Existující pravidlo typu `host` je kompatibilní s TCP připojením se SSL i bez SSL; existující `hostssl` bez TLS nefunguje. Server určuje výsledný autentizační mechanismus; SCRAM-SHA-256 je doporučený, ale nesmí být prosazován změnou pravidel bez nového rozhodnutí. `trust` nesmí být považován za splnění požadavku přihlášení heslem.
- Samostatná runtime role smí jen `USAGE` schématu a `SELECT, INSERT` příslušné tabulky; DDL a mazání pouze odděleným migračním postupem. Heslo musí být jedinečné a uchované mimo Git a logy v chráněné konfiguraci LXC.
- Přejímka začne pouze **read-only pokusem** o připojení existujícím účtem bez TLS; při neúspěchu má být hlášena blokace, nikoli automaticky změněna pravidla. Pokud je spojení povoleno, ověří se `current_database()`, `current_user`, `pg_stat_ssl.ssl = false`, skutečný přístupový rozsah a odmítnutí nesprávného hesla či DDL. Není povoleno aktivovat DB při běžném G-03 režimu.

## Přijaté riziko

**SCRAM autentizuje, ale nešifruje provoz.** Na LAN mohou být viditelné SQL příkazy, historie letadel a výsledky; kompromitovaný host na téže síti může útočit na spojení. Tato volba je určena jen pro důvěryhodnou izolovanou LAN. Pro internet, cizí VLAN, VPN nebo jiný nedůvěryhodný transport je nutné rozhodnutí přehodnotit a použít ověřené šifrování. Není důvod vypínat TLS globálně na sdíleném PostgreSQL serveru.

Toto ADR v G-04 nahrazuje dřívější požadavek na **povinné TLS** a návrh **nového pravidla `hostnossl`** (včetně příslušné části ADR 0006), nikoli ostatní požadavky na izolaci, zálohy, role a obnovu. Změna v kódu a dokumentaci **není důkazem** skutečného bezpečného nastavení DB LXC.
