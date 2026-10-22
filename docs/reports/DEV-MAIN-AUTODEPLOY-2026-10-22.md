# V2 DEV main auto-deploy — průběžný implementační report

**22. 10. 2026. Stav: CI/CD a root bridge připraveny, řízený rollback root bridge PASS, repo-scoped runner dosud NEZAREGISTROVÁN a první Actions deployment NEOVĚŘEN.**

## Implementace

- Využit existující CI TEST+SHA256 release z main, žádná duplicitní build pipeline ani příchozí SSH.
- Přidán ověřovač provenance GitHub Actions bota, SHA HEAD main/tagu/releasu, SHA256 asset manifestu a whitelistu archivu.
- DEV instalační skript zpřísněn: identita izolované služby, bezpečný archiv, atomický symlink, zdravotní kontroly, návrat na starší release při chybě.
- Samostatný repo-scoped runner `airradarv2-dev`, root sandbox, odklad při G-02 testech, sdílený flock a zablokování chybného SHA přes existující installer. Starý timer zůstává do prvního ověřeného runner deploymentu.
- Řízený rollback test s nefunkčním kandidátem `ffffffffffffffffffffffffffffffffffffffff`: healthcheck selhal, `current` se vrátil na `227a8619b590d2c536761ffefc75cd0a5633936c`, engine zůstal active a API opět vrátilo `{"status":"bootstrap","stage":"G-00"}`. Vadný release nebyl mazán.
- Testy provenance a bezpečnosti jsou připojené k Rust/web/docs CI; finální důkazy až z konkrétního zeleného CI commitu PR #10.

## Ověřené podklady

- Po sloučení G-02 do main skončila [stávající build/release pipeline #39808221884](https://github.com/Boym323/AirRadarV2/actions/runs/39808221884) úspěšně.
- **Není důkaz**, že již došlo k automatickému přepnutí Rust/Vite webu: instalaci služby v DEV LXC teprve zbývá jednorázově provést. Nenahrazovat tento chybějící důkaz výsledkem CI.

## Neuzavřené brány

Registrace runneru, první skutečný Actions deployment a odpovídající status SHA; otestování disku, systemd sandboxu a Nginx z cílového serveru; deaktivace starého timeru. Produkční G-13 zůstává NO-GO, V1/DB/receiver beze změn.

[Instalační runbook](../runbooks/DEV-MAIN-AUTODEPLOY.md)
