"""Compile unchanged union library sources, then execute a reference-only consumer.

Build the compiler and generate a primitive storage core with the documented
reference-storage-core command first. No source/seed union projections are used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--core', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--collections', action='store_true', help='Verify separately compiled ArrayList including callback import')
    parser.add_argument('--hashmap', action='store_true', help='Verify cumulative HashMap and comparer source library')
    parser.add_argument('--extensions', action='store_true', help='Also verify a separate generic extension library and consumer')
    args = parser.parse_args()
    args.hashmap = args.hashmap or args.extensions
    args.collections = args.collections or args.hashmap
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    compiler, runtime, core = (str(p.resolve()) for p in (args.compiler, args.runtime, args.core))
    commands = []

    def run(command, expected=0, stdout=None):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
        commands.append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        if result.returncode != expected or stdout is not None and result.stdout != stdout:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    manifest = HERE / ('hashmap-ownership.json' if args.hashmap else 'arraylist-ownership.json' if args.collections else 'union-ownership.json')
    consumer_source = HERE / ('hashmap-consumer.rvn' if args.hashmap else 'arraylist-consumer.rvn' if args.collections else 'union-consumer.rvn')
    seed_source = HERE / ('collection-seed.neoil' if args.collections else 'union-seed.neoil')
    sources = json.loads(manifest.read_text())['libraries'][0]['sources']
    seed = output / 'System.neox'
    library = output / 'NeoCLR.Collections.dll'
    consumer = output / 'Consumer.dll'
    run([runtime, 'assemble', str(seed_source), str(seed), '--format', 'neox'])
    common = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', str(seed),
              '--bootstrap-ownership', str(manifest), '--bootstrap-intrinsics']
    run(common + ['--library', '-o', str(library)] + sources)
    consumer_command = common + ['--reference', str(library), '-o', str(consumer), str(consumer_source)]
    run(consumer_command)
    run([runtime, 'verify', str(consumer), '--module', str(library), '--system', str(seed)])
    run([runtime, 'run', str(consumer), '--module', str(library), '--system', str(seed)], 42,
        '' if args.collections else 'Option.Some(40)\nResult.Error(7)\n')
    if args.collections:
        invalid_source = output / 'NegativeCapacity.rvn'
        invalid_source.write_text('func Main() -> int {\n    let values = System.Collections.ArrayList<int>(-1)\n    return 42\n}\n')
        invalid = output / 'invalid' / 'NegativeCapacity.dll'
        invalid.parent.mkdir()
        run(common + ['--reference', str(library), '-o', str(invalid), str(invalid_source)])
        failure = run([runtime, 'run', str(invalid), '--module', str(library), '--system', str(seed)], 1)
        if 'ArrayList capacity must be non-negative' not in failure.stderr:
            raise RuntimeError('Terminal failure adapter did not preserve the capacity error')
    extension_paths = []
    if args.extensions:
        extension_source = HERE / 'extension-library.rvn'
        extension_consumer_source = HERE / 'extension-consumer.rvn'
        extension_library = output / 'Query.dll'
        extension_consumer = output / 'QueryConsumer.dll'
        run(common + ['--library', '--reference', str(library), '-o', str(extension_library), str(extension_source)])
        run(common + ['--reference', str(library), '--reference', str(extension_library),
                      '-o', str(extension_consumer), str(extension_consumer_source)])
        modules = ['--module', str(library), '--module', str(extension_library), '--system', str(seed)]
        run([runtime, 'verify', str(extension_consumer)] + modules)
        run([runtime, 'run', str(extension_consumer)] + modules, 42, '')
        unsupported_source = output / 'UnsupportedExtension.rvn'
        unsupported_source.write_text('public extension Unsupported for int { val Value: int => self }\n')
        unsupported_output = output / 'UnsupportedExtension.dll'
        failure = run(common + ['--library', '--reference', str(library), '-o', str(unsupported_output), str(unsupported_source)], 1)
        if unsupported_output.exists() or 'implemented instance extension methods' not in failure.stderr:
            raise RuntimeError('Unsupported extension property did not reject before publication')
        extension_paths = [extension_source, extension_consumer_source, extension_library, extension_consumer, unsupported_source]
    rejected = output / 'MissingLibrary.dll'
    run(common + ['-o', str(rejected), str(consumer_source)], 1)
    if rejected.exists():
        raise RuntimeError('Missing source-library dependency published output')
    bad_source = output / 'DuplicateSeed.neoil'
    bad_source.write_text((HERE / 'union-seed.neoil').read_text() + '\n.type System.Option<T>\n.end\n')
    bad_seed = output / 'DuplicateSeed.neox'
    run([runtime, 'assemble', str(bad_source), str(bad_seed), '--format', 'neox'])
    bad_args = common.copy()
    bad_args[bad_args.index('--runtime-seed') + 1] = str(bad_seed)
    failure = run(bad_args + ['--library', '-o', str(rejected)] + sources, 1)
    if rejected.exists() or 'duplicates a source-owned declaration' not in failure.stderr:
        raise RuntimeError('Duplicate seed ownership was not rejected before output')
    compiler_payloads = [Path(compiler).parent / name for name in
                         ['Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll']]
    paths = [Path(__file__).resolve(), Path(compiler), Path(runtime), Path(core), seed, library, manifest,
             HERE / 'union-seed.neoil', seed_source, consumer_source] + [consumer] + ([invalid_source, invalid] if args.collections else []) + [ROOT / p for p in sources] + extension_paths + compiler_payloads
    revisions = {}
    for name, directory in [('runtime', ROOT), ('compiler', Path(compiler).parent)]:
        revisions[name] = subprocess.check_output(
            ['git', '-C', str(directory), 'rev-parse', 'HEAD'], text=True).strip()
    evidence = dict(revisions=revisions, scope=('Native generic extension library and separate consumer execute alongside the HashMap gate. Full query library remains blocked by object-to-generic conversion in OfType.' if args.extensions else 'HashMap separate native import: collisions, growth, replacement, missing keys, callback policies, interface dispatch and shared object identity execute. Full dual-target gate remains open.' if args.hashmap else 'ArrayList separate native import, callbacks, mutation, copying and iteration execute. Full dual-target library/application gate remains open.' if args.collections else 'Native unchanged Option/Result plus iteration contracts; separate native import and execution. Not the full dual-target class-library gate.'),
                    commands=commands, artifacts=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in paths])
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print('PASS separately compiled native generic extension library and HashMap' if args.extensions else 'PASS separately compiled native HashMap and comparers' if args.hashmap else 'PASS separately compiled native ArrayList with callback import' if args.collections else 'PASS unchanged source Option/Result native library and separate consumer')


if __name__ == '__main__':
    main()
