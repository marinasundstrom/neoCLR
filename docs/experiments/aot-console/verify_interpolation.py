#!/usr/bin/env python3
"""Qualify Raven concat conversions and expose the remaining boxed-display AOT gate."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
for name in ("compiler", "runtime", "aot", "bundle", "output"):
    parser.add_argument("--" + name, type=Path, required=True)
args = parser.parse_args()
compiler, runtime, aot, bundle, output = (
    getattr(args, name).resolve() for name in ("compiler", "runtime", "aot", "bundle", "output")
)
output.mkdir(parents=True, exist_ok=False)
base = Path(__file__).resolve().parent
faults = base.parent / "aot-fault-details"
core, seed, library, ownership = (
    bundle / "lib" / name for name in ("Core.dll", "System.runtime.neox", "System.Runtime.dll", "ownership.json")
)
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
compiler_inputs = {compiler, *compiler.parent.glob("*.dll"), *compiler.parent.glob("*.deps.json"), *compiler.parent.glob("*.runtimeconfig.json")}
adapters = [base / "text-host.c", faults / "render.c", base / "console.c", base.parent / "aot-scalar/console.c", base / "text-arena.c"]
inputs = compiler_inputs | {runtime, aot, core, seed, library, ownership, Path(__file__).resolve(), base / "text-arena.h", *adapters}
report = dict(profile="raven-concat-conversions-v1", baseRevision=subprocess.check_output(
    ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), SDKROOT=os.environ.get("SDKROOT"),
    inputs={str(path): sha(path) for path in sorted(inputs)}, commands=[])


def save():
    (output / "validation.json").write_text(json.dumps(report, indent=2) + "\n")


def run(command, expected=0):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=120)
    report["commands"].append(dict(command=list(map(str, command)), exit=result.returncode,
                                   stdout=result.stdout.decode(), stderr=result.stderr.decode()))
    save()
    assert result.returncode == expected, (command, result.stdout, result.stderr)
    return result


context = ["--system", seed, "--module", library, "--object-root", library]
flags = context + ["--compile-system", "--bind-user-fault", "--bind-console-write-line",
                   "--bind-utf8-text", "--bind-int32-to-string", "--reference-arena"]
expected_numeric = "".join(f"Value: {value}\n{value}\nValue: {value}\n" for value in (42, -2147483648, 2147483647)) + "Null: \n"
for stem, expected_text in (("interpolation", expected_numeric), ("interpolation-text", "Text: hé😀/z\x00end\n")):
    source, assembly = base / (stem + ".rvn"), output / (stem + ".dll")
    report["inputs"][str(source)] = sha(source)
    run(["dotnet", compiler, "neoclr", "--core-reference", core, "--runtime-seed", seed,
         "--reference", library, "--bootstrap-intrinsics", "--bootstrap-ownership", ownership,
         "--object-library", "System.Runtime", "-o", assembly, source])
    run([runtime, "verify", assembly, *context])
    interpreted = run([runtime, "run", assembly, *context])
    assert interpreted.stdout == expected_text.encode() and not interpreted.stderr
    inspection = json.loads(run([aot, "--inspect", assembly, "@entry", "--closed-world", *flags]).stdout)
    obj = output / (stem + ".o")
    report[stem] = dict(assemblySha256=sha(assembly), stdout=expected_text, admission=inspection["admission"])
    if stem == "interpolation":
        assert inspection["admission"] == dict(accepted=False, phase="selection",
            firstError="Object display with boxing or arrays requires a later receiver/metadata profile")
        run([aot, "--closed-world", assembly, "@entry", obj, *flags], expected=1)
        assert not obj.exists(), "Rejected boxed display published a native object"
        continue
    assert inspection["admission"]["accepted"]
    run([aot, "--closed-world", assembly, "@entry", obj, *flags])
    binary = output / stem
    run(["clang", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", *adapters, obj, "-o", binary])
    imports = set(run(["nm", "-u", obj]).stdout.decode().split())
    assert imports == {"_neoclr_console_write_line_utf8_v1", "_neoclr_string_concat_v1"}, imports
    dependencies = [line.split()[0] for line in run(["otool", "-L", binary]).stdout.decode().splitlines()[1:]]
    assert dependencies == ["/usr/lib/libSystem.B.dylib"], dependencies
    with tempfile.TemporaryDirectory() as directory:
        installed = Path(directory) / "interpolation"
        shutil.copy2(binary, installed)
        native = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
        assert (native.returncode, native.stdout, native.stderr) == (0, interpreted.stdout, b"")
    report[stem].update(executableSha256=sha(binary), dynamicDependencies=dependencies,
                        executableOnlyDirectory=True, emptyEnvironment=True, nativeMatchesInterpreter=True)
save()
print("Passed: native CIL concat conversions; standalone text interpolation; boxed-display AOT rejection retained")
