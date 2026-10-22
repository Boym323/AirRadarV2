# AirRadar V2 — první nasazení do vývojového LXC

**Datum:** 22. 10. 2026. **Stav:** připravený postup; **v LXC dosud nebyl vykonán**. Pouze G-00/G-01 statický web a Rust health. Bez DB, ADS-B a produkce.

## Výchozí prostředí

Správce potvrdil existující oddělený neprivilegovaný **Debian 13.7, x86_64, 1 vCPU, 8 GiB RAM, 20 GiB disk**. Swap uvnitř kontejneru hlásí 0 B. Kvůli globální IPv6 **nesmí** dev web poslouchat na všech rozhraních; používá pouze privátní IPv4:8080 s filtrem LAN.

Rust a Vite sestavuje **GitHub Actions**, nikoliv 1vCPU LXC. Uvnitř LXC není potřeba instalovat Node.js, npm, Rust, SQLx ani PostgreSQL.

## Krok 1 — Bootstrap uvnitř nového DEV LXC, jako root

~~~bash
apt-get update
apt-get install -y --no-install-recommends curl ca-certificates
curl -fsSLo /root/airradarv2-bootstrap.sh \
  https://raw.githubusercontent.com/Boym323/AirRadarV2/main/deploy/bootstrap-dev-lxc.sh
bash -n /root/airradarv2-bootstrap.sh
sed -n '1,240p' /root/airradarv2-bootstrap.sh
# doplň privátní IPv4 a povolenou podsíť tohoto LXC
bash /root/airradarv2-bootstrap.sh <IP_DEV_LXC> <LAN_CIDR>
~~~

Bootstrap instaluje nginx, vytvoří neprivilegovaného uživatele airradarv2, samostatnou cestu /opt/airradar-v2/releases, systemd Rust jednotku (127.0.0.1:4101) a nginx s filtrem LAN na daném IPv4:8080. Nezasahuje do V1, nenasazuje aplikaci ani neaktivuje nginx, dokud není připravený balíček. Vyžaduje **samostatný prázdný DEV LXC** bez cizích nginx sites; jinak skončí chybou.

## Krok 2 — GitHub Actions, ruční sestavení

Otevři [Ruční DEV balíček](https://github.com/Boym323/AirRadarV2/actions/workflows/dev-package.yml), vyber **Run workflow → main** a z úspěšného běhu stáhni artifact ZIP. Uvnitř budou:

~~~text
airradarv2-dev-linux-amd64-<sha12>.tar.gz
airradarv2-dev-linux-amd64-<sha12>.tar.gz.sha256
~~~

Balíček obsahuje release Rust binárku, statické web/ a VERSION se SHA commitu. GitHub workflow má jen oprávnění číst repozitář, **nikdy se samo nepřipojuje k LXC**. Oba soubory z artefaktu jsou nutné; samotné ZIP ještě není přímo instalovatelný tar.gz.

## Krok 3 — bezpečný přenos obou souborů

Na Macu rozbal stažený ZIP a přenes oba soubory. Pokud je v DEV LXC **již bezpečně nakonfigurovaný SSH přístup**, použij vestavěný macOS scp. Nezřizuj kvůli tomuto kroku veřejné root SSH s heslem.

Bez SSH v LXC lze použít hostitelský Proxmox: přes zabezpečené SSH přenes oba soubory na **Proxmox host**, ověř přes pct list správné ID nového DEV LXC (ne produkční V1), a na **Proxmox hostu** spusť:

~~~bash
pct list
pct push <ID_DEV_LXC> /root/<archive>.tar.gz /root/<archive>.tar.gz
pct push <ID_DEV_LXC> /root/<archive>.tar.gz.sha256 /root/<archive>.tar.gz.sha256
~~~

Nepotřebuješ instalovat nic dalšího na Mac. Konkrétní ID a privátní IP neukládat do veřejného GitHubu.

## Krok 4 — instalace uvnitř DEV LXC, jako root

~~~bash
curl -fsSLo /root/airradarv2-install.sh \
  https://raw.githubusercontent.com/Boym323/AirRadarV2/main/deploy/install-dev-bundle.sh
bash -n /root/airradarv2-install.sh
sed -n '1,220p' /root/airradarv2-install.sh
bash /root/airradarv2-install.sh /root/<archive>.tar.gz
~~~

Skript porovná SHA-256, ověří strukturu a commit archivu, vydá ho pod /opt/airradar-v2/releases/ a atomicky přepne symlink current. Zapne systemd a nginx, provede health testy a při selhání vrátí předchozí verzi. Veřejná IPv6 není poslouchaná.

## Krok 5 — přejímka

Spusť uvnitř DEV LXC:

~~~bash
systemctl --no-pager --full status airradarv2-engine nginx
curl -fsS http://127.0.0.1:4101/api/v1/health
curl -fsS http://127.0.0.1:4101/api/v1/aircraft
ss -lntp
journalctl -u airradarv2-engine -n 30 --no-pager
readlink -f /opt/airradar-v2/current
cat /opt/airradar-v2/current/VERSION
~~~

Ve webovém prohlížeči z LAN otevři **http://<IP_DEV_LXC>:8080/**.

Správné G-00/G-01 API explicitně odpovídá status=bootstrap, live_ingest=false a database_connected=false. Endpoint aircraft vrací NOT_IMPLEMENTED a žádná letadla. Jde o úspěšný vývojový bootstrap, **nikoliv o skutečný radar**.

## Rollback a omezení

Chceš-li vrátit předchozí release, spusť instalační skript na jeho dříve uloženém tar.gz spolu s původním sha256 souborem. Systém nevytváří druhou DB ani nové produkční služby.

Na Proxmox firewallu doporučujeme zakázat nevyžádané příchozí IPv6 na DEV LXC. Žádná veřejná reverse proxy, V1 readsb ani PostgreSQL se do G-00/G-01 nezapojují. Živá data patří do G-02/G-03, historie do G-04, produkční release do G-13.
