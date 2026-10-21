# Podrobný katalog všech funkcí AirRadar V1 pro rozhodnutí o V2

**Datum revize:** 21. 10. 2026  
**Zdroj seznamu:** [`contracts/feature-matrix.json`](../../contracts/feature-matrix.json) — aktuálně 79 KEEP a 99 REMOVE.  
**Způsob práce:** tento katalog **vysvětluje zamýšlený význam jednotlivých položek**, nikoli potvrzenou aktuální implementaci ve V1 nebo funkční stav ve V2. Podrobný rozsah z V1 nebyl pro každé ID znovu testován; popisy jsou pracovní produktovou interpretací názvů a projektových požadavků.

Popis je napsaný tak, aby bylo možné odpovědět „ponechat / vyřadit / zjednodušit“ bez otevírání starého zdrojového kódu. **Dokud vlastník projektový výběr neschválí, neměň volbu KEEP/REMOVE ani nepřidávej podpůrné moduly automaticky.**

## Část I — nyní vybrané funkce (KEEP: 79)

### Příjem ADS-B a základní provoz — J (10)

| ID | Funkce | K čemu slouží |
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

### Mapa a práce s letadly — R (16)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **R01** | Základní interaktivní mapa MapLibre · základ systému | Zobrazuje hlavní interaktivní mapu, živé značky letadel a běžné přibližování, posun i výběr cíle. |
| **R02** | Přepínání LOCAL / EXTENDED pokrytí | Přepíná zobrazení pouze lokálně přijatých letadel a rozšířeného provozu z externích zdrojů, s jasným označením původu. |
| **R03** | Ikony letadel podle typu a směru | Volí symbol podle známé kategorie letadla a otáčí jej podle směru letu; při neznámém typu použije bezpečnou obecnou ikonu. |
| **R04** | Popisky letadel a řízení jejich kolizí | Zobrazuje u letadel stručné popisky, například callsign či výšku, a omezuje jejich překrývání ve zhuštěném provozu. |
| **R05** | Barevné režimy letadel podle provozních údajů | Barví značky podle zvoleného provozního údaje, například výšky, vertikálního pohybu či zdroje; legendou vysvětluje význam. |
| **R06** | Živý seznam provozu a filtrování letadel | Nabízí živý seznam letadel s hledáním, řazením a filtry například podle callsignu, typu či zdroje příjmu. |
| **R07** | Detail vybraného letadla v mapovém panelu | Otevírá panel s aktuálními údaji vybraného letadla přímo vedle mapy; nejde automaticky o samostatnou historickou stránku. |
| **R08** | Zobrazení živé letové stopy (trail) | Kreslí nedávnou posloupnost skutečně přijatých poloh vybraného letadla; nejde o predikovanou budoucí trasu. |
| **R09** | Sledování letadla kamerou mapy (follow) | Udržuje vybrané letadlo v záběru mapové kamery, dokud uživatel automatické sledování nevypne. |
| **R10** | Současné sledování více letadel | Umožňuje označit a současně sledovat více letadel, případně zobrazovat jejich oddělené základní informace. |
| **R11** | Zobrazení trasy mezi letišti a jejího průběhu | Zobrazuje spojení a průběh trasy mezi letišti, pokud jsou známa ověřená data o původu a cíli; nejde o výpočet ETA. |
| **R12** | Vrstvy okolních letišť | Zobrazuje značky okolních letišť jako samostatně ovladatelnou mapovou vrstvu a umožňuje přejít na detail. |
| **R14** | Mapové podklady, vzhled a předvolby vrstev | Umožňuje výběr mapového podkladu, vzhledu a zapamatovaných předvoleb viditelných vrstev. |
| **R15** | Mapová HUD lišta a rychlé akce | Poskytuje kompaktní ovládací lištu mapy, například rychlý přístup k filtrům, vrstvám, vycentrování a panelům. |
| **R17** | ATC sektory a vysílače přímo v mapě | Vykresluje vybrané ATC sektory a polohy vysílačů přímo do mapy. Vyžaduje dořešit zdroj minimálních dat, protože T01/T03 jsou REMOVE. |
| **R18** | Volitelné počasí a vzdušné prostory přímo v mapě | Zapíná a vypíná meteorologické překryvy a vzdušné prostory nad mapou; jednotlivé vrstvy musí mít doložený datový zdroj. |

