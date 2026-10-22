# G-02.11 — bezpečná přejímka ingestu na samostatném DEV LXC

**Stav:** NEPROVEDENO. Tento postup není dokladem výsledků. Vývojové G-02 změny jsou v [PR #2](https://github.com/Boym323/AirRadarV2/pull/2). V1, její LXC, DB a readsb jsou nedotknutelné.

## První brána: reprodukovatelný izolovaný benchmark

1. Ověř, že vývojový kontejner je správný Debian 13 x86_64 DEV LXC, má přidělený **1 vCPU** a žádná následující operace nemění jeho systemd jednotku `airradarv2-engine`. Nepracuj na Proxmox hostu ani na produkční V1.
2. Na GitHubu spusť [ruční sestavení DEV balíčku](https://github.com/Boym323/AirRadarV2/actions/workflows/dev-package.yml) se zdrojovou větví PR #2. Běh pouze sestavuje artefakt; nic nenainstaluje ani nenasadí.
3. Stáhni ZIP s dvojicí `airradarv2-dev-linux-amd64-<SHA12>.tar.gz` a `.tar.gz.sha256`; **oba soubory** bezpečně přenes do `/root/` DEV LXC (dle existujícího [runbooku](DEV-LXC-PRVNI-NASAZENI.md)). SHA balíčku musí odpovídat otestovanému HEAD PR #2. Neinstaluj artefakt přes produkční instalační skript; aktivní služba zůstává beze změny.
4. Přímo uvnitř **izolovaného** DEV LXC ověř checksum a rozbal pouze do testovací cesty. V následujících příkazech nahraď `<ARCHIV>` přesným souborem ze staženého artefaktu:

~~~bash
cd /root
sha256sum -c <ARCHIV>.sha256
mkdir -p /root/g02-acceptance
tar -xzf <ARCHIV> -C /root/g02-acceptance
cd /root/g02-acceptance
cat VERSION
nproc
free -m
uname -a

# Tyto režimy nepřipojují readsb, DB, HTTP ani V1.
timeout 60s ./airradarv2-engine --g02-replay
timeout 180s ./airradarv2-engine --g02-benchmark | tee /root/g02-acceptance/benchmark.log

# Aktivní původní G-00/G-01 zůstává nezměněná.
systemctl is-active airradarv2-engine nginx
curl -fsS http://127.0.0.1:4101/api/v1/health
~~~

**Záznam přejímky:** SHA `VERSION`, CPU limit a počet vCPU, datum, architektura, velikost RAM, výstup benchmarku (p95, RSS, raw CPU ticks), 1×/2×/4×, odmítnutí 5001. tracku, počet neplatných zpráv a stav služeb před/po. Benchmark je syntetický a cílový DEV výkon nelze nahradit pouze hodnotami z GitHub Actions.

## Druhá brána: dlouhodobý reconnect/soak

Zejména při přerušení TCP, chybách JSON, změnách pořadí zpráv a zahlcení musí zůstat RAM ohraničená a identita zachovaná. **GitHub CI poskytuje syntetické unit/loopback testy, ne hodinový soak reálného přijímače.**

Dokud nebude schválen read-only odběr konkrétního izolovaného **testovacího** readsb/BEAST zdroje a zajištěno, že nezačne druhý produkční konzument, **nespouštěj `--g02-live`**. Testovací příjem musí být pouze čtecí, bez SQL, API, push, SSE a přepnutí systemd; výpadek nesmí ovlivnit V1. Po bezpečném otestování zapiš dobu souvislého běhu, počet reconnectů a zahazovaných pozorování, p95, CPU/RSS i chování při obnovení zdroje.

## Pravidlo závěru

G-02 smí dostat **HOTOVO** a primární KEEP ID případně `ACCEPTED` teprve po úspěchu obou bran, přesném SHA, doložených hodnotách a bezpečnostním auditu. V opačném případě zachovej **10/11, ROZPRACOVÁNO, TESTED**. Žádné automatické nasazení do DEV LXC ani produkce.

## G-02 LIVE po vypnutí periodického offline timeru (údržba 22. 10. 2026)

Běžné DEV CI/CD již testuje Rust replay/benchmarky a bezpečně nasazuje
schválený immutable release. `airradarv2-g02-auto.timer` zůstává vypnutý,
LIVE poller `airradarv2-g02-live-auto.timer` zůstává aktivní.

LIVE test se nově autorizuje **výhradně podle aktivního**
`/opt/airradar-v2/current/VERSION`, přesného SHA aktuálního GitHub `main`,
botem vydaného immutable `g02-dev-<SHA>` prerelease a krátkodobého
jednorázového `g02-live-<SHA>-<run-id>` požadavku. Dále se ověřuje
Git tag, manifest SHA-256 a nepovolené položky archivu. Test používá
jedinou předem schválenou privátní IPv4 adresu BEAST OUTPUT portu 30005.

**Starý** `g02-status.json.sha` a `PASS_SYNTHETIC_LXC` se nemění:
jsou historickým offline důkazem. Nový výsledek se zapisuje jen do
`g02-status.json.liveSoak` a obsahuje vlastní přesné SHA a requestRunId.
Veřejný LIVE GitHub gate kontroluje právě `liveSoak.sha`, nikoli
historické `status.sha`.

### Jednorázová instalace opraveného LIVE agenta do DEV LXC

Protože LIVE agent běží ze souborů vlastněných rootem v `/usr/local/libexec`,
samotný push na GitHub jej **neaktualizuje**. Na ověřeném DEV LXC po úspěšném
CI, pouze pokud LIVE služba právě neběží, proveď:

~~~bash
cd /opt/airradarv2
git fetch origin main
git merge --ff-only origin/main
python3 -m unittest discover -s tools -p 'test_g02_live_*.py'
systemctl is-active --quiet airradarv2-g02-live-auto.service && {
  echo 'LIVE test právě běží: nyní skripty nepřepisovat'; exit 1;
}
install -o root -g root -m 0755 deploy/g02-live-auto.py /usr/local/libexec/airradarv2-g02-live-auto.py
install -o root -g root -m 0755 deploy/g02-live-soak.py /usr/local/libexec/airradarv2-g02-live-soak.py
systemctl is-enabled airradarv2-g02-live-auto.timer
systemctl is-active airradarv2-g02-live-auto.timer
~~~

Tento postup **nezapíná** offline timer, neregis­truje nový runner ani
nepřepisuje receiver IP, DB či běžící engine. K ručnímu spuštění LIVE
přejímky použij samostatný GitHub workflow s explicitním aktuálním SHA
a potvrzením `SPUSTIT_READONLY_60MIN`; po hodině ověř
`liveSoak.requestRunId` a metriky. Nespouštěj jej bez schválení
read-only provozu; předchozí 60min PASS se novému SHA nepřipisuje.
