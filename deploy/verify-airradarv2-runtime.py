#!/usr/bin/env python3
"""Kompatibilní název stage-aware DEV runtime verifieru."""

from pathlib import Path
import runpy

here = Path(__file__).resolve().parent
for candidate in (here / "airradarv2-verify-g03-runtime.py", here / "verify-g03-runtime.py"):
    if candidate.is_file():
        runpy.run_path(str(candidate), run_name="__main__")
        break
else:
    raise SystemExit("chybí stage-aware runtime verifier")
