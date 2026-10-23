# Automatické nasazování `main` do DEV LXC

**Aktuální provozní stav 23. 10. 2026 — FUNKČNÍ, DEV ACCEPTED.** G-03 běží na izolovaném LXC s `--g03-live-api`, read-only BEAST, SSE snapshot/delta a heartbeat; [Actions #39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759) pro `29ee2ee75feab9279123b77d50895d648c5419d7` má všech šest jobů PASS včetně runner deploymentu/post-deploy ověření. Jednorázový bootstrap root-owned G-03 instalátoru už proběhl, nepouštět jej rutinně před každým push. Další push do `main` po úspěšném CI automaticky vytváří SHA-ověřený prerelease a nasazuje bez schvalování. Starší incidentní pokyny níže jsou **historické** a nejsou blokací dnešního G-03 provozu.

**Zbývající rizika:** nový rollback nebyl úmyslně znovu vyvolán na živém DEV; OS TCP socket memory a dlouhý soak nejsou finálně změřeny. G-11/G-13 produkční přejímka zůstává samostatná.

## Cílová cesta přes repo-scoped runner

GitHub-hosted workflow `g02-dev-candidate.yml` po úspěšném G-02 replay,
benchmarku, Rust/web testech a publikaci prerelease `g02-dev-<SHA>` spouští
deploy job výhradně na runneru s labelem `airradarv2-dev`. Job neprovádí
checkout zdrojů; stahuje jen dva assety přesného prerelease, ověří metadata,
SHA-256, `VERSION` a aktuální `main` a přes jedno sudo pravidlo volá
root-owned `/usr/local/libexec/airradarv2-runner-deploy.sh`.

Runner se instaluje z checkoutu jako účet `airradarv2-runner`:

```bash
RUNNER_REGISTRATION_TOKEN='krátkodobý-repo-token' deploy/install-runner-dev.sh
```

Token je platný pouze pro registraci, není uložen v repozitáři ani v systemd.
Runner používá již registrovanou službu
`actions.runner.Boym323-AirRadarV2.airradarv2-dev.service` a pracovní adresář
`/opt/airradarv2-actions-runner`; instalační oprava registraci nemaže ani
nevytváří druhou službu. Dočasné release soubory jsou v jednorázovém
`/var/lib/airradarv2-runner/incoming/<SHA>.<run-id>/`, vlastněném pouze účtem
runneru. Nepoužívá V1 checkout ani credentials.

Root bridge sdílí `/run/lock/airradarv2-main-deploy.lock` s G-02 offline/LIVE
službami. Při aktivním G-02 vrací 75 a GitHub job opakuje pokus. Starý polling
timer je po ověření runneru vypnutý a nový deployment ho nepoužívá:

```bash
systemctl is-enabled airradarv2-main-deploy.timer # disabled
systemctl is-active airradarv2-main-deploy.timer  # inactive
```

Obnova release probíhá přes stejný root bridge a rollback uvnitř instalačního
mechanismu; nevytváří se paralelní privilegovaná cesta.

## Incident 22. 10. 2026

Systemd journal doložil opakované ukončení služby:

```text
DEV auto-deploy rejected safely: ValueError Unsafe archive member
airradarv2-main-deploy.service: Main process exited, code=exited, status=1/FAILURE
```

Příčinou je validátor tarballu v pull agentu a installeru. GNU tar při
`tar -C <dist> .` zapisuje kořenový adresář jako `.` nebo `./`; původní kód
odstranil pouze prefix `./` a záznam `.` proto odmítl jako nebezpečný. Nejde o
odklad kvůli G-02 ani o selhání healthchecku. Ověření SHA, původu release,
kontrola obsahu, sandbox a rollback musí zůstat aktivní.

## Bezpečná oprava

Kořenové adresářové záznamy `.` a `./` se mají normalizovat na prázdný název a
přeskočit pouze tehdy, když jsou adresářové. Ostatní cesty se dál kontrolují
proti absolutní cestě, `..`, symlinkům, hardlinkům, zařízením, duplicitám a
seznamu povolených souborů. Tato oprava musí být nasazena současně do pull
agenta i installeru.

Regresní test je v `tests/test_archive_validation.py` a pokrývá GNU tar kořen,
traversal a symlink.

## Oprava incidentu s `incoming`

Runner běží jako `airradarv2-runner`, nikoli jako root. Původní workflow mazalo
sdílený obsah `incoming/*`; root-owned soubor z předchozí cesty proto vyvolal
`Permission denied`. Workflow nyní nepřepisuje ani nemaže cizí položky: vytvoří
nový adresář pro přesný SHA a run ID, stáhne pouze dva assety, provede kontrolu
manifestu z tohoto skutečného adresáře a po dokončení jej odstraní vlastním
účtem. Root bridge navíc kontroluje režim, vlastníka a tvar staging cesty.

Kontrola `VERSION` používá výstup GNU tar do souboru, nikoli křehkou kontrolu
přes stdout command substitution; validace SHA-256 a bezpečného tar allowlistu
zůstává povinná.

## Historický provozní stav před G-03 (22. 10. 2026)

Runner `airradarv2-dev` je registrovaný, aktivní a běží jako
`airradarv2-runner`. Ověřený release je
`a1319067ae9fb5d01be3851eb3f899d25686e1e3`; Rust engine, Nginx, lokální API a
web jsou aktivní. Workflow po CI a immutable prerelease provádí post-deploy
kontrolu přímo na tomto runneru: služby, přesný `VERSION`, umístění symlinku,
JSON health API, web a krátkou kontrolu, že engine nespadl.

G-02 replay a benchmarky zůstávají v CI. Veřejný `g02-status.json` ani
periodický `PASS_SYNTHETIC_LXC` nejsou podmínkou běžného deploymentu. Timer
`airradarv2-g02-auto.timer` je vypnutý, `airradarv2-g02-live-auto.timer` zůstává
oddělený podle aktuálního provozního stavu a `airradarv2-main-deploy.timer` je
vypnutý. Při kolizi s G-02 deployment bezpečně čeká přes sdílený lock.

## Historický úklid CI/CD (22. 10. 2026) — nyní čtyři aktivní workflow

- `ci.yml`: jediná kompletní sada Rust fmt/Clippy/test, G-02 replay/benchmark, HTTP smoke, Vite/React, OpenAPI/schema a české dokumentace. Spouští se pro PR, ručně a jako **reusable workflow** před vydáním DEV releasu. Samostatný push trigger na main je záměrně odebraný, protože tytéž testy provádí volání z release.
- `g02-dev-candidate.yml`: jediná cesta **main → sdílená CI → release Rust/Vite → immutable SHA+SHA256 prerelease → self-hosted Runner → DEV LXC → místní post-deploy health a rollback**. Název souboru zůstává kvůli existujícím odkazům, ale zobrazený název je obecný DEV release; nevytváří se další G-02 workflow. Na ručním workflow_dispatch lze vydat artefakt jen z main; automatické nasazení provede pouze push do main.
- `g04-postgres-migrations.yml`: **čtvrtý, oddělený DB-only PostgreSQL 18 CI workflow** bez nasazení V2 DB na skutečné LXC a bez release/deploy. Nezaměňovat jej s G-03.
- `development-metrics.yml`: denně dopočítá README / JSON / SVG metriky. Commit měnící pouze `README.md` a `docs/metrics/**` nemá vydávat ani nasazovat aplikaci; proto jsou tyto cesty u DEV release podmíněně ignorované. Změní-li stejný commit i jiný soubor, release se běžně spustí.

Odstraněná historická workflow (stále dostupná v Gitu): `dev-package.yml` byl duplicitní manuální build bez nasazení; `g02-live-dev-acceptance.yml` byla jednorázová hodinová G-02.11 přejímka, která již proběhla a má důkaz [run #39795592425](https://github.com/Boym323/AirRadarV2/actions/runs/39795592425). Python přejímací nástroje, golden fixture, regresní testy a záznamy skutečného provozu zůstávají zachovány. Historický `PASS_LIVE_BEAST_60MIN` není vyžadován při každém běžném deploymentu.

Postup ověření: PR CI (tři PASS joby); potom kontrola, že jeden nový push main spustí jen **DEV release workflow**, které nejprve zavolá reusable CI, sestaví a ověří release a provede Runner deployment se healthcheckem. **Znovunasazení samostatného LXC nebo změna V1 se neprovádí.**

## Historický P0 incident: dočasná G-03 release blokace (22. 10. 2026)

**Historický stav k 22. 10. (již neplatí):** G-03 měla 10/10 izolovaných CI testů a tehdy bylo dovoleno sloučit **pouze implementační kód** do `main`. Workflow `g02-dev-candidate.yml` po merge spustí sdílené CI a sestaví SHA-bound artefakt pro prohlížení v Actions, ale step `G-03 — bezpečná stage gate před publikací` nastaví výstup `g03_runtime=true` při existenci `contracts/openapi.g03.yaml` a povinného `web/g03-runtime-required.txt`. Job `publish-dev-candidate` má `if: ... g03_runtime == 'false'`, takže se nepublikuje žádný `g02-dev-<SHA>` prerelease; závislý `deploy-dev` je přes `needs:` rovněž SKIPPED. Vlastní G-00 DEV služba dál běží bez restartu.

**Toto je politika bez přepínače:** její změna vyžaduje separátní auditovaný PR po instalaci a testu root-owned runtime verifieru, skutečného G-03 systemd nastavení, Nginx streamu a atomického G-03↔G-00 rollbacku; není povoleno nastavit pouze repository variable a nasazení tím obejít. Automatický puller nevydá G-03 bez odpovídajícího prerelease; jeho stav a služby musí zůstat beze změny.

**Kontrola po merge:** očekávat v GitHub Actions `validate=PASS`, `test-and-build=PASS`, `publish-dev-candidate=SKIPPED`, `deploy-dev=SKIPPED`. Pokud by publish/deploy běžel, je to P0 incident: zastavit workflow před kontaktem s DEV LXC.

## Historický P1 návrh G-03 runtime přejímky (nyní aktivováno)

SHA-připnutý balíček s `contracts/openapi.g03.yaml` dostane při buildu `web/g03-runtime-required.txt`. Před deploymentem Runner ověřuje přítomnost nainstalovaného root instalátoru `G03_STAGE_GATE_V1` a verifikátoru `verify-g03-runtime.py`; pokud chybí, deploy odmítne **před přepnutím symlinku**. Instalátor vyžaduje aktivní systemd `--g03-live-api` a následně ověřuje HTTP health `stage=G-03`, pravdivý `live`/`waiting_for_observations`, nový `snapshot` s epoch/seq, stream přes DEV Nginx a 15s heartbeat do 40s. `bootstrap` ani `degraded` nesmí projít. Chyba G-03 streamu ještě uvnitř root EXIT trap vyvolá rollback.

**Historický stav před 23. 10. — překonáno:** tehdejší G-03 cutover byl NO-GO, protože PR neinstalovalo do LXC nový root-owned skript. Před prvním G-03 deploymentem je potřeba bezpečně aktualizovat instalátor z ověřeného commitu, schválit změnu systemd režimu a ověřit celý config+binary rollback (starý G-00 binary nemusí umět `--g03-live-api`). Dokud se neověří tato atomická konfigurace a cílový DEV streaming, neslučovat G-03 do main. Provozní G-00/G-02 Runner workflow zůstává neměnné.


## Historie G-03: obnovení automatické distribuce z main (22. 10. 2026)

Na výslovné přání vlastníka se odstraňuje zákaz G-03 release. Každý push do `main` nyní automaticky provede `shared CI → immutable Rust/Vite build → SHA256 prerelease → existující self-hosted V2 runner → root-owned instalátor → stage-aware health HTTP/SSE`. Žádné další ruční schválení jednotlivých vydání není potřeba. Nový instalátor transakčně připraví systemd `--g03-live-api`, převezme dříve schválený read-only BEAST vstup z G-02 služby a přepne verzi; každé selhání vrací předchozí verzi **i systemd režim**.

**Historický postup, již proveden 23. 10.:** Při prvním zprovoznění byla nutná jednorázová aktualizace root-owned installeru (existující runner sudoers nedovolují samotnému GitHub jobu nahradit root-owned soubory). Na `lo-dev-airradarv2` jako root:

```bash
cd /opt/airradarv2
git fetch origin main
git switch main
git pull --ff-only origin main
bash deploy/activate-g03-main-autodeploy.sh
```

Jednorázový skript přísně ověřuje původ a SHA `main`, root-owned checkout, schválenou BEAST G-02 službu, běžící V2 engine/Nginx/runner i `nginx -t`, potom pouze instaluje root-owned bridge, archivový validátor, G-03 runtime verifier a transakční instalátor. **Nerestartuje** běžící engine. Následující push do `main` spustí plný automatický deployment. Pokud bootstrap není proveden, workflow při preflightu selže před změnou služby, místo aby tvrdilo falešný úspěch.

Pro ověření skutečné aktivace: GitHub Actions workflow musí mít všech šest jobů PASS (zejména `deploy-dev`), na LXC `systemctl show airradarv2-engine.service -p ExecStart`, `curl -fsS http://127.0.0.1:4101/api/v1/health` vrátí `stage=G-03`, `/api/v1/stream` odešle `event: snapshot` a Nginx postupně doručí heartbeat. **Aktualizace 23. 10.: první i další automatické DEV cutover jsou již OVEŘENY** — [#39905085484](https://github.com/Boym323/AirRadarV2/actions/runs/39905085484), [#39905632012](https://github.com/Boym323/AirRadarV2/actions/runs/39905632012) a [#39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759) všechny `SUCCESS`.
