#!/usr/bin/env python3
"""Raven -> neoCLR metadata/IL -> native ARM64 Hello World (macOS experiment)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True, help='Native-enabled rvnc.dll')
    parser.add_argument('--compiler-revision', required=True, help='Source revision of the selected compiler')
    parser.add_argument('--aot', type=Path, required=True, help='Built neoclr-aot-poc executable')
    parser.add_argument('--output', type=Path, required=True, help='Fresh evidence/artifact directory')
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('Native execution qualification requires macOS ARM64')
    compiler, aot, output = (p.resolve() for p in (args.compiler, args.aot, args.output))
    if not compiler.is_file() or not aot.is_file():
        parser.error('Compiler and AOT tool must already be built')
    output.mkdir(parents=True, exist_ok=False)
    source = ROOT / 'docs/experiments/aot-hello/hello.rvn'
    host = ROOT / 'docs/experiments/aot-hello/main.c'
    service = ROOT / 'docs/experiments/aot-scalar/console.c'
    assembly, obj, binary = (output / p for p in ('RavenHello.dll', 'hello.o', 'hello'))
    report = {'profile': 'raven-neoclr-cil-hello-v1', 'compilerRevision': args.compiler_revision,
              'neoClrBaseRevision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'host': platform.platform(), 'SDKROOT': os.environ.get('SDKROOT'), 'commands': []}

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
        report['commands'].append(dict(command=result.args, exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    # The selected compiler's existing primitive/console bootstrap emits #Neo bodies.
    # This does not emit a .NET program and import its CLI projection for AOT.
    run(['dotnet', compiler, 'neoclr', '-o', assembly, source])
    run([aot, assembly, '@entry', obj, '--console'])
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, service, obj, '-o', binary])
    dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
    assert dependencies == ['/usr/lib/libSystem.B.dylib'], dependencies
    with tempfile.TemporaryDirectory(prefix='neoclr-raven-aot-') as isolated:
        installed = Path(isolated) / 'hello'
        shutil.copy2(binary, installed)
        result = subprocess.run([str(installed)], cwd=isolated, env={}, capture_output=True, timeout=10)
        assert result.returncode == 0 and result.stdout == b'Hello, world!\n' and result.stderr == b'', result
        assert list(Path(isolated).iterdir()) == [installed]
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    inputs = [source, host, service, compiler, aot]
    inputs += [compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report.update(inputs={str(p): sha(p) for p in inputs},
                  artifacts={p.name: sha(p) for p in (assembly, obj, binary)},
                  execution=dict(stdout='Hello, world!\n', stderr='', exit=0, emptyEnvironment=True,
                                 executableOnlyDirectory=True, dynamicDependencies=dependencies),
                  limits=['explicit compiler primitive/console bootstrap', 'literal console intrinsic',
                          'no general library dependency compilation', 'no trimming', 'no HTTP Server'])
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed: Raven source -> neoCLR metadata/IL -> standalone ARM64 Hello World')


if __name__ == '__main__':
    main()
