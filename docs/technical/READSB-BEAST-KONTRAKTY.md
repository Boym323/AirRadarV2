# readsb JSON, BEAST TCP a deduplikace — návrh kontraktů G-01/G-02

**Stav:** implementační kontrakt **k ověření fixture**. **Aktivní funkce:** J01, J02, J03, J04, J05, J11, návazně L06, W08. Lokalita a konfigurace přijímače jsou neveřejné.

## 1. readsb JSON: přesný vstup a význam času

\`aircraft.json\` obsahuje kořenový \`now\` (Unix sekundy), \`messages\` (celkový čítač zpráv od startu readsb) a pole \`aircraft\`. Každý prvek má \`hex\`, případně volitelná pole. \`~abcdef\` je **ne-ICAO**, nikoliv ICAO \`abcdef\`.

- \`seen\`: sekundy od poslední jakékoli zprávy vzhledem ke kořenovému \`now\`.
- \`seen_pos\`: sekundy od **poslední polohy**, ne od poslední zprávy; mít oddělený \`lastMessageAt\` a \`positionAt\`.
- \`lat\`, \`lon\`: volitelné; neexistující polohu **nedoplňovat** jako (0, 0), neuvažovat nulové souřadnice bez kontextu automaticky za chybu.
- \`flight\`, \`alt_baro\` včetně možné hodnoty \`ground\`, \`alt_geom\`, \`gs\`, \`track\`, \`baro_rate\`, \`squawk\`, \`category\`, \`r\`, \`t\` jsou volitelné/zdrojově závislé; nenahrazovat neznámou hodnotu nulou.
- \`squawk\`: **čtyřznakový oktální řetězec** \`[0-7]{4}\`, nikoliv číslo; zachovat úvodní nuly.
- \`messages\` je kumulativní počítadlo od restartu readsb; pokud klesne, je to restart nebo změna zdroje, ne negativní tok zpráv.
- Jakákoli volitelná metadata \`r\`/\`t\` mohou vyžadovat místní readsb DB; jejich absence není chyba.
- Poloha \`positionAt = now - seen_pos\` pouze pokud jsou validní \`now\`, \`seen_pos\` a \`lat/lon\`. U klientských hodin neodvozovat tuto hodnotu z aktuálního \`Date.now()\`.

Minimální syntetický fixture (\`hex\`/callsign smyšlené, nikoli skutečné letadlo):

~~~json
{
  "now": 1792627200.0,
  "messages": 12345,
  "aircraft": [{
    "hex": "abc123",
    "flight": "TEST123 ",
    "lat": 49.9, "lon": 15.3,
    "seen": 0.2, "seen_pos": 1.5,
    "alt_baro": 12300, "gs": 180.0,
    "track": 270.0,
    "squawk": "7600"
  }, {
    "hex": "~abc123", "seen": 2.0
  }]
}
~~~

**Nutná vstupní validace:** absolutní UTC \`now\` nesmí být neuvěřitelně daleko od monotónního času příjmu/mapovaného wall clocku (výchozí tolerance 10 s + konfigurace receiveru); \`seen\`/\`seen_pos\` konečné nezáporné hodnoty, \`lat\` -90..90, \`lon\` -180..180, validní oktální squawk. Přidat maximální velikost JSON snapshotu (např. 4 MiB) a maximální počet prvků (např. 5 000), fuzz testy a odmítnutí nevalidních hodnot. Tyto limity jsou startovací bezpečnostní omezení, ne změřená potřeba.

## 2. BEAST TCP: stavový rámcovač

| Typ | Prefix | MLAT čas | Síla signálu | Payload |
|---|---|---|---|---|
| Mode A/C | \`1a 31\` | 6 B | 1 B | 2 B |
| Mode-S short | \`1a 32\` | 6 B | 1 B | 7 B |
| Mode-S long | \`1a 33\` | 6 B | 1 B | 14 B |

Uvnitř každého rámce je výskyt \`0x1a\` zakódován jako \`0x1a 0x1a\`. TCP může rámce libovolně rozdělit/spojit; dekodér musí být **inkrementální stavový automat**, který udržuje přesný počet *unescaped* bajtů, rozpoznává neplatný prefix a resynchronizuje se bez nekonečné fronty. Typ \`0x34\` může existovat pro stavové zprávy na některých zařízeních; nekonstruovat obecnou délku neznámého payloadu — bezpečně přeskočit/obnovit synchronizaci podle adapteru.

Šestibajtový BEAST čas je čítač okolo **12 MHz**, **nikoli sám o sobě UTC**. Může přetékat, restartovat se či být neplatný/zero; ukládat odděleně od Rust monotonic receive instant a \`receivedAt\` UTC. Výchozí validní position time získat až po korektní synchronizaci/rekonstrukci a příslušných pravidlech.

**Mode-S CRC24/parita:** implementovat dle DF a příslušného způsobu parity (u některých zpráv se XORuje adresa / překrývá parita), nikoli zásadou \`syndrome==0\` na **všech** rámcích. Mode A/C nesprávně neinterpretovat jako ADS-B s ICAO. CPR poloha vyžaduje validní even/odd páry a pravidla času/latitude; nikdy nevydat nekorektní souřadnice jako skutečné. K jednotlivým DF/CPR vytvořit golden fixture a negativní rámce.

## 3. Normalizovaná interní událost (návrh, nikoli už existující API)

~~~text
AircraftObservation {
  identity: { kind: "icao" | "nonicao", hex6: lowercaseHex6 },
  source: "json" | "beast",       // pouze lokální zdroje
  receivedAtUtc: timestamp,        // servery označené časem
  receiverTicks?: u48,             // pouze BEAST; není absolutní čas
  messageAtUtc?: timestamp,        // pouze když bezpečně odvozeno
  positionAtUtc?: timestamp,       // pouze čerstvá skutečná validní poloha
  lat?: float, lon?: float,
  altitudeBaroFt?: integer | "ground",
  groundSpeedKt?: finite float,
  trackDegrees?: finite float,
  squawkOctal?: fourChars,
  callsign?: trimmed string,
  validity: perFieldValidity,
  format: rawFrameKind
}
~~~

- Chybějící pole znamená **absence pozorování**, nikoli implicitní zrušení již platného stavu. Neplatný údaj se nesmí publikovat.
- Identifikátory \`icao:abcdef\` a \`nonicao:abcdef\` jsou dvě oddělené identity. Callsign se může měnit.
- Priorita zdrojů se řídí **čerstvostí skutečného pole** a kvalitou validace. BEAST nesmí přepisovat novější polohu starou zprávou jen proto, že přijel později.
- Čas polohy se nikdy nesmí obnovit pouze novou zprávou bez polohy. U JSON je \`seen_pos\` primární stáří poslední polohy.
- Oddělit \`lastMessageAt\` pro dostupnost letadla od \`lastPositionAt\` pro zobrazení mapy.

## 4. Deduplikace a plynulý fallback

1. **Duplicitní zprávy:** malá TTL cache (výchozí 2 s) fingerprint \`zdroj přijímače + dekódovaný payload + typ + časový koš\` pro BEAST; stejné rámce z obou způsobů mohou mít odlišný čas, proto klíčovat pouze na *stejné validní normalizované pole a okamžik* při slučování stavu. Vyhnout se zamaskování platné opakované zprávy trvalým hashem payloadu.
2. **JSON snapshot:** aplikovat pouze hodnoty novější než dosavadní \`positionAtUtc\`/fieldTime; stejný \`aircraft_key\` jako u BEAST nevytváří druhý objekt. Neměnit lastPositionAt jen proto, že je v novém snapshotu stejná poloha se starým \`seen_pos\`.
3. **Dedup squawk alarmu:** L06 pouze u validního emergency kódu; detekce přechodu do 7500/7600/7700, jednoznačný idempotency key \`aircraft_key + code + event-start\`, debounce/rearm otestovat na přerušeném příjmu. Doručování do Web Push/Pushover musí být explicitně povolené a v testech/shadow zakázané.
4. **Stáří pro mapu:** doporučená hranice čerstvé pozice **15 s**, stáří letadla podle zpráv **60 s**. Po 15 s bez polohy viditelně označit stale nebo dočasně skrýt polohový symbol; po 60 s bez jakékoli zprávy odstranit track a vygenerovat SSE \`removed\`. Časy ověřit realitou přijímače.
5. **Výpadek BEAST:** základní watchdog \`5 s\` bez validních dat *při potvrzeném živém JSON* ⇒ použít JSON. Návrat na BEAST až po \`10 s\` souvisle zdravého streamu; preferovat aktuální pozorování, nesnižovat kvalitu. Délky prahů přizpůsobit skutečnému JSON refresh a reálnému provozu; **ticho v provozu samo neznamená chybu**. Ověřovat také stav TCP spojení a readsb metrics.
6. **Kapacita RAM:** výchozí max 5 000 identit a 64 posledních bodů na letadlo, přebytek ošetřit explicitním TTL/eviction a metrikou. Bez neomezených hashmap a backlogů. Letová stopa pro mapu může držet výřez posledních **20 minut** s maximem 64 bodů, pokud takový interval odpovídá nastavené frekvenci pozic.

## 5. Golden fixture a akceptační testy G-01/G-02

- validní Mode-S krátké/dlouhé, escape \`0x1a\`, split/chunked TCP, invalid length, neznámé typy, restart čítače a short read;
- CRC/DF/CPR včetně AP parity, falešný syndrom, odd/even mimo časové okno, nemožná poloha;
- stejné ICAO v JSON/BEAST a odlišná \`~hex\` identita, opakování, chybějící souřadnice, \`alt_baro="ground"\`, validní nula \`lat/lon\`;
- chybné/stale \`seen_pos\`, přehazování časů, skok wall clocku, restart readsb \`messages\`;
- přerušení BEAST, následně čerstvý JSON, obnova BEAST; žádné „teleporty“ při přepnutí;
- validní squawk 7500/7600/7700 a nevalidní oktální hodnoty, duplicita výstrahy;
- deterministický replay 1×/2×/4×, CPU/RSS a paměťové limity pouze v odděleném testovacím prostředí.

**Důkazy do G-01:** \`contracts/readsb-json.schema.json\`, popis interních DTO, fixture binárních rámců + očekávané stavy a Rust testy. Tato rešerše neznamená, že uvedené soubory již byly vytvořeny.

## Zdroje

- [readsb README-json.md](https://github.com/wiedehopf/readsb/blob/dev/README-json.md) — \`now\`, \`seen\`, \`seen_pos\`, volitelnost polí a ne-ICAO \`hex\`.
- [BEAST protokol](https://github.com/firestuff/adsb-tools/blob/master/protocols/beast.md) — escaping, délky a MLAT čítač.

## Důsledek deduplikace pro SQL a šetření SSD

Kanonická RAM identita a pole `lastPositionAt` jsou základ pro [úsporný adaptivní sampler](RAM-CACHE-A-DEDUP-ZAPISU.md). JSON/BEAST opakovaně reportující již známou polohu **nesmí vytvořit nový PostgreSQL vzorek**, i kdyby se měnil čas posledního příjmu zprávy. Primární SQL unikátní `(bucket_at,aircraft_key)` s 30s bucketem je druhá ochrana, nikoli hlavní filtr. Čítače zpráv a běžné SSE aktualizace se agregují v RAM, bez SQL na každý rámec.