### Informace o letadle a historie — H (3)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **H03** | Fotografie letadel z Planespotters | Načítá dostupnou fotografii konkrétního letadla od Planespotters podle registrace či jiné podporované identity. |
| **H07** | Výškový, rychlostní a vertikální profil letu | Zobrazuje časový graf skutečně zaznamenané výšky, rychlosti a stoupání či klesání během letu. |
| **H14** | Souhrny letadla za 7/30 dní a celé období | Počítá souhrny z nových záznamů V2 pro jedno letadlo za 7 a 30 dní a za celé období od spuštění V2. |

### Letiště — A (2)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **A01** | Katalog letišť a vyhledávání podle ICAO | Udržuje katalog letišť a umožňuje je vyhledávat podle ICAO kódu, názvu či dalších podporovaných údajů. |
| **A02** | Detail letiště – dráhy, poloha, vybavení a frekvence | Zobrazuje polohu letiště, dráhy, vybrané technické vybavení a publikované frekvence podle doložených zdrojů. |

### ATC, ATS a navigace — T (5)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **T08** | ATS tratě České republiky | Zobrazuje české ATS tratě a jejich publikovanou geometrii pro navigační orientaci. |
| **T09** | ATS tratě Slovenska a Rakouska | Doplňuje ATS tratě Slovenska a Rakouska jako regionální rozšíření; neznamená automaticky obnovu ATC SK/AT. |
| **T10** | NAVAID/FIX databáze a navigační průzkumník | Zpřístupňuje navigační body FIX a prostředky NAVAID v katalogu, vyhledávání a mapovém průzkumníku. |
| **T13** | Navigation Integrity – podezřelé anomálie ADS-B | Vyhodnocuje podezřelé skoky a nekonzistence ADS-B polohy či návazných údajů; výsledek označuje jako anomálii, ne potvrzený útok. |
| **T14** | Navigation Integrity – regionální mapa, historie a analýza | Zobrazuje oblasti, vývoj v čase a přehled historicky zaznamenaných Navigation Integrity anomálií z nových dat V2. |

### Počasí — W (6)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **W01** | Aktuální METAR a TAF letišť | Načítá a zobrazuje aktuální zprávy METAR a předpovědi TAF pro vybraná letiště. |
| **W02** | METAR značky a informace přímo v mapě | Přidává na mapu meteorologické značky letišť a rychlý přehled podmínek z METAR. |
| **W03** | Meteorologický radar ČHMÚ s časovou osou | Zobrazuje snímky meteorologického radaru ČHMÚ v mapě a umožňuje procházet jejich časovou osu. |
| **W06** | SIGMET a AirSIGMET výstrahy | Zobrazuje prostorově i časově platné výstrahy SIGMET a AirSIGMET včetně jejich významu a platnosti. |
| **W08** | Počasí měřené letadly z Mode-S / BDS 4,4 | Dekóduje meteorologické údaje dostupné ze zpráv Mode-S / BDS 4,4 u letadel, která je skutečně vysílají. |
| **W09** | Profil a kvalita počasí pozorovaného letadlem | Zobrazuje vývoj a důvěryhodnost meteorologických pozorování konkrétního letadla; neodhaduje chybějící měření jako fakta. |

### Oznámení a sledování — L (2)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **L11** | Push notifikace do prohlížeče | Umožňuje přihlášení prohlížeče k Web Push a technické doručení oznámení; samo neurčuje, jaká událost má zprávu vyvolat. |
| **L12** | Externí Pushover upozornění | Odesílá oznámení přes službu Pushover, eviduje reakci poskytovatele a stav doručení; pravidla spouštění nejsou automaticky součástí. |

