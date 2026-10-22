# DEV main auto-deploy

Zdokumentovaný incident G-13 je v [runbooku](../docs/runbooks/DEV-MAIN-AUTODEPLOY.md).
`archive_validation.py` je referenční bezpečná validace release archivu. Při
úpravě instalovaného agenta musí být stejná normalizace použita v pull agentu i
root-owned installeru; samotné rozšíření validace nesmí vypnout SHA, podpis
manifestu, kontrolu typu souboru ani rollback.