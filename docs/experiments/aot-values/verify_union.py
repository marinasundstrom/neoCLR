#!/usr/bin/env python3
"""Build an ordinary Raven union app through native metadata to standalone ARM64."""
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
    for name in ('compiler', 'bundle', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    parser.add_argument('--sample', choices=['union-app', 'result-app', 'pattern-app'], default='union-app')
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        parser.error('Native execution requires macOS ARM64')
    compiler, bundle, runtime, aot, output = (getattr(args, key).resolve() for key in ('compiler', 'bundle', 'runtime', 'aot', 'output'))
    core, system, library, ownership = (bundle / 'lib' / name for name in ('Core.dll', 'System.runtime.neox', 'System.Runtime.dll', 'ownership.json'))
    source = ROOT / 'docs/experiments/aot-values' / (args.sample + '.rvn')
    host = ROOT / 'docs/experiments/aot-hello/main.c'
    probe = ROOT / 'docs/experiments/aot-values/pattern-deconstruction.rvn'
    inputs = [compiler, runtime, aot, core, system, library, ownership, source, host]
    if args.sample == 'pattern-app':
        inputs.append(probe)
    inputs += [compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    for path in inputs:
        if not path.is_file():
            parser.error('Missing input: ' + str(path))
    output.mkdir(parents=True, exist_ok=False)
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    report = dict(profile='raven-closed-world-union-v1', sample=args.sample, declaredCompilerRevision=args.compiler_revision,
                  neoClrBaseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  host=platform.platform(), SDKROOT=os.environ.get('SDKROOT'), commands=[], inputs={str(p): sha(p) for p in inputs})

    def save():
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')

    def run(command, expected_error=None):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
        report['commands'].append(dict(command=result.args, exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
        save()
        if expected_error is not None:
            assert result.returncode != 0 and expected_error in result.stdout + result.stderr, result
        elif result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    assembly, obj, binary = (output / name for name in (args.sample + '.dll', args.sample + '.o', args.sample))
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', system,
         '--reference', library, '--bootstrap-intrinsics', '--bootstrap-ownership', ownership,
         '--object-library', 'System.Runtime', '-o', assembly, source])
    if args.sample == 'pattern-app':
        rejected = output / 'deconstruction.dll'
        run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', system,
             '--reference', library, '--bootstrap-intrinsics', '--bootstrap-ownership', ownership,
             '--object-library', 'System.Runtime', '-o', rejected, probe],
            expected_error='BoundAssignmentStatement (BoundPatternAssignmentExpression)')
        assert not rejected.exists()
        report['deconstructionProbe'] = 'Rejected by Raven native emitter: BoundPatternAssignmentExpression'
    assert run([runtime, 'run', assembly, '--system', system, '--module', library, '--object-root', library]) == ''
    report['selection'] = json.loads(run([aot, '--closed-world', assembly, '@entry', obj]))
    run(['clang', '-arch', 'arm64', '-Wall', '-Wextra', '-Werror', host, obj, '-o', binary])
    assert run(['nm', '-u', obj]) == ''
    dependencies = [line.split()[0] for line in run(['otool', '-L', binary]).splitlines()[1:]]
    assert dependencies == ['/usr/lib/libSystem.B.dylib'], dependencies
    with tempfile.TemporaryDirectory(prefix='neoclr-native-union-') as directory:
        installed = Path(directory) / binary.name
        shutil.copy2(binary, installed)
        result = subprocess.run([str(installed)], cwd=directory, env={}, capture_output=True, timeout=10)
        assert result.returncode == 0 and result.stdout == b'' and result.stderr == b'', result
    report['native'] = dict(exit=0, stdout='', stderr='', emptyEnvironment=True,
                            executableOnlyDirectory=True, dynamicDependencies=dependencies)
    report['artifacts'] = {p.name: sha(p) for p in (assembly, obj, binary)}
    save()
    print('Passed: Raven ' + args.sample + ' -> standalone ARM64')


if __name__ == '__main__':
    main()
