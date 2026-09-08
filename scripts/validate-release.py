#!/usr/bin/env python3
"""Validate a committed source archive without tagging or publishing it (Python 3.9+)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tarfile
import tempfile


def run(argv, cwd, capture=False, env=None):
    print("+ " + " ".join(map(str, argv)), flush=True)
    result = subprocess.run(list(map(str, argv)), cwd=cwd, env=env, check=True,
                            stdout=subprocess.PIPE if capture else None, text=True, encoding="utf-8")
    return result.stdout if capture else None


def audit_notices(source):
    manifest = json.loads((source / "third-party/manifest.json").read_text(encoding="utf-8"))
    packages = {}
    for block in (source / "Cargo.lock").read_text(encoding="utf-8").split("[[package]]")[1:]:
        fields = dict(re.findall(r'^([a-z_]+) = "([^"\n]*)"$', block, re.MULTILINE))
        if fields.get("source", "").startswith("registry+"):
            packages[(fields["name"], fields["version"])] = fields
    declared = {(p["name"], p["version"]): p for p in manifest["packages"]}
    if declared.keys() != packages.keys():
        raise RuntimeError("notice inventory differs from locked registry packages")
    count = 0
    for identity, package in declared.items():
        if package["checksum"] != packages[identity]["checksum"]:
            raise RuntimeError("locked dependency checksum differs from notice manifest")
        for notice in package["notices"]:
            path = (source / notice["path"]).resolve()
            if source not in path.parents:
                raise RuntimeError("notice path escapes source archive")
            if hashlib.sha256(path.read_bytes()).hexdigest() != notice["sha256"]:
                raise RuntimeError("notice hash mismatch: " + notice["path"])
            count += 1
    return len(packages), count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--revision", default="HEAD")
    parser.add_argument("--toolchain", default="stable")
    parser.add_argument("--output", type=Path, help="new directory for archive, extraction and report")
    parser.add_argument("--smoke-only", action="store_true", help="skip full tests; report explicitly records this")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    output = args.output.resolve() if args.output else Path(tempfile.mkdtemp(prefix="neoclr-release-"))
    if args.output:
        output.mkdir(parents=True, exist_ok=False)
    report = {"status": "running", "platform": platform.platform(), "toolchain": args.toolchain,
              "full_tests": False, "smoke_programs": [],
              "validator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    print("Validation output: " + str(output), flush=True)
    try:
        revision = run(["git", "rev-parse", "--verify", args.revision + "^{commit}"], repo, True).strip()
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise RuntimeError("revision did not resolve to one commit")
        report["commit"] = revision
        archive = output / "neoclr-source.tar"
        run(["git", "-c", "core.autocrlf=false", "-c", "core.eol=lf", "archive", "--format=tar", "--prefix=neoclr-source/", "--output=" + str(archive), revision], repo)
        report["archive_sha256"] = hashlib.sha256(archive.read_bytes()).hexdigest()
        expected = set(run(["git", "ls-tree", "-r", "--name-only", revision], repo, True).splitlines())
        with tarfile.open(archive) as tar:
            actual = set()
            for member in tar.getmembers():
                path = Path(member.name)
                if path.is_absolute() or ".." in path.parts or path.parts[0] != "neoclr-source":
                    raise RuntimeError("unsafe archive member")
                if not (member.isfile() or member.isdir()):
                    raise RuntimeError("source archive contains a link or special file")
                if member.isfile():
                    actual.add(path.relative_to("neoclr-source").as_posix())
            if actual != expected:
                raise RuntimeError("archive membership differs from tracked tree")
            # Every member was checked above; permit Python versions predating filters.
            tar.extractall(output)
        report["tracked_files"] = len(actual)
        source = (output / "neoclr-source").resolve()
        for name in ["Cargo.toml", "Cargo.lock", "build.rs", "LICENSE", "README.md",
                     "CHANGELOG.md", "AGENTS.md", "THIRD_PARTY_NOTICES.md",
                     "runtime/System.neoil", ".github/workflows/ci.yml"]:
            if not (source / name).is_file():
                raise RuntimeError("required source member missing: " + name)
        report["registry_packages"], report["notice_files"] = audit_notices(source)
        env = dict(os.environ)
        env["CARGO_TARGET_DIR"] = str(output / "target")
        report["rustc"] = run(["rustc", "+" + args.toolchain, "-Vv"], source, True, env).strip()
        cargo = ["cargo", "+" + args.toolchain]
        if not args.smoke_only:
            run(cargo + ["test", "--locked", "--all-targets", "--no-fail-fast"], source, env=env)
            report["full_tests"] = True
        run(cargo + ["build", "--locked"], source, env=env)
        executable = output / "target/debug" / ("neoclr.exe" if os.name == "nt" else "neoclr")
        for name in ["counter", "collections", "outputs", "reflection", "reference-identity",
                     "interfaces", "arrays", "control-flow", "typeof"]:
            program = source / "examples/source" / (name + ".neo")
            artifact = output / (name + ".neo.json")
            run([executable, "verify", program], source)
            direct = run([executable, "run", program], source, True)
            run([executable, "assemble", program, artifact], source)
            run([executable, "verify", artifact], source)
            restored = run([executable, "run", artifact], source, True)
            if direct != restored:
                raise RuntimeError("source/artifact output mismatch: " + name)
            report["smoke_programs"].append(name)
        run(cargo + ["run", "--locked", "--example", "build_native"], source, env=env)
        run([executable, "run", "examples/pinvoke.neoil"], source)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = str(error)
        raise
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print("Evidence: " + str(output / "report.json"), flush=True)


if __name__ == "__main__":
    main()
