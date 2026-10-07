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
    for name in ('compiler', 'core', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('Native execution requires macOS ARM64')
    compiler, core, runtime, aot, output = (getattr(args, key).resolve() for key in ('compiler', 'core', 'runtime', 'aot', 'output'))
    for path in (compiler, runtime, aot):
        if not path.is_file():
            parser.error('Missing built tool: ' + str(path))
    output.mkdir(parents=True, exist_ok=False)
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report = dict(profile='raven-value-library-v1', declaredCompilerRevision=args.compiler_revision,
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
    sources = [ROOT / 'docs/experiments/aot-library' / name for name in ('library.rvn', 'app.rvn')]
    inputs += [core] + sources
    library, assembly, obj, binary = (output / name for name in ('Values.dll', 'App.dll', 'app.o', 'app'))
    run(['dotnet', compiler, 'neoclr', '--library', '-o', library, sources[0]])
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--reference', library, '-o', assembly, sources[1]])
    assert run([runtime, 'run', assembly, '--module', library]) == ''
    inspection = json.loads(run([aot, '--inspect', assembly, '@entry', '--closed-world', '--module', library]))
    assert inspection['admission']['accepted'] is True
    report['selection'] = json.loads(run([aot, '--closed-world', assembly, '@entry', obj, '--module', library]))
    assert inspection['selection'] == report['selection']
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', ROOT / 'docs/experiments/aot-hello/main.c', obj, '-o', binary])
    assert run(['nm', '-u', obj]) == ''
    dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
    assert dependencies == ['/usr/lib/libSystem.B.dylib'], dependencies
    with tempfile.TemporaryDirectory(prefix='neoclr-aot-library-') as directory:
        installed = Path(directory) / 'app'
        shutil.copy2(binary, installed)
        result = subprocess.run([str(installed)], cwd=directory, env={}, capture_output=True, timeout=10)
        assert result.returncode == 0 and result.stdout == b'' and result.stderr == b'', result
    report['native'] = dict(artifacts={p.name: sha(p) for p in (library, assembly, obj, binary)},
                           exit=0, emptyEnvironment=True, executableOnlyDirectory=True,
                           dynamicDependencies=dependencies)
    report['inputs'] = {str(p): sha(p) for p in inputs}
    save()
    print('Passed: separate Raven value library and application -> standalone ARM64')


if __name__ == '__main__':
    main()
