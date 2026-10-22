# ADR 0008 — Identita a původ lokálního ADS-B pozorování

**Datum:** 22. října 2026. **Stav:** přijato jako minimální kontrakt pro G-01/G-02. Neimplementováno; jde o přípravu správné datové identity, nikoli o rozšíření aktivního rozsahu 45 funkcí. Navazuje na [ADR 0007](0007-ram-first-deduplikace-a-usporny-zapis.md).

## Problém

Dva technické transporty jednoho `readsb` (`aircraft.json` a BEAST TCP) **nejsou dvě nezávislé přijímací stanice**. Samotné pole `source: json|beast` neposkytuje stabilní identitu původu a může vést k dvojímu započítání příjmu, nekorektnímu součtu statistik, opakovanému SSE eventu a redundantnímu zápisu do PostgreSQL.

## Minimální rozhodnutí, které musíme zavést už teď

1. Každá interní normalizovaná observace má **stabilní neveřejné `receiverId`** přidělené důvěryhodnou konfigurací enginu. Pro dnešní instalaci jde o jednu pevnou hodnotu, např. `rx-primary`. Nevytváříme zatím uživatelskou správu stanic, externích providerů ani nový proces.
2. `transport` označuje pouze způsob vstupu (`readsb_json` či `beast_tcp`). `receiverId` označuje *fyzické/logické místo příjmu*; tyto dva pojmy nejsou zaměnitelné.
3. `aircraftKey` zůstává `icao:abcdef` nebo `nonicao:abcdef`. Nesestavovat jej z `receiverId` či callsignu. Identita letadla má zůstat konzistentní při JSON/BEAST fallbacku.
4. `receivedAtUtc`, `positionAtUtc`, zdrojový čas a dostupnost jsou **různé hodnoty**. BEAST 12MHz MLAT čítač není bez kalibrace UTC. `seen_pos` je stáří polohy, nikoliv nového přijetí.
5. Před deduplikací a volbou kanonického pole validovat jeho čerstvost, původ, stav dekodéru a fyzikální plausibilitu. Naivně nepreferovat poslední doručený paket. Nededukovat „vyšší kvalitu“ jen z BEAST transportu.
6. V RAM uchovávat drobnou neveřejnou informaci o původu hodnot. **REST/SSE zveřejní jen schválenou abstraktní informaci o aktuálnosti/kvalitě**, nikoliv adresu nebo souřadnice přijímače; v G-01 nadefinovat přesný seznam zveřejňovaných položek.
7. PostgreSQL J09 ukládá pouze finální adaptivně vzorkovanou platnou polohu z kanonického Rust RAM stavu. Nevytvářet další raw tabulky, zdrojové per-packet historie, nový sekundární index ani další DB writer. Pokud pro produkt není potřeba dlouhodobě držet receiver identifikátor, **nepřidávat ho do dnešní fyzické tabulky**.
8. Zdraví receiveru a čítače počítat z reálně dostupných čítačů zařízení; stejné `readsb` JSON/BEAST pozorování nesmí být vydáváno za dvě unikátně přijaté zprávy. Při nejistotě vykazovat obě transportní metriky odděleně, nikoli jejich falešný součet.

## Minimální interní kontrakt G-01

~~~ts
type LocalAircraftObservation = {
  receiverId: string;                         // např. "rx-primary", neveřejné
  transport: "readsb_json" | "beast_tcp";
  aircraftKey: `icao:${string}` | `nonicao:${string}`;
  receivedAtUtc: string;                     // čas přijetí na serveru
  positionAtUtc?: string;                    // čas platné polohy
  receiverTicks?: number;                    // BEAST; nikdy ne automaticky UTC
  position?: { lat: number; lon: number };
  validity: "valid" | "stale" | "invalid";
};
~~~

Jde o dokumentační typ, ne o již existující TypeScript/Rust definici. Skutečné DTO musí odpovídat G-01 schématům a testům. Rust si má minimální `receiverId` připravit v konfiguraci tak, aby se **nemusela měnit identita letadla nebo přepisovat RAM state** při pozdějším schváleném rozšíření.

## Co se výslovně nezavádí

Žádný další síťový feed, nová API integrace, UI přepínač, správa přijímačů, model multi-site statistik, licenční agregátor ani nová DB tabulka. **Nápady na případné produktové rozšíření existují výhradně v [jediném archivu](../backlog/FUNKCE-PRO-BUDOUCI-ROZSIRENI.md)**. Nečíst archiv při běžné implementaci G-01/G-02; schválených 45 funkcí zůstává beze změny.

## Testy a omezení

- Stejná fyzická lokální stanice přes JSON a BEAST → jedno `receiverId`, jedno `aircraftKey`, žádné zdvojení polohy/statistik/SQL.
- Neplatné či chybějící `receiverId` z nepodloženého vstupu nelze povýšit na důvěryhodné; engine zná konfiguraci, nikoli náhodný řetězec z klientského payloadu.
- `nonicao:abc123` a `icao:abc123` zůstávají oddělené.
- Doporučený 60s adaptivní sampling i 1Hz SSE zůstávají nezměněné a nesmějí být svázané s jedním konkrétním transportem.

**Nebyly provedeny změny runtime kódu, konfigurace LXC, schématu PostgreSQL ani produkce.**
