# Archiv nápadů pro budoucí rozšíření AirRadar V2

**Stav: ODLOŽENO, mimo aktuální implementaci. Celkem: 133 funkcí.** Toto je jediné místo v aktuálním stromu projektu, kde uchováváme **ID, názvy a popisy** odložených funkcí. Tento dokument se běžně nenačítá do kontextu implementace.

**Poslední rozhodnutí 21. října 2026:** vyřadit také přepínač **R02 — LOCAL/EXTENDED**, protože radar nyní pracuje výhradně s lokálními daty. Zbývajících 45 funkcí je v aktivní matici. Nenavrhovat ani neimplementovat nic z této evidence bez nového výslovného schválení.

## Jak případně funkci vrátit

1. Projít náklady CPU/RAM/DB, dostupnost zdroje a jeho licenci, rizika, skutečnou potřebu a způsob ověření.
2. Nechat vlastníka potvrdit konkrétní ID a očekávané zjednodušení.
3. Přesunout vybranou položku z tohoto souboru do `contracts/feature-matrix.json` se stavem `NOT_STARTED`, přiřadit modul, české zadání a testy.
4. Teprve potom začít implementovat. Žádné automatické kopírování zdrojů, historie či konfigurace V1.

## Kompletní seznam odložených funkcí

### Místní příjem letadel (1)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **J10** | Demo provoz se simulovanými letadly | Generuje fiktivní letadla a pohyb pro demonstrační režim uživatelského radaru; to není totéž co neveřejné syntetické fixture pro testy. |

### Mapa (10)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **R02** | Přepínání LOCAL / EXTENDED pokrytí | Přepíná zobrazení pouze lokálně přijatých letadel a rozšířeného provozu z externích zdrojů, s jasným označením původu. |
| **R05** | Barevné režimy letadel podle provozních údajů | Barví značky podle zvoleného provozního údaje, například výšky, vertikálního pohybu či zdroje; legendou vysvětluje význam. |
| **R07** | Detail vybraného letadla v mapovém panelu | Otevírá panel s aktuálními údaji vybraného letadla přímo vedle mapy; nejde automaticky o samostatnou historickou stránku. |
| **R10** | Současné sledování více letadel | Umožňuje označit a současně sledovat více letadel, případně zobrazovat jejich oddělené základní informace. |
| **R11** | Zobrazení trasy mezi letišti a jejího průběhu | Zobrazuje spojení a průběh trasy mezi letišti, pokud jsou známa ověřená data o původu a cíli; nejde o výpočet ETA. |
| **R15** | Mapová HUD lišta a rychlé akce | Poskytuje kompaktní ovládací lištu mapy, například rychlý přístup k filtrům, vrstvám, vycentrování a panelům. |
| **R17** | ATC sektory a vysílače přímo v mapě | Vykresluje vybrané ATC sektory a polohy vysílačů přímo do mapy. Vyžaduje dořešit zdroj minimálních dat, protože T01/T03 jsou REMOVE. |
| **R18** | Volitelné počasí a vzdušné prostory přímo v mapě | Zapíná a vypíná meteorologické překryvy a vzdušné prostory nad mapou; jednotlivé vrstvy musí mít doložený datový zdroj. |
| **R13** | Poloha přijímače, kružnice dosahu a měřítko | Zobrazuje pozici přijímače, vzdálenostní kružnice a mapové měřítko; přesná soukromá lokalita vyžaduje zvláštní ochranu. |
| **R16** | Režim prezentace mapy | Zjednodušuje mapové rozhraní do prezentačního režimu například pro velkou obrazovku. |

### Informace o letadlech (14)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **H03** | Fotografie letadel z Planespotters | Načítá dostupnou fotografii konkrétního letadla od Planespotters podle registrace či jiné podporované identity. |
| **H07** | Výškový, rychlostní a vertikální profil letu | Zobrazuje časový graf skutečně zaznamenané výšky, rychlosti a stoupání či klesání během letu. |
| **H14** | Souhrny letadla za 7/30 dní a celé období | Počítá souhrny z nových záznamů V2 pro jedno letadlo za 7 a 30 dní a za celé období od spuštění V2. |
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

### Letiště (10)

| ID | Odložená funkce | Zamýšlené chování |
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

