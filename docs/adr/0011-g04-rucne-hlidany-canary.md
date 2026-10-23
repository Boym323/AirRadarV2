# ADR 0011 — První G-04 DEV canary bez automatického monitoringu DB LXC

**Rozhodnutí vlastníka:** 23. 10. 2026. **Stav:** schválen způsob přípravy, nikoli samotné spuštění. **Rozsah:** jednorázový, osobně dozorovaný G-04/J09 canary v samostatném DEV LXC.

## Kontext

Implementace a regresní testy G-04 prošly. V2 má vlastní SHA-pinned binárku a opt-in systemd službu. Původně vyžadovaný automatický dohled DB potřeboval nový SSH účet, klíč a forced-command helper na sdíleném PostgreSQL LXC. Vlastník výslovně rozhodl pokračovat **bez automatického monitoringu databázového LXC**.

## Rozhodnutí

- První canary bude jednorázový, časově omezený a **osobně hlídaný správcem**. Nevyžaduje SSH monitorovací účet ani monitorovací wrapper. Služba `airradarv2-g04-canary.service` spouští přímo nezměněný engine z commitu `fe0ac66fdebea25d21a2d211cd74cab922dd7d05`, na loopback portu 4114.
- Zachovat: `CPUQuota=50%`, `MemoryMax=2G`, `Restart=no`, `RuntimeMaxSec=1875`, `EnvironmentFile` root:root 0600, oddělený BEAST klient a databázi `airradar_v2_dev`. Service se nesmí povolit při bootu. G-03 na portu 4101 zůstává nedotčený.
- **Přijaté riziko:** bez automatického DB dohledu se G-04 **samo nezastaví při zaplnění DB filesystemu nebo přetížení PostgreSQL CPU/RAM**. Systemd CPU/RAM limity chrání jen aplikační proces, nikoliv sdílenou DB a jiné aplikace.
- Před spuštěním se ručně zaznamená baseline DB LXC a G-03. Po dobu testu musí být člověk přítomen a nejpozději každých 5 minut zkontrolovat DB volné místo, WAL, CPU/load, dostupnou RAM a zdraví G-03 i dalších důležitých služeb (PP Studio).
- **Okamžité STOP pouze G-04**, pokud DB volné místo klesne pod 2 GiB, zátěž DB je trvale vysoká, WAL roste neobvykle, SQL přestane odpovídat, G-03/PP Studio degraduje nebo správce nemůže dohlížet. Stop: `systemctl stop airradarv2-g04-canary.service`. G-03 se nikdy nerestartuje a databázové řádky se nemažou.
- Po doběhnutí se výsledky `eligible/committed/dropped_*`, `pending/inflight`, `sql_timeouts`, readback V2 DB, SSE a ručně doložená hostitelská měření vyhodnotí stávajícím evaluatorem. Limit ztrát nejvýše 1 %. Evaluator je **následné vyhodnocení**, nikoliv automatický živý dohled.
- Připojení zůstává v důvěryhodné LAN bez TLS (ADR 0009); zálohy postradatelné V2 historie nejsou povinné (ADR 0010). Žádné změny `pg_hba.conf`, firewallu, clusterového PostgreSQL, V1 ani PP Studio. Budoucí DDL migrace vyžadují samostatný schválený účet/postup.

## Hranice schválení

Toto rozhodnutí platí jen pro **první ručně dozorovaný DEV canary**. Není automatickým souhlasem k jeho startu ani k bezobslužnému/trvalému zapnutí historie. Monitoring návrh zůstává pouze neaktivním historickým podkladem a **není blokací GO pro tento jednotlivý test**. Reálný GO pro spuštění musí vlastník vydat zvlášť po posledním preflightu.
