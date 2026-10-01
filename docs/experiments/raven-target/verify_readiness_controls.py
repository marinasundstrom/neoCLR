"""Compare CLI emission with the legacy importer/runtime, not native-backend acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("inventory", type=Path, help="Completed --readiness-inventory output")
parser.add_argument("output", type=Path, help="Fresh control output directory")
parser.add_argument("--bridge", type=Path, required=True)
parser.add_argument("--runtime", type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
inventory = args.inventory.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=False)
bridge = args.bridge.resolve()
runtime = args.runtime.resolve()
core = root / "api-docs/reference/NeoCLR.CoreProbe.dll"
system = output / "System.Collections.neoil"
system.write_text(build(root / "runtime/System.neoil"))
expected_outputs = {
    "application-interfaces": "42\n99\n",
    "application-inheritance": "7\n42\n",
    "application-delegates": "8\n42\n99\n12\n15\n42\n42\n123\n123\n1\n-2147483648\n",
    "application-order-collections": (
        root / "docs/experiments/raven-target/samples/application-order-collections.expected.txt"
    ).read_text(),
}
reports = []


def command(arguments):
    process = subprocess.run(
        [str(argument) for argument in arguments], capture_output=True, text=True, timeout=120
    )
    return {"exitCode": process.returncode, "stdout": process.stdout, "stderr": process.stderr}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


for name, expected in expected_outputs.items():
    destination = output / name
    source = inventory / (name + "-target.cli.dll")
    imported = command(["dotnet", bridge, "--import", source, core, destination])
    verification = run = None
    if imported["exitCode"] == 0:
        verification = command([runtime, "verify", destination / "App.neoil", "--system", system])
        if verification["exitCode"] == 0:
            run = command([runtime, "run", destination / "App.neoil", "--system", system])
    passed = run is not None and run["exitCode"] == 0 and run["stdout"] == expected
    reports.append({
        "sample": name, "cliSha256": sha256(source), "import": imported,
        "verification": verification, "run": run, "expectedStdout": expected, "passed": passed,
    })
    print(name, passed, flush=True)
    (output / "validation.json").write_text(json.dumps({
        "methodology": "Current Raven ordinary CLI target emission -> current legacy bridge -> "
            "existing generated collection library -> current runtime. This is a control, not "
            "direct metadata-backend or fresh full-library compilation evidence.",
        "bridgeSha256": sha256(bridge), "coreSha256": sha256(core),
        "systemSha256": sha256(system), "runtimeSha256": sha256(runtime), "cases": reports,
    }, indent=2) + "\n")
# An exploratory report deliberately records failures instead of freezing them as expectations.
