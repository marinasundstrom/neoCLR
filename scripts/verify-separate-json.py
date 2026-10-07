#!/usr/bin/env python3
"""Compile source-free consumers and execute separate Runtime/Data JSON acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'runtime-library-directory', 'data', 'runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    directory = args.runtime_library_directory.resolve()
    library = directory / 'System.Runtime.dll'
    seed = directory / 'System.runtime.neox'
    ownership = directory / 'ownership.json'
    inputs = [Path(__file__), args.compiler, args.core, args.data, args.runtime, library, seed, ownership]
    inputs += [args.compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  compilerRevision=args.compiler_revision, commands=[], hashes={})
    expected = ('Model constructed\nName assigned\nModel constructed\nInvalid input begins\n'
                'Invalid input ends\nNative JSON object mapping passed\n')
    for name, stdout in [('array-reflection-consumer', ''), ('json-object-consumer', expected)]:
        source = ROOT / 'docs/experiments/extended-cli-metadata/bootstrap' / (name + '.rvn')
        artifact = output / (name + '.dll')
        compile_command = ['dotnet', args.compiler, 'neoclr', '--core-reference', args.core,
                           '--runtime-seed', seed, '--bootstrap-ownership', ownership,
                           '--reference', library, '--reference', args.data,
                           '--object-library', 'System.Runtime', '-o', artifact, source]
        common = [artifact, '--system', seed, '--module', library, '--module', args.data,
                  '--object-root', library]
        commands = [(compile_command, 0, None), ([args.runtime, 'verify'] + common, 0, None),
                    ([args.runtime, 'run'] + common + ['--instructions', '100000000'], 42, stdout)]
        for command, status, expected_stdout in commands:
            result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
            report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                           stdout=result.stdout, stderr=result.stderr))
            (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
            if result.returncode != status or expected_stdout is not None and (result.stdout != expected_stdout or result.stderr):
                raise RuntimeError('Acceptance failed: ' + name + ': ' + result.stderr)
        inputs += [source, artifact]
    report['hashes'] = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Separate Runtime/Data JSON and array reflection acceptance passed')


if __name__ == '__main__':
    main()
