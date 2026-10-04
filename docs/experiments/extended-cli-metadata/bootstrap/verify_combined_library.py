#!/usr/bin/env python3
"""Build the combined source subset as a schema-3 library and execute separate consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'core', 'seed', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--ownership', type=Path, default=HERE / 'offset-ownership.json')
    parser.add_argument('--services', action='store_true', help='Also compile source UTF-8 and file services')
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
              '--bootstrap-ownership', output / 'ownership.json']
    sources = [ROOT / 'runtime/raven/src/System/IO' / (name + '.rvn')
               for name in ('InputStream', 'OutputStream', 'SeekableStream', 'StreamError', 'MemoryStream')]
    if args.services:
        sources += [ROOT / 'runtime/raven/src/System/IO' / (name + '.rvn') for name in ('FileInputStream', 'FileOutputStream')]
        sources += [ROOT / 'runtime/raven/src/System/Text' / (name + '.rvn') for name in ('Utf8', 'InvalidUtf8Error')]
    manifest = json.loads(args.ownership.read_text())
    declaration = manifest['libraries'][0]
    declaration['sources'] += [str(p.relative_to(ROOT)) for p in sources]
    declaration['types'] += ['System.IO.' + p.stem if p.parent.name == 'IO' else 'System.Text.' + p.stem for p in sources]
    manifest['unit'] = dict(assemblyName='NeoCLR.CoreProbe', typeName='System.Void')
    (output / 'ownership.json').write_text(json.dumps(manifest, indent=2) + '\n')
    sources = [ROOT / p for p in declaration['sources']]
    library = output / (declaration['assemblyName'] + '.dll')
    run(common + ['--library', '-o', library] + sources)
    if library.stat().st_size <= 1024 * 1024:
        raise RuntimeError('Combined fixture no longer exceeds the old envelope; review this gate')
    fixture = output / 'file.bin'
    source = output / 'ServiceConsumer.rvn'
    source.write_text((HERE / ('service-consumer.rvn' if args.services else 'stream-consumer.rvn')).read_text().replace('__TEST_FILE__', json.dumps(str(fixture), ensure_ascii=False)[1:-1]))
    app = output / 'ServiceConsumer.dll'
    run(common + ['--reference', library, '-o', app, source])
    dependencies = ['--module', library, '--system', args.seed.resolve()]
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies, 42)
    if args.services and fixture.read_bytes() != bytes([40, 2]):
        raise RuntimeError('Native file mutation differs from expected bytes')
    broad = ROOT / 'docs/experiments/raven-target/samples/application-order-collections.rvn'
    app = output / 'Application.dll'
    run(common + ['--reference', library, '-o', app, broad])
    run([args.runtime.resolve(), 'verify', app] + dependencies)
    run([args.runtime.resolve(), 'run', app] + dependencies)
    if commands[-1]['stdout'].replace('\r\n', '\n') != broad.with_suffix('.expected.txt').read_text():
        raise RuntimeError('Broad application output differs')
    inputs = sources + [source, HERE / ('service-consumer.rvn' if args.services else 'stream-consumer.rvn'), broad, broad.with_suffix('.expected.txt'),
                        args.compiler.resolve(), args.runtime.resolve(), args.core.resolve(), args.seed.resolve(),
                        output / 'ownership.json'] + sorted(output.glob('*.dll'))
    revision = lambda directory: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=directory, text=True).strip()
    evidence = dict(runtimeRepositoryRevision=revision(ROOT), compilerRepositoryRevision=revision(args.compiler.resolve().parent),
                    scope=f'{len(sources)} unchanged sources in one native library PE >1 MiB. Source-free stream and unchanged broad application consumers execute; services={args.services}; full System remains open.',
                    hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=commands)
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