### Navigace (13)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **T08** | ATS tratě České republiky | Zobrazuje české ATS tratě a jejich publikovanou geometrii pro navigační orientaci. |
| **T09** | ATS tratě Slovenska a Rakouska | Doplňuje ATS tratě Slovenska a Rakouska jako regionální rozšíření; neznamená automaticky obnovu ATC SK/AT. |
| **T13** | Navigation Integrity – podezřelé anomálie ADS-B | Vyhodnocuje podezřelé skoky a nekonzistence ADS-B polohy či návazných údajů; výsledek označuje jako anomálii, ne potvrzený útok. |
| **T14** | Navigation Integrity – regionální mapa, historie a analýza | Zobrazuje oblasti, vývoj v čase a přehled historicky zaznamenaných Navigation Integrity anomálií z nových dat V2. |
| **T01** | ATC sektory včetně geometrie a výškových hranic | Ukládá nebo načítá ATC sektory jako přesné polygony s výškovými hranicemi a platností. |
| **T02** | Samostatný detail ATC sektoru | Zobrazuje samostatný detail vybraného ATC sektoru, jeho výšky a publikované služby. |
| **T03** | ATC frekvence, vysílače a publikované služby | Zpřístupňuje publikované ATC frekvence, polohy vysílačů a rozsah příslušných služeb. |
| **T04** | Pravděpodobné ATC přiřazení vybraného letadla | Podle polohy, výšky a geometrie odhaduje, který ATC sektor by mohl být pro letadlo relevantní; neurčuje skutečně naladěnou frekvenci. |
| **T05** | ATC handoff – odhad přechodu mezi sektory | Odvozuje možný budoucí přechod letadla mezi sektory (handoff), nikoliv potvrzený přechod spojení. |
| **T06** | Historie provozu a přechodů mezi sektory – API | Uchovává analytickou historii pravděpodobného provozu a přechodů mezi ATC sektory. |
| **T07** | Validace a diagnostika ATC dat | Provádí validaci geometrie, správnosti a aktuálnosti importovaných ATC dat a zobrazuje diagnostiku. |
| **T11** | Letové procedury SID/STAR a procedurální data | Udržuje a zobrazuje publikované postupy SID/STAR a další procedury odletu/příletu. |
| **T12** | AUP/UUP – plánovaná aktivace vzdušného prostoru | Zobrazuje plánované aktivace, omezení či vyhrazení vzdušných prostorů podle AUP/UUP. |

### Počasí (10)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **W01** | Aktuální METAR a TAF letišť | Načítá a zobrazuje aktuální zprávy METAR a předpovědi TAF pro vybraná letiště. |
| **W02** | METAR značky a informace přímo v mapě | Přidává na mapu meteorologické značky letišť a rychlý přehled podmínek z METAR. |
| **W04** | Echo Top – výška radarových odrazů | Zobrazuje odhad maximální výšky radarových odrazů (Echo Top), pokud poskytovatel takový produkt skutečně dodává; není to běžný 2D radarový snímek. |
| **W05** | Model větru ICON-EU v různých letových hladinách | Načítá modelovaný vítr ICON-EU pro zvolené hladiny a zobrazuje jeho prostorový průběh. |
| **W07** | PIREP/AIREP hlášení pilotů | Zobrazuje meteorologická hlášení z letadel podaná piloty (PIREP/AIREP), odlišná od automatických BDS dat. |
| **W10** | Weather Fusion – spojení více zdrojů počasí | Spojuje několik meteorologických zdrojů do jednoho odvozeného výstupu včetně nejistoty a časové platnosti. |
| **W11** | Weather Avoidance – vyhodnocení možné odchylky kvůli počasí | Vyhodnocuje, zda počasí může vést k odchylce letu od předpokládané trasy; jde o hypotézu, ne potvrzenou příčinu. |
| **W12** | Počasí na trase zvoleného letu | Sestavuje přehled počasí podél známé nebo plánované trasy vybraného letu. |
| **W13** | Historické mapové počasí v Time Machine | V režimu historického přehrávání synchronizuje mapové počasí s historickým časem. |
| **W14** | Weather Corridor – počasí v předpokládaném koridoru | Přepočítává očekávané počasí v budoucím koridoru predikovaného letu. |

