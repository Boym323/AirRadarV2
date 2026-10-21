# G-09 — Původní OGN a externí obohacení — ZRUŠENO

**Stav:** VYŘAZENO (bez implementace)
**Závislosti:** žádná
**Primární KEEP ID:** žádná; interní brána nebo zrušený modul

## Cíl

Původní etapa se po vyřazení E04–E10/H03 **nerealizuje**.

## Postup

1. Nepřipojovat OGN/DDB/SoftRF, FlightAware, ADSBDB, Planespotters, RXW Hub; nepsat nepotřebnou cache či integrace.
2. Nezavádět staré kontaktní klíče ani funkcionalitu jen proto, že byly ve V1.

## Testy a měření


- Ověřit nulový aktivní externí provider a nulové požadavky na jejich credentials v čistém G-00/G-02.

## Co neimplementovat

Neobnovovat E04–E10 ani H03 bez explicitní změny produktové matice.

## Kritéria dokončení

Neprovádět žádnou novou implementaci. V plánu etapu evidovat jako zrušenou a zachovat pouze návaznost dalších G-XX.

## Bezpečnost a provoz

AirRadar V1 a jeho LXC/databáze jsou pouze read-only reference. Žádná etapa sama neopravňuje ke změně produkčního provozu nebo reálnému rozesílání notifikací.
