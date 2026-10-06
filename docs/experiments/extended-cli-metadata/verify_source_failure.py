"""Compile unchanged System.Fail and verify its terminal fault through a separate native consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'probe', 'library', 'ownership', 'seed', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    compiler, probe, library, ownership, seed, runtime, output = (
        getattr(args, name).resolve() for name in
        ('compiler', 'probe', 'library', 'ownership', 'seed', 'runtime', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    sources = [ROOT / path for path in ('runtime/raven/src/System/Functions.rvn',
        'runtime/raven/native/RuntimeFailure.rvn', 'runtime/raven/native/RuntimeFailureCalls.rvn')]
    manifest = json.loads(ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='Failure',
        sources=[str(path.relative_to(ROOT)) for path in sources],
        types=['System.Runtime.CompilerServices.RuntimeFailure']))
    selected = output / 'ownership.json'
    selected.write_text(json.dumps(manifest, indent=2) + '\n')
    core, failure, consumer = [output / name for name in ('Core.dll', 'Failure.dll', 'Consumer.dll')]
    commands = []

    def run(command, expected_exit=0):
        command = [str(part) for part in command]
        result = subprocess.run(command, cwd=output, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=command, cwd=str(output), exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected_exit:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    run(['dotnet', probe, '--reference-source-failure-core', core])
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
              '--bootstrap-ownership', selected, '--reference', library]
    run(common + ['--library', '-o', failure] + sources)
    sample = Path(__file__).resolve().parent / 'bootstrap/failure-consumer.rvn'
    run(common + ['--reference', failure, '-o', consumer, sample])
    dependencies = ['--system', seed, '--module', library, '--module', failure]
    run([runtime, 'verify', consumer] + dependencies)
    result = run([runtime, 'run', consumer] + dependencies, expected_exit=1)
    if result.stdout or not result.stderr.startswith('Fault: source failure [code=UserFault]'):
        raise RuntimeError('Expected the source Fail diagnostic and no consumer output')
    inputs = sources + [Path(__file__), sample, library, ownership, selected, core, seed,
                        failure, consumer, compiler, probe, runtime]
    inputs += [compiler.parent / name for name in
               ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    inputs += [ROOT / path for path in ('src/native.rs', 'tests/fault_codes.rs',
               'docs/experiments/raven-target/CoreDeclarations.cs', 'docs/experiments/raven-target/Program.cs')]
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    evidence = dict(runtimeRevision=revision, compilerRevision=args.compiler_revision,
                    commands=commands,
                    hashes={str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
