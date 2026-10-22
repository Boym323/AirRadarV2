# G-02.11 — automatická přejímka v izolovaném DEV LXC

**Stav 22. 10. 2026: časovač nainstalován podle uživatele; offline výsledky z cílového LXC doloženy HTTP 200 / PASS_SYNTHETIC_LXC na SHA `daa15703610b219b5b413319a4920659e19a8694`.** Podrobné hodnoty a metodika jsou v [G-02 reportu](../reports/G-02-VYSLEDEK.md). Tento postup je alternativou k ručnímu sestavení, stahování ZIP a kopírování souborů. V1 a její data zůstávají mimo rozsah. Není to automatické nasazení webu.

## Co se děje automaticky

1. Po `push` do důvěryhodného `main` nebo vývojové větve `codex/g02-auto-lxc-20261022` GitHub Actions provede Rust fmt/Clippy, **26 Rust testů**, JSON/BEAST replay, zátěž 1×/2×/4×, dokumentační a webové testy.
2. Pokud všechny kontroly projdou, sestaví release Rust binárku a statický Vite web. Vytvoří **veřejný prerelease připnutý k plnému SHA**, se dvěma soubory `tar.gz` a `tar.gz.sha256`. Publikace není nasazení. V případě FAIL žádný kandidát nevznikne. [Workflow](../../.github/workflows/g02-dev-candidate.yml).
3. Jednorázově nainstalovaný timer v DEV LXC každých přibližně **15 minut** zkontroluje GitHub API (pouze odchozí HTTPS). U nového kandidáta ověří tag, VERSION, SHA-256 obou souborů včetně GitHub digestu a bezpečně vyjme pouze binárku. Nepoužívá GitHub token, žádný Docker ani self-hosted runner.
4. Rust engin běží samostatně jako neprivilegovaný systémový uživatel `airradarv2`, pouze `--g02-replay` a `--g02-benchmark`. Neotevírá port, nečte readsb, nemění `current` symlink, Nginx web ani stávající API. Výsledky podle konkrétního SHA zapíše do JSON.
5. Souhrn můžeš prohlédnout na **`http://<IP_DEV_LXC>:8080/g02-status.json`**, pouze přes stávající LAN-only Nginx. `status=PASS_SYNTHETIC_LXC` potvrzuje offline syntetický test v cílovém LXC; **neznamená HOTOVO G-02.11 ani reálný live-soak.**

## Jednorázové zapnutí na správném DEV LXC

Zkontroluj, že jsi v **odděleném Debian 13 DEV LXC** (1 vCPU, bez produkční V1), nikoli na Proxmox hostu či v jiné aplikaci. Skript ověřuje existující G-00/G-01 bootstrap `/opt/airradar-v2/current`, unit `airradarv2-engine` a LAN-only Nginx.

