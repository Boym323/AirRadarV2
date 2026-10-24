#!/usr/bin/env python3
"""G-05.01–.09 browser smoke bez reálného RF/DB/tile serveru."""
import json
from pathlib import Path
from io import BytesIO
from PIL import Image
import subprocess
import time
from contextlib import closing
from urllib.request import urlopen
from playwright.sync_api import sync_playwright, expect

URL = "http://127.0.0.1:4100"
STYLE = {"version": 8, "name": "CI offline map", "sources": {},
         "layers": [{"id": "land", "type": "background",
                     "paint": {"background-color": "#1c3445"}}]}

# Syntetická bodová fixture; skutečný veřejný katalog se tvoří zvlášť
# z pinned/unlicensed OurAirports CSV při npm run build.
AIRPORTS = {
    "version":1,
    "source":{"name":"OurAirports",
              "commit":"3b10d0b967cc04c62eb6411939e4e9936a88e701"},
    "airports":[
        {"id":"CZ-TEST1","name":"Testovací letiště CZ-TEST1",
         "type":"small_airport","lat":49.23,"lon":17.67,"country":"CZ"},
        {"id":"CZ-TEST2","name":"Testovací letiště CZ-TEST2",
         "type":"small_airport","lat":49.28,"lon":17.84,"country":"CZ"},
        {"id":"CZ-TEST3","name":"Testovací letiště CZ-TEST3",
         "type":"small_airport","lat":49.10,"lon":17.48,"country":"CZ"},
    ],
}

MOCK = r"""
(() => {
  const now = Date.now() / 1000;
  const observation = (id, lon) => ({
    id, lastMessageAt: now, lastPositionAt: now,
    position: {lat: 49.2, lon}, positionOrigin: 'adsb', stale: false,
    callsign: id === 'icao:abc123' ? 'OKTEST' : 'OKTWO',
    squawk: null, telemetry: {trackDegrees: 105, altitudeBaroFt: 6400}
  });
  window.__radarFixture = {
    initial: {kind:'snapshot', epoch:'g05-ci', seq:0,
              generatedAt: new Date().toISOString(),
              aircraft:[observation('icao:abc123', 17.5),observation('icao:bbbbbb',17.6)]},
    next: {kind:'delta', epoch:'g05-ci', seq:1,
           generatedAt: new Date().toISOString(),
           upsert:[observation('icao:abc123',17.51)], removed:['icao:bbbbbb']}
  };
  const sources = [];
  window.__emitRadar = (type, payload) => {
    for (const source of sources) {
      if (!source.closed) source.dispatchEvent(
        new MessageEvent(type, {data: JSON.stringify(payload)})
      );
    }
  };
  window.EventSource = class extends EventTarget {
    constructor(url) {
      super();
      this.url = url;
      this.closed = false;
      sources.push(this);
      setTimeout(() => {
        if (!this.closed) this.dispatchEvent(new MessageEvent('snapshot', {
          data: JSON.stringify(window.__radarFixture.initial)
        }));
      }, 50);
    }
    close() { this.closed = true; }
  };
})();
"""

