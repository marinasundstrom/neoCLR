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
    manifest['failure'] = dict(assemblyName='Failure', namespaceName='System', functionName='Fail')
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
    flow_sample = Path(__file__).resolve().parent / 'bootstrap/failure-flow-consumer.rvn'
    flow_app = output / 'Flow.dll'
    run(common + ['--reference', failure, '-o', flow_app, flow_sample])
    run([runtime, 'verify', flow_app] + dependencies)
    flow = run([runtime, 'run', flow_app] + dependencies, expected_exit=42)
    if flow.stdout or flow.stderr:
        raise RuntimeError('Successful let-else must produce no output')
    source_directory = output / 'source-flow'
    source_directory.mkdir()
    source_app = source_directory / 'Failure.dll'
    run(common + ['-o', source_app] + sources + [flow_sample])
    source_dependencies = ['--system', seed, '--module', library]
    run([runtime, 'verify', source_app] + source_dependencies)
    source_flow = run([runtime, 'run', source_app] + source_dependencies, expected_exit=42)
    if source_flow.stdout or source_flow.stderr:
        raise RuntimeError('Source-owner let-else must produce no output')
    absent_sample = output / 'absent.rvn'
    absent_sample.write_text(flow_sample.read_text().replace('Some(42)', 'None()'))
    absent_app = output / 'Absent.dll'
    run(common + ['--reference', failure, '-o', absent_app, absent_sample])
    absent = run([runtime, 'run', absent_app] + dependencies, expected_exit=1)
    if absent.stdout or not absent.stderr.startswith('Fault: missing value [code=UserFault]'):
        raise RuntimeError('Absent let-else must execute the terminal function')
    wrong_manifest = json.loads(selected.read_text())
    wrong_manifest['failure']['assemblyName'] = 'Numbers'
    wrong = output / 'wrong-owner.json'
    wrong.write_text(json.dumps(wrong_manifest, indent=2) + '\n')
    wrong_common = [wrong if part == selected else part for part in common]
    rejected = output / 'Rejected.dll'
    invalid = run(wrong_common + ['--reference', failure, '-o', rejected, sample], expected_exit=1)
    if rejected.exists() or 'failure contract' not in invalid.stderr:
        raise RuntimeError('Wrong terminal owner must reject before publication')
    inputs = sources + [Path(__file__), sample, library, ownership, selected, core, seed,
                        failure, consumer, compiler, probe, runtime, flow_sample, flow_app, source_app, absent_sample, absent_app, wrong]
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