### Upozornění (12)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **L01** | Watchlist – pravidla sledování letadel | Spravuje pravidla, která letadla nebo situace má uživatel aktivně sledovat. |
| **L02** | Hlídání ICAO, registrace, callsignu, typu a aerolinky | Podporuje pravidla odpovídající konkrétnímu ICAO, registraci, callsignu, typu či aerolince. |
| **L03** | Upozornění podle vzdálenosti od přijímače | Vyvolává upozornění při dosažení definované vzdálenosti cíle od přijímače. |
| **L04** | Fleet Explorer – vlastní sledované flotily | Umožňuje sestavit vlastní flotily a prohlížet či sledovat jejich členy jako skupinu. |
| **L05** | Geofencing – vlastní sledované geografické oblasti | Umožňuje vlastní geografické zóny a upozornění při vstupu či výstupu letadla. |
| **L07** | Historie upozornění a událostí | Ukládá a zobrazuje časovou historii vytvořených upozornění a událostí. |
| **L08** | Pravidla cooldownu a odstranění duplicit | Uplatňuje časovou prodlevu a deduplikaci, aby stejná událost neposílala opakovaně oznámení. |
| **L09** | Notification Center – přijaté/nepřečtené zprávy | Nabízí centrum zpráv s označením přečtených a nepřečtených upozornění. |
| **L10** | Ztlumení oznámení pro letadlo nebo pravidlo | Dovoluje uživateli ztlumit oznámení pro konkrétní letadlo či pravidlo. |
| **L13** | Delivery Health – stav doručování zpráv | Sleduje čekající, odeslané, neúspěšné a opakované pokusy o doručení upozornění. |
| **L14** | Alert Rule Simulator – testování pravidel | Simuluje průchod pravidly na zkušebních datech bez skutečného odesílání zpráv. |
| **L15** | Alert Effectiveness Analytics – účinnost upozornění | Měří užitečnost a výsledky vyvolaných upozornění, například podíl relevantních a duplicitních. |

### Doplňující zdroje (10)

| ID | Odložená funkce | Zamýšlené chování |
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
| **E08** | ACARS H1/FPN waypointy a mapová vrstva | Převádí ACARS zprávy H1/FPN obsahující waypointy na mapovou vrstvu plánované či oznámené trasy. |

### Statistiky (6)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **S02** | Denní/7denní/30denní agregované statistiky – API | Vypočítává denní a souhrnné statistiky za 7 a 30 dní z dat nově získaných V2 a zpřístupňuje je přes API. |
| **S03** | Receiver Coverage – polární mapa dosahu | Vytváří polární mapu geografického dosahu přijímače podle skutečně lokálně přijatých poloh a směrů. |
| **S04** | Analýza slabých sektorů a spolehlivosti příjmu | Vyhodnocuje slabší směrové sektory, dostupnost signálu a stabilitu místního příjmu z měřitelných ukazatelů. |
| **S05** | Rekordy vzdálenosti a přijatých letadel | Udržuje rekordní hodnoty, například nejvzdálenější zachycené letadlo nebo historický maximální počet. |
| **S07** | Historická heatmapa a profil provozu – API | Vytváří historickou heatmapu intenzity příjmu či provozu a další časové profily přes API. |
| **S08** | Denní a týdenní recap – zbývající API | Automaticky sestavuje denní nebo týdenní rekapitulaci provozu; odlišuje se od jednoduchého S09 logbooku. |

### Provoz (7)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **O09** | T58 production soak, stall canary a CPU profiling | Umožňuje řízený dlouhodobý provozní test T58, sledování zaseknutí a profilování CPU na bezpečné testovací zátěži. |
| **O10** | Výkonové benchmarky a zátěžové testy | Poskytuje opakovatelné benchmarky a zátěžové testy pro ingest, SSE, UI a databázi s porovnatelnými vstupy. |
| **O11** | Automatické CI, testy, CodeQL a bezpečnostní kontroly · základ systému | Zajišťuje automatické testy, lint, CodeQL či obdobný bezpečnostní audit a kontrolu přesného commitu v GitHub Actions. |
| **O12** | Automatické nasazení, health check a rollback · základ systému | Připravuje automatizované sestavení a nasazení samostatné V2 s kontrolami health, možností návratu a bez zásahu do V1. |
| **O13** | Import/synchronizace eAIP, ATS a letištních dat | Bezpečně synchronizuje a verzovaně importuje publikovaná eAIP, ATS a letištní data, s kontrolou původu. |
| **O15** | Správa přihlášení administrátora a bezpečnostních oprávnění · základ systému | Zajišťuje přihlášení správce, role/oprávnění a ochranu neveřejných API a administračních operací. |
| **O14** | Automatická dokumentace, changelog a metriky růstu kódu | Automaticky generuje část projektové dokumentace, changelog a grafy velikosti či růstu zdrojového kódu. Ručně vedený CHANGELOG může existovat i bez této funkce. |

### Uživatelské prostředí (2)

