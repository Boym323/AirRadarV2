#!/usr/bin/env python3
"""G-05: read-only vizuální přejímka veřejného náhledu V2 po DEV releasu.

Měří dostupnost a layout, pořizuje dvě skutečné screenshoty ze sítě runneru.
Bez fixture/live API mutací. Nejde o fyzické GPU měření.
"""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright

URL = os.getenv("G05_PUBLIC_URL", "https://airradarv2.pomykal.cz")
SHA = os.getenv("G05_EXPECTED_SHA", "NEZNÁMÉ")
OUTPUT = Path(".artifacts/g05-public-after-release")
OUTPUT.mkdir(parents=True, exist_ok=True)


def inspect(browser, name, width, height, mobile=False):
    page = browser.new_page(viewport={"width": width, "height": height},
                            is_mobile=mobile, has_touch=mobile)
    try:
        response = page.goto(URL, wait_until="domcontentloaded", timeout=30000)
        if response is None or response.status != 200:
            raise AssertionError(f"HTTP {response.status if response else 'bez odpovědi'}")
        page.locator(".maplibregl-canvas").wait_for(state="visible", timeout=18000)
        page.get_by_role("heading", name="AirRadar", exact=True).wait_for(timeout=10000)
        page.get_by_role("button", name="Vrstvy").wait_for(timeout=10000)
        # Počkáme na první stabilní render, ne na celé dlouhotrvající síťové idle.
        page.wait_for_timeout(2200)
        state = page.evaluate("""() => {
          const map = document.querySelector(".maplibregl-canvas").getBoundingClientRect();
          const header = document.querySelector(".radar-header").getBoundingClientRect();
          const body = document.querySelector("body").getBoundingClientRect();
          return {
            viewport: {width: innerWidth, height: innerHeight},
            map: {width: map.width, height: map.height},
            header: {top: header.top, height: header.height},
            noHorizontalScroll: document.documentElement.scrollWidth <= innerWidth,
            hasFooter: !!document.querySelector(".radar-footer"),
            hasMapAttribution: !!document.querySelector(".maplibregl-ctrl-attrib"),
            title: document.title,
            bodyHeight: body.height,
            trafficStatus: document.querySelector(".connection")?.textContent?.trim() ?? ""
          };
        }""")
        if not state["noHorizontalScroll"]:
            raise AssertionError(f"{name}: horizontální přetékání")
        if state["hasFooter"]:
            raise AssertionError(f"{name}: zbytečná technická patička")
        if not state["hasMapAttribution"]:
            raise AssertionError(f"{name}: chybí MapLibre atribuce")
        if state["map"]["height"] < height - 3:
            raise AssertionError(f"{name}: mapa není fullscreen: {state['map']['height']} / {height}")
        page.screenshot(path=str(OUTPUT / f"{name}.png"), full_page=True)
        return {"name": name, "status": "PASS", **state}
    finally:
        page.close()


def main():
    report = {"url": URL, "expectedCommit": SHA, "kind": "public V2 DEV (nikoli V1 produkce)",
              "checks": [], "gpuBenchmark": False}
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(
                channel="chrome", headless=True,
                args=["--enable-webgl", "--use-gl=angle", "--use-angle=swiftshader",
                      "--enable-unsafe-swiftshader"])
            try:
                report["checks"].append(inspect(browser, "desktop-1440x900", 1440, 900))
                report["checks"].append(inspect(browser, "mobile-390x844", 390, 844, True))
            finally:
                browser.close()
        report["result"] = "PASS"
    except Exception as exc:
        report["result"] = "NEOVĚŘENO/FAIL"
        report["error"] = str(exc)[:800]
        raise
    finally:
        (OUTPUT / "evidence.json").write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