### Externí datové zdroje — E (9)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **E01** | ADSBHub – příjem externího síťového provozu | Přijímá okolní síťový provoz z ADSBHub a odděleně jej označuje jako externí zdroj. |
| **E02** | ADSB.lol – surový BEAST/SBS stream | Přijímá živá BEAST/SBS data z ADSB.lol, hlídá výpadky a omezuje zdrojové přetížení. |
| **E03** | ADSB.lol – HTTP geografická záloha | Při selhání streamového spojení používá geografické HTTP dotazy ADSB.lol s časovým omezením a limity. |
| **E04** | OGN/FLARM – letadla, kluzáky a další cíle | Samostatně zpracovává OGN/FLARM cíle, například kluzáky a další vysílače, a zobrazuje je odděleně od ADS-B. |
| **E05** | OGN Device Database – identifikace a ochrana soukromí | Používá databázi OGN zařízení k rozpoznání typu a hlavně k dodržení příznaků ochrany soukromí, včetně noTracking. |
| **E06** | SoftRF SQLite – nouzová OGN whitelist databáze | Poskytuje lokální záložní SQLite seznam SoftRF/OGN identit pro určené nouzové scénáře; neobchází pravidla soukromí. |
| **E07** | ACARS/VDL2 metadata přes RXW Hub | Zobrazuje nebo doplňuje informace z ACARS/VDL2 přes RXW Hub, nikoliv automaticky mapové waypointy z ACARS H1/FPN. |
| **E09** | FlightAware – doplňování letových plánů a diagnostika využití | Na požádání doplňuje dostupné letové plány či metadata z FlightAware a měří využití či omezení API. |
| **E10** | ADSBDB – externí metadata letadel | Načítá pomocné informace o letadlech z ADSBDB, například typ či registraci, v mezích dostupnosti poskytovatele. |

### Statistiky a pokrytí — S (6)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **S01** | Aktuální počítadla letadel a zpráv za sekundu | Zobrazuje okamžitý počet sledovaných letadel a rychlost příjmu ADS-B zpráv za sekundu. |
| **S02** | Denní/7denní/30denní agregované statistiky – API | Vypočítává denní a souhrnné statistiky za 7 a 30 dní z dat nově získaných V2 a zpřístupňuje je přes API. |
| **S03** | Receiver Coverage – polární mapa dosahu | Vytváří polární mapu geografického dosahu přijímače podle skutečně lokálně přijatých poloh a směrů. |
| **S04** | Analýza slabých sektorů a spolehlivosti příjmu | Vyhodnocuje slabší směrové sektory, dostupnost signálu a stabilitu místního příjmu z měřitelných ukazatelů. |
| **S06** | Statistiky typů letadel, aerolinek a tratí – API | Připravuje agregované přehledy podle typu letadla, aerolinky či tratě, pokud mají záznamy dostatek identifikačních dat. |
| **S09** | Logbook a agregované souhrny přijímače | Zpřístupňuje logbook a přehledy činnosti přijímače, například denní počet pozorování, typy a průběh sběru. |

### Provoz, administrace a diagnostika — O (14)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **O01** | System Dashboard – stav všech komponent | Soustřeďuje do dashboardu přehled zdraví enginu, webu, databáze, přijímače a externích providerů. |
| **O02** | Veřejná kontrola zdraví aplikace a verze · základ systému | Poskytuje veřejný sanitizovaný health endpoint a verzi buildu bez zveřejnění tajných provozních údajů. |
| **O03** | Receiver Monitoring – zdraví SDR/readsb příjmu | Sleduje dostupnost SDR/readsb, poslední zprávu, typ zdroje, výpadky a základní kvalitu příjmu. |
| **O04** | Diagnostika externích poskytovatelů a integrací | U každé externí integrace zobrazuje poslední úspěch, chyby, timeouty, limity a aktuální degradaci. |
| **O05** | CPU, RAM, GC a event-loop telemetrie | Měří CPU, RAM a zátěž běžících služeb; u Node části GC/event-loop, u Rust enginu odpovídající runtime a frontové metriky. |
| **O06** | Historie výkonu a aktuálních bottlenecků | Ukládá omezenou časovou řadu výkonu a identifikuje aktuální úzká místa, bez neomezeného monitorovacího archivu. |
| **O07** | Databázová latence, fronty a diagnostika zapisování | Ukazuje latence SQL, počet čekajících zápisů, vytížení connection poolu a chybovost databázové persistence. |
| **O08** | Systémové diagnostické SSE streamy | Posílá aktuální stav diagnostiky přes SSE, aby panel nemusel pravidelně znovu dotazovat všechny sondy. |
| **O09** | T58 production soak, stall canary a CPU profiling | Umožňuje řízený dlouhodobý provozní test T58, sledování zaseknutí a profilování CPU na bezpečné testovací zátěži. |
| **O10** | Výkonové benchmarky a zátěžové testy | Poskytuje opakovatelné benchmarky a zátěžové testy pro ingest, SSE, UI a databázi s porovnatelnými vstupy. |
| **O11** | Automatické CI, testy, CodeQL a bezpečnostní kontroly · základ systému | Zajišťuje automatické testy, lint, CodeQL či obdobný bezpečnostní audit a kontrolu přesného commitu v GitHub Actions. |
| **O12** | Automatické nasazení, health check a rollback · základ systému | Připravuje automatizované sestavení a nasazení samostatné V2 s kontrolami health, možností návratu a bez zásahu do V1. |
| **O13** | Import/synchronizace eAIP, ATS a letištních dat | Bezpečně synchronizuje a verzovaně importuje publikovaná eAIP, ATS a letištní data, s kontrolou původu. |
| **O15** | Správa přihlášení administrátora a bezpečnostních oprávnění · základ systému | Zajišťuje přihlášení správce, role/oprávnění a ochranu neveřejných API a administračních operací. |

