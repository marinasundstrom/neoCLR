#!/usr/bin/env python3
"""Build and qualify the shared HTTPS server cases on Windows x64."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import subprocess
import sys
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
COMPILER = '371d1d7078bffa05e5e2607b6d3cd3833d6e8404'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, help='Matching development bundle; otherwise rebuild from pinned bootstrap')
    args = parser.parse_args()
    if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
        parser.error('requires Windows x64 with MSVC in PATH')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), compilerRevision=COMPILER, commands=[])

    def run(label, command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=1200)
        (out / (label + '.stdout.log')).write_bytes(result.stdout)
        (out / (label + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(label=label, command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(label + ' failed; see retained logs')

    try:
        report['revision'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        if args.bundle:
            bundle = args.bundle.resolve()
        else:
            spec = importlib.util.spec_from_file_location('windows_project', ROOT / 'scripts/validate-windows-project.py')
            bootstrap = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(bootstrap)
            archive = out / bootstrap.ARCHIVE
            urllib.request.urlretrieve(bootstrap.URL, archive)
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            if digest != bootstrap.BUNDLE_SHA:
                raise ValueError('Bootstrap archive hash mismatch')
            report['bootstrap'] = dict(url=bootstrap.URL, sha256=digest)
            with tarfile.open(archive) as source:
                source.extractall(out, filter='data')
            run('development-bundle', [sys.executable, ROOT / 'scripts/prepare-native-development-bundle.py',
                '--bundle', out / 'neoclr-native-poc', '--output', out / 'development',
                '--build-compiler', '--compiler-revision', COMPILER])
            bundle = out / 'development/bundle'
        run('aot-build', ['cargo', 'build', '--locked', '--manifest-path', ROOT / 'tools/aot-poc/Cargo.toml'])
        for label, flags in [('https', ['--https-server']), ('cleartext', [])]:
            run(label + '-build', [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http',
                '--project', ROOT / 'docs/experiments/https-server/Native.rvnproj', '--bundle', bundle,
                '--aot', ROOT / 'tools/aot-poc/target/debug/neoclr-aot-poc.exe',
                '--output', out / label, *flags])
        run('https-validation', [sys.executable, ROOT / 'docs/experiments/https-server/validate.py',
            '--bundle', bundle, '--project-build', out / 'https', '--cleartext-build', out / 'cleartext',
            '--output', out / 'validation'])
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
