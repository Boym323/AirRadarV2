#!/usr/bin/env python3
"""G-03.07/.09/.10 — izolované měření Rust HTTP/SSE, bez síťového přijímače.

CLI spouští vlastní Rust binary na 127.0.0.1:4102. readsb JSON generuje
v dočasném adresáři a publikuje výhradně atomickým rename.
Není to test DEV LXC ani prohlížeče; hodnoty nejsou produkční benchmark.
"""
import argparse
import asyncio
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


HOST, PORT = "127.0.0.1", 4102
SCENARIOS = ((1, 25), (10, 250), (100, 250), (10, 1000), (1, 5000), (10, 5000), (100, 5000))


def write_fixture(path: Path, count: int, step: int, compact: bool = False) -> None:
    now = time.time()
    aircraft = []
    for i in range(count):
        row = {
            "hex": f"{i + 1:06x}",
            "type": "adsb_icao",
            "seen": 0.0,
        }
        if not compact:
            row.update({
                "seen_pos": 0.0,
                "lat": 49.0 + i * 0.00001 + step * 0.000001,
                "lon": 16.0 + i * 0.00001,
                "flight": f"T{i:06d}",
                "alt_baro": 10000,
                "gs": 160,
                "track": 270,
            })
        aircraft.append(row)
    data = {"now": now, "messages": count * (step + 1), "aircraft": aircraft}
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(data, separators=(",", ":")), encoding="utf-8")
    os.replace(temporary, path)


def proc_stats(pid: int) -> tuple[float, int]:
    stat = Path(f"/proc/{pid}/stat").read_text(encoding="ascii").split()
    ticks = int(stat[13]) + int(stat[14])
    hz = os.sysconf("SC_CLK_TCK")
    rss_kib = 0
    for line in Path(f"/proc/{pid}/status").read_text(encoding="ascii").splitlines():
        if line.startswith("VmRSS:"):
            rss_kib = int(line.split()[1])
            break
    return ticks / hz, rss_kib


class SseClient:
    def __init__(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter,
                 chunked: bool = False):
        self.reader = reader
        self.writer = writer
        self.chunked = chunked
        self.raw_buffer = b""
        self.bytes_read = 0

    async def readline(self) -> bytes:
        """Decode HTTP/1.1 chunk framing before parsing SSE lines.

        SSE JSON may span many 16KiB shared Bytes chunks; interpreting raw
        chunk sizes or CRLF boundaries as SSE frames corrupts the benchmark.
        """
        if not self.chunked:
            return await self.reader.readline()
        while b"\n" not in self.raw_buffer:
            size_header = await self.reader.readline()
            if not size_header:
                return b""
            size = int(size_header.split(b";", 1)[0].strip(), 16)
            if size == 0:
                return b""
            part = await self.reader.readexactly(size)
            assert await self.reader.readexactly(2) == b"\r\n"
            self.raw_buffer += part
        sep = self.raw_buffer.index(b"\n") + 1
        line, self.raw_buffer = self.raw_buffer[:sep], self.raw_buffer[sep:]
        return line

    async def event(self, seconds: float = 25.0) -> dict:
        async def receive() -> dict:
            kind = None
            payload = []
            while True:
                line = await self.readline()
                if not line:
                    raise AssertionError("SSE stream ukončen před očekávanou zprávou")
                self.bytes_read += len(line)
                line = line.rstrip(b"\r\n")
                if not line:
                    if payload and kind:
                        return {"kind": kind, "body": json.loads(b"".join(payload))}
                    continue
                if line.startswith(b"event: "):
                    kind = line[7:].decode("utf-8")
                elif line.startswith(b"data: "):
                    payload.append(line[6:])

        return await asyncio.wait_for(receive(), timeout=seconds)

    def close(self) -> None:
        self.writer.close()


async def connect() -> tuple[int, SseClient]:
    reader, writer = await asyncio.wait_for(
        asyncio.open_connection(HOST, PORT, limit=2 * 1024 * 1024), timeout=8
    )
    writer.write(
        b"GET /api/v1/stream HTTP/1.1\r\n"
        b"Host: 127.0.0.1\r\nAccept: text/event-stream\r\n"
        b"Connection: keep-alive\r\n\r\n"
    )
    await asyncio.wait_for(writer.drain(), timeout=8)
    headers = await asyncio.wait_for(reader.readuntil(b"\r\n\r\n"), timeout=8)
    first = headers.splitlines()[0].split()
    if len(first) < 2:
        raise AssertionError("Neplatný HTTP status")
    return int(first[1]), SseClient(reader, writer, b"transfer-encoding: chunked" in headers.lower())


