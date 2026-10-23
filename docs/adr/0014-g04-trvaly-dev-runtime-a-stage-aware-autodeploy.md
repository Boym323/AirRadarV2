# ADR 0014 — Trvalý G-04 DEV runtime a stage-aware autodeploy

**Datum:** 23. 10. 2026. **Stav:** přijato pro izolovaný DEV LXC. **Rozsah:** G-04/J09, nikoli produkční G-13.

## Kontext

G-04/J09 bylo v DEV přijato, ale `main → runner → LXC` stále instaloval pouze
`--g03-live-api` a post-deploy kontrola vyžadovala `database_connected=false`.
Merge tedy nasadil nový binární artefakt, ale nepřepnul runtime na novou aktivní
funkci.

## Rozhodnutí

- Release obsahuje `web/runtime-stage.txt`, vytvořený z verzovaného
  `deploy/runtime-stage`.
- Po přijetí G-04/J09 root-owned instalátor atomicky přepíná immutable release i
  systemd režim na `--g04-live-history`; G-03 zůstává součástí stejného
  HTTP/SSE runtime a read-only BEAST vstupu.
- Přihlašovací údaje zůstávají pouze v existujícím root-owned
  `/etc/airradar-v2/g04-canary.env`; do artefaktu, argumentů ani logů se
  nekopírují. PostgreSQL se nemění a nepoužívá V1 databázi.
- Stage-aware healthcheck vyžaduje shodu release markeru, API `stage`, živého
  ingestu a příznaku databáze. Při nesouladu se provede rollback release i
  systemd override.
- Budoucí přijaté etapy musí změnit marker společně s kódem, testy, reportem a
  runtime kontraktem; neschválené etapy se do běžící služby nepřidají implicitně.

## Důsledky a hranice

G-04 writer běží jako doplňkový DEV proces; živý RAM/SSE příjem nesmí čekat na
PostgreSQL. Produkční V1, její LXC, DB, firewall a readsb konfigurace se
nemění. G-04 canary `--g04-dev-history` zůstává samostatný testovací režim.

## Důkaz

Konzumenti: Rust G-04/J09, root-owned DEV instalátor, GitHub Actions release a
post-deploy healthcheck. Testy: Rust health, `cargo test`, shell/Python syntaxe,
stage boundary a workflow regrese; release se smí označit PASS až po CI a
lokálním healthchecku přesného SHA.
