#!/usr/bin/env python3
"""Add the current JIT backend/host to a Windows SDK bundle and archive its catalog."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def archive_bundle(bundle, archive):
    bundle = bundle.resolve()
    manifest = bundle / 'manifest.json'
    catalog = json.loads(manifest.read_text(encoding='utf-8'))
    members = []
    for relative, digest in catalog['files'].items():
        path = (bundle / relative).resolve()
        if bundle not in path.parents or sha(path) != digest:
            raise ValueError('Bundle catalog mismatch: ' + relative)
        members.append((relative, path))
    with tarfile.open(archive, 'w:gz') as stream:
        stream.add(manifest, arcname='neoclr-native-poc/manifest.json', recursive=False)
        for relative, path in sorted(members):
            stream.add(path, arcname='neoclr-native-poc/' + relative, recursive=False)
    return dict(archiveSha256=sha(archive), manifestSha256=sha(manifest))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
        parser.error('Requires Windows x64 with MSVC initialized')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])

    def run(name, command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=1200)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(name=name, command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(name + ' failed; see retained logs')

    try:
        run('backend', ['cargo', 'build', '--locked', '--release', '--manifest-path', ROOT / 'tools/aot-poc/Cargo.toml'])
        backend = ROOT / 'target/release/neoclr-aot-poc.exe'
        project = out / 'host-source'
        project.mkdir()
        (project / 'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project="$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props"/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>JitHttp</AssemblyName></PropertyGroup></Project>', encoding='utf-8')
        shutil.copyfile(ROOT / 'docs/experiments/http-server/Server.rvn', project / 'Main.rvn')
        run('host-recipe', [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http',
                           '--project', project / 'App.rvnproj', '--bundle', args.bundle.resolve(),
                           '--aot', backend, '--output', out / 'native'])
        run('jit-bundle', [sys.executable, ROOT / 'scripts/prepare-jit-bundle.py', '--bundle', args.bundle.resolve(),
                          '--runtime', args.runtime.resolve(), '--jit-compiler', backend,
                          '--host-recipe', out / 'native/build.json', '--output', out / 'jit'])
        report.update(archive_bundle(out / 'jit/bundle', out / 'neoclr-native-poc.tar.gz'))
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
