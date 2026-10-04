#!/usr/bin/env python3
"""Build source-owned String and execute Unicode text operations through native imports."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--ownership', type=Path, default=HERE / 'string-ownership.json')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, error=None):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    # Flatten the checked seed and remove exactly the System.String type block.
    # Keep the runtime services and bootstrap Char (grapheme) declaration.
    def expand(path):
        import re
        return re.sub(r'^\.include "([^"]+)"$', lambda m: expand(path.parent / m[1]), path.read_text(), flags=re.M)
    lines = expand(HERE / 'numeric-seed.neoil').splitlines()
    retained = []
    depth = 0
    removed = 0
    for line in lines:
        if line == '.type System.String':
            if depth or removed: raise ValueError('Duplicate String seed definition')
            depth = 1
            removed += 1
        elif depth:
            if line.startswith('.method '): depth += 1
            elif line == '.end': depth -= 1
        else:
            retained.append(line)
    if depth or removed != 1: raise ValueError('Missing or incomplete String seed definition')
    seed_source = output / 'seed.neoil'
    seed_source.write_text('\n'.join(retained) + '\n')
    seed = output / 'System.neox'
    run([args.runtime.resolve(), 'assemble', seed_source, seed, '--format', 'neox'])
    ownership = json.loads(args.ownership.read_text())
    manifest = output / 'ownership.json'
    manifest.write_text(json.dumps(ownership, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', manifest]
    sources = [ROOT / source for source in ownership['libraries'][0]['sources']]
    library = output / 'Numbers.dll'
    run(common + ['--library', '-o', library] + sources)
    app = output / 'Consumer.dll'
    consumer = HERE / 'source-string-consumer.rvn'
    run(common + ['--reference', library, '-o', app, consumer])
    dependencies = ['--module', library, '--system', seed]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    numeric = output / 'NumericConsumer.dll'
    run(common + ['--reference', library, '-o', numeric, HERE / 'native-number-consumer.rvn'])
    run([args.runtime.resolve(), 'run', numeric] + dependencies, 99)
    conflicting_seed = output / 'ConflictingSystem.neox'
    run([args.runtime.resolve(), 'assemble', HERE / 'numeric-seed.neoil', conflicting_seed, '--format', 'neox'])
    conflicting = common.copy()
    conflicting[conflicting.index('--runtime-seed') + 1] = conflicting_seed
    rejected = output / 'DuplicateOwner.dll'
    run(conflicting + ['--reference', library, '-o', rejected, consumer], 1, 'duplicates a source-owned declaration')
    if rejected.exists(): raise RuntimeError('Duplicate String ownership published output')
    missing = output / 'MissingProvider.dll'
    run(common + ['-o', missing, consumer], 1)
    if missing.exists(): raise RuntimeError('Missing String owner published output')
    inputs = sources + [consumer, manifest, seed_source, seed, args.compiler.resolve(), args.runtime.resolve(),
                        args.core.resolve(), args.ownership.resolve(), library, app, numeric, HERE / 'native-number-consumer.rvn']
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Source-owned String with numeric library; separate native consumer. Unicode text with grapheme and scalar views, UTF-8 storage. Char remains bootstrap-owned. Revisions are bases of the tested working trees.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
