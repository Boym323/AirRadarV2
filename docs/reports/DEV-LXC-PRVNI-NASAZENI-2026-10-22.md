# První nasazení G-00/G-01 do vývojového LXC

**Datum:** 22. 10. 2026. **Stav:** ÚSPĚŠNĚ NASAZENO podle správcem dodaného výpisu z terminálu. Nezávislý přístup ChatGPT do LXC neproběhl.

## Verze a prostředí

- Commit nasazeného CI balíčku: `227a8619b590d2c536761ffefc75cd0a5633936c`.
- Balíček `airradarv2-dev-linux-amd64-227a8619b590.tar.gz` se souborem `.sha256`; instalátor ověřil integritu: **OK**.
- Samostatný neprivilegovaný Debian 13.7 x86_64 LXC, 1 vCPU, 8 GiB RAM, 20 GiB disk. Konkrétní privátní IP ani SSH se do veřejného repozitáře nezapisují.
- Přenos přes `scp` z macOS, spuštění `deploy/install-dev-bundle.sh` v LXC, ohlášeno **PASS**.
- `nginx -t`: úspěšně. `systemctl is-active airradarv2-engine nginx`: **active / active**.

## Ověřený bootstrap a správně neimplementovaný radar

~~~json
{"status":"bootstrap","stage":"G-00","live_ingest":false,"database_connected":false}
~~~

~~~json
{"aircraft":[],"source":"bootstrap","status":"NOT_IMPLEMENTED"}
~~~

Obě odpovědi byly doloženy skutečným `curl` na loopback Rust API. `NOT_IMPLEMENTED` je očekávaný výsledek před G-02/G-03, nikoli chyba instalace. Instalační skript také při PASS provádí smoke test Nginx HTML.

## Dodatečná síťová kontrola předaná správcem

- `curl -fsS` proti webovému LAN endpointu v novém LXC vrátil HTML dokument v češtině (`<!doctype html>`, `<html lang="cs">`) s názvem AirRadar V2. To potvrzuje skutečný HTTP provoz přes Nginx z kontejneru; nevypovídá to samo o sobě o dostupnosti z jiných zařízení v LAN.
- `ss -lntp | grep -E '4101|8080'` potvrdil Rust **jen na IPv4 loopback `127.0.0.1:4101`** a Nginx **jen na explicitní privátní IPv4 portu `8080`**. Na těchto dvou portech se ve výpisu neobjevil IPv6 listener ani wildcard listener.
- **Bezpečnostní omezení:** tento výpis neprokazuje zákaz veřejného IPv6 provozu na všech *ostatních* portech ani pravidla Proxmox firewallu. To je oddělená síťová kontrola.
- **Podklad:** výpis konzole DEV LXC, poskytnutý správcem dne 22. 10. 2026. Přístup ChatGPT k LXC nebyl použit.

## Co ještě nebylo doloženo

- Interaktivní otevření webu v prohlížeči uživatele přes LAN (zatím není potvrzené).
- Kontrola pravidel firewallu Proxmoxu včetně IPv6 (nebyla provedena); listenery služeb na příslušných portech byly ověřeny níže.
- Dlouhodobý soak, restart LXC a provozní metriky.
- Živý readsb/BEAST, SSE a PostgreSQL: tyto funkce **zatím nemají implementaci**.

## Bezpečné následné ověření

~~~bash
curl -fsS http://127.0.0.1:4101/api/v1/health
ss -lntp
systemctl --no-pager --full status airradarv2-engine nginx
journalctl -u airradarv2-engine -n 30 --no-pager
readlink -f /opt/airradar-v2/current
~~~

Další etapou je **G-02**: nejprve izolované syntetické testy readsb JSON/BEAST a jediného Rust RAM state. Bez přímého připojení k V1 či databázi a bez produkčního release G-13.
