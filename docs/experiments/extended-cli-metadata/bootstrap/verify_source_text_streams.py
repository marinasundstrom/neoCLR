#!/usr/bin/env python3
"""Build unchanged text streams against native encoding/text, then run artifact-only consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'base-library', 'encoding-library', 'ownership', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, expected=0, contains=None):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or expected == 42 and result.stdout or contains and contains not in result.stderr:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    names = ('StreamReader', 'StreamWriter', 'TextReader', 'TextWriter', 'TextReadError')
    sources = [ROOT / 'runtime/raven/src/System/IO' / (name + '.rvn') for name in names]
    manifest = json.loads(args.ownership.read_text())
    manifest['libraries'].append(dict(assemblyName='TextStreams', sources=[str(p.relative_to(ROOT)) for p in sources],
        types=['System.IO.' + name for name in names]))
    ownership = output / 'ownership.json'
    ownership.write_text(json.dumps(manifest, indent=2) + '\n')
    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve(), '--reference', args.encoding_library.resolve()]
    library = output / 'TextStreams.dll'
    run(common + ['--library', '-o', library] + sources)
    common[common.index('--bootstrap-ownership') + 1] = ownership
    common += ['--reference', library]
    dependencies = ['--module', args.base_library.resolve(), '--module', args.encoding_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    for name, source, expected, message in [
        ('TextStreamConsumer', HERE / 'text-stream-consumer.rvn', 42, None),
        ('MatchReturnConsumer', HERE / 'match-return-consumer.rvn', 42, None),
    ]:
        image = output / (name + '.dll')
        run(common + ['-o', image, source])
        run([args.runtime.resolve(), 'verify', image] + dependencies)
        run([args.runtime.resolve(), 'run', image] + dependencies, expected, message)
    rejected = output / 'NestedReturn.dll'
    run(common + ['-o', rejected, HERE / 'match-return-nested-rejected.rvn'], 1,
        'value block cannot exit its enclosing expression')
    if rejected.exists():
        raise RuntimeError('Unsupported nested return published output')
    inputs = sources + [args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
                        args.base_library.resolve(), args.encoding_library.resolve(), args.ownership.resolve(), ownership, Path(__file__)]
    inputs += [HERE / name for name in ('text-stream-consumer.rvn', 'match-return-consumer.rvn', 'match-return-nested-rejected.rvn')]
    inputs += sorted(output.glob('*.dll'))
    inputs += list(args.compiler.resolve().parent.glob('Raven.CodeAnalysis*.dll'))
    inputs += list(args.compiler.resolve().parent.glob('NeoCLR.Metadata*.dll'))
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    evidence = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(args.compiler.resolve().parent),
                    scope='Five unchanged text-stream sources, native artifact-only encoding/text dependencies and executable consumers. Revisions are bases of tested working trees; hashes identify binaries. JSON remains open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
