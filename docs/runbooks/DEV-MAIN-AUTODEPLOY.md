# Automatické nasazování `main` do DEV LXC

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

## Aktuální provozní stav

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
