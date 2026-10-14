#!/usr/bin/env python3
"""Build and verify the bounded native structural union/intersection POC."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--raven', required=True, type=Path, help='Matching Raven feature checkout')
    parser.add_argument('--library', required=True, type=Path, help='Matching native library with structural TypeInfo APIs')
    args = parser.parse_args()
    raven, library = args.raven.resolve(), args.library.resolve()
    output = ROOT / 'target/structural-poc'
    output.mkdir(parents=True, exist_ok=True)
    report = {'status': 'running', 'scope': 'Closed-reference interpreter POC', 'checks': []}
    report_path = output / 'results.json'

    def save():
        report_path.write_text(json.dumps(report, indent=2) + '\n')

    def run(label, command, cwd=ROOT, expected=0):
        print(label, flush=True)
        log = output / (str(len(report['checks']) + 1).zfill(2) + '.log')
        started = time.monotonic()
        record = {'label': label, 'command': [str(x) for x in command], 'cwd': str(cwd), 'expectedExitCode': expected, 'log': str(log)}
        report['checks'].append(record)
        save()
        with log.open('w') as stream:
            result = subprocess.run(record['command'], cwd=cwd, stdout=stream, stderr=subprocess.STDOUT, timeout=900)
        record.update(exitCode=result.returncode, seconds=round(time.monotonic() - started, 2))
        save()
        if result.returncode != expected:
            raise RuntimeError(f'{label}: expected exit {expected}, got {result.returncode}; see {log}')
        print(f'PASS {label}', flush=True)

    metadata = ROOT / 'tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj'
    properties = [f'-p:NeoClrMetadataProject={metadata}', '-p:UseRavenCoreReference=false', '-p:WarningLevel=0']
    try:
        for file in ['Core.dll', 'System.Runtime.dll', 'System.runtime.neox', 'NeoCLR.ClassLibrary.props', 'bundle.json']:
            if not (library / file).is_file():
                raise RuntimeError(f'Missing native library prerequisite: {library / file}')
        report['revisions'] = {name: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
                               for name, path in [('neoclr', ROOT), ('raven', raven)]}
        report['workingTreeChanges'] = {name: subprocess.check_output(['git', 'status', '--porcelain'], cwd=path, text=True)
                                      for name, path in [('neoclr', ROOT), ('raven', raven)]}
        run('Default runtime rejects compositions', ['cargo', 'test', '-p', 'neoclr', '--test', 'composition_constraints'])
        run('Native structural semantics and constraint proofs', ['cargo', 'test', '-p', 'neoclr', '--features', 'experimental-compositions', '--test', 'structural_compositions', '--test', 'composition_constraints', '--test', 'self_types', '--test', 'generic_bounds'])
        run('Build experimental interpreter', ['cargo', 'build', '-p', 'neoclr', '--features', 'experimental-compositions', '--bin', 'neoclr'])
        run('Metadata authoring and negative constraints', ['dotnet', 'run', '--project', 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '--', '--authored-structural-constraints', output / 'Authored.neox'])
        for flag in ['--structural-method-constraints', '--open-constrained-calls']:
            run('Metadata ' + flag, ['dotnet', 'run', '--project', 'tools/metadata/NeoCLR.Metadata.Experimental.Tests', '--no-build', '--', flag])
        run('Build matching Raven compiler', ['dotnet', 'build', 'src/Raven.Compiler/Raven.Compiler.csproj', '-f', 'net10.0', *properties], cwd=raven)
        run('Raven native constraint emission and import', ['dotnet', 'run', '--project', 'tools/NeoClrMetadataProbe', *properties, '--', '--structural-constraints', library, output / 'constraints'], cwd=raven)
        run('Raven native source signature roundtrip', ['dotnet', 'run', '--project', 'tools/NeoClrMetadataProbe', '--no-build', '--', '--source-compositions', library, output / 'signatures'], cwd=raven)
        compiler = raven / 'src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll'
        for kind in ['unions', 'intersections']:
            run('Source structural ' + kind, ['python3', ROOT / f'docs/experiments/source-structural-{kind}/verify.py', '--compiler', compiler, '--library', library, '--output', output / kind])
        runtime = ROOT / 'target/debug/neoclr'
        common = ['--system', library / 'System.runtime.neox', '--module', library / 'System.Runtime.dll', '--object-root', library / 'System.Runtime.dll', '--execution', 'interpreter', '--instructions', '100000000']
        for consumer, dependency in [('ConstraintSource', None), ('ConstraintConsumer', 'ConstraintLibrary'), ('TraitSource', None), ('ExternalTraits', 'TraitLibrary'), ('ConstructedTraits', 'TraitLibrary')]:
            command = [runtime, 'run', output / 'constraints' / (consumer + '.neox'), *common]
            if dependency:
                command += ['--module', output / 'constraints' / (dependency + '.neox')]
            run(consumer + ' execution', command, expected=42)
        run('External union constraint consumer execution', [runtime, 'run', output / 'constraints/ExternalBoundConsumer.neox', *common,
            '--module', output / 'constraints/ExternalBoundLibrary.neox', '--module', output / 'constraints/TraitLibrary.neox'], expected=42)
        for kind in ['Union', 'Intersection']:
            run(kind + ' separate consumer execution', [runtime, 'run', output / 'signatures' / f'Authored{kind}Consumer.neox', *common, '--module', output / 'signatures' / f'Authored{kind}.neox'], expected=42)
        run('Host-authored constraints execution', [runtime, 'run', output / 'Authored.neox', '--execution', 'interpreter'], expected=42)
        run('API documentation snapshot', ['python3', 'scripts/build-api-docs.py', '--check'])
        files = [runtime, compiler, metadata.parent / 'Model/MethodILGenerator.cs',
                 ROOT / 'docs/experiments/source-structural-unions/Main.rvn',
                 ROOT / 'docs/experiments/source-structural-intersections/Main.rvn',
                 raven / 'tools/NeoClrMetadataProbe/StructuralConstraintChecks.cs',
                 *[library / name for name in ['Core.dll', 'System.Runtime.dll', 'System.runtime.neox']],
                 *output.glob('constraints/*.neox'), *output.glob('signatures/*.neox')]
        report['sha256'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in files}
        report['status'] = 'passed'
    except Exception as error:
        report['status'] = 'failed'
        report['error'] = str(error)
        raise
    finally:
        save()
    print(f'POC acceptance passed: {report_path}', flush=True)


if __name__ == '__main__':
    main()
