#!/usr/bin/env python3
"""G-03.08: skutečný Chromium EventSource přes Vite a Nginx; bez V1.

Izolované CI: Rust --g03-live-api z lokálního fixture, Vite same-origin proxy,
Nginx same-origin proxy. Skutečný prohlížeč musí přijmout initial snapshot,
pozdější delta ještě během otevřeného streamu a reconnect full snapshot.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.request

from playwright.sync_api import sync_playwright, TimeoutError as BrowserTimeout
from benchmark_g03_sse import write_fixture


def health(url: str) -> bool:
    try:
        with urllib.request.urlopen(url, timeout=1) as response:
            return response.status == 200
    except (OSError, ValueError):
        return False


def ready(url: str) -> None:
    for _ in range(120):
        if health(url):
            return
        time.sleep(0.2)
    raise AssertionError(f"Server se nespustil: {url}")


JS_INSTALL = """() => {
    window.__g03 = { messages: [], errors: 0 };
    window.__g03.conn = new EventSource("/api/v1/stream");
    for (const type of ["snapshot", "delta", "unavailable"]) {
        window.__g03.conn.addEventListener(type, event => {
            const payload = JSON.parse(event.data);
            window.__g03.messages.push({ type, payload, id: event.lastEventId, at: performance.now() });
        });
    }
    window.__g03.conn.onerror = () => { window.__g03.errors++; };
}"""


def browser_case(browser, origin: str, fixture: Path, step: int) -> dict:
    page = browser.new_page()
    try:
        page.goto(origin, wait_until="domcontentloaded", timeout=20000)
        page.evaluate(JS_INSTALL)
        page.wait_for_function(
            "() => window.__g03?.messages?.some(x => x.type === 'snapshot')",
            timeout=20000,
        )
        first = page.evaluate("() => window.__g03.messages[0]")
        assert first["type"] == "snapshot"
        assert first["payload"]["kind"] == "snapshot"
        assert first["id"] == f'{first["payload"]["epoch"]}:{first["payload"]["seq"]}'
        assert len(first["payload"]["aircraft"]) == 25
        assert "receiverId" not in json.dumps(first)
        before_seq = first["payload"]["seq"]

        # Po počátečním snapshotu zůstane EventSource otevřený; nová data
        # musí projít přes tentýž proxy stream bez čekání na ukončení odpovědi.
        write_fixture(fixture, 25, step)
        page.wait_for_function(
            "(oldSeq) => window.__g03.messages.some(x => x.type === 'delta' "
            "&& x.payload.seq > oldSeq)", arg=before_seq, timeout=15000
        )
        latest = page.evaluate("() => window.__g03.messages.filter(x => x.type === 'delta').at(-1)")
        assert latest["payload"]["epoch"] == first["payload"]["epoch"]
        assert latest["id"] == f'{latest["payload"]["epoch"]}:{latest["payload"]["seq"]}'
        assert latest["payload"]["upsert"], "Po změně JSON očekáváme upsert"
        assert not page.evaluate("() => window.__g03.errors"), "EventSource se neočekávaně rozpojil"

        # Last-Event-ID není serverový replay; nové spojení začne full snapshotem.
        page.evaluate("() => { window.__g03.conn.close(); window.__g03.messages=[]; }")
        page.evaluate(JS_INSTALL)
        page.wait_for_function(
            "() => window.__g03.messages.some(x => x.type === 'snapshot')", timeout=15000
        )
        reconnect = page.evaluate("() => window.__g03.messages[0]")
        assert reconnect["payload"]["epoch"] == first["payload"]["epoch"]
        assert reconnect["payload"]["seq"] >= latest["payload"]["seq"]
        assert reconnect["type"] == "snapshot"
        page.evaluate("() => window.__g03.conn.close()")
        return {
            "origin": origin,
            "snapshot_seq": before_seq,
            "delta_seq": latest["payload"]["seq"],
            "reconnect_seq": reconnect["payload"]["seq"],
            "browser": browser.browser_type.name,
            "result": "PASS",
        }
    finally:
        page.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--web-dir", default="apps/web")
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    root = Path(args.web_dir).resolve()
    assert binary.is_file()
    assert (root / "package.json").is_file()
    with tempfile.TemporaryDirectory(prefix="airradarv2-g03-browser-") as tmp:
        fixture = Path(tmp) / "aircraft.json"
        write_fixture(fixture, 25, 1)
        env = dict(os.environ)
        env["AIRRADAR_V2_BIND"] = "127.0.0.1:4102"
        env["AIRRADAR_V2_READSB_JSON"] = str(fixture)
        env.pop("AIRRADAR_V2_BEAST_ADDR", None)
        engine = subprocess.Popen(
            [str(binary), "--g03-live-api"], env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
            stdin=subprocess.DEVNULL,
        )
        vite = subprocess.Popen(
            ["npm", "run", "dev"], cwd=root,
            env={**env, "AIRRADAR_V2_G03_API": "1"},
            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
            stdin=subprocess.DEVNULL,
        )
        try:
            ready("http://127.0.0.1:4102/api/v1/health")
            ready("http://127.0.0.1:4100/")
            ready("http://127.0.0.1:8082/")
            # Ověř skutečné publikování do 25 letadel.
            for _ in range(45):
                with urllib.request.urlopen("http://127.0.0.1:4102/api/v1/aircraft", timeout=3) as r:
                    state = json.load(r)["data"]
                if len(state["aircraft"]) == 25:
                    break
                time.sleep(0.2)
            else:
                raise AssertionError("Rust RAM nevytvořil 25 testovacích letadel")
            with sync_playwright() as p:
                browser = p.chromium.launch(channel="chrome", headless=True, args=["--no-sandbox"])
                try:
                    results = [
                        browser_case(browser, "http://127.0.0.1:4100/", fixture, 2),
                        browser_case(browser, "http://127.0.0.1:8082/", fixture, 3),
                    ]
                    for result in results:
                        print("G03_BROWSER_EVENTSOURCE=" + json.dumps(result, sort_keys=True), flush=True)
                finally:
                    browser.close()
        except BrowserTimeout as e:
            raise AssertionError(f"Chromium nedostal SSE event včas: {e}") from e
        finally:
            for proc in (vite, engine):
                proc.terminate()
            for proc in (vite, engine):
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
    print("PASS G-03.08: Chromium EventSource snapshot/delta/reconnect přes Vite i Nginx")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