### Uživatelské rozhraní — U (6)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **U01** | Responzivní rozhraní mobil/tablet/desktop · základ systému | Rozvrhuje mapu, seznam a panely pro mobilní telefon, tablet i desktop včetně dotykového ovládání. |
| **U02** | Instalovatelná PWA aplikace | Umožňuje instalovat web jako Progressive Web App s ikonami, manifestem a řízeným cache režimem. |
| **U03** | Offline cache posledního známého stavu | Bezpečně uloží poslední známý veřejný stav pro prohlížení offline a výslovně jej označí jako neaktuální. |
| **U04** | Česká a anglická lokalizace | Nabízí uživatelské rozhraní česky i anglicky; čeština je výchozí jazyk projektu. |
| **U05** | Globální Command Search a navigace | Poskytuje rychlé globální vyhledání letadla, letiště, příkazu nebo části aplikace z jednoho pole. |
| **U08** | Klávesové zkratky a rychlé ovládání | Povoluje běžné akce klávesami, například přechod do hledání, výběr panelu či ovládání mapy. |

## Část II — nyní vyřazené funkce (REMOVE: 99)

**REMOVE neznamená, že se funkce nesmí vrátit.** Znamená to pouze, že nebude budována bez nového rozhodnutí. Některé funkce mohou být užitečné v jednodušší podobě.

### Příjem ADS-B a základní provoz — J (1)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **J10** | Demo provoz se simulovanými letadly | Generuje fiktivní letadla a pohyb pro demonstrační režim uživatelského radaru; to není totéž co neveřejné syntetické fixture pro testy. |

### Mapa a práce s letadly — R (2)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **R13** | Poloha přijímače, kružnice dosahu a měřítko | Zobrazuje pozici přijímače, vzdálenostní kružnice a mapové měřítko; přesná soukromá lokalita vyžaduje zvláštní ochranu. |
| **R16** | Režim prezentace mapy | Zjednodušuje mapové rozhraní do prezentačního režimu například pro velkou obrazovku. |

### Informace o letadle a historie — H (11)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **H01** | Samostatný detail letadla – registrace, typ, operátor | Otevírá samostatnou stránku profilu letadla s registrací, typem, operátorem a dalšími základními údaji. |
| **H02** | Katalog a automatické doplňování metadat letadel | Udržuje katalog identit a metadat letadel a automaticky doplňuje chybějící údaje od externích zdrojů. |
| **H04** | Letový plán, volací znak a letiště OD/DO | Zobrazuje dostupný letový plán, callsign a letiště odletu/příletu u samostatného detailu letadla. |
| **H05** | Historie konkrétního letadla a jeho letů | Vypisuje historické jednotlivé lety jednoho letadla ze starší databáze, s možností otevřít detail. |
| **H06** | Detail jednotlivého letu (Flight Story V2) | Skládá podrobný „příběh“ jednoho letu z událostí, stopy, výšky a souvisejících metadat. |
| **H08** | Historické přehrávání konkrétního letu | Umožňuje přehrát dřívější konkrétní let na mapě v čase. |
| **H09** | Seznam a vyhledávání historických letů | Nabízí katalog historických letů s vyhledáním a filtrováním napříč letadly. |
| **H10** | Porovnání dvou letů | Srovnává dva historické lety z hlediska trasy, rychlosti, výšky a dalších údajů. |
| **H11** | Time Machine – přehrávání všech letadel v čase | Přehrává historický stav všech dostupných letadel v daném regionu v určitém čase. |
| **H12** | Event Replay – ±10 minut kolem události | Přehrává průběh provozu v časovém okně přibližně deset minut před a po vybrané události. |
| **H13** | Historická identita NEW / RARE / RETURNING | Označuje letadla jako nová, vzácná nebo vracející se podle staré historické identity a návštěvnosti. |

