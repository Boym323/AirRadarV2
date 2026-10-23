# V2 DEV main auto-deploy — historický implementační report (22. 10. 2026)

> **Nový stav 23. 10. 2026:** původní příprava níže již byla dokončena. `main → shared CI → SHA256 release → V2 runner → DEV LXC` je funkční a **ACCEPTED** pro G-03: [run #39906245759](https://github.com/Boym323/AirRadarV2/actions/runs/39906245759), runtime `29ee2ee75feab9279123b77d50895d648c5419d7`, všech šest jobů PASS včetně lokálního HTTP/SSE healthchecku. Staré věty o chybějícím auto-deployi či třech workflow níže jsou **původním záznamem incidentu**, nikoliv novým omezením. V repozitáři jsou nyní čtyři workflow včetně samostatné izolované G-04 DB CI; V1 a reálná DB se tím nemění. Aktuální instrukce jsou v [runbooku](../runbooks/DEV-MAIN-AUTODEPLOY.md).


**Historický záznam 22. 10. 2026.** Původní přípravná část reportu zachovává
stav před registrací runneru; aktuální uzavření je uvedeno níže.

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

## Historické neuzavřené brány

Následující seznam popisoval stav před registrací runneru a prvním deploymentem:
registrace, ověření cílových služeb a deaktivace starého timeru. Tyto provozní
body jsou nyní uzavřené; produkční G-13 stále zůstává NO-GO a V1/DB/receiver
zůstávají beze změn.

## Finální ověření stabilizace

Runner `airradarv2-dev` je registrovaný a aktivní jako účet
`airradarv2-runner`. Release `a1319067ae9fb5d01be3851eb3f899d25686e1e3` je
aktivní v `/opt/airradar-v2/current`; engine, Nginx, Rust health API a web byly
ověřeny. Běžný workflow již nečeká na `g02-status.json`: po deployi provádí
kontrolu přímo na cílovém runneru. G-02 replay/benchmarky zůstávají v CI jako
nezávislá diagnostická brána.

`airradarv2-g02-auto.timer` a `airradarv2-main-deploy.timer` jsou `disabled` a
`inactive`; `airradarv2-g02-live-auto.timer` zůstává samostatný. Rollback root
bridge byl dříve řízeně ověřen a zachovává předchozí funkční release.

[Instalační runbook](../runbooks/DEV-MAIN-AUTODEPLOY.md)

## Konsolidace automatizací (22. 10. 2026)

Po funkčním self-hosted Runner deployi [#39828853055](https://github.com/Boym323/AirRadarV2/actions/runs/39828853055) byl připraven úklid na přesně tři aktivní workflow: jeden sdílený testovací `ci.yml`, jedno `main` immutable release + Runner nasazení a jedny denní metriky. Zbytečné ruční build a 60min G-02 LIVE request YAML byly odstraněny pouze z aktivních GitHub workflow, ne z Git historie; již doložený G-02.11 PASS zůstává platný. Push s pouze statistikami nemá restartovat DEV. Dokončení tohoto úklidu vyžaduje novou CI přejímku konkrétního commitu a jednu skutečnou Runner deploy přejímku po merge; do té doby nejde o potvrzený provozní efekt této konkrétní změny.
