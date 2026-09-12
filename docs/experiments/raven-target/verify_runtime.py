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
if len(report.get("ResultImportRejections", {})) != 4:
    raise AssertionError("Run Result import rejection checks first")
if len(report.get("OptionImportRejections", {})) != 3:
    raise AssertionError("Run Option import rejection checks first")
if len(report.get("VoidImportRejections", {})) != 3:
    raise AssertionError("Run Void projection/import checks first")
results = {}
for name in ("CoreOnly", "CoreEmpty", "CoreNested", "CoreInt32", "CoreLibrary", "CoreUnion", "CoreOption", "CoreVoid"):
    artifact = output / (name + ".neoil")
    subprocess.run([str(runtime), "verify", str(artifact)], check=True, capture_output=True, text=True)
    run = subprocess.run([str(runtime), "run", str(artifact)], check=True, capture_output=True, text=True)
    expected = {"CoreVoid": "Completed without a payload\nNot completed\n", "CoreOption": "42\nProduct not found\n", "CoreUnion": "42\nOverflow\n", "CoreOnly": "Hello from Raven on neoCLR\n", "CoreLibrary": "42\n1\n0\nLibrary calls from Raven\n"}.get(name, "")
    if run.stdout != expected:
        raise AssertionError(f"{name}: expected {expected!r}, received {run.stdout!r}")
    results[name] = {"stdout": run.stdout, "stderr": run.stderr, "verified": True}
print(json.dumps(results, indent=2))
