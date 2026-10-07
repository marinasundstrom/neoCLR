#!/usr/bin/env python3
"""Record separate optional-library compilation against an emitted Runtime candidate."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'runtime-library-directory', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    parser.add_argument('--runtime', type=Path, help='Also compile and execute the unchanged networking consumer')
    parser.add_argument('--include-web', action='store_true', help='Build Web against the emitted Data and Networking artifacts')
    args = parser.parse_args()
    directory = args.runtime_library_directory.resolve()
    compiler = args.compiler.resolve()
    core = args.core.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    ownership = directory / 'ownership.json'
    library = directory / 'System.Runtime.dll'
    seed = directory / 'System.runtime.neox'
    manifest = json.loads(ownership.read_text())
    if manifest['libraries'][0]['assemblyName'] != 'System.Runtime':
        raise ValueError('Expected the explicitly selected System.Runtime candidate')
    inputs = [Path(__file__), compiler, core, ownership, library, seed]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  compilerRevision=args.compiler_revision,
                  scope='Compilation frontier only; emission is not execution acceptance.', cases=[])
    artifacts = {}
    groups = ('Data', 'Networking', 'Web') if args.include_web else ('Data', 'Networking')
    for group in groups:
        if group == 'Web' and any(case['exitCode'] != 0 for case in report['cases']):
            raise RuntimeError('Data and Networking must compile before the Web audit')
        sources = sorted((ROOT / 'runtime/raven/src/System' / group).rglob('*.rvn'))
        if group == 'Networking':
            sources += [ROOT / 'runtime/raven/native' / (name + '.rvn')
                        for name in ('RuntimeNetworkCalls', 'RuntimeNetworkServices')]
        inputs += sources
        artifact = output / ('System.' + group + '.dll')
        artifacts[group] = artifact
        references = [library] if group != 'Web' else [library, artifacts['Data'], artifacts['Networking']]
        command = ['dotnet', compiler, 'neoclr', '--core-reference', core,
                   '--runtime-seed', seed, '--bootstrap-ownership', ownership,
                   '--object-library', 'System.Runtime', '--bootstrap-intrinsics',
                   '--library', '-o', artifact]
        for reference in references:
            command += ['--reference', reference]
        command += sources
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
        entry = dict(name=group, command=result.args, exitCode=result.returncode,
                     stdout=result.stdout, stderr=result.stderr, sourceCount=len(sources),
                     outputPublished=artifact.exists())
        if artifact.exists():
            inputs.append(artifact)
        report['cases'].append(entry)
        report['hashes'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        (output / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode != 0 and artifact.exists():
            raise RuntimeError('Failed compilation published an output artifact')
        print(group, result.returncode, flush=True)
    if args.runtime:
        if next(case for case in report['cases'] if case['name'] == 'Networking')['exitCode'] != 0:
            raise RuntimeError('Networking must compile before execution acceptance')
        artifact = artifacts['Networking']
        runtime = args.runtime.resolve()
        source = ROOT / 'docs/experiments/network-cancellation/Main.rvn'
        consumer = output / 'NetworkConsumer.dll'
        common = ['--system', seed, '--module', library, '--module', artifact,
                  '--object-root', library]
        commands = [
            ['dotnet', compiler, 'neoclr', '--core-reference', core,
             '--runtime-seed', seed, '--bootstrap-ownership', ownership,
             '--reference', library, '--reference', artifact, '--object-library',
             'System.Runtime', '-o', consumer, source],
            [runtime, 'verify', consumer] + common,
            [runtime, 'run', consumer] + common,
        ]
        report['scope'] = 'Optional-library compilation frontier and source-free networking execution.'
        report['execution'] = []
        for command in commands:
            result = subprocess.run(list(map(str, command)), cwd=ROOT,
                                    capture_output=True, text=True, timeout=180)
            report['execution'].append(dict(command=result.args, exitCode=result.returncode,
                                            stdout=result.stdout, stderr=result.stderr))
            (output / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
            if result.returncode != 0:
                raise RuntimeError('Networking acceptance failed: ' + str(command[1]))
        expected = 'Network token cancellation checks passed\n'
        if result.stdout != expected:
            raise RuntimeError('Networking stdout mismatch: ' + repr(result.stdout))
        report['expectedStdout'] = expected
        report['hashes'].update({str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                                 for p in (runtime, source, consumer)})
        (output / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
        print('Networking execution passed', flush=True)


if __name__ == '__main__':
    main()
