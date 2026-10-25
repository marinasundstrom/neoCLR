"""Compatibility entry point; implementation lives in runtime/native/gc/concurrent."""
from pathlib import Path
import runpy

runpy.run_path(str(Path(__file__).resolve().parents[3] / "runtime/native/gc/concurrent/native-providers.py"), run_name="__main__")
