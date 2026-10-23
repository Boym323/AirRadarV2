#!/usr/bin/env python3
"""G-03.08/G-04 — přísná lokální DEV přejímka HTTP/SSE runtime stage.

Spouští výhradně schválený root-owned DEV installer/Runner, nikoli veřejné API.
Nezaměnit úspěšné HTTP 200 bootstrap G-00 za aktivní G-03.
"""
import argparse
import json
import urllib.request

MAX_EVENT_BYTES = 1024 * 1024 + 8192


def validate_health(data: object, expected_stage: str = "G-03") -> None:
    if not isinstance(data, dict):
        raise ValueError("Health musí být JSON objekt")
    if data.get("stage") != expected_stage or data.get("status") not in (
        "live", "waiting_for_observations"
    ):
        raise ValueError(f"{expected_stage} není aktivní nebo je degradovaná; bootstrap není přejímka")
    if not isinstance(data.get("live_ingest"), bool):
        raise ValueError("Health live_ingest není boolean")
    if data["live_ingest"] != (data["status"] == "live"):
        raise ValueError("Nekonzistentní health status/live_ingest")
    expected_database = expected_stage == "G-04"
    if data.get("database_connected") is not expected_database:
        raise ValueError("Health DB příznak neodpovídá runtime stage")


def validate_snapshot(data: object) -> None:
    if not isinstance(data, dict) or data.get("kind") != "snapshot":
        raise ValueError("SSE nepředal skutečný G-03 snapshot")
    if not isinstance(data.get("epoch"), str) or not data["epoch"]:
        raise ValueError("Snapshot epoch chybí")
    if isinstance(data.get("seq"), bool) or not isinstance(data.get("seq"), int) or data["seq"] < 0:
        raise ValueError("Snapshot seq není monotónní nezáporné číslo")
    if not isinstance(data.get("aircraft"), list) or len(data["aircraft"]) > 5000:
        raise ValueError("Snapshot překročil limit 5000 letadel")
    if not isinstance(data.get("generatedAt"), str) or not data["generatedAt"].endswith("Z"):
        raise ValueError("Chybí UTC čas snapshotu")


def parse_first_event(raw: bytes) -> dict:
    if len(raw) > MAX_EVENT_BYTES:
        raise ValueError("SSE úvodní frame je příliš velký")
    event = None
    event_id = None
    data = []
    for line in raw.decode("utf-8", "strict").splitlines():
        if line.startswith("event: "):
            event = line[7:]
        elif line.startswith("id: "):
            event_id = line[4:]
        elif line.startswith("data: "):
            data.append(line[6:])
    if event != "snapshot" or not data:
        raise ValueError("Po spojení nepřišel full snapshot; nelze prohlásit G-03 za hotové")
    payload = json.loads("\n".join(data))
    validate_snapshot(payload)
    if event_id != f"{payload['epoch']}:{payload['seq']}":
        raise ValueError("SSE id, epoch a sequence nejsou atomické")
    return payload


def json_get(url: str) -> dict:
    request = urllib.request.Request(url, headers={"Accept": "application/json"})
    with urllib.request.urlopen(request, timeout=5) as response:
        if response.status != 200:
            raise ValueError(f"{url}: HTTP {response.status}")
        if "application/json" not in response.headers.get("Content-Type", ""):
            raise ValueError(f"{url}: chybí application/json")
        body = response.read(MAX_EVENT_BYTES + 1)
    if len(body) > MAX_EVENT_BYTES:
        raise ValueError(f"{url}: příliš velká HTTP odpověď")
    return json.loads(body)


def first_stream_snapshot(url: str, require_accel: bool, require_heartbeat: bool = False) -> dict:
    request = urllib.request.Request(url, headers={"Accept": "text/event-stream"})
    # 15s is the contract heartbeat interval; allow a full interval plus
    # scheduling/proxy jitter so a busy DEV LXC is not called broken at 20s.
    with urllib.request.urlopen(request, timeout=40 if require_heartbeat else 7) as response:
        if response.status != 200:
            raise ValueError(f"{url}: SSE HTTP {response.status}")
        if not response.headers.get("Content-Type", "").startswith("text/event-stream"):
            raise ValueError(f"{url}: chybí text/event-stream")
        if "no-transform" not in response.headers.get("Cache-Control", ""):
            raise ValueError(f"{url}: chybí no-transform")
        if require_accel and response.headers.get("X-Accel-Buffering", "").lower() != "no":
            raise ValueError(f"{url}: server nevypnul SSE buffering")
        frame = bytearray()
        snapshot = None
        while len(frame) < MAX_EVENT_BYTES:
            line = response.readline(MAX_EVENT_BYTES - len(frame) + 1)
            if not line:
                raise ValueError(f"{url}: SSE předčasně skončilo")
            if len(line) > MAX_EVENT_BYTES:
                raise ValueError(f"{url}: SSE line překročila limit")
            if snapshot is not None and line.startswith(b": heartbeat"):
                return snapshot
            frame.extend(line)
            if line in (b"\n", b"\r\n"):
                if b"event: snapshot" in frame or b"event: unavailable" in frame:
                    snapshot = parse_first_event(bytes(frame))
                    if not require_heartbeat:
                        return snapshot
                frame.clear()
        raise ValueError(f"{url}: SSE frame překročil bezpečný limit nebo heartbeat nedorazil")


def verify(direct: str, proxy: str, expected_stage: str = "G-03") -> dict:
    direct = direct.rstrip("/")
    proxy = proxy.rstrip("/")
    if not direct.startswith("http://127.0.0.1:"):
        raise ValueError("Direct API musí být DEV loopback")
    if not proxy.startswith("http://"):
        raise ValueError("Proxy musí být výhradně interní DEV HTTP")
    health = json_get(direct + "/api/v1/health")
    validate_health(health, expected_stage)
    aircraft = json_get(direct + "/api/v1/aircraft")
    if aircraft.get("status") not in ("live", "waiting_for_observations"):
        raise ValueError("Aircraft API není aktivní")
    validate_snapshot(aircraft.get("data"))
    stream = first_stream_snapshot(direct + "/api/v1/stream", require_accel=True)
    proxied = first_stream_snapshot(proxy + "/api/v1/stream", require_accel=False, require_heartbeat=True)
    if stream["epoch"] != aircraft["data"]["epoch"] or proxied["epoch"] != stream["epoch"]:
        raise ValueError("Nginx SSE nepoužívá stejný Rust epoch")
    return {"stage": health["stage"], "status": health["status"], "seq": stream["seq"],
            "proxySeq": proxied["seq"], "aircraft": len(stream["aircraft"])}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--direct", default="http://127.0.0.1:4101")
    parser.add_argument("--proxy", required=True)
    parser.add_argument("--expected-stage", choices=("G-03", "G-04"), default="G-03")
    args = parser.parse_args()
    result = verify(args.direct, args.proxy, args.expected_stage)
    print(f"PASS {args.expected_stage} DEV direct HTTP/SSE + Nginx no-buffer full snapshot: " +
          json.dumps(result, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