### Letiště — A (10)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **A03** | Airport Live Board – přehled provozu letiště | Nabízí samostatnou aktuální tabuli provozu konkrétního letiště s přehledem viditelných letů. |
| **A04** | Odvozené přílety, odlety a průlety | Odvozuje z pohybu letadel, zda šlo o přílet, odlet nebo průlet v okolí letiště; nejde o ověřený letištní systém. |
| **A05** | Vyhodnocení použitých přistávacích drah | Zkouší z trajektorie letu a orientace letiště odhadnout konkrétní použitou přistávací dráhu. |
| **A06** | Arrival Sequence / Approach Queue | Řadí blížící se letadla do orientační přibližovací sekvence a fronty na přílet. |
| **A07** | Arrival Flow, vytížení a stabilita drah (V8) | Vyhodnocuje vývoj příletových toků, stabilitu přiblížení a zatížení drah v pokročilém modelu V8. |
| **A08** | Odhad terminálové poptávky / příletů (V9) | Modeluje pravděpodobnou budoucí poptávku a přílety na letišti podle experimentálního modelu V9. |
| **A09** | Provozní výjimky a konfliktní údaje letiště | Vyhledává rozpory nebo neobvyklé stavy v letištních údajích či odvozeném provozu. |
| **A10** | Live Airport Network – okolní aktivní letiště | Zobrazuje propojenou živou síť okolních aktivních letišť a jejich vztahy podle sledovaného provozu. |
| **A11** | Porovnání letišť | Srovnává vybraná letiště podle metrik, například počtu pozorovaných letů nebo podmínek. |
| **A12** | Historická statistika provozu letiště za 7/30 dní | Vytváří archivní přehled provozu jednoho letiště za posledních 7 či 30 dní. |

### ATC, ATS a navigace — T (9)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **T01** | ATC sektory včetně geometrie a výškových hranic | Ukládá nebo načítá ATC sektory jako přesné polygony s výškovými hranicemi a platností. |
| **T02** | Samostatný detail ATC sektoru | Zobrazuje samostatný detail vybraného ATC sektoru, jeho výšky a publikované služby. |
| **T03** | ATC frekvence, vysílače a publikované služby | Zpřístupňuje publikované ATC frekvence, polohy vysílačů a rozsah příslušných služeb. |
| **T04** | Pravděpodobné ATC přiřazení vybraného letadla | Podle polohy, výšky a geometrie odhaduje, který ATC sektor by mohl být pro letadlo relevantní; neurčuje skutečně naladěnou frekvenci. |
| **T05** | ATC handoff – odhad přechodu mezi sektory | Odvozuje možný budoucí přechod letadla mezi sektory (handoff), nikoliv potvrzený přechod spojení. |
| **T06** | Historie provozu a přechodů mezi sektory – API | Uchovává analytickou historii pravděpodobného provozu a přechodů mezi ATC sektory. |
| **T07** | Validace a diagnostika ATC dat | Provádí validaci geometrie, správnosti a aktuálnosti importovaných ATC dat a zobrazuje diagnostiku. |
| **T11** | Letové procedury SID/STAR a procedurální data | Udržuje a zobrazuje publikované postupy SID/STAR a další procedury odletu/příletu. |
| **T12** | AUP/UUP – plánovaná aktivace vzdušného prostoru | Zobrazuje plánované aktivace, omezení či vyhrazení vzdušných prostorů podle AUP/UUP. |

### Počasí — W (8)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **W04** | Echo Top – výška radarových odrazů | Zobrazuje odhad maximální výšky radarových odrazů (Echo Top), pokud poskytovatel takový produkt skutečně dodává; není to běžný 2D radarový snímek. |
| **W05** | Model větru ICON-EU v různých letových hladinách | Načítá modelovaný vítr ICON-EU pro zvolené hladiny a zobrazuje jeho prostorový průběh. |
| **W07** | PIREP/AIREP hlášení pilotů | Zobrazuje meteorologická hlášení z letadel podaná piloty (PIREP/AIREP), odlišná od automatických BDS dat. |
| **W10** | Weather Fusion – spojení více zdrojů počasí | Spojuje několik meteorologických zdrojů do jednoho odvozeného výstupu včetně nejistoty a časové platnosti. |
| **W11** | Weather Avoidance – vyhodnocení možné odchylky kvůli počasí | Vyhodnocuje, zda počasí může vést k odchylce letu od předpokládané trasy; jde o hypotézu, ne potvrzenou příčinu. |
| **W12** | Počasí na trase zvoleného letu | Sestavuje přehled počasí podél známé nebo plánované trasy vybraného letu. |
| **W13** | Historické mapové počasí v Time Machine | V režimu historického přehrávání synchronizuje mapové počasí s historickým časem. |
| **W14** | Weather Corridor – počasí v předpokládaném koridoru | Přepočítává očekávané počasí v budoucím koridoru predikovaného letu. |

