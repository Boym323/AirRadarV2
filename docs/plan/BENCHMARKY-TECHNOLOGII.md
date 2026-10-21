# Benchmarkový a validační plán technologií AirRadar V2

**Stav:** REPRODUKOVATELNÝ PLÁN, **nikoli hotová měření**. Žádný zde popsaný benchmark nebyl v rámci tohoto technologického průzkumu na produkčním LXC, skutečném readsb ani reálné PostgreSQL/SQLite databázi spuštěn.  
**Pravidlo:** žádná tabulka bez poznámky „NEZMĚŘENO“ nesmí být vyplněna odhadem či čísly z cizího projektu.

## Závazná aktualizace měření

PostgreSQL bylo definitivně vybráno a existuje v samostatném DB LXC. **B05 již nemusí rozhodovat mezi SQLite a PostgreSQL**: musí měřit skutečné PostgreSQL/SQLx zápisy, SQL dotazy, pool, síťovou latenci, TLS, retenci a backup+restore na oddělené DEV DB. Níže popsané SQLite srovnání je pouze starší referenční scénář, nikoliv úkol nebo důvod zavádět SQLite do V2.

G-00 web již používá Vite, případné srovnání s Next.js má pouze diagnostický význam.

## Protokol společný pro všechna měření

1. Použít izolovaný vývojový checkout, přesné SHA, lockfily, konkrétní verze Rust/Go/Node, OS/kernel, cgroup limit CPU/RAM a hardware LXC.
2. Stejné **syntetické či anonymizované** vstupy se známými očekávanými výsledky; vstupy verzovat mimo soukromou produkční DB.
3. Rozlišit *studený start*, *zahřátý běh* a *soak*; každou kombinaci spustit nejméně třikrát po restartu, reportovat medián, p95/p99 a variabilitu.
4. Fixovat concurrent clients, délku běhu, velikost jednotlivých snapshotů, počet letadel, intervaly změn, TLS/proxy konfiguraci, nastavení komprese a případný scheduler/throttle.
5. Měřit CPU user/system, RSS peak/steady, alokace/GC tam, kde existují, disk read/write/WAL, síť bytes/s, latenci vstup→RAM, RAM→SSE, vstup→browser, chyby a dropped updates.
6. Zvýšení rychlosti nevyvažovat skrytým rizikem nesprávné identity, špatných poloh, neplatného squawk nebo nekonečného růstu front. Správnost je **povinná před výkonem**.
7. Produkční V1 a její LXC/DB se nesmějí stát objektem destruktivního či zátěžového testu. read-only shadow až po schválení a prokázání minimálního dopadu.

## B01 — Runtime ingest: Rust vs Go vs Node

**Minimální parity prototyp:** číst stejné anonymizované BEAST frame fixture i readsb JSON, udržovat identický normalizovaný RAM state a zpřístupnit jednoduché HTTP/SSE. Není férové porovnat kompletní Rust decoder s Go serverem, který pouze přeposílá původní JSON.

**Vstupy:** 25, 100, 500 a 2 000 syntetických současně živých cílů; replay 1×, 2× a 4×; CPU 1/2/4 vCPU podle dostupné izolované instance. BEAST/JSON parity podle golden expected state.

**Kritéria:** parity správných identit a poloh; CPU %, RSS MiB, p95/p99 latence ingest, rps SSE, drop/retry count a délka/údržba implementace. Go a Node mohou být malé měřicí spike, **nesmějí se zavádět do produkčního repozitáře** bez přínosu a ADR.

**Aktuální výsledky:** NEZMĚŘENO.

## B02 — Výchozí readsb JSON versus BEAST

**Scénáře:** JSON 1 s (dle skutečné konfigurace), případně 0,5 s, BEAST rámce podle skutečného replay; měřit postupně, pak oba v jednom engine a přechod BEAST→JSON→BEAST.

**Kritéria:** správná ICAO/ne-ICAO identita, no phantom tracks, CPR/CRC parity, latence aktualizace, CPU, celkový objem přenesených dat, doba odpojení/recovery, absence leaků. U složitých binárních měření BDS použít speciální golden data.

**Výsledek:** NEZMĚŘENO. **Očekávání** nižší latence BEAST není naměřené potvrzení.

## B03 — SSE full versus delta a alternativy

**Vstupy:** 25/100/500/2 000 letadel; 1/10/100 klientů, 1Hz a 2Hz serializace; jeden z klientů extrémně pomalý; krátký i několika hodinový soak.

**Kritéria:** p50/p95/p99 RAM→client, MB/s na klienta i celkem, počet alokací/CPU na klienta, heartbeat/jitter, reconnect, ztracené sekvence, limit front. Hlídat maximální délku recovery a korektní počáteční snapshot.

**Porovnávat:** full snapshot SSE, initial full + shared delta SSE, případně prostý HTTP polling jako kontrolní jednoduché řešení. WebSocket až s doloženým obousměrným požadavkem.

**Výsledek:** NEZMĚŘENO.

## B04 — Web: Next.js server, Next.js export, Vite + React

**Férové implementace:** tatáž statická česká úvodní stránka i totožný malý MapLibre/React radarový prototyp s fiktivním SSE serverem; stejný React, MapLibre, UI komponenty a množství dat. Rozlišovat „browser cost“ od „server cost“.

