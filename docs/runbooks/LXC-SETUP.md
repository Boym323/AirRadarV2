# Nový samostatný Proxmox LXC pro AirRadar V2

**Stav: pouze plán. Z tohoto repozitáře nebyl vytvořen ani upraven žádný LXC.** Původní AirRadar V1 musí zůstat beze změny.

## Předběžné parametry

| Parametr | Doporučený výchozí návrh |
|---|---|
| Distribuce | Debian 13, architektura x86_64 |
| Procesor | 4 vCPU |
| Operační paměť | 8 GiB |
| Disk | 60 GiB |
| Kontejner | neprivilegovaný LXC |
| Přístup | vlastní hostname/IP, pouze nezbytné porty |

Jde o **odhad pro začátek vývoje, ne změřené minimální požadavky**. Skutečnou spotřebu ověřit při 1×/2×/4× provozu; před instalací nativních Rust/Node knihoven zohlednit omezení LXC a konfiguraci Proxmoxu.

## Oddělení služeb

- Samostatný systémový uživatel `airradarv2` bez interaktivního přihlášení.
- Aplikační cesta například `/opt/airradar-v2`, interní stav `/var/lib/airradar-v2` s omezenými právy.
- Rust engine prozatím `127.0.0.1:4101`, Next.js web `4100`, při vývoji bez veřejné domény.
- Budoucí systemd `airradarv2-engine.service` a `airradarv2-web.service`. Nezakládat služby dřív, než je reálně funkční jejich build/start/stop a bezpečnostní kontrola.
- Nová databáze `airradar_v2` (DEV `airradar_v2_dev`), samostatná minimální oprávnění; fyzický PostgreSQL server lze sdílet teprve po ověření volné kapacity a izolace.
- Tajemství a credential soubory jen v zabezpečené správě, nikdy v GitHubu. Nikdy nekopírovat `.env`, SSH, TLS nebo GitHub runner konfiguraci V1.

## Bezpečný postup přípravy

1. Vytvořit nový LXC ručně v Proxmoxu; zkontrolovat síť, disk, RAM, firewall a možnost restartu.
2. Doplnit skutečné parametry do **neveřejné provozní evidence**; do veřejného GitHubu nedávat citlivé údaje.
3. Nainstalovat požadovaný stabilní Rust a Node a ověřit projekt pouze s testovacími daty.
4. Otestovat HTTP health, statickou stránku, stop/start, dostupnost logů a paměťové limity.
5. Teprve při G-05 připravit V2 databázi a bezpečné migrace s otestovanou obnovou.
6. Teprve při G-14 uvažovat o **read-only** připojení druhého konzumenta k readsb/BEAST, po měření dopadu na V1.
7. Vlastní release, veřejnou doménu a proxy řešit samostatně až v G-15 s výslovným souhlasem a návratovým plánem.

## Co se nesmí stát

V2 nesmí převzít port, adresář, DB, systemd název, GitHub runner ani hesla V1. Nesmí vysílat demo upozornění skutečným uživatelům ani zapisovat historie do V1.

**Přejímka:** obě instalace mohou běžet odděleně; po vypnutí V2 je V1 beze změn; záloha V2 se dá obnovit bez zásahu do staré DB.

