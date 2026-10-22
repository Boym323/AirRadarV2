# DEV AirRadarV2: automatické vydávání ověřeného GitHub main

**Stav 22. 10. 2026: implementace připravena v PR #10, jednorázová aktivace v LXC ještě neověřena.** Jen vývojové LXC, nikoli produkční G-13 cutover, V1 nebo databáze.

## Jak to funguje

1. Merge do **main** spustí stávající GitHub Actions [G-02 DEV kandidát](../../.github/workflows/g02-dev-candidate.yml): Rust fmt, clippy, unit/replay, Vite lint/typecheck/test/build a kontrolu dokumentace. Teprve po PASS publikuje bot immutable GitHub release **g02-dev-SHA** s tar.gz a SHA256 manifestem.
2. Samostatný DEV systemd timer každých přibližně pět minut zjišťuje aktuální **main HEAD**. Bere jen release odpovídající přesnému SHA, vydaný **github-actions[bot]**, se správným tagem, URL, SHA-256 manifestem, původem assetů, VERSION a bezpečným obsahem tar.
3. Pokud běží G-02 offline nebo LIVE test, změna se odkládá. Pokud se main mezitím posunul, starší balíček se nenasadí.
4. Instalátor uloží release do /opt/airradar-v2/releases, atomicky přepne **current**, restartuje pouze **airradarv2-engine.service** a reloaduje DEV Nginx, provede HTTP healthcheck a ověření Vite indexu. Při chybě obnoví předchozí release a chybný SHA se nezkouší v nekonečné smyčce.
5. Poslední výsledek je v souboru /dev-deploy-status.json: DEPLOYED / WAITING_FOR_CI / POSTPONED_G02_ACCEPTANCE / ROLLED_BACK / ERROR_NO_DEPLOY. Podrobnou diagnostiku lze přečíst v systemd journalu.

**Bez SSH, webhooku do LAN, self-hosted runneru, dotyků V1, DB migrací nebo změn konfigurace readsb.**

## Jednorázová instalace v DEV LXC (nikoli Proxmox host nebo V1)

Před instalací musí být PR #10 sloučený do main a finální CI SHA zelené. Použij celý neměnný commit SHA výsledného schváleného PR (ne staré G-02 release SHA). Uvnitř **lo-dev-airradarv2, jako root**:

~~~bash
G13_SOURCE_REF=SEM_DOSAD_CELY_40ZNAKOVY_SHA_MERGE
curl -fsSLo /root/install-main-dev-auto.sh \
  "https://raw.githubusercontent.com/Boym323/AirRadarV2/$G13_SOURCE_REF/deploy/install-main-dev-auto.sh"
bash -n /root/install-main-dev-auto.sh
G13_SOURCE_REF="$G13_SOURCE_REF" bash /root/install-main-dev-auto.sh
~~~

Po úspěšné instalaci je timer automaticky aktivní a další vydání už žádnou konzoli LXC nevyžadují. Na první kontrolu:

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
