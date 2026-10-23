#!/usr/bin/env python3
"""G-04.06/.12: stejná deterministická rekonstruovaná trasa, 15/30/60 s.

Výstup je kvalitativní replay sampleru, nikoli tvrzení o skutečném RF letu.
"""
from __future__ import annotations

import json
import math


def distance_m(a: dict, b: dict) -> float:
    lat = math.radians((a["lat"] + b["lat"]) / 2)
    dx = math.radians(b["lon"] - a["lon"]) * math.cos(lat)
    dy = math.radians(b["lat"] - a["lat"])
    return 6_371_000 * math.sqrt(dx * dx + dy * dy)


def heading_delta(a: float, b: float) -> float:
    diff = abs(a - b) % 360
    return min(diff, 360 - diff)


def make_trace() -> list[dict]:
    lat, lon = 49.0, 17.0
    trace = []
    for t in range(601):
        if t:
            heading = 90 + max(0, min(t - 180, 60)) * 1.5
            distance = 0.00072
            lat += distance * math.cos(math.radians(heading))
            lon += distance * math.sin(math.radians(heading)) / math.cos(math.radians(lat))
        heading = 90 + max(0, min(t - 180, 60)) * 1.5
        altitude = 10_000 + (max(0, min(t - 300, 60)) * 25)
        trace.append({"t": t, "lat": lat, "lon": lon, "heading": heading, "altitude": altitude})
    return trace


def adaptive(trace: list[dict]) -> list[int]:
    selected = [0]
    for index in range(1, len(trace)):
        last = trace[selected[-1]]
        current = trace[index]
        dt = current["t"] - last["t"]
        distance = distance_m(last, current)
        heading = heading_delta(last["heading"], current["heading"])
        altitude = abs(current["altitude"] - last["altitude"])
        if dt >= 30 and (heading >= 30 or altitude >= 1500):
            selected.append(index)
        elif dt >= 180 or (dt >= 60 and (distance >= 2000 or heading >= 15 or altitude >= 500)):
            selected.append(index)
    return selected


def fixed(trace: list[dict], seconds: int) -> list[int]:
    result = list(range(0, len(trace), seconds))
    if result[-1] != len(trace) - 1:
        result.append(len(trace) - 1)
    return result


def max_reconstruction_error(trace: list[dict], selected: list[int]) -> float:
    maximum = 0.0
    for left, right in zip(selected, selected[1:]):
        a, b = trace[left], trace[right]
        span = max(1, right - left)
        for index in range(left, right + 1):
            ratio = (index - left) / span
            reconstructed = {
                "lat": a["lat"] + (b["lat"] - a["lat"]) * ratio,
                "lon": a["lon"] + (b["lon"] - a["lon"]) * ratio,
            }
            maximum = max(maximum, distance_m(trace[index], reconstructed))
    return maximum


def main() -> None:
    trace = make_trace()
    profiles = [("15s", fixed(trace, 15)), ("30s", fixed(trace, 30)), ("60s", fixed(trace, 60)), ("adaptive", adaptive(trace))]
    for name, selected in profiles:
        rows = [
            {"t": trace[i]["t"], "lat": trace[i]["lat"], "lon": trace[i]["lon"], "heading": trace[i]["heading"], "altitude": trace[i]["altitude"]}
            for i in selected
        ]
        turn_rows = sum(180 <= row["t"] <= 240 for row in rows)
        payload_bytes = sum(len(json.dumps(row, separators=(",", ":"))) for row in rows)
        print(
            f"G-04 sampling {name}: rows={len(rows)} max_error_m={max_reconstruction_error(trace, selected):.2f} "
            f"turn_rows={turn_rows} payload_bytes={payload_bytes}"
        )


if __name__ == "__main__":
    main()