### Oznámení a sledování — L (13)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **L01** | Watchlist – pravidla sledování letadel | Spravuje pravidla, která letadla nebo situace má uživatel aktivně sledovat. |
| **L02** | Hlídání ICAO, registrace, callsignu, typu a aerolinky | Podporuje pravidla odpovídající konkrétnímu ICAO, registraci, callsignu, typu či aerolince. |
| **L03** | Upozornění podle vzdálenosti od přijímače | Vyvolává upozornění při dosažení definované vzdálenosti cíle od přijímače. |
| **L04** | Fleet Explorer – vlastní sledované flotily | Umožňuje sestavit vlastní flotily a prohlížet či sledovat jejich členy jako skupinu. |
| **L05** | Geofencing – vlastní sledované geografické oblasti | Umožňuje vlastní geografické zóny a upozornění při vstupu či výstupu letadla. |
| **L06** | Emergency squawk 7500/7600/7700 | Rozpoznává nouzové transpondérové kódy 7500, 7600 a 7700 a nabízí navázaná upozornění. |
| **L07** | Historie upozornění a událostí | Ukládá a zobrazuje časovou historii vytvořených upozornění a událostí. |
| **L08** | Pravidla cooldownu a odstranění duplicit | Uplatňuje časovou prodlevu a deduplikaci, aby stejná událost neposílala opakovaně oznámení. |
| **L09** | Notification Center – přijaté/nepřečtené zprávy | Nabízí centrum zpráv s označením přečtených a nepřečtených upozornění. |
| **L10** | Ztlumení oznámení pro letadlo nebo pravidlo | Dovoluje uživateli ztlumit oznámení pro konkrétní letadlo či pravidlo. |
| **L13** | Delivery Health – stav doručování zpráv | Sleduje čekající, odeslané, neúspěšné a opakované pokusy o doručení upozornění. |
| **L14** | Alert Rule Simulator – testování pravidel | Simuluje průchod pravidly na zkušebních datech bez skutečného odesílání zpráv. |
| **L15** | Alert Effectiveness Analytics – účinnost upozornění | Měří užitečnost a výsledky vyvolaných upozornění, například podíl relevantních a duplicitních. |

### Externí datové zdroje — E (1)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **E08** | ACARS H1/FPN waypointy a mapová vrstva | Převádí ACARS zprávy H1/FPN obsahující waypointy na mapovou vrstvu plánované či oznámené trasy. |

### Statistiky a pokrytí — S (3)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **S05** | Rekordy vzdálenosti a přijatých letadel | Udržuje rekordní hodnoty, například nejvzdálenější zachycené letadlo nebo historický maximální počet. |
| **S07** | Historická heatmapa a profil provozu – API | Vytváří historickou heatmapu intenzity příjmu či provozu a další časové profily přes API. |
| **S08** | Denní a týdenní recap – zbývající API | Automaticky sestavuje denní nebo týdenní rekapitulaci provozu; odlišuje se od jednoduchého S09 logbooku. |

### Provoz, administrace a diagnostika — O (1)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **O14** | Automatická dokumentace, changelog a metriky růstu kódu | Automaticky generuje část projektové dokumentace, changelog a grafy velikosti či růstu zdrojového kódu. Ručně vedený CHANGELOG může existovat i bez této funkce. |

### Uživatelské rozhraní — U (2)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **U06** | Oblíbená letiště a lokální uživatelské preference | Umožňuje uložit oblíbená letiště a další místní osobní preference uživatele. |
| **U07** | Sdílené záložkové odkazy na filtrování a analýzy | Vytváří sdílitelnou URL s konkrétním filtrem nebo analytickým stavem obrazovky. |

