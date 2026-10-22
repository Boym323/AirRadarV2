# V2 DEV main auto-deploy — průběžný implementační report

**22. 10. 2026. Stav: připraveno v PR #10, nový systemd timer dosud NEINSTALOVÁN, první automatický release/rollback provozně NEOVĚŘENO.**

## Implementace

- Využit existující CI TEST+SHA256 release z main, žádná duplicitní build pipeline ani příchozí SSH.
- Přidán ověřovač provenance GitHub Actions bota, SHA HEAD main/tagu/releasu, SHA256 asset manifestu a whitelistu archivu.
- DEV instalační skript zpřísněn: identita izolované služby, bezpečný archiv, atomický symlink, zdravotní kontroly, návrat na starší release při chybě.
- Samostatný systemd timer každých 5 minut, root sandbox, odklad při G-02 testech a zablokování chybného SHA. Jen diagnostika bez interních adres na /dev-deploy-status.json.
- Testy provenance a bezpečnosti jsou připojené k Rust/web/docs CI; finální důkazy až z konkrétního zeleného CI commitu PR #10.

## Ověřené podklady

- Po sloučení G-02 do main skončila [stávající build/release pipeline #39808221884](https://github.com/Boym323/AirRadarV2/actions/runs/39808221884) úspěšně.
- **Není důkaz**, že již došlo k automatickému přepnutí Rust/Vite webu: instalaci služby v DEV LXC teprve zbývá jednorázově provést. Nenahrazovat tento chybějící důkaz výsledkem CI.

## Neuzavřené brány

První skutečný deployment v DEV LXC a odpovídající status SHA; bezpečný řízený rollback na testovací vadné verzi; otestování disku, systemd sandboxu a Nginx z cílového serveru; automatická retence starších releases. Produkční G-13 zůstává NO-GO, V1/DB/receiver beze změn.

[Instalační runbook](../runbooks/DEV-MAIN-AUTODEPLOY.md)
