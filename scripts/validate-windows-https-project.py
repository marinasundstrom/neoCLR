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
COMPILER = '6ceeb1be8790e105a1394cb99fded41fafad5b4c'


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
        print(label, flush=True)
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=1200)
        (out / (label + '.stdout.log')).write_bytes(result.stdout)
        (out / (label + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(label=label, command=result.args, exitCode=result.returncode))
        if result.returncode:
            print(result.stdout[-8000:].decode('utf-8', errors='replace'), flush=True)
            print(result.stderr[-8000:].decode('utf-8', errors='replace'), flush=True)
            raise RuntimeError(label + ' failed; see retained logs')

    try:
        report['revision'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
        if args.bundle:
            bundle = args.bundle.resolve()
            manifest_path = bundle / 'manifest.json'
            manifest = json.loads(manifest_path.read_text())
            backend = bundle / manifest['jit']['compiler']
            report['bundleManifestSha256'] = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
            report['compilerRevision'] = json.loads((bundle / 'provenance.json').read_text())['libraryBuild']['compilerRevision']
            report['backendSha256'] = hashlib.sha256(backend.read_bytes()).hexdigest()
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
        if not args.bundle:
            run('aot-build', ['cargo', 'build', '--locked', '--manifest-path', ROOT / 'runtime/codegen/Cargo.toml'])
            backend = ROOT / 'target/debug/neoclr-codegen.exe'
        for label, flags in [('https', ['--https-server']), ('cleartext', [])]:
            run(label + '-build', [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile', 'windows-http',
                '--project', ROOT / 'docs/experiments/https-server/Native.rvnproj', '--bundle', bundle,
                '--aot', backend,
                '--output', out / label, *flags])
        run('https-validation', [sys.executable, ROOT / 'docs/experiments/https-server/validate.py',
            '--bundle', bundle, '--project-build', out / 'https', '--cleartext-build', out / 'cleartext',
            '--output', out / 'validation', '--windows-jit'])
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
