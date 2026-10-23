# ADR 0009 — PostgreSQL V2 v důvěryhodné LAN bez TLS

**Rozhodnuto vlastníkem:** 23. 10. 2026. **Stav:** schválená cílová konfigurace G-04, provozně NEOVĚŘENO.  
**Rozsah:** pouze G-04 / J09 a samostatná databáze `airradar_v2_dev`.

## Kontext

Původní návrh vyžadoval mezi aplikačním a databázovým LXC šifrování TLS (`verify-full`). Vlastník výslovně zvolil přímé připojení v důvěryhodné místní síti **bez TLS** a přihlášení samostatným uživatelským jménem a heslem.

## Rozhodnutí

- SQLx v režimu `--g04-dev-history` používá `PgSslMode::Disable`; konfigurace vyžaduje `AIRRADAR_V2_DEV_DB_HOST`, `_PORT`, `_USER` a `_PASSWORD`, nikoli CA certifikát.
- Přístup z konkrétní IP aplikačního LXC (`/32`) přes privátní síť; TCP 5432 mimo tuto cestu blokován host/Proxmox firewallem. Žádný veřejný přístup ani zbytečné otevírání jiných VLAN.
- PostgreSQL pravidlo `hostnossl airradar_v2_dev airradar_v2_dev_app <IP_APP>/32 scram-sha-256`; nikdy `trust` ani široké povolení `all`. Zásahy do sdíleného `pg_hba.conf` jsou pouze po schválení správce a bez dopadu na ostatní služby.
- Samostatná runtime role smí jen `USAGE` schématu a `SELECT, INSERT` příslušné tabulky; DDL a mazání pouze odděleným migračním postupem. Heslo musí být jedinečné a uchované mimo Git a logy v chráněné konfiguraci LXC.
- Přejímka musí ověřit `current_database()`, `current_user`, `pg_stat_ssl.ssl = false`, omezení přístupu podle zdrojové IP a odmítnutí nesprávného hesla či DDL. Není povoleno aktivovat DB při běžném G-03 režimu.

## Přijaté riziko

**SCRAM autentizuje, ale nešifruje provoz.** Na LAN mohou být viditelné SQL příkazy, historie letadel a výsledky; kompromitovaný host na téže síti může útočit na spojení. Tato volba je určena jen pro důvěryhodnou izolovanou LAN. Pro internet, cizí VLAN, VPN nebo jiný nedůvěryhodný transport je nutné rozhodnutí přehodnotit a použít ověřené šifrování. Není důvod vypínat TLS globálně na sdíleném PostgreSQL serveru.

Toto ADR v G-04 nahrazuje pouze dřívější požadavek na **povinné TLS** (včetně příslušné části ADR 0006), nikoli ostatní požadavky na izolaci, zálohy, role a obnovu. Změna v kódu a dokumentaci **není důkazem** skutečného bezpečného nastavení DB LXC.
