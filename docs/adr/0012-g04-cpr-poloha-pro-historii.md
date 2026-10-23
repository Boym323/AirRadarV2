# ADR 0012 — G-04: skutečná BEAST/CPR poloha bez přeznačení času

**Datum:** 23. 10. 2026. **Stav:** přijato pro opravu PR #21. **Rozsah:** G-01/G-02 ingest a G-04 sampler; bez nového zdroje dat nebo služby.

## Kontext

BEAST-only DEV canary přijal data, ale G-04 vykázal `eligible=0`, protože
`contract::beast_arrival()` nastavoval `timeBasis=server_arrival` a historický
kontrakt povoloval pouze `derived_readsb` a `receiver_utc`. Současně je nutné
odlišit skutečnou polohu rekonstruovanou z validních DF17/CPR rámců od libovolné
pozice dodané s časem příjmu serverem.

## Rozhodnutí

- Validní globální even/odd nebo bezpečně lokální CPR z BEAST dostane
  `positionOrigin=adsb_cpr`; čas zůstane `timeBasis=server_arrival` a
  `positionAtUtc=receivedAtUtc`. Není to důkaz receiver UTC.
- Historický vzorek je způsobilý pouze pro `rx-primary` + `beast_tcp` + přesně
  `adsb_cpr` + `server_arrival`, se shodným časem zprávy/příjmu, čerstvou platnou
  souřadnicí a po existující CRC/CPR, deduplikaci a fyzikální plausibilitě.
- Obecný `server_arrival` vzorek zůstává odmítnutý. `beast_arrival()` se nemění
  na historický vstup jen změnou časového označení.
- RAM-first adaptivní sampler, 30s bucket, bounded fronta 2048/128, omezený
  retry a jediný SQLx writer se nemění. JSON+BEAST stejného `receiverId` nadále
  sdílí jediný RAM stav a watermark.

## Důkazy a hranice

Regrese pokrývá BEAST-only validní CPR → RAM → sampler → SQLx → izolovaný
PostgreSQL readback a negativní případy neplatného CPR, staré polohy, duplicit,
nesprávných časů a JSON+BEAST slučování. Sdílené PostgreSQL, G-03, V1 a PP
Studio se nemění. Provozní DEV canary se tímto ADR nespouští ani neschvaluje.