### Digital Twin a experimentální predikce — D (23)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **D01** | Digital Twin – základní projekce letu na 30 minut | Projektuje odhadnutou pozici letadla na horizont až 30 minut podle pohybu a modelu trajektorie. |
| **D02** | 4D koridor – budoucí poloha, výška a čas | Vypočítává časoprostorový 4D koridor, tedy možné budoucí polohy, výšky a časy namísto jediné linie. |
| **D03** | Prediction Timeline v detailu letadla | Zobrazuje v detailu letadla časovou osu očekávaných budoucích predikovaných bodů či událostí. |
| **D04** | Budoucí průniky ATC sektorů a ATS tras | Hledá předpokládané budoucí průniky projekce letu s ATC sektory nebo ATS tratěmi. |
| **D05** | Průniky s plánovanými AUP/UUP prostory | Zjišťuje možné budoucí průniky trajektorie s plánovaně aktivními AUP/UUP prostory. |
| **D06** | Průniky s Navigation Integrity anomáliemi | Propojuje predikovanou trajektorii s oblastmi, v nichž jsou pozorovány navigační anomálie. |
| **D07** | Operational Focus – prioritní situace letadla | Sestavuje prioritní provozní souvislosti jednoho letadla a určuje, na co se operátor zaměřit. |
| **D08** | Operational Focus Change – změny priorit | Sleduje změny takto vypočtené provozní priority v čase a jejich vysvětlení. |
| **D09** | Regional Situation Graph – vazby mezi letadly | Vytváří graf vztahů mezi letadly v regionu, například jejich vzájemného kontextu nebo predikovaných návazností. |
| **D10** | Operational Attention – regionální upozornění | Generuje odvozená regionální provozní upozornění podle analytických priorit. |
| **D11** | Regional Focus Queue – pořadí prioritních letadel | Řadí několik předpokládaně důležitých letadel do společné fronty pozornosti. |
| **D12** | Regionální projekce za 5/15/30 minut | Vypočítává výhled regionální situace na 5, 15 a 30 minut. |
| **D13** | Validace koridoru proti pozdějším pozicím | Porovnává minulou předpovězenou trajektorii s později skutečně přijatými polohami letadla. |
| **D14** | Validace predikovaných událostí | Kontroluje, zda predikované události skutečně nastaly, případně s jakou časovou odchylkou. |
| **D15** | Truth-first Validation – vyhodnocování podle skutečných událostí | Stanovuje metodiku ověřování predikcí podle reálných výsledků, nikoli podle dalších predikcí. |
| **D16** | Ukládání anonymních výsledků kalibrace | Ukládá anonymizované výsledky kalibračních porovnání pro další ladění modelů. |
| **D17** | Regional Attention Outcome – kontrola regionálních predikcí | Vyhodnocuje následné skutečné dopady dřívějších regionálních priorit a odhadů. |
| **D18** | Regional Attention Graduation – pravidla povýšení modelu | Stanovuje kritéria, podle kterých může regionální predikční model přejít do vyššího stupně nasazení. |
| **D19** | Trajectory Quality V2 – alternativní vertikální model | Používá alternativní vertikální model V2 pro kvalitu odhadované trajektorie. |
| **D20** | Trajectory Quality V3 – výkonově orientovaný shadow model | Zkouší stínový výkonově laděný model trajektorie V3 bez přímého veřejného používání. |
| **D21** | Outcome Validation pro modely V2 a V3 | Porovnává výsledky modelů V2 a V3 s následnými skutečnými polohami či událostmi. |
| **D22** | Graduation/Promotion – výběr schváleného modelu | Rozhoduje na základě definovaných metrik, který predikční model lze schválit či povýšit. |
| **D23** | Admin Calibration Center – diagnostika modelů | Poskytuje administrační centrum pro kalibraci, výsledky a porovnávání predikčních modelů. |

### Flight Intelligence a předpovědi — P (15)

