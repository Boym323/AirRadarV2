# DEV AirRadarV2: automatické vydávání ověřeného GitHub main

**Stav 22. 10. 2026: implementace připravena v PR #10, jednorázová aktivace v LXC ještě neověřena.** Jen vývojové LXC, nikoli produkční G-13 cutover, V1 nebo databáze.

## Jak to funguje

1. Merge do **main** spustí stávající GitHub Actions [G-02 DEV kandidát](../../.github/workflows/g02-dev-candidate.yml): Rust fmt, clippy, unit/replay, Vite lint/typecheck/test/build a kontrolu dokumentace. Teprve po PASS publikuje bot immutable GitHub release **g02-dev-SHA** s tar.gz a SHA256 manifestem.
2. Samostatný DEV systemd timer přibližně každou **jednu minutu** zjišťuje aktuální **main HEAD**. Bere jen release odpovídající přesnému SHA, vydaný **github-actions[bot]**, se správným tagem, URL, SHA-256 manifestem, původem assetů, VERSION a bezpečným obsahem tar.
3. Pokud běží G-02 offline nebo LIVE test, změna se odkládá. Pokud se main mezitím posunul, starší balíček se nenasadí.
4. Instalátor uloží release do /opt/airradar-v2/releases, atomicky přepne **current**, restartuje pouze **airradarv2-engine.service** a reloaduje DEV Nginx, provede HTTP healthcheck a ověření Vite indexu. Při chybě obnoví předchozí release a chybný SHA se nezkouší v nekonečné smyčce.
5. Poslední výsledek je v souboru /dev-deploy-status.json: DEPLOYED / WAITING_FOR_CI / POSTPONED_G02_ACCEPTANCE / ROLLED_BACK / ERROR_NO_DEPLOY. Podrobnou diagnostiku lze přečíst v systemd journalu.

**Bez SSH, webhooku do LAN, self-hosted runneru, dotyků V1, DB migrací nebo změn konfigurace readsb.**

## Jednorázová instalace v DEV LXC (nikoli Proxmox host nebo V1)

Před instalací musí být PR #10 i následná optimalizace minutové kontroly sloučené do main a finální CI SHA zelené. Použij celý neměnný commit SHA výsledného schváleného PR (ne staré G-02 release SHA). Uvnitř **lo-dev-airradarv2, jako root**:

~~~bash
G13_SOURCE_REF=SEM_DOSAD_CELY_40ZNAKOVY_SHA_MERGE
curl -fsSLo /root/install-main-dev-auto.sh \
  "https://raw.githubusercontent.com/Boym323/AirRadarV2/$G13_SOURCE_REF/deploy/install-main-dev-auto.sh"
bash -n /root/install-main-dev-auto.sh
G13_SOURCE_REF="$G13_SOURCE_REF" bash /root/install-main-dev-auto.sh
~~~

Po úspěšné instalaci je timer automaticky aktivní a další vydání už žádnou konzoli LXC nevyžadují. **Pokud již existuje dřívější 5minutový timer, spusť znovu celý tento bezpečný jednorázový instalátor z nejnovějšího schváleného SHA** — aktualizuje root-owned Python agent i pevně definovaný systemd timer a provede daemon-reload. Pouhé `systemctl daemon-reload` bez nové jednotky interval nezmění. Na první kontrolu:

~~~bash
systemctl status airradarv2-main-deploy.timer --no-pager
systemctl start airradarv2-main-deploy.service
journalctl -u airradarv2-main-deploy.service -n 60 --no-pager
cat /var/cache/airradarv2-main-deploy/status.json
cat /opt/airradar-v2/current/VERSION
curl -fsS http://127.0.0.1:4101/api/v1/health
~~~

**Diagnostická služba nemá zobrazení V1 údajů.** Nový Nginx endpoint vrací pouze hash commitu, výsledek a čas.

## Ochrana a omezení