**Měřit:** cold/warm \`npm run build\`, velikost gzip/brotli JS/CSS, počet načtených souborů, LCP, INP, velikost JS parse/execute, počet React commitů, počet MapLibre \`setData\`/\`updateData\`, FPS a dlouhé úlohy při pan/zoom, RSS případného Next serveru.

**Testy:** prohlížeč Chromium i mobilní režim (ideálně skutečné pomalejší zařízení), klávesnice, PWA manifest, service worker/HTTPS a bezpečnost web push. Vite bundle musí splnit stejné produktové scénáře.

**Závěr podle dokumentace:** Vite nepotřebuje produkční Node server, zatímco Next.js server mode ano. Samotný výkon mapy **NEZMĚŘENO**.

## B05 — SQLite WAL versus PostgreSQL + SQLx

### Shodný dotazový model

- \`flight_samples\` s identitou, timestampem, pozicí, výškou, rychlostí, případně squawk a provenance.
- Potřebné indexy pro konkrétní dotazy aktivních funkcí, ne všechny hypotetické indexy z V1.
- INSERT vzorků přes **jeden bounded writer**, batch v transakci, podobné retention/cleanup.
- Query: poslední body letadla, počet typů/aerolinek/tras v agregátu, stručný logbook, cleanup a restart.

**Příklady kapacity — pouze ilustrační model, nikoliv měření:** 50 současně ukládaných letadel, 1 vzorek na letadlo za 15 sekund při trvalé přítomnosti = 288 000 řádků/den, tj. 8,64 milionu za 30 dní. Čistých hypotetických 100 bajtů na řádek by dalo přibližně 864 MB bez indexů, stránkování, WAL a metadat; skutečný disk bude jiný a zvolený sampling musí být odvozen z reálného provozu.

**Zátěže:** 100/500/2 000 zapisovaných vzorků na dávku, v různé frekvenci; 1/5/20 čtecích klientů, současná retence, nucený checkpoint SQLite, kontrolovaný pád writeru, restart a obnova ze zálohy.

**Výstupy:** rows/s, transaction p95/p99, query p95/p99, CPU/RSS obou procesů i DB, WAL/DB size, checkpoint duration/blocked readers, počet ztracených vzorků, složitost obnovy.

**Automatické rozhodnutí neprovádět z jediné latence!** SQLite vybrat, pokud stabilně a s rezervou pokrývá realistic workload, backup i retention a nepotřebujeme síťový multiwriter přístup. Jinak PostgreSQL.

**Výsledek:** NEZMĚŘENO.

## B06 — MapLibre \`setData\` versus \`updateData\`

**Vstupy:** stabilní unique feature IDs, 25/100/500/2 000 letadel, 1/2/5 aktualizací/s, trail 30/120/600 bodů, pohyb mapy a vyhledávání současně.

**Varianta A:** kompletní GeoJSON \`setData\` v batch. **Varianta B:** stabilní GeoJSON source a diff \`updateData\`. Testovat stejnou verzi MapLibre, totožnou style/layer konfiguraci a mobilní GPU.

**Měřit:** FPS, p95 render duration, main-thread long tasks, worker CPU, paměť prohlížeče, čas setData/updateData, React commits, chybovost při ztraceném contextu.

**Výsledek:** NEZMĚŘENO. Diff strategie musí prokazovat smysl; jinak použít jednodušší A.

## B07 — LXC runtime, záloha a bezpečnost

**Vlastní V2 LXC, nikdy V1.** Porovnat dvě varianty:
- Vite statika + jeden Rust engine + SQLite lokální soubor + proxy.
- Vite statika + jeden Rust engine + oddělená PostgreSQL + proxy.

**Měřit:** RAM po bootu/idle/steady/p95 peak, CPU, disk I/O, storage growth, p95 health response, čas start/restart/rollback/backup/restore, OOM a zaplnění disku. Zatížit pouze vývojový container s limity.

**Bezpečnostní brána:** V2 service accounts, tajemství, readsb read-only, public DTO sanitizace, proxy SSE buffering, HTTP/2/browser, TLS/PWA/Push, skutečně oddělená V1 infrastruktura.

**Výsledek:** NEZMĚŘENO; původní návrh 4 vCPU / 8 GiB / 60 GiB **není měřené minimum**.

## Připravená šablona pro skutečný report

~~~markdown
# Výsledek experimentu BXX

- Commit a přesný SHA:
- Datum, LXC VMID (interní evidence), CPU, RAM, OS/kernel:
- Implementace/alternativy a přesné lockfily:
- Dataset (SHA256 fixture), počet letadel, klientů, interval:
- Příkazy a nastavení, počet opakování:
- Správnost / regrese: PASS / FAIL:
- CPU p50/p95, RSS peak/steady:
- Latence p50/p95/p99, propustnost:
- Disk/DB/WAL, síť, ztráty:
- Omezení měření a variabilita:
- Doporučení a důvody:
- Odkaz na artefakty a CI:
~~~

## Co blokuje finální uzavření rozhodnutí

Bez referenčních anonymizovaných JSON+BEAST sekvencí, přístupu do izolovaného V2 LXC, stejného zátěžového harnessu a skutečných výsledků **nelze vydávat kvantitativní srovnání** Rust/Go/Node ani PostgreSQL/SQLite. Při zahájení G-01 nejdřív připravit fixtures a syntetický SSE/DB workload. Neprovádět zátěžové testy na produkci V1.
