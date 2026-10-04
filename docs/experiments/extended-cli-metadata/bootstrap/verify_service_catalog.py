#!/usr/bin/env python3
"""Exercise catalog-backed UTF-8, file I/O and worker callbacks with native libraries."""
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

    def run(command, expected=0, error=None):
        result = subprocess.run([str(x) for x in command], cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=[str(x) for x in command], exitCode=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or error and error not in result.stderr or expected == 42 and result.stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))

    common = ['dotnet', args.compiler.resolve(), 'neoclr', '--core-reference', args.core.resolve(),
              '--runtime-seed', args.seed.resolve(), '--bootstrap-intrinsics',
              '--bootstrap-ownership', args.ownership.resolve(), '--reference', args.base_library.resolve()]
    sources = [ROOT / 'runtime/raven/src/System/IO' / (name + '.rvn')
               for name in ('InputStream', 'OutputStream', 'SeekableStream', 'StreamError', 'MemoryStream', 'FileInputStream', 'FileOutputStream')]
    sources += [ROOT / 'runtime/raven/src/System/Text' / (name + '.rvn') for name in ('Utf8', 'InvalidUtf8Error')]
    library = output / 'ServiceLibrary.dll'
    run(common + ['--library', '-o', library] + sources)
    fixture = output / 'file.bin'
    source = output / 'ServiceConsumer.rvn'
    source.write_text((HERE / 'service-consumer.rvn').read_text().replace('__TEST_FILE__', json.dumps(str(fixture), ensure_ascii=False)[1:-1]))
    app = output / 'ServiceConsumer.dll'
    run(common + ['--reference', library, '-o', app, source])
    dependencies = ['--module', args.base_library.resolve(), '--module', library, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    if fixture.read_bytes() != bytes([40, 2]):
        raise RuntimeError('Native file mutation differs from expected bytes')
    worker = output / 'WorkerContracts.dll'
    run(common + ['--library', '-o', worker, HERE / 'worker-contracts.rvn'])
    app = output / 'WorkerConsumer.dll'
    run(common + ['--reference', worker, '-o', app, HERE / 'worker-consumer.rvn'])
    dependencies = ['--module', args.base_library.resolve(), '--module', worker, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    inputs = sources + [source, HERE / 'service-consumer.rvn', HERE / 'worker-contracts.rvn', HERE / 'worker-consumer.rvn',
                        args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
                        args.base_library.resolve(), args.ownership.resolve()] + sorted(output.glob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope='Nine unchanged source files independently compiled: UTF-8 and real file I/O. Separate worker callback contract executes; full source Tasks/Workers remains open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
