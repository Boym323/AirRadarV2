# Podrobný katalog aktuálně schválených funkcí

**Rozsah: 45 funkcí.** Texty jsou zamýšleným chováním, nikoli důkazem dokončené implementace. Oddělená evidence případných budoucích rozšíření je v [archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md).

## Místní příjem letadel (10)

| ID | Funkce | Popis |
|---|---|---|
| **J01** | Příjem lokálních letadel přes readsb JSON · základ systému | Načítá pravidelné JSON snapshoty z místního přijímače readsb a z nich zjišťuje viditelná letadla, polohy, výšky a další údaje. |
| **J02** | Přímý příjem lokálních BEAST TCP rámců | Připojuje se přímo na lokální TCP stream BEAST a z dekódovaných zpráv získává podrobné údaje o provozu bez čekání na JSON snapshoty. |
| **J03** | Rust akcelerace BEAST parseru a CRC24 | Zpracovává binární zprávy BEAST/Mode-S a ověřuje CRC24. Ve V2 půjde o nativní Rust dekodér, nikoli o akceleraci Node.js. |
| **J04** | Automatické přepínání BEAST / JSON při výpadku | Pokud se přeruší BEAST nebo JSON, bezpečně přejde na dostupný zdroj a po obnově se vrátí bez zbytečné ztráty identity letadel. |
| **J05** | Správa živého stavu letadel v RAM · základ systému | Udržuje v paměti jediný aktuální stav každého letadla, aktualizuje hodnoty a odstraňuje zastaralé objekty podle času. |
| **J06** | Veřejné API aktuálních letadel · základ systému | Poskytuje aktuální seznam a základní údaje letadel přes veřejné HTTP API pro web a případné klienty. |
| **J07** | Živý stream SSE V1 – celé snapshoty · základ systému | Průběžně rozesílá klientům přes Server-Sent Events úplné snapshoty letadel. Slouží i pro jednoduché klienty či obnovu synchronizace. |
| **J08** | Optimalizovaný SSE V2 – delta aktualizace | Po úvodním snapshotu posílá jen změněná a zmizelá letadla. Snižuje objem dat i opakovanou práci klienta. |
| **J09** | Ukládání vzorkované historie do PostgreSQL | Vzorkuje vybrané letové pozice a ukládá je do nové PostgreSQL pro profily, statistiky a budoucí historii; neukládá každý ADS-B rámec. |
| **J11** | Kontinuita identity letadel, detekce výpadků a deduplikace · základ systému | Spojuje pozorování stejného cíle, řeší změnu volacího znaku, duplikáty a krátké výpadky, aby letadla na mapě nepřeskakovala mezi identitami. |

## Mapa (8)

| ID | Funkce | Popis |
|---|---|---|
| **R01** | Základní interaktivní mapa MapLibre · základ systému | Zobrazuje hlavní interaktivní mapu, živé značky letadel a běžné přibližování, posun i výběr cíle. |
| **R03** | Ikony letadel podle typu a směru | Volí symbol podle známé kategorie letadla a otáčí jej podle směru letu; při neznámém typu použije bezpečnou obecnou ikonu. |
| **R04** | Popisky letadel a řízení jejich kolizí | Zobrazuje u letadel stručné popisky, například callsign či výšku, a omezuje jejich překrývání ve zhuštěném provozu. |
| **R06** | Živý seznam provozu a filtrování letadel | Nabízí živý seznam letadel s hledáním, řazením a filtry například podle callsignu, typu či zdroje příjmu. |
| **R08** | Zobrazení živé letové stopy (trail) | Kreslí nedávnou posloupnost skutečně přijatých poloh vybraného letadla; nejde o predikovanou budoucí trasu. |
| **R09** | Sledování letadla kamerou mapy (follow) | Udržuje vybrané letadlo v záběru mapové kamery, dokud uživatel automatické sledování nevypne. |
| **R12** | Vrstvy okolních letišť | Zobrazuje značky okolních letišť jako samostatně ovladatelnou mapovou vrstvu a umožňuje přejít na detail. |
| **R14** | Mapové podklady, vzhled a předvolby vrstev | Umožňuje výběr mapového podkladu, vzhledu a zapamatovaných předvoleb viditelných vrstev. |

## Letiště (2)

| ID | Funkce | Popis |
|---|---|---|
| **A01** | Katalog letišť a vyhledávání podle ICAO | Udržuje katalog letišť a umožňuje je vyhledávat podle ICAO kódu, názvu či dalších podporovaných údajů. |
| **A02** | Detail letiště – dráhy, poloha, vybavení a frekvence | Zobrazuje polohu letiště, dráhy, vybrané technické vybavení a publikované frekvence podle doložených zdrojů. |

## Navigace (1)

| ID | Funkce | Popis |
|---|---|---|
| **T10** | NAVAID/FIX databáze a navigační průzkumník | Zpřístupňuje navigační body FIX a prostředky NAVAID v katalogu, vyhledávání a mapovém průzkumníku. |

## Počasí (4)

