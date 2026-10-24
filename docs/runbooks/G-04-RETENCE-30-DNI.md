# G-04.09 — bezpečná pravidelná retence historie AirRadar V2

**Datum:** 24. 10. 2026 · **Rozsah:** G-04.09/J09, výhradně databáze `airradar_v2_dev` na existujícím samostatném DB LXC. **Stav:** zdrojový skript, systemd šablony a izolovaný CI test připraveny; **na skutečném DB LXC NENASAZENO ani nezapnuto**. Produkční V1 a ostatní databáze jsou nedotknutelné.

## Proč nejde mazat v Rust writeru

Aktuální runtime role má záměrně práva pouze `SELECT` a `INSERT`; přidání `DELETE` do jejího účtu by porušilo izolaci. Údržba je proto DBA-managed, mimo RAM ingest i SQLx batch writer. Historie starší než 30 dnů smí být odstraněna jen v samostatné V2 databázi.

## Implementovaná ochrana

- [`g04_retention_bounded.sh`](../../database/maintenance/g04_retention_bounded.sh) vyžaduje přesné `PGDATABASE=airradar_v2_dev`, `PGUSER=airradar_v2_dev_maintenance` a `AIRRADAR_G04_RETENTION_APPROVED=YES`; výslovný kontrolní `DO` blok běží také uvnitř každé SQL transakce.
- Každá dávka maže maximálně 1 000 řádků starších 30 dnů a limity `statement_timeout=5s`, `lock_timeout=1s`, celkem maximálně 20 dávek / 20 000 řádků / 90 s v jednom běhu. Při chybě SQL skončí s chybovým kódem; žádné TRUNCATE, DROP, změna indexů nebo dalších DB.
- Samostatné šablony `database/maintenance/systemd/` definují denní timer a systemd jednotku spouštěnou dedikovaným systémovým účtem na **DB LXC**, nikoli na DEV app LXC. Není implementovaná automatická instalace ani enable timeru.
- GitHub PostgreSQL 18 izolovaný test `tools/test_g04_retention_bounded.sh` ověřuje zápornou bránu, oddělenou SQL roli, smazání 32denního vzorku a zachování jednodenního. CI test není důkaz volného místa skutečného hostitele.

## Kontrolovaný postup pro DBA

1. Nejdřív **read-only** změřit volné místo dat/WAL filesystemu, zatížení, nejstarší vzorek, počet řádků starších než 30 dnů a současný přístup do vyhrazené V2 DEV databáze. Potvrdit, že běžící PP Studio/V1 nesmí být dotčeno.
2. Pouze po výslovném schválení vlastníka vytvořit samostatný PostgreSQL login `airradar_v2_dev_maintenance` bez `SUPERUSER/CREATEDB/CREATEROLE` s `CONNECT` pouze do `airradar_v2_dev`, `USAGE` na schéma a `SELECT,DELETE` výhradně na `airradar_v2.flight_samples`. **Žádné úpravy `pg_hba.conf`, firewallu ani oprávnění runtime role.**
3. Na DB LXC vytvořit systémový účet `airradarv2-retention` bez interaktivního shellu. Nainstalovat root-owned skript jako `/usr/local/libexec/airradarv2-g04-retention.sh` (0755) a šablony do `/etc/systemd/system/`; zabezpečit `/etc/airradar-v2/g04-retention.env` s přihlašovacím údajem mimo Git (0600). Před schválením ponechat `AIRRADAR_G04_RETENTION_APPROVED=NO`.
4. Ověřit `systemd-analyze verify`, nezměněný běh G-03/G-04 a odmítnutí neplatné role/DB. Po testovací záloze či snapshotu **pouze V2 DEV** spustit jednorázově s explicitním `YES`; zaznamenat počet skutečně odstraněných řádků, WAL/CPU/disk a odezvu dalších DB.
5. Teprve po vyhodnocení testu samostatně povolit `systemctl enable --now airradarv2-g04-retention.timer` na DB LXC. Pokud hrozí méně než 2 GiB volného DB/WAL prostoru nebo rušení ostatních aplikací, timer nezapínat, zastavit jen údržbu a oznámit správci.
6. Denně kontrolovat `systemctl --failed`, journal retenční služby, staré řádky a reálnou kapacitu hostitele. Záporný průběh, absence přihlášení nebo nedostatek volného místa jsou **NO-GO**; neobcházet je globálním navýšením oprávnění.

## Provozní omezení

Limity záměrně dovolují zůstat v retenci starší data, pokud jedno provedení nedoběhne. Nesmí se kvůli tomu násilně navýšit dávkování bez měření WAL/IOPS a kapacity sdílené PostgreSQL instance. Jednorázový 30denní SQL skript zůstává pro řízenou údržbu. Bez aktivovaného timeru **není automatická retence nasazena**. Toto řešení je pouze pro vývojovou databázi V2; od produkčního G-13 je oddělené.
