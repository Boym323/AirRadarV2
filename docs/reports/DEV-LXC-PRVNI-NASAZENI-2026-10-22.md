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

## Co ještě nebylo doloženo

- Interaktivní otevření webu v prohlížeči uživatele přes LAN.
- Ověření skutečných poslouchajících portů (včetně veřejné IPv6) a kontrola firewallu Proxmoxu.
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
