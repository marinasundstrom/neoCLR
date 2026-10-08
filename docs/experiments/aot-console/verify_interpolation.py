#!/usr/bin/env python3
"""Qualify standalone Raven interpolation, boxed integers and String Object views."""
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
parser.add_argument("--sample", action="append", choices=["interpolation", "interpolation-text", "boxed-int32", "character-text-identity", "string-interface-views"],
                    help="Qualify only selected samples; default qualifies all.")
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
report = dict(profile="raven-string-interface-views-v1", baseRevision=subprocess.check_output(
    ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), SDKROOT=os.environ.get("SDKROOT"),
    inputs={str(path): sha(path) for path in sorted(inputs)}, commands=[])


def save():
    # Preserve small evidence; complete command bytes are hashed before compaction.
    for command in report["commands"]:
        for stream in ("stdout", "stderr"):
            value = command[stream]
            if len(value) > 8000:
                command[stream + "Sha256"] = hashlib.sha256(value.encode()).hexdigest()
                command[stream + "OriginalBytes"] = len(value.encode())
                if "--inspect" in command["command"]:
                    details = json.loads(value)
                    command[stream] = json.dumps({key: details[key] for key in ("schema", "admission", "notice")})
                else:
                    command[stream] = value[-4000:]
    (output / "validation.json").write_text(json.dumps(report, indent=2) + "\n")


def run(command, expected=0, **kwargs):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=120, **kwargs)
    report["commands"].append(dict(command=list(map(str, command)), exit=result.returncode,
                                   stdout=result.stdout.decode(), stderr=result.stderr.decode()))
    save()
    assert result.returncode == expected, (command, result.stdout, result.stderr)
    return result


context = ["--system", seed, "--module", library, "--object-root", library]
flags = context + ["--compile-system", "--bind-user-fault", "--bind-console-write-line",
                   "--bind-utf8-text", "--bind-int32-to-string", "--reference-arena"]
expected_numeric = "".join(f"Value: {value}\n{value}\nValue: {value}\n" for value in (42, -2147483648, 2147483647)) + "Null: \n"
cases = [("string-interface-views", "hé😀\0z\n\n"), ("interpolation", expected_numeric), ("interpolation-text", "Text: hé😀/z\x00end\n"),
         ("boxed-int32", "-2147483648\n-1\n0\n1\n42\n2147483647\n"),
         ("character-text-identity", "".join(f"Item: {i}\n{char}\n" for i, char in enumerate(["A", "å", "😀", "é", "👨‍👩‍👧‍👦", "🇸🇪", "\0"])))]
for stem, expected_text in cases:
    if args.sample and stem not in args.sample:
        continue
    sample_flags = flags + (["--bind-character-text"] if stem == "character-text-identity" else [])
    native_text = []
    if stem == "character-text-identity":
        manifest = ROOT / "tools/aot-native-text/Cargo.toml"
        run(["cargo", "build", "--locked", "--release", "--manifest-path", manifest])
        native_text = [manifest.parent / "target/release/libneoclr_aot_native_text.a"]
        for path in [manifest, manifest.parent / "Cargo.lock", manifest.parent / "src/lib.rs", *native_text]:
            report["inputs"][str(path)] = sha(path)
    source, assembly = base / (stem + ".rvn"), output / (stem + ".dll")
    report["inputs"][str(source)] = sha(source)
    run(["dotnet", compiler, "neoclr", "--core-reference", core, "--runtime-seed", seed,
         "--reference", library, "--bootstrap-intrinsics", "--bootstrap-ownership", ownership,
         "--object-library", "System.Runtime", "-o", assembly, source])
    run([runtime, "verify", assembly, *context])
    interpreted = run([runtime, "run", assembly, *context])
    assert interpreted.stdout == expected_text.encode() and not interpreted.stderr
    inspection = json.loads(run([aot, "--inspect", assembly, "@entry", "--closed-world", *sample_flags]).stdout)
    obj = output / (stem + ".o")
    report[stem] = dict(assemblySha256=sha(assembly), stdout=expected_text, admission=inspection["admission"])
    assert inspection["admission"]["accepted"]
    if stem == "string-interface-views":
        selected = inspection["selection"]
        interfaces = selected["stringInterfaceViews"]
        assert interfaces, "String conformance was not retained"
        report[stem]["stringInterfaceViews"] = [row for row in selected["types"] if row["compiledIndex"] in interfaces]
    run([aot, "--closed-world", assembly, "@entry", obj, *sample_flags])
    binary = output / stem
    run(["clang", "-arch", "arm64", "-std=c11", "-Wall", "-Wextra", "-Werror", *adapters, obj, *native_text, "-o", binary])
    imports = set(run(["nm", "-u", obj]).stdout.decode().split())
    expected_imports = {"_neoclr_console_write_line_utf8_v1"}
    if stem in ("boxed-int32", "interpolation", "character-text-identity"):
        expected_imports |= {"_neoclr_allocate_object_v1", "_neoclr_int32_to_string_v1"}
    if stem != "boxed-int32":
        expected_imports.add("_neoclr_string_concat_v1")
    if stem == "string-interface-views":
        expected_imports.add("_neoclr_allocate_object_v1")
    if native_text:
        expected_imports.add("_neoclr_is_single_grapheme_v1")
    assert imports == expected_imports, imports
    dependencies = [line.split()[0] for line in run(["otool", "-L", binary]).stdout.decode().splitlines()[1:]]
    assert dependencies == ["/usr/lib/libSystem.B.dylib"], dependencies
    with tempfile.TemporaryDirectory() as directory:
        installed = Path(directory) / "interpolation"
        shutil.copy2(binary, installed)
        native = subprocess.run([installed], cwd=directory, env={}, capture_output=True, timeout=10)
        assert (native.returncode, native.stdout, native.stderr) == (0, interpreted.stdout, b"")
    if stem in ("interpolation", "character-text-identity", "string-interface-views"):
        read_end, write_end = os.pipe()
        os.close(read_end)
        try:
            failed = run([runtime, "run", assembly, *context], expected=1,
                         pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1))
            native_failed = run([binary], expected=1, env={},
                                pass_fds=(write_end,), preexec_fn=lambda: os.dup2(write_end, 1))
            assert failed.stderr == native_failed.stderr and failed.stderr
            assert not failed.stdout and not native_failed.stdout
            report[stem]["brokenPipeFaultParity"] = failed.stderr.decode()
        finally:
            os.close(write_end)
    report[stem].update(executableSha256=sha(binary), dynamicDependencies=dependencies,
                        executableOnlyDirectory=True, emptyEnvironment=True, nativeMatchesInterpreter=True)
save()
print("Passed: selected standalone Console samples; exact interpreter output and output-fault parity")
