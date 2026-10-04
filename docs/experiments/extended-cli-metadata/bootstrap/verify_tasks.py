#!/usr/bin/env python3
"""Compile source-owned Tasks/Workers and execute separate native consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--ownership', type=Path, default=HERE / 'offset-ownership.json')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, error=None, stdout=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout != (stdout or ""):
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    manifest = json.loads(args.ownership.read_text())
    declaration = json.loads((HERE / 'tasks-library.json').read_text())
    manifest['libraries'].append(declaration)
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', ownership, '--reference', args.base_library.resolve()]
    library = output / (declaration['assemblyName'] + '.dll')
    run(common + ['--library', '-o', library] + [ROOT / source for source in declaration['sources']])
    dependencies = ['--module', args.base_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    for name, source, expected, error in [('Consumer', 'tasks-consumer.rvn', 42, None),
                                          ('WrongQueue', 'tasks-wrong-queue.rvn', 1, 'source-owned TaskQueue contract'),
                                          ('DuplicateQueue', 'tasks-duplicate-queue.rvn', 1, 'registered once')]:
        app = output / (name + '.dll')
        run(common + ['--reference', library, '-o', app, HERE / source])
        run([args.runtime.resolve(), 'verify', app] + dependencies)
        run([args.runtime.resolve(), 'run', app] + dependencies, expected, error)
    app = output / 'EntryDrain.dll'
    run(common + ['--reference', library, '-o', app, HERE / 'tasks-entry-drain.rvn'])
    run([args.runtime.resolve(), 'run', app] + dependencies, 42, stdout='drained\n')
    inputs = [ROOT / source for source in declaration['sources']]
    inputs += [HERE / name for name in ('tasks-library.json', 'tasks-consumer.rvn', 'tasks-wrong-queue.rvn', 'tasks-duplicate-queue.rvn', 'tasks-entry-drain.rvn')] + [ownership]
    inputs += [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
               args.base_library.resolve(), args.ownership.resolve()] + sorted(output.glob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Six actual Tasks/Concurrency source files build as a separate native library. Consumer execution covers scheduling, unit values, callback composition, cancellation, worker results, identity, interface callbacks and entry drain. Source queue service calls explicitly select their own type; generic async language lowering remains outside this gate.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
