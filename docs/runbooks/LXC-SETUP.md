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
- Rust engine `127.0.0.1:4101`, Vite dev web `127.0.0.1:4100`; produkční web je statický `dist/` přes HTTPS proxy.
- Budoucí systemd pouze pro `airradarv2-engine.service`; statický web nepotřebuje Node systemd službu.
- PostgreSQL je již v samostatném DB LXC. Pro V2 vyhradit `airradar_v2` (DEV `airradar_v2_dev`) a nové omezené účty; ověřit síť/TLS a izolaci. [Postup](../../database/POSTGRES-LXC.md).
- Tajemství a credential soubory jen v zabezpečené správě, nikdy v GitHubu. Nikdy nekopírovat `.env`, SSH, TLS nebo GitHub runner konfiguraci V1.

## Bezpečný postup přípravy

1. Vytvořit nový LXC ručně v Proxmoxu; zkontrolovat síť, disk, RAM, firewall a možnost restartu.
2. Doplnit skutečné parametry do **neveřejné provozní evidence**; do veřejného GitHubu nedávat citlivé údaje.
3. Rust engine a Vite `dist/` sestavit v CI; na produkčním aplikačním LXC nevyžadovat běžící Node.
4. Otestovat HTTP health, statickou stránku, stop/start, dostupnost logů a paměťové limity.
5. Teprve při G-04 připravit vlastní V2 databázi v existujícím PostgreSQL LXC a bezpečné SQLx migrace s otestovanou obnovou.
6. Teprve při G-12 uvažovat o **read-only** připojení druhého konzumenta k readsb/BEAST, po měření dopadu na V1.
7. Vlastní release, veřejnou doménu a proxy řešit samostatně až v G-13 s výslovným souhlasem a návratovým plánem.

## Co se nesmí stát

V2 nesmí převzít port, adresář, DB, systemd název, GitHub runner ani hesla V1. Nesmí vysílat demo upozornění skutečným uživatelům ani zapisovat historie do V1.

**Přejímka:** obě instalace mohou běžet odděleně; po vypnutí V2 je V1 beze změn; záloha V2 se dá obnovit bez zásahu do staré DB.