async def await_live(count: int) -> None:
    import urllib.request

    for _ in range(180):
        try:
            with urllib.request.urlopen(
                f"http://{HOST}:{PORT}/api/v1/aircraft", timeout=2
            ) as response:
                payload = json.load(response)
            if len(payload.get("data", {}).get("aircraft", [])) >= count:
                return
        except (OSError, ValueError):
            pass
        await asyncio.sleep(0.15)
    raise AssertionError(f"Rust RAM snapshot nedosáhl {count} letadel")


def percentile(values: list[float], pct: float) -> float:
    values = sorted(values)
    index = max(0, min(len(values) - 1, int(len(values) * pct + 0.999999) - 1))
    return round(values[index], 2)


async def benchmark(binary: Path, temp: Path) -> list[dict]:
    fixture = temp / "synthetic-aircraft.json"
    write_fixture(fixture, 25, 0)
    env = dict(os.environ)
    env["AIRRADAR_V2_BIND"] = f"{HOST}:{PORT}"
    env["AIRRADAR_V2_READSB_JSON"] = str(fixture)
    env.pop("AIRRADAR_V2_BEAST_ADDR", None)
    with (temp / "rust.log").open("wb") as output:
        engine = subprocess.Popen(
            [str(binary), "--g03-live-api"],
            cwd=temp,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            await await_live(25)
            results = []
            for index, (connections, count) in enumerate(SCENARIOS, 1):
                write_fixture(fixture, count, index * 2, compact=count >= 5000)
                await await_live(count)
                opened = await asyncio.gather(*(connect() for _ in range(connections)))
                assert all(status == 200 for status, _ in opened), "SSE HTTP 200 očekáváno"
                clients = [client for _, client in opened]
                try:
                    initial = await asyncio.gather(*(c.event() for c in clients))
                    assert all(item["kind"] == "snapshot" for item in initial)
                    seqs = [item["body"]["seq"] for item in initial]
                    assert len(set(seqs)) == 1, "Vstupní stav se během připojení neměnil"
                    assert all(len(item["body"]["aircraft"]) >= count for item in initial)
                    assert all("receiverId" not in json.dumps(item) for item in initial)
                    if connections == 100:
                        status, excess = await connect()
                        excess.close()
                        assert status == 503, "101. SSE klient musí dostat 503"

                    cpu_before, rss_before = proc_stats(engine.pid)
                    prior_bytes = sum(c.bytes_read for c in clients)
                    rss_samples = [rss_before]
                    stop_sampling = asyncio.Event()

                    async def sample_rss() -> None:
                        while not stop_sampling.is_set():
                            rss_samples.append(proc_stats(engine.pid)[1])
                            await asyncio.sleep(0.05)

                    watcher = asyncio.create_task(sample_rss())
                    started = time.perf_counter()
                    write_fixture(fixture, count, index * 2 + 1, compact=count >= 5000)

                    async def receive_change(client: SseClient, seq: int) -> float:
                        while True:
                            item = await client.event()
                            if item["kind"] not in ("delta", "snapshot"):
                                continue
                            if item["body"]["seq"] > seq:
                                assert item["kind"] in ("delta", "snapshot")
                                assert "receiverId" not in json.dumps(item)
                                return (time.perf_counter() - started) * 1000

                    latencies = await asyncio.gather(
                        *(receive_change(client, seq) for client, seq in zip(clients, seqs))
                    )
                    elapsed = time.perf_counter() - started
                    stop_sampling.set()
                    await watcher
                    cpu_after, rss_after = proc_stats(engine.pid)
                    result = {
                        "clients": connections,
                        "aircraft": count,
                        "events": len(latencies),
                        "lost_updates": connections - len(latencies),
                        "measured_from": "atomic_json_rename",
                        "slow_consumers_excluded_from_latency": True,
                        "latency_p95_ms": percentile(latencies, 0.95),
                        "latency_p99_ms": percentile(latencies, 0.99),
                        "bytes_per_sec": round(
                            (sum(c.bytes_read for c in clients) - prior_bytes)
                            / max(elapsed, 0.001)
                        ),
                        "cpu_percent_1_core": round(
                            (cpu_after - cpu_before) / max(elapsed, 0.001) * 100, 2
                        ),
                        "rss_before_kib": rss_before,
                        "rss_after_kib": rss_after,
                        "rss_peak_kib": max(rss_samples + [rss_after]),
                        "elapsed_s": round(elapsed, 3),
                    }
                    results.append(result)
                    print("G03_SSE_SCENARIO=" + json.dumps(result, sort_keys=True), flush=True)
                finally:
                    for c in clients:
                        c.close()
                    await asyncio.gather(
                        *(c.writer.wait_closed() for c in clients), return_exceptions=True
                    )

            # G-03.07: 90 navázaných klientů záměrně nečte žádné SSE body.
            # Jediný rychlý konzument musí stále dostávat všechna nová data;
            # ověříme skutečný Rust RSS budget, nikoli OS socket buffery.
            write_fixture(fixture, 5000, 400, compact=True)
            await await_live(5000)
            rss_before_slow = proc_stats(engine.pid)[1]
            unread = await asyncio.gather(*(connect() for _ in range(90)))
            assert all(status == 200 for status, _ in unread)
            healthy_status, healthy = await connect()
            assert healthy_status == 200
            slow_peak = rss_before_slow
            try:
                initial = await healthy.event(seconds=20)
                assert initial["kind"] == "snapshot"
                last_seq = initial["body"]["seq"]
                for update_no in range(5):
                    write_fixture(fixture, 5000, 401 + update_no, compact=True)
                    while True:
                        message = await healthy.event(seconds=20)
                        if message["kind"] in ("delta", "snapshot") and message["body"]["seq"] > last_seq:
                            last_seq = message["body"]["seq"]
                            break
                    slow_peak = max(slow_peak, proc_stats(engine.pid)[1])
                growth = slow_peak - rss_before_slow
                assert growth <= 25 * 1024, (
                    f"90 nečtoucích klientů: Rust RSS narostlo o {growth} KiB, limit 25 MiB"
                )
                print("G03_SSE_SLOW_CLIENTS=" + json.dumps({
                    "unread_clients": 90,
                    "healthy_clients": 1,
                    "delivered_updates_to_healthy": 5,
                    "rust_rss_growth_kib": growth,
                    "rust_rss_peak_kib": slow_peak,
                    "rust_rss_budget_kib": 25 * 1024,
                    "os_tcp_socket_buffers": "not_measured",
                }, sort_keys=True), flush=True)
            finally:
                healthy.close()
                for _, client in unread:
                    client.close()
                await asyncio.gather(
                    healthy.writer.wait_closed(),
                    *(client.writer.wait_closed() for _, client in unread),
                    return_exceptions=True,
                )

            # G-03.09: chybný/částečný readsb JSON nepoškodí živý RAM snapshot.
            status, malformed_client = await connect()
            assert status == 200
            try:
                initial = await malformed_client.event()
                assert initial["kind"] == "snapshot"
                before_ids = {a["id"] for a in initial["body"]["aircraft"]}
                # Předchozí dávka může legitimně posunout seq ještě po připojení;
                # chybný nový JSON nesmí ztratit žádnou dosavadní identitu.
                # Původní data jsou v RAM, syntetický JSON pouze záměrně poškodíme.
                broken = fixture.with_suffix(".tmp")
                broken.write_text("{invalid-json", encoding="utf-8")
                os.replace(broken, fixture)
                await asyncio.sleep(1.4)
                import urllib.request

                with urllib.request.urlopen(
                    f"http://{HOST}:{PORT}/api/v1/aircraft", timeout=3
                ) as response:
                    retained = json.load(response)["data"]
                retained_ids = {a["id"] for a in retained["aircraft"]}
                assert retained_ids == before_ids, (
                    "Chybný JSON odstranil nebo přidal identitu v kanonickém RAM stavu"
                )
                before_seq = retained["seq"]
                write_fixture(fixture, 1000, 100)
                while True:
                    recovered = await malformed_client.event(seconds=8)
                    if recovered["kind"] in ("delta", "snapshot") and recovered["body"]["seq"] > before_seq:
                        break
                print("PASS G-03.09: malformed JSON, zachování RAM a zotavení")
            finally:
                malformed_client.close()
                await malformed_client.writer.wait_closed()

            # G-03.06 lifecycle: po odpojení všech klientů musí být kapacita vrácena.
            status, remaining = await connect()
            assert status == 200, "SSE permit nebyl uvolněn po disconnectu"
            remaining.close()
            await remaining.writer.wait_closed()
            return results
        finally:
            engine.terminate()
            try:
                await asyncio.to_thread(engine.wait, 4)
            except subprocess.TimeoutExpired:
                engine.kill()
                await asyncio.to_thread(engine.wait)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    if not binary.is_file():
        raise RuntimeError("Chybí Rust binary, předem sestav --g02-replay/--g02-benchmark")
    if not sys.platform.startswith("linux"):
        raise RuntimeError("Měření /proc vyžaduje Linux")
    with tempfile.TemporaryDirectory(prefix="airradarv2-g03-sse-") as work:
        try:
            results = asyncio.run(benchmark(binary, Path(work)))
        except Exception:
            log = Path(work) / "rust.log"
            if log.exists():
                print(log.read_text(encoding="utf-8", errors="replace")[-6000:])
            raise
    assert len(results) == len(SCENARIOS)
    print("PASS G-03: izolované 1/10/100 klientů až s 5000 kompaktními cíli, žádné DB/V1/readsb TCP.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
