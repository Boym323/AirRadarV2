# G-02.11: hodinová read-only přejímka ze skutečného přijímače

**Stav: NEPROVEDENO.** G-02 = 10/11. Jediný konzument BEAST OUTPUT TCP :30005. Žádné změny V1, readsb, SDR, DB, V2 webu ani existující služby. Aktivní KEEP: J01/J02/J03/J04/J05/J11.

## 1. Ověř připojení z DEV LXC (ne z Macu ani V1)

~~~bash
timeout 4 bash -c ': </dev/tcp/192.168.1.50/30005' \
  && echo 'BEAST OUTPUT 30005 dostupný'
~~~

Standardní BEAST output port readsb je 30005; port 30004 je vstupní a je zakázaný. Pokud spojení neprojde, nic na přijímači ani V1 nepřekonfigurovávej bez souhlasu. Volitelný JSON diagnostický endpoint tar1090 lze vyzkoušet bezpečným GET:

~~~bash
curl -fsS --max-time 4 http://192.168.1.50/tar1090/data/aircraft.json | head -c 300
~~~

JSON endpoint nemusí existovat. Samotný G-02 live ingest umí BEAST TCP; JSON adapter nyní očekává lokální readsb JSON soubor, proto není HTTP GET automaticky připojení druhého transportu.

## 2. Jednorázové spuštění omezené služby na DEV LXC

Spusť pouze jako root v ověřeném DEV Debian 13 LXC. Nejprve zkontroluj stávající systemd službu a plný SHA posledního úspěšného syntetického reportu.

~~~bash
set -euo pipefail
systemctl is-active --quiet airradarv2-engine
systemctl is-active --quiet airradarv2-g02-auto.timer
curl -fsS https://airradarv2.pomykal.cz/g02-status.json \
 | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x["status"],x["sha"])'
curl -fsSLo /usr/local/libexec/airradarv2-g02-live-soak.py \
  https://raw.githubusercontent.com/Boym323/AirRadarV2/codex/g02-live-receiver-soak-20261022/deploy/g02-live-soak.py
python3 -m py_compile /usr/local/libexec/airradarv2-g02-live-soak.py
chmod 0755 /usr/local/libexec/airradarv2-g02-live-soak.py
systemd-run --unit=airradarv2-g02-live-soak \
  --description='AirRadarV2 DEV BEAST read-only 60min test' \
  --property=Type=exec --property=User=airradarv2 \
  --property=Group=airradarv2 \
  --property=WorkingDirectory=/var/lib/airradar-v2/g02-auto \
  --property=NoNewPrivileges=yes --property=PrivateTmp=yes \
  --property=ProtectSystem=strict --property=ProtectHome=yes \
  --property='ReadWritePaths=/var/lib/airradar-v2/g02-auto /var/cache/airradarv2-g02-public' \
  --property=MemoryMax=384M --property=CPUQuota=50% \
  --property=CPUWeight=10 --property=RuntimeMaxSec=3900 \
  /usr/bin/python3 /usr/local/libexec/airradarv2-g02-live-soak.py \
    --receiver 192.168.1.50 --port 30005 \
    --sha d0cefc4a5c3332eadccf8ab21c96bae51987abe9 --seconds 3600
~~~

Povinný preflight: veřejný report musí obsahovat právě SHA d0cefc4a5c3332eadccf8ab21c96bae51987abe9 a status PASS_SYNTHETIC_LXC, jinak tester bezpečně odmítne start. Skript používá existující SHA-256 ověřovacího agenta k přímému stažení přesné release binárky. Nepřepisuje ani nespouští žádnou produkční jednotku; běží pod uživatelem airradarv2.

## 3. Kontrola

~~~bash
systemctl status airradarv2-g02-live-soak.service --no-pager
journalctl -u airradarv2-g02-live-soak.service -n 30 --no-pager
curl -fsS https://airradarv2.pomykal.cz/g02-status.json | python3 -m json.tool
~~~

Do veřejného g02-status.json přibude po skončení jen klíč liveSoak: doba, počty pozorování, tracků, drops, CPU, peak RSS, výsledek. Neobsahuje interní IP adresu ani polohy/callsigny letadel. Podrobné soukromé logy: /var/lib/airradar-v2/g02-auto/real-soak/real-beast.log.

Přerušení jen testu: systemctl stop airradarv2-g02-live-soak.service.

## 4. Přijetí

- PASS_LIVE_BEAST_60MIN: alespoň 3 600 s, živé zprávy DF17 a alespoň chvíli zdravý BEAST zdroj, žádný channel/capacity drop, RSS menší než 256 MiB a proces bez předčasného pádu.
- INCONCLUSIVE: nedostatek skutečných letových zpráv (může být ticho v éteru).
- FAIL: předčasný konec, dropy, překročená paměť nebo selhání preflight.
- CPU se změří a posoudí; výstupní syntetické P95 není automaticky P95 skutečného RF ingestu. V1 a databáze zůstanou beze změny.

Až po skutečných naměřených údajích posoudit G-02 jako 11/11 a případně stav KEEP. Příprava tohoto skriptu není důkaz provedení hodinového testu.
