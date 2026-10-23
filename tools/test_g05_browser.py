#!/usr/bin/env python3
"""G-05.01/.02/.05/.06 browser smoke bez reálného RF/DB/tile serveru."""
import json
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
            expect(page.get_by_role("heading", name="Živý letecký provoz")).to_be_visible()
            expect(page.get_by_text("2 letadel")).to_be_visible(timeout=12000)
            canvas = page.locator(".maplibregl-canvas")
            expect(canvas).to_be_visible(timeout=12000)
            canvas.evaluate("(el) => { el.dataset.g05Mounted = 'once'; }")
            page.get_by_role("button", name="OKTWO").click()
            expect(page.get_by_role("heading", name="OKTWO")).to_be_visible()
            page.evaluate("window.__emitRadar('delta', window.__radarFixture.next)")
            expect(page.get_by_text("1 letadel")).to_be_visible(timeout=12000)
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
            expect(page.get_by_text("2 letadel")).to_be_visible(timeout=12000)
            expect(page.get_by_role("heading", name="OKTWO")).to_have_count(0)
            page.get_by_role("button", name="OKTEST").click()
            expect(page.get_by_role("heading", name="OKTEST")).to_be_visible()
            assert canvas.get_attribute("data-g05-mounted") == "once", "MapLibre remount během SSE"
            page.get_by_label("Mapový podklad").select_option("fiord")
            expect(canvas).to_be_visible()
            page.get_by_label("Hledat volací znak nebo ICAO").fill("neexistuje")
            expect(page.get_by_text("Žádná odpovídající letadla.")).to_be_visible()
            browser.close()
        print("PASS G-05 Chromium: bez RF/DB, stabilní MapLibre canvas, snapshot/delta, odstranění, filtr, výběr a styl")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

if __name__ == "__main__":
    main()
