#!/usr/bin/env python3
"""Kompatibilní název stage-aware DEV runtime verifieru."""

from pathlib import Path
import runpy

runpy.run_path(str(Path(__file__).with_name("verify-g03-runtime.py")), run_name="__main__")
