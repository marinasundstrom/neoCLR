#!/usr/bin/env python3
"""Compile the Raven value samples through neoCLR metadata to native ARM64."""
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
    for name in ('compiler', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    parser.add_argument('--samples', nargs='+', choices=('counter', 'copies', 'nested', 'outputs'), default=['counter', 'copies', 'nested', 'outputs'])
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('Native execution requires macOS ARM64')
    compiler, runtime, aot, output = (getattr(args, key).resolve() for key in ('compiler', 'runtime', 'aot', 'output'))
    for path in (compiler, runtime, aot):
        if not path.is_file():
            parser.error('Missing built tool: ' + str(path))
    output.mkdir(parents=True, exist_ok=False)
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report = dict(profile='raven-output-values-v3', declaredCompilerRevision=args.compiler_revision,
                  neoClrBaseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  host=platform.platform(), SDKROOT=os.environ.get('SDKROOT'), commands=[], samples={})

    def save():
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
        report['commands'].append(dict(command=result.args, exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
        save()
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    inputs = [compiler, runtime, aot, ROOT / 'docs/experiments/aot-hello/main.c']
    inputs += [compiler.parent / n for n in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    for name in args.samples:
        source = ROOT / f'docs/experiments/aot-values/{name}.rvn'
        assembly, obj, binary = (output / (name + extension) for extension in ('.dll', '.o', ''))
        inputs.append(source)
        run(['dotnet', compiler, 'neoclr', '-o', assembly, source])
        assert run([runtime, 'run', assembly]) == ''
        run([aot, assembly, '@entry', obj])
        run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', ROOT / 'docs/experiments/aot-hello/main.c', obj, '-o', binary])
        assert run(['nm', '-u', obj]) == ''
        dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
        assert dependencies == ['/usr/lib/libSystem.B.dylib'], dependencies
        with tempfile.TemporaryDirectory(prefix='neoclr-aot-values-') as directory:
            installed = Path(directory) / name
            shutil.copy2(binary, installed)
            result = subprocess.run([str(installed)], cwd=directory, env={}, capture_output=True, timeout=10)
            assert result.returncode == 0 and result.stdout == b'' and result.stderr == b'', result
        report['samples'][name] = dict(artifacts={p.name: sha(p) for p in (assembly, obj, binary)},
                                      exit=0, stdout='', stderr='', emptyEnvironment=True,
                                      executableOnlyDirectory=True, dynamicDependencies=dependencies)
    report['inputs'] = {str(p): sha(p) for p in inputs}
    save()
    print('Passed: Raven value samples -> neoCLR IL -> independent ARM64 executables')


if __name__ == '__main__':
    main()