- Root-owned Python agent běží v systemd oneshot s **ProtectSystem=strict, ProtectHome=yes, NoNewPrivileges=yes, MemoryMax=768M, CPUQuota=60 % a TimeoutStartSec=300**. Zapisovat smí jen do DEV release adresáře, soukromého stavu a veřejného statusu.
- Balíček musí mít správné SHA-256 a pouze pravidelné soubory v allowlistu, bez symlinků/hardlinků/traversalu; archiv je maximálně 150 MiB komprimovaně, 300 MiB rozbaleně.
- Rollback zachovává předchozí release. **Retence starých verzí dosud není automatická**, protože před smazáním je nutné vyhodnotit 20GiB disk a případnou zálohu.
- Při chybě stejného SHA zůstává kandidát zablokován; nová verze main může být nainstalována. Diagnostika: journalctl -u airradarv2-main-deploy.service.
- Vypnutí dalšího automatického nasazování: **systemctl disable --now airradarv2-main-deploy.timer**. Běžící web a Rust engine tím nepřestanou fungovat.
- Merge do main zatím **nezapíná G-03 živý API režim**: ten má opt-in, který patří do vlastní vývojové etapy. G-04 DB migrace a produkční G-13 release zůstávají samostatně schvalované.
- Kdo může změnit main nebo vydat release, může ovlivnit DEV binárku. Doporučena branch protection a povinné CI; žádný široký token/heslo v LXC není potřebný.

**Do doby doloženého prvního nasazení v LXC nelze automatiku označit za provozně aktivní.**

## Rychlý release a minutová kontrola — změna z 22. 10. 2026

Běžná CI pro vydání balíčku v [g02-dev-candidate.yml](../../.github/workflows/g02-dev-candidate.yml) **končí ihned po úspěšném vydání immutable balíčku**, bez čekání na 180s DEV LXC přejímku (dříve až 25 minut). Přejímka se spouští **nezávisle** pomocí [g02-dev-lxc-acceptance.yml](../../.github/workflows/g02-dev-lxc-acceptance.yml) po úspěchu releasu nebo ručně, s původními kontrolami SHA a read-only veřejného reportu. Dlouhý běh samostatné přejímky neblokuje vydání nové verze ani pull-only DEV nasazování.

Systemd timer používá `OnBootSec=1min`, `OnUnitInactiveSec=1min`, `RandomizedDelaySec=0` a `AccuracySec=1s`. To **není záruka přesně jedné minuty od merge**: balíček musí projít kompilací/testy, síť může být pomalejší a čas běhu instalace či G-02 acceptance může přidat odklad. Na úspěšný hotový release obvykle navazuje kontrola v následujícím minutovém intervalu.

**Bez překročení GitHub REST API limitu:** každou minutu se veřejné `main` SHA předběžně zjišťuje z GitHub Atom feedu `https://github.com/Boym323/AirRadarV2/commits/main.atom` (bez GitHub REST API). Před skutečným spuštěním nového kódu se SHA znovu autoritativně ověří pomocí GitHub REST API. Pokud CI pro nový commit nevytvoří release, provede se až 10 kontrol v minutovém intervalu; další kontroly 404 jsou omezené na jednou za 5 minut, aby při dlouhé chybě buildu nevyčerpaly anonymní limit. Pokud Atom feed nejde načíst/validovat, proces **fail-closed**, nikdy nepřepne neověřenou verzi. Nejde o webhook ani GitHub API token v LXC.

**Jednorázová aktualizace již aktivního timeru** uvnitř samostatného DEV LXC:

~~~bash
# Po přijetí oprav do main dosaď celý aktuální SHA commitu s opravou:
SHA=NAHRAD_CELY_40ZNAKOVY_SHA
curl -fsSLo /root/install-main-dev-auto.sh \
  "https://raw.githubusercontent.com/Boym323/AirRadarV2/$SHA/deploy/install-main-dev-auto.sh"
G13_SOURCE_REF="$SHA" bash /root/install-main-dev-auto.sh
systemctl cat airradarv2-main-deploy.timer
systemctl list-timers --all 'airradarv2-main-deploy*'
~~~

U **dříve nainstalovaného** agenta je tato opakovaná instalace nutná pro změnu systemd timeru i samotného mechanismu levného minutového dotazování. Po instalaci zkontroluj `journalctl -u airradarv2-main-deploy.service -n 70 --no-pager`, `cat /var/cache/airradarv2-main-deploy/status.json` a `/opt/airradar-v2/current/VERSION`. Žádný přímý přístup do LXC nebyl z tohoto GitHub PR proveden.
