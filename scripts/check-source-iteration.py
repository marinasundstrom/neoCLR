#!/usr/bin/env python3
"""Ordinary-driver acceptance for source-owned iteration contracts on both targets."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/experiments/extended-cli-metadata/bootstrap/iteration-ownership.json"
CONSUMER = MANIFEST.with_name("iteration-consumer.rvn")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--raven-root", type=Path, required=True)
    parser.add_argument("--core", type=Path, required=True, help="Primitive-only CoreProbe, generated with --reference-primitive-core")
    parser.add_argument("--runtime", type=Path, default=ROOT / "target/release/neoclr")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    driver = args.raven_root.resolve() / "src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll"
    runtime_config = driver.with_suffix(".runtimeconfig.json")
    core, runtime, output = args.core.resolve(), args.runtime.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    manifest = json.loads(MANIFEST.read_text())
    library = manifest["libraries"][0]
    commands = []
    evidence = {
        "scope": "source-built iteration contracts; no retained System seed is referenced",
        "manifestSha256": sha(MANIFEST), "consumerSha256": sha(CONSUMER),
        "sources": {p: sha(ROOT / p) for p in library["sources"]},
        "artifacts": {str(p): sha(p) for p in [driver, runtime_config, core, runtime]},
        "revisions": {str(p): subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=p, text=True).strip()
                      for p in [ROOT, args.raven_root.resolve()]},
        "commands": commands, "passed": False,
    }

    def run(arguments, expected=0, stdout=None, diagnostic=None):
        result = subprocess.run([str(a) for a in arguments], capture_output=True, text=True, timeout=120)
        commands.append({"arguments": [str(a) for a in arguments], "exitCode": result.returncode,
                         "expectedExitCode": expected, "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != expected or stdout is not None and (result.stdout != stdout or result.stderr):
            raise RuntimeError(f"Unexpected command result: {arguments}\n{result.stdout}\n{result.stderr}")
        if diagnostic and diagnostic not in result.stdout + result.stderr:
            raise RuntimeError("Missing diagnostic: " + diagnostic)

    try:
        for target in ["dotnet", "neoclr"]:
            directory = output / target
            directory.mkdir()
            source_dir = directory / "library-source"
            source_dir.mkdir()
            sources = []
            for index, path in enumerate(library["sources"]):
                copied = source_dir / f"{index}-{Path(path).name}"
                shutil.copyfile(ROOT / path, copied)
                sources.append(copied)
            artifact = directory / (library["assemblyName"] + ".dll")
            application = directory / "IterationConsumer.dll"
            prefix = ["dotnet", driver]
            if target == "neoclr":
                prefix += ["neoclr", "--core-reference", core]
                library_options = ["--library"]
                references = ["--reference", artifact]
            else:
                prefix += ["--framework", "net10.0", "--emit-core-types-only"]
                library_options = ["--output-type", "classlib"]
                references = ["--refs", artifact]
            run([*prefix, "--bootstrap-ownership", MANIFEST, *library_options, "-o", artifact, *sources])
            shutil.rmtree(source_dir)
            run([*prefix, "--bootstrap-ownership", MANIFEST, *references, "-o", application, CONSUMER])
            if target == "neoclr":
                run([runtime, "verify", application, "--module", artifact])
                run([runtime, "run", application, "--module", artifact], expected=42, stdout="")
            else:
                run(["dotnet", "exec", "--runtimeconfig", runtime_config, application], expected=42, stdout="")
            for scenario in ["wrong-owner", "duplicate-owner", "unsupported-version"]:
                invalid = json.loads(MANIFEST.read_text())
                if scenario == "wrong-owner":
                    invalid["libraries"][0]["assemblyName"] = "WrongOwner"
                    invalid["iteration"]["assemblyName"] = "WrongOwner"
                    expected_text = "requires exactly one declaration"
                elif scenario == "duplicate-owner":
                    invalid["libraries"][0]["types"].append(invalid["libraries"][0]["types"][0])
                    expected_text = "Duplicate or invalid bootstrap type owner"
                else:
                    invalid["version"] = 2
                    expected_text = "Unsupported bootstrap ownership"
                invalid_path = directory / (scenario + ".json")
                invalid_path.write_text(json.dumps(invalid))
                destination = directory / (scenario + ".dll")
                run([*prefix, "--bootstrap-ownership", invalid_path, *references, "-o", destination, CONSUMER],
                    expected=1, diagnostic=expected_text)
                if destination.exists():
                    raise RuntimeError("Failed configuration published output: " + scenario)
            if target == "neoclr":
                destination = directory / "duplicate-bootstrap.dll"
                run(["dotnet", driver, "neoclr", "--core-reference", ROOT / "api-docs/reference/NeoCLR.CoreProbe.dll",
                     "--bootstrap-ownership", MANIFEST, *references, "-o", destination, CONSUMER], expected=1,
                    diagnostic="requires exactly one declaration")
                if destination.exists():
                    raise RuntimeError("Conflicting bootstrap published output")
            evidence[target] = {"librarySha256": sha(artifact), "applicationSha256": sha(application), "exitCode": 42}
        evidence["passed"] = True
        print("PASS source-owned iteration: .NET and NeoCLR 42; ownership failures publish no output")
    finally:
        (output / "validation.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