| ID | Odložená funkce | Zamýšlené chování |
|---|---|---|
| **U06** | Oblíbená letiště a lokální uživatelské preference | Umožňuje uložit oblíbená letiště a další místní osobní preference uživatele. |
| **U07** | Sdílené záložkové odkazy na filtrování a analýzy | Vytváří sdílitelnou URL s konkrétním filtrem nebo analytickým stavem obrazovky. |

### Predikce (23)

| ID | Odložená funkce | Zamýšlené chování |
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

### Analytika (15)

| ID | Odložená funkce | Zamýšlené chování |
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

## Důvod zachování archivu

Nechceme přijít o dřívější nápady, ale nechceme je ani držet v aktivních zadáních, testovacích maticích nebo infrastrukturním plánu. Historie verzí a V1 zůstává nedotčená. Inženýrské minimum (regresní testy, bezpečnost, zálohy a návrat) platí bez ohledu na to, zda někdy vzniknou samostatné produktové rozšiřující funkce.

## Návrh rozšíření příjmu a pokrytí — pouze odložený koncept (22. 10. 2026)

**Status:** NEROZHODNUTO / NENÍ VE 45 AKTIVNÍCH FUNKCÍCH. Tento záznam není povolením implementace, nasazení, síťového připojení, ani automatickým znovuzařazením jakékoli odložené funkce. Stávající seznam 133 ID a jejich počet se nemění; jde o nový scénář k budoucímu posouzení, nikoli o nové schválené ID.

### A. Druhá vlastní vzdálená stanice — preferovaná první varianta rozšíření

```text
rx-primary (readsb JSON/BEAST) ─┐
                                ├─ [jeden centrální Rust engine] → RAM stav → SSE/Mapa
rx-remote-2 (readsb BEAST) ─ VPN ┘                       └─ adaptivní vzorky → PostgreSQL
```

- Další fyzický ADS-B přijímač má vlastní `receiverId`, vlastní readsb/SDR a **žádnou vlastní databázi AirRadar**. Do centrály se BEAST rámce posílají přes ověřený soukromý kanál (preferenčně WireGuard VPN s minimálními právy); nevystavovat nezabezpečený TCP port `30005` do veřejného internetu.
- Dvě stanice přijímající jednu ADS-B zprávu **nevytvoří dvě letadla** ani dva historické řádky; kanonický `aircraftKey` zůstává společný. Každé pozorování má `receiverId` a oddělené `transport`.
- JSON a BEAST na jedné stanici **nejsou dva nezávislé přijímače**. Při připojení remote BEAST zachovat vazbu na příslušnou stanici z autentizovaného VPN/socket profilu; nespoléhat na volně dodaný ID v payloadu.
- BEAST MLAT čítače z různých stanic nelze navzájem porovnávat jako UTC a prostým rozdílem počítat polohu. Vlastní **MLAT algoritmus nespadá do tohoto návrhu**; potřebuje synchronizované přijímače, další práci a samostatné rozhodnutí.
- Stejná zpráva z různých stanic: transportní dedup zvlášť pro každou stanici, pozorovací slučování podle platného ICAO/non-ICAO, skutečné časové čerstvosti a fyzikální plausibility. **Nikdy nezprůměrovávat** nesouhlasné pozice; rozdíly mohou znamenat chybu, MLAT odlišné kvality, zpoždění nebo spoofing. U nevysvětlitelného konfliktu označit nejistotu a nevkládat sporný bod do historie.
- Konkrétní data o dostupnosti stanice, RSSI a dosahu držet ve výchozím stavu jen v **bounded RAM metrikách**. Nezapisovat každou zprávu nebo RSSI do PostgreSQL. Nezveřejňovat přesné umístění vzdálené stanice.
- Ověření před schválením: výpadek a obnova jedné lokality; překryv pokrytí, 10× opakované zprávy, síťový jitter 0–5 s; chyba hodin; duplicitní hex s fyzikálně nemožnou vzdáleností; nárůst RSS, CPU, frame/s, SSE, DB write/s; oddělení ostatních aplikací v DB LXC.

**Architektonické cesty:** (1) centrální Rust přijímá autentizované samostatné BEAST TCP streamy s `receiverId` — lepší provenance a diagnostika; (2) centrální readsb kombinuje více Beast feedů a Rust čte jeho JSON — nejmenší implementační náklad, ale **je nutné ověřit, zda verze readsb zachová identitu zdrojové stanice v použitém výstupu**. Pokud centrální JSON provenance ztratí, nepředstírat historii jednotlivých stanic.

