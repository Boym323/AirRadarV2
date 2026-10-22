# DEV main auto-deploy

Zdokumentovaný incident G-13 je v [runbooku](../docs/runbooks/DEV-MAIN-AUTODEPLOY.md).
`archive_validation.py` je referenční bezpečná validace release archivu. Při
úpravě instalovaného agenta musí být stejná normalizace použita v pull agentu i
root-owned installeru; samotné rozšíření validace nesmí vypnout SHA, podpis
manifestu, kontrolu typu souboru ani rollback.

Nový runner se instaluje přes `install-runner-dev.sh`. Pokud už existuje
registrovaný runner, skript jeho registraci zachová a pouze synchronizuje
provozní bridge; nevytváří druhou službu ani nevyžaduje nový token. Artefakty
Actions se ukládají do jednorázových adresářů
`incoming/<full-SHA>.<run-id>/`, takže úklid cizích/root-owned souborů nemůže
zablokovat další běh. `runner-deploy.sh` je jediný privilegovaný vstup z runneru.

Starý pull timer `airradarv2-main-deploy.timer` je po ověření runner deploymentu
vypnutý. Běžná cesta vede přes GitHub Actions runner; root bridge a G-02 stále
sdílejí lock a při kolizi má G-02 přednost. Periodický veřejný G-02 report není
podmínkou běžného deploymentu.
