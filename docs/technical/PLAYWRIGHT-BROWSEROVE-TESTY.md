# Playwright — prohlížečové akceptační testy AirRadar V2

**Stav:** návrh automatizace pro G-01/G-05/G-10/G-11; knihovna ani browsery dosud nebyly přidány do repozitáře a **žádný Playwright E2E test nebyl spuštěn**. Microsoft Playwright je testovací nástroj, **není potřeba .NET, Azure ani SQL Server**.

## 1. Co musí testovat browser, ne unit test React komponenty

| Test | Kdy | Očekávaný výsledek |
|---|---|---|
| E2E-00 | první PR s Playwright | Na statickém Vite preview existuje \`lang="cs"\`, nadpis a pravdivé upozornění, že není připojen živý provoz |
| E2E-01 | G-03/G-05 | Z počátečního \`snapshot\` se vykreslí seznam/mapa, \`delta.upsert\` změní konkrétní cíl a \`removed\` jej odstraní |
| E2E-02 | G-03/G-05 | Přerušení EventSource, restart epoch a vynechaná seq vyvolá nový snapshot, žádné zdvojené letadlo |
| E2E-03 | G-05 | Klik na cíl ze seznamu, filtry, trail a follow, pan/zoom kamerou, ruční vypnutí follow |
| E2E-04 | G-05 | 250/500/1 000 deterministických letadel, po upsertu žádná duplicitní feature ID |
| E2E-05 | G-10 | Mobilní viewport, keyboard/ARIA a ovladatelnost filtrů bez překryvů |
| E2E-06 | G-10 | PWA instalovatelnost, offline s označením stáří a návrat online |
| E2E-07 | G-08 | Squawk alarm v testovacím dry-run; **nikdy skutečně neodeslat push** |
| E2E-08 | G-11 | Chrome/Firefox/WebKit bez přístupu k real receiveru a produkční DB |

## 2. Doporučená konfigurace (návrh pro samostatný PR)

~~~ts
// apps/web/playwright.config.ts — až po přidání @playwright/test
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: 'http://127.0.0.1:4100',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
    { name: 'mobile-chromium', use: { ...devices['Pixel 7'] } },
  ],
  webServer: {
    command: 'npm run preview',
    url: 'http://127.0.0.1:4100',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
~~~

Není zde třeba druhý Node backend; Playwright spouští pouze HTTP server pro **test hotového statického buildu**. Před spuštěním: \`npm ci && npm run build && npx playwright test\`. Browsery instalovat odděleně \`npx playwright install --with-deps\`. Vlastní Rust/SSE fixture server spouštět pouze v testech, pokud daný scénář vyžaduje reálné streamované eventy.

~~~ts
// apps/web/e2e/bootstrap.spec.ts — první skutečně proveditelný test G-00
import { test, expect } from '@playwright/test';

test('česká stránka nepředstírá živý radar', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('html')).toHaveAttribute('lang', 'cs');
  await expect(page.getByRole('heading', { name: 'AirRadar V2' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Není připojen živý provoz' }))
    .toBeVisible();
});
~~~

**Žádné pevné \`waitForTimeout(5000)\`** jako synchronizační mechanismus. Používat \`getByRole\`, \`getByLabel\`, asynchronní \`expect(...).toBeVisible()\` a jednoznačné testovací ID tam, kde sémantika nestačí.

## 3. Testovací data a SSE

- Jestliže testuješ jen jednu HTTP odpověď, Playwright \`page.route(...)\` může vracet syntetické JSON. Skutečné **postupné SSE eventy, reconnect a heartbeat** ověřuj pomocí samostatného lokálního fixture serveru a jeho deterministického scénáře; jednorázové \`route.fulfill\` se statickým body neprokazuje dlouhodobé streamování.
- Ve fixture používat např. \`icao:abc123\` a \`nonicao:abc123\`, syntetické souřadnice a časy; nikdy necílit na reálný readsb/DB LXC.
- Mapové podklady fixovat či podvrhnout stabilním legálně použitým stylem. Externí dlaždice, meteorologické API a mapové fonty nesmějí rozhodovat o PASS/FAIL.
- Testy nouzového squawk používají v G-08 výslovně \`DRY_RUN\` a nemají VAPID/Pushover produkční credentials.

## 4. CI a důkazy

- **První fáze:** krátký Chromium smoke E2E nad dnešním bootstrap; v CI oddělený job \`e2e-chromium\` jen po explicitním přidání dependency a lockfile v jednom commitu.
- **Při mapě/PWA:** rozšířit o Firefox + WebKit + mobilní emulaci; nejprve jeden worker v CI kvůli stabilitě.
- Při neúspěchu uložit Playwright trace/screenshot/HTML report jako GitHub Actions artifact, s rozumnou retencí a bez neveřejných dat.
- **CI PASS znamená funkční browserové scénáře**, nikoliv ověřený výkon grafického procesoru. FPS a GPU profilovat na skutečném cílovém hardware v samostatném reportu.
- Timeouty, retries, headless GPU a případné flaky testy reportovat otevřeně; nevypínat testy kvůli zelené CI.

## Oficiální dokumentace

- [Playwright CI](https://playwright.dev/docs/ci)
- [Playwright webServer](https://playwright.dev/docs/test-webserver)
- [Playwright auto-retrying assertions](https://playwright.dev/docs/test-assertions)
- [Playwright API mocking](https://playwright.dev/docs/mock)
- [Playwright trace viewer](https://playwright.dev/docs/trace-viewer)
