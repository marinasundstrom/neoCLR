#!/usr/bin/env python3
"""Compile and execute source-free consumers of separate Runtime/Data/Networking/Web assemblies."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'runtime-library-directory', 'data', 'networking', 'web', 'runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    directory = args.runtime_library_directory.resolve()
    library = directory / 'System.Runtime.dll'
    seed = directory / 'System.runtime.neox'
    ownership = directory / 'ownership.json'
    inputs = [Path(__file__), args.compiler, args.core, args.data, args.networking, args.web, args.runtime, library, seed, ownership]
    inputs += [args.compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  compilerRevision=args.compiler_revision, commands=[], hashes={})
    cases = [
        ('deadline', ['extended-cli-metadata/bootstrap/network-deadline-consumer.rvn'], 'Network deadline checks passed\n'),
        ('headers', ['http-headers/Main.rvn'], 'HTTP header lookup checks passed\n'),
        ('base', ['http-base/Main.rvn'], 'HTTP base address and overload checks passed\n'),
        ('json', ['http-json-client/Main.rvn'], 'HTTP JSON client checks passed\n'),
        ('routes', ['http-routing/Routes.rvn', 'http-routing/Direct.rvn', 'http-routing/Main.rvn'], 'Route parsing checks passed\n'),
    ]
    for name, paths, stdout in cases:
        sources = [ROOT / 'docs/experiments' / path for path in paths]
        artifact = output / (name + '.dll')
        compile_command = ['dotnet', args.compiler, 'neoclr', '--core-reference', args.core,
                           '--runtime-seed', seed, '--bootstrap-ownership', ownership,
                           '--reference', library, '--reference', args.data, '--reference', args.networking, '--reference', args.web,
                           '--object-library', 'System.Runtime', '--async-library', 'System.Runtime', '-o', artifact] + sources
        common = [artifact, '--system', seed, '--module', library, '--module', args.data,
                  '--module', args.networking, '--module', args.web, '--object-root', library]
        commands = [(compile_command, 0, None), ([args.runtime, 'verify'] + common, 0, None),
                    ([args.runtime, 'run'] + common + ['--instructions', '100000000'], 0, stdout)]
        for command, status, expected_stdout in commands:
            result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
            report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                           stdout=result.stdout, stderr=result.stderr))
            (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
            if result.returncode != status or expected_stdout is not None and (result.stdout != expected_stdout or result.stderr):
                raise RuntimeError('Acceptance failed: ' + name + ': ' + result.stderr)
        inputs += sources + [artifact]
    report['hashes'] = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    (output / 'evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Separate Runtime/Data/Networking/Web acceptance passed')


if __name__ == '__main__':
    main()
