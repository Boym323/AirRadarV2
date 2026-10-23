# ADR 0013 — G-04: časově omezené řízené ukončení SQL writeru

**Datum:** 23. 10. 2026. **Stav:** implementováno v PR #21, **živá přejímka nové implementace dosud neprovedena**. Platí jen pro opt-in `--g04-dev-history`.

## Důvod

První živý BEAST/CPR canary úspěšně uložil 1 489 ze 1 502 kandidátů ve 63 SQL dávkách bez chyby, ale `systemd RuntimeMaxSec=1875` ukončil proces výsledkem `timeout` se 13 položkami `pending`. Průběžně nulové `dropped_*` nepotvrzují bezpečné vyprázdnění při zastavení.

## Rozhodnutí

1. Po 1 802 s samostatného G-04 canary Rust **zastaví svůj lokální příjem** BEAST/JSON, bez jakékoliv změny souběžného G-03.
2. Existující `HistorySampler` vyčká uzavření posledního 30s UTC bucketu a 5s ochranné lhůty; nevytvoří umělou pozici a nepřepíše časové razítko. Uzavřené vzorky odesílá přes původní jediný SQLx writer, po každé dávce čeká na výsledek `COMMIT`/ack.
3. Drain smí trvat maximálně 60 s; dokončení writer tasku maximálně další 3 s. `pending=0` a `inflight=0`, potvrzený návrat workeru i závěrečný diagnostický snapshot jsou podmínkou úspěšného návratu procesu (`exit 0`).
4. `RuntimeMaxSec=1875` zůstává **vnější ochranný limit** systemd. Pokud se drain nepodaří, neprohlašovat PASS. Při nouzovém `systemctl stop` zůstává prioritou rychlé zastavení G-04 i za cenu ztráty postradatelných posledních V2 vzorků; nesmí blokovat ochranu sdíleného PostgreSQL.
5. Žádný nový proces, časovač, DB monitoring/SSH účet, index, migrace nebo zásah do G-03/V1/PP Studio. Starý release ponechat beze změny; novou binárku instalovat do samostatného připnutého adresáře a až po kontrolách opakovat ručně dozorovaný DEV canary.

## Regresní a provozní brána

- Automatické testy: potvrzené vyprázdnění pending po ack, záporný ack, nepřijatý ack s deadlinem, skutečný izolovaný SQLx/PostgreSQL readback přes stejnou drain funkci.
- Nový živý canary: `exit 0` bez `Result=timeout`, poslední `pending/inflight=0`, `committed+drops=eligible`, `sql_timeouts=0`, `connection=Up`, alespoň 1 800 s dokladů, ztráty nejvýše 1 %, pozitivní **nové** SQL řádky, živé G-03 SSE a časově doložené disk/WAL/CPU/RAM DB LXC.
- Schválení GO se **nepřenáší na nový release ani na bezobslužný provoz**. Neověřené údaje se nesmí doplňovat odhadem ani synteticky.

