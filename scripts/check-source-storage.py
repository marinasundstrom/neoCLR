#!/usr/bin/env python3
"""Driver acceptance for an explicit storage bootstrap and separately compiled native helper."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/experiments/extended-cli-metadata/bootstrap/iteration-ownership.json"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--raven-root", type=Path, required=True)
    parser.add_argument("--core", type=Path, required=True, help="Explicit --reference-storage-core bootstrap")
    parser.add_argument("--runtime", type=Path, default=ROOT / "target/release/neoclr")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output, core, runtime = args.output.resolve(), args.core.resolve(), args.runtime.resolve()
    driver = args.raven_root.resolve() / "src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll"
    output.mkdir(parents=True, exist_ok=False)
    evidence = {"scope": "driver bootstrap storage; not ArrayList or broad application completion",
                "revisions": {str(p): subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=p, text=True).strip()
                              for p in [ROOT, args.raven_root.resolve()]},
                "artifacts": {str(p): sha(p) for p in [core, driver, runtime]},
                "sourceHashes": {str(p): sha(p) for p in [Path(__file__),
                    ROOT / "docs/experiments/raven-target/CoreDeclarations.cs",
                    ROOT / "docs/experiments/raven-target/Program.cs",
                    args.raven_root.resolve() / "src/Raven.Compiler/NeoClrCommand.cs"]},
                "ownershipManifestSha256": sha(MANIFEST), "commands": [], "passed": False}

    def run(arguments, expected=0, diagnostic=None, silent=False):
        result = subprocess.run([str(a) for a in arguments], capture_output=True, text=True, timeout=180)
        evidence["commands"].append({"arguments": [str(a) for a in arguments], "exitCode": result.returncode,
                                     "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != expected or silent and (result.stdout or result.stderr):
            raise RuntimeError(f"Unexpected result: {arguments}\n{result.stdout}\n{result.stderr}")
        if diagnostic and diagnostic not in result.stdout + result.stderr:
            raise RuntimeError("Missing diagnostic: " + diagnostic)

    try:
        # Reuse the ordinary dual-target library build and unchanged source ownership checks.
        run([sys.executable, ROOT / "scripts/check-source-iteration.py", "--raven-root", args.raven_root,
             "--core", core, "--runtime", runtime, "--output", output / "iteration"])
        contracts = output / "iteration/neoclr/NeoCLR.Collections.dll"
        prefix = ["dotnet", driver, "neoclr", "--core-reference", core,
                  "--bootstrap-ownership", MANIFEST, "--reference", contracts]
        source = output / "Storage.rvn"
        source.write_text("""namespace Storage
public func Reserve<T>(length: int) -> T[] {
    return System.Runtime.CompilerServices.CheckedStorage.Reserve<T>(length)
}
""")
        evidence["librarySourceSha256"] = sha(source)
        denied = output / "Disabled.dll"
        run([*prefix, "--library", "-o", denied, source], expected=1, diagnostic="NEOMETA001")
        if denied.exists():
            raise RuntimeError("Disabled intrinsic published output")
        for extra, diagnostic in [(["--bootstrap-intrinsics"], "requires an explicit --core-reference"),
                                  (["--bootstrap-intrinsics", "--bootstrap-intrinsics"], "Specify --bootstrap-intrinsics once")]:
            denied = output / "Invalid.dll"
            run(["dotnet", driver, "neoclr", *extra, "-o", denied, source], expected=1, diagnostic=diagnostic)
            if denied.exists():
                raise RuntimeError("Invalid bootstrap configuration published output")
        library = output / "StorageHelpers.dll"
        run([*prefix, "--bootstrap-intrinsics", "--library", "-o", library, source])
        source.unlink()
        evidence["librarySha256"] = sha(library)
        for unread in [False, True]:
            consumer = output / ("Unread.rvn" if unread else "Written.rvn")
            consumer.write_text("""import Storage.*
func Main() -> int {
    let values = Reserve<int>(2)
    values[0] = 40
    let alias = values
    ASSIGNMENT
    return values[0] + alias[1]
}
""".replace("ASSIGNMENT", "" if unread else "alias[1] = 2"))
            application = consumer.with_suffix(".dll")
            # The consumer imports native signatures; it neither compiles helper source nor enables intrinsics.
            run([*prefix, "--reference", library, "-o", application, consumer])
            run([runtime, "verify", application, "--module", contracts, "--module", library])
            run([runtime, "run", application, "--module", contracts, "--module", library],
                expected=1 if unread else 42, diagnostic="uninitialized" if unread else None, silent=not unread)
            evidence[consumer.stem] = {"sourceSha256": sha(consumer), "artifactSha256": sha(application)}
        evidence["passed"] = True
        print("PASS driver storage: separate generic helper returns 42; unread slot faults; explicit bootstrap required")
    finally:
        (output / "validation.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