Nejdříve vyčkej na úspěšné [GitHub Actions G-02 kandidáta](https://github.com/Boym323/AirRadarV2/actions/workflows/g02-dev-candidate.yml), zkontroluj zdrojové soubory a v **tomto správném LXC jako root** spusť:

~~~bash
cd /root
if ! command -v python3 >/dev/null; then
  apt-get update
  apt-get install -y python3-minimal
fi
curl -fsSLo g02-auto-pull.py \
  https://raw.githubusercontent.com/Boym323/AirRadarV2/codex/g02-auto-lxc-20261022/deploy/g02-auto-pull.py
curl -fsSLo install-g02-auto-lxc.sh \
  https://raw.githubusercontent.com/Boym323/AirRadarV2/codex/g02-auto-lxc-20261022/deploy/install-g02-auto-lxc.sh
python3 -m py_compile g02-auto-pull.py
bash -n install-g02-auto-lxc.sh
sed -n '1,240p' install-g02-auto-lxc.sh
bash install-g02-auto-lxc.sh /root/g02-auto-pull.py
systemctl list-timers --all 'airradarv2-g02-auto*'
systemctl start airradarv2-g02-auto.service
journalctl -u airradarv2-g02-auto.service -n 50 --no-pager
~~~

**Poznámka:** obě stažení výše cílí na *vývojovou větev*. Po sloučení nahraď `codex/g02-auto-lxc-20261022` konkrétním ověřeným SHA nebo `main` pro jednorázové nastavení; během běžného provozu timer už žádné skripty neaktualizuje, stahuje jen immutable kompilované balíčky.

Instalační skript na DEV LXC vytvoří izolované systemd jednotky, přidá přesně jednu read-only cestu do existujícího LAN-only Nginx serveru a aktivuje timer. **Nespouští automatickou instalaci aplikace ani produkční nasazení.** Nevyžaduje další software na macOS.

## Sledování a vypnutí

- GitHub: [automatické buildy kandidátů](https://github.com/Boym323/AirRadarV2/actions/workflows/g02-dev-candidate.yml) a [release](https://github.com/Boym323/AirRadarV2/releases)
- DEV web: `/g02-status.json`, obsahuje SHA, datum, PASS/FAIL, p95, RSS a raw CPU ticks
- LXC: `systemctl status airradarv2-g02-auto.timer` a `journalctl -u airradarv2-g02-auto.service -n 100 --no-pager`
- Vypnutí: `systemctl disable --now airradarv2-g02-auto.timer`. Web a hlavní `airradarv2-engine` zůstávají v provozu.
- Opakování stejného SHA: timer úspěšný či neúspěšný SHA automaticky znovu netestuje, aby se neopakovaly těžké testy; další commit vytvoří nový kandidát.

## Co automatika zatím NEOVĚŘUJE

Trvalý příjem **živého** BEAST/readsb, dlouhodobý reconnect/soak, výpadek fyzické stanice a bezpečnostní kontroly produkčního nasazení. Připojení V1 přijímače bez výslovného schválení nepouštět. G-02 nadále **10/11, TESTED**, dokud nebude doložen provozní test a přejímka. GitHub runner pro veřejný repozitář v domácí LAN záměrně nepoužíváme.

## Potvrzená první automatická přejímka

Uživatel dodal 22. 10. 2026 HTTP výstup z veřejné reverzní proxy: `/g02-status.json` vrací `PASS_SYNTHETIC_LXC` (14:02 SELČ, SHA `daa15703`, replay i benchmark PASS). **Potvrzuje to dostupnost status endpointu i funkční testovací řetězec z pohledu uživatelova Macu**. Z prostředí asistenta nemohl být endpoint nezávisle načten. Nyní je status rovněž veřejně dostupný přes `https://airradarv2.pomykal.cz/g02-status.json`, nejen v LAN; neuváděj do něj interní IP, tajemství ani diagnostické údaje mimo přejímku. Druhá brána live-soak nadále neproběhla.

## Další automatický test při novém GitHub commitu

Nová release binárka G-02.11 spouští po syntetickém 5 000-track benchmarku též integrovaný **loopback JSON/BEAST reconnect soak** bez externího přijímače. Není nutná opětovná instalace Python časovače, protože jeho stávající `--g02-benchmark` nyní zahrnuje i tento scénář. V `/g02-status.json` kontroluj v `benchmarkLog` řádek `G-02 isolated-loopback soak PASS: ...`, nikoli jen `status=PASS_SYNTHETIC_LXC`. Až tento řádek opravdu vznikne **na cílovém LXC**, doplníme číselné výsledky a rozhodneme o provozním live-soak. Tento test nenahrazuje test skutečného RF toku.

Optimalizované `--g02-benchmark` nyní obsahuje **180 sekund** syntetického BEAST/JSON loopback provozu s reconnecty; CI debug měří jen 5 sekund. Celý `benchmark` zůstává pod stávajícím per-command 300s timeoutem. V `benchmarkLog` sleduj `duration_ms` alespoň `180000`, nulové `queue_drops`, p95 a `rss_peak_kib`. Nepovažuj CI záznam s `duration_ms=5000` za důkaz 180sekundového LXC soak.

## Automatické vzdálené potvrzení bez dalších zásahů

Nový samostatný GitHub Actions job `verify-dev-lxc` po úspěšném vydání balíčku pravidelně **pouze čte** veřejný HTTPS `https://airradarv2.pomykal.cz/g02-status.json`. Porovnává plné `sha` s právě vydaným commitem a vyžaduje `replay=PASS`, `benchmark=PASS`, všech šest zátěžových variant a izolovaný TCP/JSON soak >=180 000 ms, TCP reconnect, JSON recovery, nulové `queue_drops`, platné RSS i vyčištěný TTL. Nejvýše 25 jednou za minutu opakovaných pokusů; žádné SSH, CI tokeny v LXC ani zásah do prod V1. Když není nový report, brána FAIL/NEOVĚŘENO, ne falšovaný PASS. Kontrola výsledků je stále pouze *syntetická*. Reálný hodinový test autorizovaného readsb zůstává otevřeným rizikem.
