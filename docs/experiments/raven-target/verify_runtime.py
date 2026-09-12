#!/usr/bin/env python3
"""Verify imported Raven programs with an explicitly selected neoCLR executable."""
import json
from pathlib import Path
import subprocess
import sys

runtime = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
report = json.loads((output / "report.json").read_text())
if len(report.get("StaticImportRejections", {})) != 3:
    raise AssertionError("Run the complete emission/import probe, including rejection checks, first")
results = {}
for name in ("CoreOnly", "CoreEmpty", "CoreNested", "CoreInt32"):
    artifact = output / (name + ".neoil")
    subprocess.run([str(runtime), "verify", str(artifact)], check=True, capture_output=True, text=True)
    run = subprocess.run([str(runtime), "run", str(artifact)], check=True, capture_output=True, text=True)
    expected = ("Hello from Raven on neoCLR\n" if name == "CoreOnly" else "") + "=> Void\n"
    if run.stdout != expected:
        raise AssertionError(f"{name}: expected {expected!r}, received {run.stdout!r}")
    results[name] = {"stdout": run.stdout, "stderr": run.stderr, "verified": True}
print(json.dumps(results, indent=2))
