# Automatické nasazování `main` do DEV LXC

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

## Aktuální provozní stav

Aktivní release zůstává `227a8619b590d2c536761ffefc75cd0a5633936c`; status byl
`ERROR_NO_DEPLOY`. V tomto běhu nebylo možné zapsat `/usr/local/libexec`,
spustit systemd přes D-Bus ani ověřit síťový GitHub release, proto provozní
oprava a nové automatické nasazení zůstávají neověřené.