# G-02.11 — spouštění hodinového LIVE testu přímo z GitHub Actions

**Stav:** implementace připravena v PR #7, před aktivací je potřeba jednorázová instalace služby v izolovaném DEV LXC. Žádná změna V1, DB ani stávající V2 engine. KEEP J01/J02/J03/J04/J05/J11.

## Princip

GitHub má samostatný workflow v defaultní větvi \`main\`: **AirRadar V2 — G-02 LIVE přejímka (DEV)**. Vlastník vybere potvrzení \`SPUSTIT_READONLY_60MIN\`; GitHub ověří existující SHA-připnutý \`g02-dev-<SHA>\` release a vydá jednorázovou release značku \`g02-live-<SHA>-<run_id>\`. Samotný GitHub runner **nevidí domácí LAN**, nevydává shell příkazy pro kontejner a neobsahuje domácí IP.

Oddělený systemd timer \`airradarv2-g02-live-auto.timer\` (přibližně každé 3 minuty) vyžádá odchozím HTTPS veřejný seznam GitHub releasů. Agent \`deploy/g02-live-auto.py\` ověří typ, stáří do 45 minut, autora \`github-actions[bot]\`, plné SHA tagu i cílového commitu, přítomnost identického předchozího \`PASS_SYNTHETIC_LXC\` a neexistenci dříve vyřízeného request ID. GitHub release nikdy neobsahuje příkaz ke spuštění.

Pouze po úspěšném ověření spustí existující \`deploy/g02-live-soak.py\` přes jediný read-only BEAST OUTPUT port \`30005\` s pevnou délkou 3 600 sekund. Nijak nemění přijímač, V1, databázi, produktové API/SSE ani stávající systémovou jednotku \`airradarv2-engine\`. Sekundové trace zůstanou lokálně. Veřejný \`/g02-status.json\` obsahuje jen agregovanou sekci \`liveSoak\`, s \`requestRunId\`, SHA, CPU/RSS, drops a PASS/FAIL/INCONCLUSIVE. GitHub workflow tuto sekci porovnává se stejným workflow run ID a hodnotí PASS teprve po skutečných 60 minutách.

## Jednorázová aktivace v DEV LXC

**Nepouštět v Proxmox hostu ani v produkčním V1 kontejneru.** Nejdříve musí projít CI posledního PR #7 a tlačítko workflow z PR #6 musí být v \`main\`.

Při jednorázové aktivaci vezmi **aktuální plný ověřený SHA z PR #7** (nikoli SHA staré binárky G-02) a uvnitř **V2 DEV LXC jako root** spusť:

~~~bash
SOURCE_SHA=NAHRAD_40ZNAKOVYM_OVERENYM_SHA_PR7
curl -fsSLo /root/g02-live-installer.sh \
  "https://raw.githubusercontent.com/Boym323/AirRadarV2/$SOURCE_SHA/deploy/install-g02-live-auto.sh"
G02_SOURCE_REF="$SOURCE_SHA" bash /root/g02-live-installer.sh PRIVATNI_IP_ADRESA_PRIJIMACE
~~~

Instalátor na správném Debian 13 x86_64 ověří již nainstalovaný G-02 auto timer, existenci V2 enginu a účet \`airradarv2\`; stáhne skripty pod týmž immutable SHA, kontroluje Python syntaxi a založí pouze **novou** oneshot službu a timer. Limity: \`NoNewPrivileges=yes\`, \`ProtectSystem=strict\`, \`ProtectHome=yes\`, \`CPUQuota=50%\`, \`MemoryMax=384M\`, \`TimeoutStartSec=3900\`, zapisuje jen do vlastních \`/var/lib/airradar-v2/g02-auto\` a \`/var/cache/airradarv2-g02-public\`. Neinstaluje další démon, Docker, SSH ani GitHub token.

Kontrola instalace:

~~~bash
systemctl status airradarv2-g02-live-auto.timer --no-pager
systemctl list-timers --all 'airradarv2-g02-live-auto*'
systemctl status airradarv2-engine --no-pager
~~~

## Každý další test — už pouze GitHub

1. Otevřít Actions → **AirRadar V2 — G-02 LIVE přejímka (DEV)** → **Run workflow**.
2. \`approved_sha\`: plný SHA **posledního skutečně offline ověřeného** G-02 releasu (výchozí testovaný \`d0cefc4a5c3332eadccf8ab21c96bae51987abe9\`). Automat v LXC úmyslně odmítne jiný SHA.
3. \`confirm\`: **SPUSTIT_READONLY_60MIN**, potvrdit.
4. GitHub workflow čeká až 79 minut na read-only veřejný report pro přesné SHA + request ID. Když spojení nelze navázat nebo test selže, nedojde k falešnému PASS. Při úspěchu zobrazí výsledky v GitHub Actions. Zobrazí se i v \`https://airradarv2.pomykal.cz/g02-status.json\`.

Poznámka: standardní GitHub Actions job nemůže automaticky oznámit výsledek před skončením celé hodiny. Běžící test lze přerušit z DEV LXC příkazem \`systemctl stop airradarv2-g02-live-auto.service\`, nebo ve workflow zastavit čekající GitHub úlohu (to nezaručuje zastavení samotného LXC testu). Opakované stisknutí tlačítka v době testu se kvůli GitHub concurrency zařadí až po skončení předchozího běhu.

## Bezpečnostní hranice a otevřená akceptace

- Příjem je jen čtení BEAST, nikoli připojení na vstupní port 30004; JSON lokální soubor se záměrně nepřepíná v rámci této etapy.
- Jeden fyzický receiver na PRIVATNI_IP_ADRESA_PRIJIMACE zůstává pod kontrolou současného readsb; stream ke konzumentům se nemění.
- Agent se **nikdy** nespustí z pouhé nové verze zdrojového kódu: potřebuje explicitní GitHub user kliknutí a schválenou immutable release značku.
- Při preflight FAIL nebo nedostatku živých frame končí zkouška bez G-02 11/11; výsledky nelze dopředu předpokládat.
- Zbývá posoudit skutečné CPU/RAM, ztráty a omezenou instrumentaci P95 živé RF cesty. Závěr se zapíše do přejímacího reportu G-02, nikoli automaticky do matice.

## Vypnutí

~~~bash
systemctl disable --now airradarv2-g02-live-auto.timer
systemctl stop airradarv2-g02-live-auto.service
~~~

Netýká se \`airradarv2-g02-auto.timer\` pro již existující offline testy, ani V1/webu.

## Zjištěná závada Type=oneshot — oprava pro další běhy

Doložený audit z DEV LXC potvrdil, že `RuntimeMaxSec=3900` **systemd ignoruje pro `Type=oneshot`**. Opraven instalační vzor na `TimeoutStartSec=3900` (3900 s od začátku procesu). Interní Python subprocess timeout 3800 sekund zůstává sekundární pojistkou. GitHub změna nemění již instalovanou jednotku. **Běžící 60min test nepřerušovat.** Teprve po jeho dokončení provést v DEV LXC: `systemctl edit --full airradarv2-g02-live-auto.service`, opravit jedinou direktivu, následně `systemctl daemon-reload`, `systemctl show airradarv2-g02-live-auto.service -p Type -p TimeoutStartUSec -p RuntimeMaxUSec`; jednotku neresetovat/restartovat během testu. Alternativně po doběhu jednorázově znovu nainstalovat ověřený skript s novým SHA.

**Reálný přenos již prokázán z výstupu auditu správce**: BEAST TCP `ESTABLISHED`, `preferred=Beast` 591 vzorků, 16 532 updates, nulové `channel_drops` i `capacity_drops`, 66 peak tracks, veřejná sekce `liveSoak` odpovídá requestu `39795592425` a je ve stavu `RUNNING`. **Toto není 60min PASS.**