| ID | Funkce | Popis |
|---|---|---|
| **W03** | Meteorologický radar ČHMÚ s časovou osou | Zobrazuje snímky meteorologického radaru ČHMÚ v mapě a umožňuje procházet jejich časovou osu. |
| **W06** | SIGMET a AirSIGMET výstrahy | Zobrazuje prostorově i časově platné výstrahy SIGMET a AirSIGMET včetně jejich významu a platnosti. |
| **W08** | Počasí měřené letadly z Mode-S / BDS 4,4 | Dekóduje meteorologické údaje dostupné ze zpráv Mode-S / BDS 4,4 u letadel, která je skutečně vysílají. |
| **W09** | Profil a kvalita počasí pozorovaného letadlem | Zobrazuje vývoj a důvěryhodnost meteorologických pozorování konkrétního letadla; neodhaduje chybějící měření jako fakta. |

## Upozornění (3)

| ID | Funkce | Popis |
|---|---|---|
| **L11** | Push notifikace do prohlížeče | Umožňuje přihlášení prohlížeče k Web Push a technické doručení oznámení; samo neurčuje, jaká událost má zprávu vyvolat. |
| **L12** | Externí Pushover upozornění | Odesílá oznámení přes službu Pushover, eviduje reakci poskytovatele a stav doručení; pravidla spouštění nejsou automaticky součástí. |
| **L06** | Emergency squawk 7500/7600/7700 | Rozpoznává nouzové transpondérové kódy 7500, 7600 a 7700 a nabízí navázaná upozornění. |

## Statistiky (3)

| ID | Funkce | Popis |
|---|---|---|
| **S01** | Aktuální počítadla letadel a zpráv za sekundu | Zobrazuje okamžitý počet sledovaných letadel a rychlost příjmu ADS-B zpráv za sekundu. |
| **S06** | Statistiky typů letadel, aerolinek a tratí – API | Připravuje agregované přehledy podle typu letadla, aerolinky či tratě, pokud mají záznamy dostatek identifikačních dat. |
| **S09** | Logbook a agregované souhrny přijímače | Zpřístupňuje logbook a přehledy činnosti přijímače, například denní počet pozorování, typy a průběh sběru. |

## Provoz (8)

| ID | Funkce | Popis |
|---|---|---|
| **O01** | System Dashboard – stav všech komponent | Soustřeďuje do dashboardu přehled zdraví enginu, webu, databáze, přijímače a externích providerů. |
| **O02** | Veřejná kontrola zdraví aplikace a verze · základ systému | Poskytuje veřejný sanitizovaný health endpoint a verzi buildu bez zveřejnění tajných provozních údajů. |
| **O03** | Receiver Monitoring – zdraví SDR/readsb příjmu | Sleduje dostupnost SDR/readsb, poslední zprávu, typ zdroje, výpadky a základní kvalitu příjmu. |
| **O04** | Diagnostika externích poskytovatelů a integrací | U každé externí integrace zobrazuje poslední úspěch, chyby, timeouty, limity a aktuální degradaci. |
| **O05** | CPU, RAM, GC a event-loop telemetrie | Měří CPU, RAM a zátěž běžících služeb; u Node části GC/event-loop, u Rust enginu odpovídající runtime a frontové metriky. |
| **O06** | Historie výkonu a aktuálních bottlenecků | Ukládá omezenou časovou řadu výkonu a identifikuje aktuální úzká místa, bez neomezeného monitorovacího archivu. |
| **O07** | Databázová latence, fronty a diagnostika zapisování | Ukazuje latence SQL, počet čekajících zápisů, vytížení connection poolu a chybovost databázové persistence. |
| **O08** | Systémové diagnostické SSE streamy | Posílá aktuální stav diagnostiky přes SSE, aby panel nemusel pravidelně znovu dotazovat všechny sondy. |

## Uživatelské prostředí (6)

| ID | Funkce | Popis |
|---|---|---|
| **U01** | Responzivní rozhraní mobil/tablet/desktop · základ systému | Rozvrhuje mapu, seznam a panely pro mobilní telefon, tablet i desktop včetně dotykového ovládání. |
| **U02** | Instalovatelná PWA aplikace | Umožňuje instalovat web jako Progressive Web App s ikonami, manifestem a řízeným cache režimem. |
| **U03** | Offline cache posledního známého stavu | Bezpečně uloží poslední známý veřejný stav pro prohlížení offline a výslovně jej označí jako neaktuální. |
| **U04** | Česká a anglická lokalizace | Nabízí uživatelské rozhraní česky i anglicky; čeština je výchozí jazyk projektu. |
| **U05** | Globální Command Search a navigace | Poskytuje rychlé globální vyhledání letadla, letiště, příkazu nebo části aplikace z jednoho pole. |
| **U08** | Klávesové zkratky a rychlé ovládání | Povoluje běžné akce klávesami, například přechod do hledání, výběr panelu či ovládání mapy. |

## Ověření

Před implementací definuj kontrakt, testy a mezní situace. Stav funkce měň pouze po skutečném ověření. Podrobnosti má [katalog modulů](MODULY-A-ROZSAH.md).
