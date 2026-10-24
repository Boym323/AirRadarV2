# G-05 — průběžný výsledek implementace živé mapy

## 24. 10. 2026 — mapové symboly a adaptivní štítky G-05.03/.04 (PR #29 sloučen)

- Základna: `79a019e2d9c651a9d04ad04e17563c9934d602a6`. G-05.03/R03 + G-05.04/R04.
- Přidáno `apps/web/src/radar/aircraft-icons.ts`: **10 vlastních vektorových Path2D siluet** (jednotná 64×64), deterministické mapování pouze z volitelné ADS-B emitter category; neznámý/rezervovaný typ = obecný letoun, chybějící/neplatný skutečný ground track = kruh. Žádné DOM markery a žádné obrázky třetích stran.
- `RadarMap.tsx` nyní jednou registruje ikony do MapLibre a znovu při `style.load`; používá jeden GeoJSON aircraft source, stálou Map instance, halo vybraného cíle, oddělenou vrstvu štítků a skutečný track. Živý G-03 SSE, G-04 ani API se nemění.
- Štítky jsou podle přiblížení: zoom pod 8 skryté, od 8 validní callsign, od 11 callsign + dostupná barometrická výška s `ft`; kolize řeší MapLibre bez skrývání symbolů, vybraný má přednost. Testy ověřují klasifikaci, rezervované hodnoty, směr a výsledné GeoJSON.
- **Datová hranice:** `emitterCategory` je zatím pouze volitelný frontendový slot. Rust/public DTO tuto informaci zatím neposkytuje, takže skutečný DEV radar používá obecný symbol/bod. API se nezměnilo. Pro R03 plnou akceptaci je nutný samostatný ověřený G-02/G-03 datový kontrakt a reálná grafická přejímka.
- **Zbývá:** integrační screenshot/test dark/light, interakční a vizuální hustota, G-05.11 měření GPU/FPS; G-05.10 závisí na G-06. **Ověřeno:** [PR #29](https://github.com/Boym323/AirRadarV2/pull/29), hlava `283ef886ce0badfcf981f99fda9ec44171d5b82c`; [Actions #40082045615](https://github.com/Boym323/AirRadarV2/actions/runs/40082045615) **3/3 PASS** (web lint/typecheck/Vitest/build, Rust replay/SSE, dokumentace a Chromium MapLibre smoke). Merge do `main`: `8309f0699fb3503ea0a70c01fc6c3f9543142844`. Automatický DEV release je oddělené ověření. **G-05.03/.04 zůstávají rozpracované**: reálné emitter category není v G-03 DTO; category screenshoty ani FPS/2000 a finální browser přejímka neproběhly.



## G-05 map-first UI — návazná implementace podle schválené specifikace (24. 10. 2026)

- Cíl: R01/R06/R08/R09/R14, P0 z `docs/plan/WEB-UX-UI-SPECIFIKACE.md`. Velká hlavička a trvale dvojpanelový layout nahrazeny kompaktní lištou a dominantní mapou na celou pracovní plochu.
- Seznam se otevře jen na žádost uživatele, detail cíle se otevře po výběru z mapy i seznamu. Na desktopu jsou kontextové panely nad mapou, na mobilní šířce spodní panel. Zavření vrací focus na stabilní tlačítko v liště.
- Detail zobrazuje pouze dostupnou výšku, rychlost vůči zemi, skutečný track a vertikální rychlost; neznámá data jsou „—“. Follow je explicitní a je zakázáno pro stale polohu či přerušený SSE stream. Bez nových backendových dat, mapových zdrojů a závislostí.
- MapLibre `RadarMap` a jediný `useRadarStream` zůstávají trvale namountované a beze změny. Výběr při smazání identity a reconnectu nadále podléhá stávající kontrole epoch/seq.
- Browser Chromium fixture byla rozšířena o výchozí map-first obrazovku, otevření/zavření panelů, fokus, detail, změnu stylu bez remountu a mobil 390×844 bez horizontálního scrollování.
- **Přejímka:** [PR #26](https://github.com/Boym323/AirRadarV2/pull/26) sloučen do `main` jako `c17601d55180b44b5bdbac7c53da00defe3bae46`. Izolovaná [CI #40079234896](https://github.com/Boym323/AirRadarV2/actions/runs/40079234896) **3/3 PASS** (web, Rust/SSE + Chromium desktop/mobil, česká dokumentace). **DEV release z main je samostatné ověření**; screenshot reálného DEV browseru a FPS benchmark zatím nedoloženy. E2E/FPS na fyzickém telefonu, měření G-05.11, letiště G-05.10 ani produkční release nejsou tímto krokem deklarované jako hotové.


**Aktualizace:** 24. 10. 2026 · **Stav:** ROZPRACOVÁNO, 4/11 kroků plně ověřené (G-05.01/.02/.05/.06). [PR #22](https://github.com/Boym323/AirRadarV2/pull/22) byl **sloučen do main 23. 10.**; [DEV nasazení #40028968380](https://github.com/Boym323/AirRadarV2/actions/runs/40028968380) PASS, včetně G-04 živého API/SSE. Jde o DEV náhled, nikoli produkční release nebo finální G-05 přejímku.

## Implementované části

- **G-05.01 — MapLibre klient:** MapLibre GL JS 6.15.0 připnutý v `package.json` a reprodukovatelném `package-lock.json`. Vite worker přes `?worker&url`; jeden React mount = jedna MapLibre instance, WebGL fallback na seznam. Default OpenFreeMap Liberty, bez přístupového klíče, s atribucí. 
- **G-05.02 — G-03 SSE adapter:** `apps/web/src/radar/aircraft-store.ts` a `use-radar-stream.ts`; atomické snapshot/delta, `epoch/seq` kontrola a vynucení nového snapshotu při gapu/reconnectu. 5 000 max. letadel, validace identity/souřadnic a maximálně jeden React publikovaný update za sekundu. Žádné přímé DB nebo další ADS-B ingest.
- **G-05.03/.04 — mapová reprezentace:** jediný GeoJSON aircraft source, symbolová ikona letadla pouze při známém reálném `trackDegrees`, jinak bod; popisek podle skutečného callsign. Stabilní feature ID. Nepoužívají se jednotlivé DOM markery.
- **G-05.05/.06 — seznam a výběr:** jeden store pro mapu i vyhledatelný seznam, filtr podle dostupné čerstvé polohy, výběr letadla a vysloveně neprodukční prázdný stav.
- **G-05.07/.08/.09 — navazující základ:** browser-only přijaté body stopy omezené na 64 bodů/20 minut bez extrapolace, volitelný follow s vypnutím při pan a podklady Liberty/Fiord. Pro finální TESTED je nutná prohlížečová interakční regrese.
- **Jednotkové testy:** `apps/web/tests/g05-radar.test.ts` testuje atomické delty, duplicitní cíle, gaps, epochy, odstranění, validitu souřadnic, retenci a heading. `apps/web/tests/app.test.tsx` ověřuje české UI i stav bez live dat.

## Navazující změna G-05.08 / R09 — ochrana ručního pohybu mapou

- Opravena situace, kdy se follow režim vypnul pouze po `dragstart`, ale při ručním zoomu kolečkem, dotykovém přesunu nebo navigaci šipkami se mapa mohla každou sekundu vracet k letadlu.
- MapLibre nyní uvolňuje follow při drag/rotate/pitch a při `wheel`, `touchmove` a navigačních klávesách uvnitř mapy. Programové `easeTo` sledování neruší. Přidána korektní registrace i odregistrování DOM listenerů a přístupný stav tlačítka `aria-pressed`.
- Rozšířen izolovaný Chromium regresní test o skutečné kolečko myši a obsluhu klávesnice/dotykového pohybu. **Výsledek nového CI musí být doložen před změnou statusu kroku na TESTED.** Bez RF/DB a bez tvrzení o mobilním GPU/FPS.

## Co zůstává

- **G-05.10** letištní zdroj je závislý na G-06; žádný falešný katalog nebyl přidán.
- **G-05.11** Playwright se skutečným browser/WebGL, referenční výkon/FPS pro 25/100/500/2000 a setData vs updateData se musí změřit v samostatném prostředí; čísla nejsou vydávána za ověřená.
- Ověřené části **R01 a R06 mají stav `TESTED`**: [hlavní CI #40024971130](https://github.com/Boym323/AirRadarV2/actions/runs/40024971130) PASS včetně Chromium reálného map canvasu nad deterministickým SSE a offline podkladem. Další mapové funkce mají stav IN_PROGRESS, R12 NOT_STARTED; etapa se neuzavírá.

## Hranice bezpečnosti

- G-03 zůstává jedinou autoritou v Rust RAM a web konzumuje jen `/api/v1/stream`.
- Pouze veřejná/sanitizovaná DTO; žádný nový runtime server, DB či externí datový zdroj letadel.
- Podklady OpenFreeMap potřebují internet a jejich dostupnost nesmí blokovat seznam letadel.
- Žádný zásah do AirRadar V1, PP Studio, PostgreSQL či provozního G-03.

## Důkazy

**První důkaz:** CI #40024971130 PASS: Rust, Vite ESLint/TypeScript/Vitest/build, dokumentace a skutečný Chromium canvas se SSE snapshot/delta, odstraněním cíle, výběrem, style switch a bez remountu. Skutečný GPU/FPS benchmark, mobilní browser a celkovou G-05.11 akceptaci označujeme **NEOVĚŘENO**.
