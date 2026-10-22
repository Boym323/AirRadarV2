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
službami. Při aktivním G-02 vrací 75 a GitHub job opakuje pokus. Po úspěšné
prvním ověřeném Actions deploymentu se starý timer deaktivuje:

```bash
systemctl disable --now airradarv2-main-deploy.timer
```

Do té doby zůstává aktivní jako jediná ověřená cesta obnovy.

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

Aktivní release zůstává `227a8619b590d2c536761ffefc75cd0a5633936c`, dokud
runner neprovede první ověřený deployment. Root bridge a sdílené G-02 lock
drop-iny jsou připravené; první Actions job, rollback a deaktivace starého
timeru jsou zatím provozně neověřené.
