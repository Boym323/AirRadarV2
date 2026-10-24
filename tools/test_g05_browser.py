#!/usr/bin/env python3
"""G-05.01–.09 browser smoke bez reálného RF/DB/tile serveru."""
import json
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
            page.goto(URL)
            expect(page.locator("html")).to_have_attribute("lang", "cs")
            expect(page.get_by_role("heading", name="AirRadar V2")).to_be_visible()
            expect(page.get_by_text("2 letadla")).to_be_visible(timeout=12000)
            canvas = page.locator(".maplibregl-canvas")
            expect(canvas).to_be_visible(timeout=12000)
            canvas.evaluate("(el) => { el.dataset.g05Mounted = 'once'; }")
            # G-05 R01/R06: mapa zůstává dominantní, seznam je na vyžádání.
            expect(page.get_by_role("complementary", name="Seznam letadel")).to_have_count(0)
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
            # Základní mobilní geometrie G-05 (plná a11y/GPU přejímka patří G-10).
            mobile = browser.new_page(viewport={"width": 390, "height": 844},
                                      is_mobile=True, has_touch=True)
            mobile.add_init_script(MOCK)
            mobile.route("https://tiles.openfreemap.org/**",
                         lambda route: route.fulfill(
                             status=200, content_type="application/json",
                             body=json.dumps(STYLE)))
            mobile.goto(URL)
            expect(mobile.locator(".maplibregl-canvas")).to_be_visible(timeout=12000)
            assert mobile.evaluate("document.documentElement.scrollWidth <= innerWidth"), (
                "Mobil má horizontální scroll")
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
            # Style reload musí obnovit celý sprite registr a zachovat mapu.
            page.get_by_label("Mapový podklad").select_option("fiord")
            expect(canvas).to_be_visible()
            page.get_by_label("Mapový podklad").select_option("liberty")
            expect(canvas).to_be_visible()
            assert canvas.get_attribute("data-g05-mounted") == "once", (
                "MapLibre remount po 2 000 cílech a změně podkladu")
            browser.close()
        print("PASS G-05 Chromium: map-first desktop/mobil, panely, focus, MapLibre/SSE, kategorie a 2 000 synthetic targets (SwiftShader smoke, ne FPS)")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

if __name__ == "__main__":
    main()
