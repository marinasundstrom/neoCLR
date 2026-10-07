"""Compile and execute unchanged orders against the full source-owned native library."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'audit', 'runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--compiler-revision', required=True)
    args = parser.parse_args()
    for name in ('compiler', 'core', 'audit', 'runtime', 'output'):
        setattr(args, name, getattr(args, name).resolve())
    args.output.mkdir(parents=True, exist_ok=False)
    audit_path = args.audit / 'audit.json'
    audit = json.loads(audit_path.read_text())
    case = next(c for c in audit['cases'] if c['name'] == 'full-owned-handle')
    if case['outcome'] != 'emitted' or case['runtimeSeedFinalization']['exitCode'] != 0:
        raise ValueError('The source library and finalized runtime seed must exist')
    seed = Path(case['runtimeSeed'])
    library = seed.parent / 'Numbers.dll'
    ownership = seed.parent / 'ownership.json'
    source = ROOT / 'docs/experiments/raven-target/samples/application-order-collections.rvn'
    expected = source.with_suffix('.expected.txt')
    app = args.output / 'Orders.dll'
    commands = []

    def run(command, code=0):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (args.output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != code:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    common = ['dotnet', args.compiler, 'neoclr', '--core-reference', args.core,
              '--runtime-seed', seed, '--bootstrap-ownership', ownership, '--reference', library]
    # No class-library sources, CLI projections or consumer rewrites participate.
    run(common + ['--object-library', 'Numbers', '-o', app, source])
    dependencies = ['--system', seed, '--module', library, '--object-root', library]
    run([args.runtime, 'verify', app] + dependencies)
    result = run([args.runtime, 'run', app] + dependencies)
    if result.stdout != expected.read_text() or result.stderr:
        raise RuntimeError('Orders output differs from its checked-in expectation')
    for name, selection in [('missing', ['--object-library', 'Missing']),
                            ('conflicting', ['--object-library', 'Numbers', '--source-object-root', '--library'])]:
        rejected = args.output / (name + '.dll')
        run(common + selection + ['-o', rejected, source], 1)
        if rejected.exists():
            raise RuntimeError('Invalid root selection published output')
    inputs = [Path(__file__), audit_path, seed, library, ownership, source, expected, app, args.core, args.runtime]
    inputs += [args.compiler.parent / name for name in ('rvnc.dll', 'Raven.CodeAnalysis.dll',
                                                       'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    evidence = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                    compilerRevision=args.compiler_revision, commands=commands,
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
    (args.output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(args.output / 'validation.json')


if __name__ == '__main__':
    main()