| ID | Funkce | K čemu slouží |
|---|---|---|
| **P01** | Flight Intelligence – automatické události letu | Z pozorovaného letu vytváří automaticky interpretované provozní události. |
| **P02** | Rozpoznávání holding, go-around, diversion, orbit a neobvyklých zatáček | Pokouší se rozpoznat holding, go-around, diversion, orbit nebo neobvyklé zatáčky podle trajektorie. |
| **P03** | Přehled událostí a analytická stránka | Nabízí zvláštní stránku a API s vypočtenými událostmi Flight Intelligence. |
| **P04** | Route Intelligence – interpretace letového plánu | Interpretuje strukturu dostupného letového plánu a její navigační význam. |
| **P05** | Route Corridor Intelligence – odchylka od plánované trasy | Počítá odchylku aktuální polohy od koridoru plánované trasy. |
| **P06** | Trajectory Conformance – odklon, návrat a pravděpodobné DCT | Vyhodnocuje, zda se letadlo od plánované trasy vzdálilo, vrací se, nebo letí pravděpodobně přímo na bod DCT. |
| **P07** | Predikce času příletu ETA | Odhaduje čas příletu (ETA) z pozice, dráhy letu a dalších známých okolností. |
| **P08** | Predikce přistávací dráhy | Odhaduje pravděpodobnou přistávací dráhu dříve, než ji lze spolehlivě odvodit z přiblížení. |
| **P09** | Predikce změny přistávací dráhy | Sleduje, zda se pravděpodobná budoucí přistávací dráha mění. |
| **P10** | Predikce budoucí trajektorie | Predikuje budoucí pohyb letadla v prostoru, nikoliv jen historickou stopu. |
| **P11** | Explainable Prediction – vysvětlení podkladů predikcí | Vysvětluje, ze kterých údajů a předpokladů vznikla konkrétní predikce. |
| **P12** | Prospektivní sběr a validace predikcí | Ukládá predikce předem pro pozdější nezávislé vyhodnocení. |
| **P13** | Readiness a pravidla pro zveřejnění predikcí | Stanovuje hranice přesnosti a provozní připravenosti, než je predikce zveřejněna uživatelům. |
| **P14** | Kalibrace a porovnání predikcí s výsledky | Kalibruje predikční modely podle skutečných následně přijatých výsledků a porovnává jejich přesnost. |
| **P15** | Predictive Operations Center – souhrn předpovědí | Nabízí souhrnnou obrazovku všech predikovaných provozních situací pro operátora. |

## Důležité závislosti a otevřené otázky před schválením

1. **R17 vs T01/T03:** ATC sektory a vysílače mají být v mapě, ale geometrii sektorů a databázi publikovaných frekvencí nynější výběr vylučuje. Buď vrať minimální podkladové ATC funkce, nebo R17 zjednoduš.
2. **L11/L12 bez L01–L10:** Push/Pushover jsou doručovací kanály; bez pravidel či jiné schválené události neexistuje jednoznačný původce oznámení. Rozhodni, zda chcete pouze transport, nebo i konkrétní spouštěče.
3. **R11 vs H04 a P04:** „Trasa mezi letišti“ je zachována, ale samostatný letový plán a jeho interpretace nejsou. Trasu lze ukázat jen tehdy, pokud je O/D skutečně známé z dovoleného zdroje; není důvod domýšlet plánovaný průběh.
4. **R07 vs H01/H02:** Mapový detail může zobrazovat dostupné informace, ale není to automaticky podpora samostatné profilové stránky ani kompletního katalogu letadel.
5. **H07/H14 vs H05/H09:** Profily a 7/30denní souhrny vyžadují záznamy, nikoli nutně seznam všech historických letů. Nová V2 databáze se plní od nuly.
6. **J03 Rust ve V2:** cílem je přímo nativní parser bez mezivrstvy Node, nikoli mechanicky převzatá „akcelerace“ staré implementace.
7. **O05 Rust vs Node:** `event-loop` a `GC` se týkají Node.js části; Rust engine se má měřit vhodnými CPU/RSS, latencí a frontovými metrikami.
8. **O14 vs CHANGELOG:** Automatický generátor dokumentace a metrik růstu kódu je REMOVE, ale ručně vedený `CHANGELOG.md` a Git historie jsou už součástí běžného vývoje V2.
9. **J10 demo vs testovací fixture:** vyřazený veřejný simulovaný provoz nevylučuje syntetické replay záznamy pro offline testy.
10. **Datové podklady a licence:** ATC, eAIP, ADSBHub, ADSB.lol, OGN, RXW a meteorologické služby mohou mít přístupové či licenční podmínky. Samotné zachování ID neznamená automaticky dostupný klíč či povolení veřejného šíření dat.

## Jak potvrdit výběr

Vlastník může poslat pouze změny, například:

> Z KEEP vyřaď: H14, W09.  
> Z REMOVE vrať: T01, T03, L06, H01.  
> O14 ponech jako REMOVE, ale běžný CHANGELOG zachovej.  
> Ostatní neměň.

Po potvrzení nejdříve aktualizuj `contracts/feature-matrix.json`, tento katalog a `docs/plan/MATICE-FUNKCI.md`. Až potom konkretizuj seznam modulů, úkoly G-XX a akceptační testy. **Nepřenášej automaticky celé složky staré V1.**