Oficiální readsb dokumentace: https://manpages.debian.org/unstable/readsb/readsb.1.en.html — více instancí `--net-connector`; BEAST technický čas: https://github.com/firestuff/adsb-tools/blob/master/protocols/beast.md.

### B. Síťová třetí strana — až samostatné rozhodnutí a licenční prověrka

Externí data nejsou totéž co vlastní RF pozorování. Zdroj může poskytovat **agregovaný JSON snapshot**, BEAST/SBS, **MLAT**, částečně odhadnutou polohu nebo pouze metadata. Je nutné samostatné povolení každého poskytovatele pro live zobrazení, uchování historie, odvozenou databázi a veřejnou redistribuci.

| Poskytovatel | Potenciál | Omezení, která musí být vyřešena před aktivací |
|---|---|---|
| **adsb.lol** | Veřejné síťové API, data pod **ODbL 1.0**, možné pokrytí mimo naše přijímače | Posoudit share-alike/odvozenou databázi, veřejnou atribuci, aktuální přístupové klíče a vhodnost produkčního provozu; výslovně kontaktovat správce při produkční závislosti. https://www.adsb.lol/docs/open-data/api/ |
| **OpenSky Network** | Síťové stavové vektory včetně oblasti/času | Podle aktuálních podmínek **provozní využití REST API vyžaduje předchozí písemnou dohodu** i pro nekomerční projekt; komerční licence zvlášť. https://opensky-network.org/about/terms-of-use |
| **adsb.fi** | Veřejná otevřená data a API, možná geografická oblast | **Osobní nekomerční použití**, 1 request/s na veřejném API, povinná atribuce; nesplňovat jiné podmínky znamená nepoužít. https://github.com/adsbfi/opendata |
| **ADS-B Exchange** | Rozsáhlé pokrytí a API | Smluvní použití, omezení dalšího zpřístupňování či redistribuce; vyžaduje posouzení licence a způsobu provozu. https://support.adsbexchange.com/hc/en-us/articles/37364077703693-What-is-ADS-B-Exchange-s-data-use-policy |
| **Airplanes.live** | API pro polohy a další údaje | Samotná dokumentace API neurčuje licenci na kopírování/redistribuci leteckých dat; před implementací získat aktuální podmínky a provozní limity. https://airplanes.live/api-docs/ |

**Nezaměňovat otevřený zdrojový kód API s licencí samotných letových dat.** Některé zdroje umožňují osobní zobrazování, ale zakazují redistribuci/veřejné uložení. Při smíšení více zdrojů může být nutné zachovat data oddělená a respektovat nejpřísnější smluvní omezení.

Potenciální návrh až po schválení: adaptér s `providerId`, informací o povoleném účelu (`display`, `store`, `redistribute`), TTL/stáří, limitu geografické oblasti, `positionOrigin` (ADS-B/MLAT/odhad/neznámé), explicitní degradací na poslední platnou lokální polohu a samostatnou cache. Zdroj bez důvěryhodných souřadnic se nesmí vydávat za čerstvé místní RF pozorování. Každý zdroj má vlastní rate limit, timeout a bounded cache; nikdy nesmí zastavit Rust ingest ani zaplavit SQL.

### C. Co při budoucím schvalování zvážit

1. Začít **jednou další vlastní stanicí**, ne integrací všech externích API najednou; měřit skutečně rozšířené pokrytí bez přidání duplicit a bez růstu SQL.
2. Rozhodnout, zda vzdálené stanice poskytují pouze přijaté ADS-B zprávy, nebo mají také GPS čas potřebný pro **skutečné multilaterace**; MLAT bez synchronizace neslibovat.
3. V rámci licence rozhodnout, co smíme jen zobrazovat a co trvale uchovávat v PostgreSQL; obecně externí display-only data **neukládat do J09**.
4. Zachovat **jednu mapu a jeden AircraftState**; neobnovovat přepínač LOCAL/EXTENDED. Původ zdroje a nejistota může být v budoucnu zobrazena nenápadnou ikonou/textem, až bude konkrétně schváleno.
5. Před implementací vytvořit akceptační testy identity, priority stáří/pravděpodobnosti, redundantní stanice, výpadků, licence, vypnutí provideru a paměťových limitů.
6. Produktové rozšíření přesunout do aktivních 45 funkcí **pouze na nové výslovné schválení vlastníka**, s úpravou matice, plánů, ADR, testů a changelogu. Dokud nebude schváleno, žádné importy ani nové datové zdroje do běžícího V2.