def main():
    process = subprocess.Popen(
        ["npm", "run", "preview", "--prefix", "apps/web"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    try:
        for _ in range(80):
            try:
                with closing(urlopen(URL, timeout=1)) as response:
                    if response.status == 200:
                        break
            except OSError:
                time.sleep(.25)
        else:
            raise AssertionError("Vite preview se nespustil")
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(
                channel="chrome", headless=True,
                args=["--enable-webgl", "--use-gl=angle",
                      "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
            )
            page = browser.new_page(viewport={"width": 1370, "height": 850})
            page.add_init_script(MOCK)
            page.route("https://tiles.openfreemap.org/**",
                       lambda route: route.fulfill(
                           status=200, content_type="application/json",
                           body=json.dumps(STYLE)))
            page.route("**/data/airports-ourairports.json",
                       lambda route: route.fulfill(
                           status=200, content_type="application/json",
                           body=json.dumps(AIRPORTS)))
            page.goto(URL)
            expect(page.locator("html")).to_have_attribute("lang", "cs")
            expect(page.get_by_role("heading", name="AirRadar", exact=True)).to_be_visible()
            expect(page.locator(".version-tag")).to_have_text("V2")
            expect(page.locator(".map-disclosure")).to_have_count(0)
            expect(page.locator(".radar-footer")).to_have_count(0)
            expect(page.get_by_label("Hledat letadlo")).to_be_visible()
            assert page.evaluate("""() => {
              const map = document.querySelector('.maplibregl-canvas').getBoundingClientRect();
              const header = document.querySelector('.radar-header').getBoundingClientRect();
              return Math.abs(map.height - innerHeight) <= 2 && header.top > 0 &&
                header.bottom < 100 && document.documentElement.scrollWidth <= innerWidth;
            }"""), "Radar musí mít mapu přes celý viewport a plovoucí nástroje"
            assert page.evaluate("""() => {
              const title = document.querySelector('.brand-title');
              const container = document.querySelector('.brand');
              return title.getBoundingClientRect().right <= container.getBoundingClientRect().right + 1;
            }"""), "Značka nesmí přetékat ani zalamovat název"
            expect(page.get_by_text("2 letadla")).to_be_visible(timeout=12000)
            canvas = page.locator(".maplibregl-canvas")
            expect(canvas).to_be_visible(timeout=12000)
            canvas.evaluate("(el) => { el.dataset.g05Mounted = 'once'; }")
            # G-05 R01/R06: mapa zůstává dominantní, seznam je na vyžádání.
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_have_count(0)
            page.get_by_label("Hledat letadlo").fill("OKT")
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_be_visible()
            expect(page.get_by_role("button", name="OKTEST")).to_be_visible()
            page.get_by_label("Hledat letadlo").fill("")
            page.get_by_role("button", name="Zavřít seznam").click()
            page.get_by_role("button", name="Letadla", exact=True).click()
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_be_visible()
            page.get_by_role("button", name="OKTWO").click()
            expect(page.get_by_role("heading", name="OKTWO")).to_be_visible()
            page.evaluate("window.__emitRadar('delta', window.__radarFixture.next)")
            expect(page.get_by_text("1 letadlo")).to_be_visible(timeout=12000)
            expect(page.get_by_role("button", name="OKTWO")).to_have_count(0)
            expect(page.get_by_role("heading", name="OKTWO")).to_have_count(0)
            # Re-appearance of a removed ID must not silently restore selection.
            page.evaluate("""() => {
              const fixture = window.__radarFixture;
              window.__emitRadar('delta', {
                kind: 'delta', epoch:'g05-ci', seq:2,
                generatedAt:new Date().toISOString(),
                upsert: [fixture.initial.aircraft[1]], removed:[]
              });
            }""")
            expect(page.get_by_text("2 letadla")).to_be_visible(timeout=12000)
            expect(page.get_by_role("heading", name="OKTWO")).to_have_count(0)
            page.get_by_role("button", name="Letadla", exact=True).click()
            page.get_by_role("button", name="OKTEST").click()
            expect(page.get_by_role("heading", name="OKTEST")).to_be_visible()
            Path(".artifacts").mkdir(exist_ok=True)
            page.screenshot(path=".artifacts/g05-i2-selected.png", full_page=True)
            # G-05.07: na mapě musí být skutečně žlutá stopa z nových
            # dvou časově odlišných poloh; offline styl nemá jiné žluté prvky.
            # Sekvence 0->1->2 už v testu proběhla. Nový bod není predikce.
            page.evaluate("""() => {
              const first = window.__radarFixture.initial.aircraft[0];
              const at = first.lastPositionAt + 20;
              window.__emitRadar('delta', {
                kind:'delta', epoch:'g05-ci', seq:3,
                generatedAt:new Date(at * 1000).toISOString(),
                upsert:[{...first,lastMessageAt:at,lastPositionAt:at,
                  position:{lat:49.2,lon:17.84}}],removed:[]
              });
            }""")
            # Screenshot assertion na reálném WebGL canvasu, ne pouze na store.
            # Čeká se na konkrétní barevný důkaz, nikoli pevný timeout.
            warm_pixels = 0
            stop_at = time.monotonic() + 12
            while time.monotonic() < stop_at:
                shot = canvas.screenshot()
                pixels = Image.open(BytesIO(shot)).convert("RGB")
                warm_pixels = sum(
                    1 for red, green, blue in pixels.getdata()
                    if red > 170 and green > 100 and blue < 170
                    and red > green + 25 and green > blue + 20
                )
                if warm_pixels >= 30:
                    break
                time.sleep(.25)
            assert warm_pixels >= 30, (
                f"G-05.07 stopa není vidět na WebGL canvasu; warm_pixels={warm_pixels}")
            assert canvas.get_attribute("data-g05-mounted") == "once", (
                "MapLibre remount při vytvoření trailu")
            print(f"PASS G-05.07: viditelná skutečná žlutá stopa, warm_pixels={warm_pixels}")
            assert canvas.get_attribute("data-g05-mounted") == "once", "MapLibre remount během SSE"
            # G-05.08: follow se musí vypnout při skutečném kolečku myši.
            page.get_by_role("button", name="Sledovat na mapě").click()
            expect(page.get_by_role("button", name="Ukončit sledování")).to_have_attribute(
                "aria-pressed", "true")
            location = canvas.bounding_box()
            assert location is not None
            page.mouse.move(location["x"] + location["width"] / 2,
                            location["y"] + location["height"] / 2)
            page.mouse.wheel(0, 150)
            expect(page.get_by_role("button", name="Sledovat na mapě")).to_have_attribute(
                "aria-pressed", "false")
            # Klávesnice a dotykový pohyb musejí také ukončit follow.
            page.get_by_role("button", name="Sledovat na mapě").click()
            canvas.focus()
            page.keyboard.press("ArrowLeft")
            expect(page.get_by_role("button", name="Sledovat na mapě")).to_have_attribute(
                "aria-pressed", "false")
            page.get_by_role("button", name="Sledovat na mapě").click()
            canvas.evaluate("""el => el.parentElement.dispatchEvent(
                new Event('touchmove', {bubbles: true, cancelable: true})
            )""")
            expect(page.get_by_role("button", name="Sledovat na mapě")).to_have_attribute(
                "aria-pressed", "false")
            assert canvas.get_attribute("data-g05-mounted") == "once", "MapLibre remount během ovládání"
            # G-05.09: přepnutí stylu nesmaže výběr ani MapLibre canvas.
            page.get_by_role("button", name="Vrstvy").click()
            expect(page.get_by_role("complementary", name="Nastavení mapy")).to_be_visible()
            page.get_by_label("Mapový podklad").select_option("liberty")
            expect(canvas).to_be_visible()
            expect(page.get_by_role("heading", name="OKTEST")).to_be_visible()
            page.get_by_role("button", name="Zavřít detail letadla").click()
            expect(page.get_by_role("complementary", name="Vybrané letadlo")).to_have_count(0)
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_have_count(0)
            expect(page.get_by_role("button", name="Letadla", exact=True)).to_be_focused()
            assert canvas.get_attribute("data-g05-mounted") == "once", "Mapa nesmí remountovat při zavření detailu"
            page.get_by_role("button", name="Letadla", exact=True).click()
            page.get_by_label("Hledat volací znak nebo ICAO").fill("neexistuje")
            expect(page.get_by_text("Žádná odpovídající letadla.")).to_be_visible()
            page.get_by_role("button", name="Zavřít seznam").click()
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_have_count(0)
            # G-05.10: nový izolovaný kontext = počáteční střed mapy
            # 17.67/49.23. Nepoužívat kameru z dřívějšího follow testu.
            airport_page = browser.new_page(viewport={"width": 1440, "height": 900})
            airport_page.add_init_script(MOCK)
            airport_page.route("https://tiles.openfreemap.org/**",
                               lambda route: route.fulfill(
                                   status=200, content_type="application/json",
                                   body=json.dumps(STYLE)))
            airport_page.route("**/data/airports-ourairports.json",
                               lambda route: route.fulfill(
                                   status=200, content_type="application/json",
                                   body=json.dumps(AIRPORTS)))
            airport_page.goto(URL)
            airport_canvas = airport_page.locator(".maplibregl-canvas")
            expect(airport_canvas).to_be_visible(timeout=12000)
            airport_canvas.evaluate("el => { el.dataset.g05Mounted = 'once'; }")
            airport_page.get_by_role("button", name="Vrstvy").click()
            airport_toggle = airport_page.get_by_label("Letiště", exact=True)
            expect(airport_toggle).to_be_enabled(timeout=12000)
            airport_toggle.uncheck()
            airport_toggle.check()
            # Není jisté, že worker stihl vytvořit GeoJSON symbol ve stejném
            # ticku jako React dokončil fetch, proto aktivně čekáme na WebGL pixel.
            bounds = airport_canvas.bounding_box()
            assert bounds is not None
            mid_x = bounds["x"] + bounds["width"] / 2
            mid_y = bounds["y"] + bounds["height"] / 2
            matched = False
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                image = Image.open(BytesIO(airport_canvas.screenshot())).convert("RGB")
                center = image.getpixel((int(bounds["width"] / 2), int(bounds["height"] / 2)))
                if center[0] > 155 and center[1] > 85 and center[2] < 120:
                    matched = True
                    break
                time.sleep(.2)
            assert matched, "G-05.10: bod letiště v centru WebGL nebyl vykreslen"
            airport_page.mouse.click(mid_x, mid_y)
            expect(airport_page.get_by_role("complementary", name="Vybrané letiště")).to_be_visible(
                timeout=12000)
            expect(airport_page.get_by_role("heading", name="Testovací letiště CZ-TEST1")).to_be_visible()
            airport_page.get_by_role("button", name="Vrstvy").click()
            airport_page.get_by_label("Mapový podklad").select_option("liberty")
            expect(airport_page.get_by_role("heading", name="Testovací letiště CZ-TEST1")).to_be_visible()
            assert airport_canvas.get_attribute("data-g05-mounted") == "once"
            airport_page.get_by_role("button", name="Zavřít detail letiště").click()
            expect(airport_page.get_by_role("complementary", name="Vybrané letiště")).to_have_count(0)
            airport_page.close()

            # Základní mobilní geometrie G-05 (plná a11y/GPU přejímka patří G-10).
            mobile = browser.new_page(viewport={"width": 390, "height": 844},
                                      is_mobile=True, has_touch=True)
            mobile.add_init_script(MOCK)
            mobile.route("https://tiles.openfreemap.org/**",
                         lambda route: route.fulfill(
                             status=200, content_type="application/json",
                             body=json.dumps(STYLE)))
            mobile.route("**/data/airports-ourairports.json",
                         lambda route: route.fulfill(
                             status=200, content_type="application/json",
                             body=json.dumps(AIRPORTS)))
            mobile.goto(URL)
            expect(mobile.locator(".maplibregl-canvas")).to_be_visible(timeout=12000)
            assert mobile.evaluate("document.documentElement.scrollWidth <= innerWidth"), (
                "Mobil má horizontální scroll")
            mobile.get_by_role("button", name="Vrstvy").click()
            expect(mobile.get_by_role("complementary", name="Nastavení mapy")).to_be_visible()
            assert mobile.evaluate("document.documentElement.scrollWidth <= innerWidth"), (
                "Mobilní vrstvy přetékají")
            mobile.get_by_role("button", name="Zavřít vrstvy").click()
            expect(mobile.get_by_role("button", name="Vrstvy")).to_be_focused()
            mobile.get_by_role("button", name="Letadla", exact=True).click()
            expect(mobile.get_by_role("complementary", name="Seznam letadel")).to_be_visible()
            assert mobile.evaluate("document.documentElement.scrollWidth <= innerWidth"), (
                "Mobilní panel přetéká")
            mobile.get_by_role("button", name="Zavřít seznam").click()
            expect(mobile.get_by_role("button", name="Letadla", exact=True)).to_be_focused()
            expect(mobile.locator(".maplibregl-canvas")).to_be_visible()
            mobile.close()
            # G-05.03/.11: Chromium smoke s 2 000 kategoriovanými cíli a skutečným
            # MapLibre canvasem. Jde o funkční zátěžový smoke SwiftShader,
            # NIKOLIV referenční FPS/GPU benchmark desktopu nebo telefonu.
            page.evaluate("""() => {
              const now = Date.now() / 1000;
              const aircraft = Array.from({length: 2000}, (_, i) => ({
                id: 'icao:' + i.toString(16).padStart(6, '0'),
                lastMessageAt: now, lastPositionAt: now,
                position: {lat: 48.5 + (i % 67) * .01,
                           lon: 14.5 + (i % 97) * .01},
                positionOrigin: 'adsb', stale: false,
                callsign: 'C' + i.toString().padStart(5, '0'),
                squawk: null,
                emitterCategory: i % 2 ? 'A7' : 'B1',
                telemetry: {trackDegrees: i % 360, altitudeBaroFt: 5000 + i}
              }));
              window.__emitRadar('snapshot', {
                kind: 'snapshot', epoch: 'g05-dense', seq: 0,
                generatedAt: new Date().toISOString(), aircraft
              });
            }""")
            expect(page.get_by_label("Počet zachycených letadel")).to_contain_text(
                "000 letadel", timeout=20000)
            assert canvas.get_attribute("data-g05-mounted") == "once", (
                "MapLibre remount při hustém SSE snapshotu")
            # G-05.11: 10 % změn ze stejné SSE epochy → inkrementální
            # updateData nad 2 000 stabilními identitami; bez remountu.
            page.evaluate("""() => {
              const now = Date.now()/1000;
              const changed = Array.from({length:200},(_,i)=>{
                const idx = i*10;
                return {
                  id:'icao:'+idx.toString(16).padStart(6,'0'),
                  lastMessageAt:now,lastPositionAt:now,
                  position:{lat:48.6+(idx%67)*.01,lon:14.6+(idx%97)*.01},
                  positionOrigin:'adsb',stale:false,
                  callsign:'C'+idx.toString().padStart(5,'0'),
                  squawk:null,emitterCategory:idx%2?'A7':'B1',
                  telemetry:{trackDegrees:idx%360,altitudeBaroFt:5000+idx}
                };
              });
              window.__emitRadar('delta',{
                kind:'delta',epoch:'g05-dense',seq:1,
                generatedAt:new Date().toISOString(),upsert:changed,removed:[]
              });
            }""")
            expect(page.get_by_label("Počet zachycených letadel")).to_contain_text(
                "000 letadel", timeout=12000)
            assert canvas.get_attribute("data-g05-mounted") == "once", (
                "MapLibre remount po 200 změněných identitách")

            # Style reload musí obnovit celý sprite registr a zachovat mapu.
            page.get_by_role("button", name="Vrstvy").click()
            page.get_by_label("Mapový podklad").select_option("fiord")
            expect(canvas).to_be_visible()
            page.get_by_label("Mapový podklad").select_option("liberty")
            expect(canvas).to_be_visible()
            assert canvas.get_attribute("data-g05-mounted") == "once", (
                "MapLibre remount po 2 000 cílech a změně podkladu")
            # Audit A07: po krátké chybě podkladu musí následné style.load
            # odstranit starý chybový banner a nerozpojit živé SSE.
            # Benchmark query povolí kontrolovaný přístup k již existující
            # instanci MapLibre pouze na 127.0.0.1, bez veřejného API.
            recovery = browser.new_page(viewport={"width": 1370, "height": 850})
            recovery.add_init_script(MOCK)
            recovery.route("https://tiles.openfreemap.org/**",
                           lambda route: route.fulfill(
                               status=200, content_type="application/json",
                               body=json.dumps(STYLE)))
            recovery.route("**/data/airports-ourairports.json",
                           lambda route: route.fulfill(
                               status=200, content_type="application/json",
                               body=json.dumps(AIRPORTS)))
            recovery.goto(URL + "?g05-benchmark")
            recovery.wait_for_function(
                "window.__g05BenchMap && window.__g05BenchMap.isStyleLoaded()",
                timeout=12000)
            assert recovery.evaluate("""() => {
              const map = window.__g05BenchMap;
              return ['generic-fixedwing', 'rotorcraft'].every(icon =>
                map.hasImage('radar-' + icon) &&
                map.hasImage('radar-selected-' + icon)
              );
            }"""), "Vybrané siluety musí být v MapLibre sprite registru po style.load"
            recovery.evaluate("""() =>
                window.__g05BenchMap.fire('error', {error: new Error('synthetic tile error')})
            """)
            expect(recovery.locator(".map-error")).to_have_count(0)
            recovery.evaluate("() => window.__g05BenchMap.fire('style.load')")
            expect(recovery.locator(".maplibregl-canvas")).to_be_visible()
            recovery.evaluate("""() => {
              for (let i = 0; i < 3; i++) {
                window.__g05BenchMap.fire('error', {error: new Error('repeated tile error')});
              }
            }""")
            # Ani opakovaný výpadek částí stylu nesmí vytvořit banner:
            # vnější chyba není důkazem nedostupné geografické mapy.
            expect(recovery.locator(".map-error")).to_have_count(0)
            assert recovery.evaluate("""() => {
              const map = window.__g05BenchMap;
              return !map.getSource('radar-osm-backup') &&
                !!map.getSource('radar-aircraft') &&
                !!map.getLayer('radar-aircraft-symbols');
            }"""), "Dílčí chyby nesmějí skrýt mapu ani letadla"
            recovery.close()

            # Úplný výpadek externího stylu nesmí skrýt radar ani rozkmitat canvas.
            offline = browser.new_page(viewport={"width": 1370, "height": 850})
            offline.add_init_script(MOCK)
            offline.route("https://tiles.openfreemap.org/**", lambda route: route.abort())
            offline.route("**/data/airports-ourairports.json",
                          lambda route: route.fulfill(
                              status=200, content_type="application/json",
                              body=json.dumps(AIRPORTS)))
            offline.goto(URL + "?g05-benchmark")
            expect(offline.get_by_text("2 letadla")).to_be_visible(timeout=12000)
            expect(offline.locator(".maplibregl-canvas")).to_be_visible(timeout=12000)
            offline.wait_for_function(
                "window.__g05BenchMap && !!window.__g05BenchMap.getSource('radar-osm-backup')",
                timeout=12000)
            expect(offline.locator(".maplibregl-canvas")).to_be_visible(timeout=12000)
            offline.close()
            # Vizuální evidence se ukládá zvlášť: čistá fixture není živý
            # provoz ani fyzický GPU benchmark. Screenshot slouží jako
            # reprodukovatelná layoutová baseline.
            artifacts = Path(".artifacts")
            artifacts.mkdir(exist_ok=True)
            page.get_by_role("button", name="Zavřít vrstvy").click()
            page.screenshot(path=str(artifacts / "g05-i1-desktop.png"), full_page=True)
            capture_mobile = browser.new_page(viewport={"width": 390, "height": 844},
                                              is_mobile=True, has_touch=True)
            capture_mobile.add_init_script(MOCK)
            capture_mobile.route("https://tiles.openfreemap.org/**",
                                 lambda route: route.fulfill(
                                     status=200, content_type="application/json",
                                     body=json.dumps(STYLE)))
            capture_mobile.route("**/data/airports-ourairports.json",
                                 lambda route: route.fulfill(
                                     status=200, content_type="application/json",
                                     body=json.dumps(AIRPORTS)))
            capture_mobile.goto(URL)
            expect(capture_mobile.locator(".maplibregl-canvas")).to_be_visible()
            capture_mobile.screenshot(path=str(artifacts / "g05-i1-mobile.png"),
                                      full_page=True)
            capture_mobile.close()
            # Read-only porovnání s veřejným DEV V2 před merge. Síťový
            # neúspěch nepředstírat jako screenshot ani jako vizuální PASS.
            public_result = {"url": "https://airradarv2.pomykal.cz",
                             "status": "NEOVĚŘENO"}
            public_page = browser.new_page(viewport={"width": 1370, "height": 850})
            try:
                response = public_page.goto(public_result["url"],
                                            wait_until="domcontentloaded", timeout=12000)
                if response and response.status == 200:
                    expect(public_page.locator(".maplibregl-canvas")).to_be_visible(
                        timeout=10000)
                    public_result["status"] = "DOSTUPNÉ"
                    public_result["http"] = response.status
                    public_result["dimensions"] = public_page.evaluate("""() => ({
                      viewportHeight: innerHeight,
                      mapHeight: document.querySelector('.maplibregl-canvas')
                        .getBoundingClientRect().height,
                      headerHeight: document.querySelector('.radar-header')
                        .getBoundingClientRect().height,
                      footerPresent: !!document.querySelector('.radar-footer')
                    })""")
                    public_page.screenshot(path=str(artifacts / "g05-public-before-merge.png"))
                else:
                    public_result["reason"] = f"HTTP {response.status if response else 'žádná odpověď'}"
            except Exception as exc:
                public_result["reason"] = str(exc)[:350]
            finally:
                public_page.close()
            (artifacts / "g05-public-comparison.json").write_text(
                json.dumps(public_result, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8")
            print("G-05 public preview: " + json.dumps(public_result, ensure_ascii=False))
            browser.close()
        print("PASS G-05 Chromium: kompaktní UI bez nadbytečných overlayů, 2 000 SSE cílů, funkční vrstvy a tichý partial-tile fallback (SwiftShader smoke, ne FPS)")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

if __name__ == "__main__":
    main()
