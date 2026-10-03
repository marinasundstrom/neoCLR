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
    parser.add_argument('--queries', action='store_true', help='Compile unchanged full query sources and execute an independent consumer')
    parser.add_argument('--arrays', action='store_true', help='Execute source Array<T> backing and query iteration over native vectors')
    parser.add_argument('--application', action='store_true', help='Run unchanged application-order-collections against the separately built native library')
    args = parser.parse_args()
    args.arrays = args.arrays or args.application
    args.queries = args.queries or args.arrays
    args.hashmap = args.hashmap or args.extensions or args.queries
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

    manifest = HERE / ('array-ownership.json' if args.arrays else 'query-ownership.json' if args.queries else 'hashmap-ownership.json' if args.hashmap else 'arraylist-ownership.json' if args.collections else 'union-ownership.json')
    consumer_source = HERE / ('array-consumer.rvn' if args.arrays else 'query-consumer.rvn' if args.queries else 'hashmap-consumer.rvn' if args.hashmap else 'arraylist-consumer.rvn' if args.collections else 'union-consumer.rvn')
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
    application_paths = []
    if args.application:
        override_source = HERE / 'override-consumer.rvn'
        override_app = output / 'Override.dll'
        run(common + ['--reference', str(library), '-o', str(override_app), str(override_source)])
        run([runtime, 'verify', str(override_app), '--module', str(library), '--system', str(seed)])
        run([runtime, 'run', str(override_app), '--module', str(library), '--system', str(seed)], 0, 'Multiple\n42\n')
        application_source = ROOT / 'docs/experiments/raven-target/samples/application-order-collections.rvn'
        expected = application_source.with_suffix('.expected.txt')
        application = output / 'Application.dll'
        run(common + ['--reference', str(library), '-o', str(application), str(application_source)])
        run([runtime, 'verify', str(application), '--module', str(library), '--system', str(seed)])
        run([runtime, 'run', str(application), '--module', str(library), '--system', str(seed)], 0, expected.read_text())
        application_paths = [application_source, expected, application, override_source, override_app]
    sample_paths = []
    if args.arrays:
        samples = [
            (HERE / 'array-interface-count-consumer.rvn', 42, ''),
            (HERE / 'nested-array-callback-consumer.rvn', 42, ''),
            (HERE / 'instance-callback-consumer.rvn', 42, ''),
            (HERE / 'captured-reference-consumer.rvn', 42, ''),
            (HERE / 'primitive-capture-consumer.rvn', 42, ''),
            (HERE / 'query-lifetime-consumer.rvn', 42, ''),
            (ROOT / 'docs/experiments/raven-target/samples/library-list-filters.rvn', 0,
             '7\n7\n1\n3\nAbsent\nExists\nNot all positive\n3\n7\n42\n7\n7\nAbsent\nAbsent\nAbsent\n0\nAll empty elements satisfy the predicate\n5\n7\n99\n1\n2\n'),
            (ROOT / 'docs/experiments/raven-target/samples/library-array-callbacks.rvn', 0, '7\n42\nFirst\nSecond\n'),
            (ROOT / 'docs/experiments/raven-target/samples/library-option.rvn', 0, '42\nProduct not found\n'),
            (ROOT / 'docs/experiments/raven-target/samples/library-option-propagation.rvn', 0, 'Value found\n42\nAbsent\n'),
            (ROOT / 'docs/experiments/raven-target/samples/library-collection-capabilities.rvn', 0,
             '2\n42\n2\n2\n7\n2\n9\n2\n3\n11\n'),
        ]
        for name in ['library-query-basics', 'library-query-names']:
            source = ROOT / 'docs/experiments/raven-target/samples' / (name + '.rvn')
            expected = source.with_suffix('.expected.txt')
            samples.append((source, 0, expected.read_text()))
            sample_paths.append(expected)
        for source, exit_code, expected_stdout in samples:
            app = output / (source.stem + '.dll')
            run(common + ['--reference', str(library), '-o', str(app), str(source)])
            run([runtime, 'verify', str(app), '--module', str(library), '--system', str(seed)])
            run([runtime, 'run', str(app), '--module', str(library), '--system', str(seed)],
                exit_code, expected_stdout)
            sample_paths.extend([source, app])
        mutable_source = output / 'MutableCapture.rvn'
        mutable_source.write_text('class Cell {}\nfunc Main() -> int {\n    var cell = Cell()\n    let callback: () -> object = () => cell\n    cell = Cell()\n    return 42\n}\n')
        mutable_output = output / 'MutableCapture.dll'
        failure = run(common + ['--reference', str(library), '-o', str(mutable_output), str(mutable_source)], 1)
        if mutable_output.exists() or 'closure capture requires an immutable reference or supported primitive local' not in failure.stderr:
            raise RuntimeError('Mutable capture did not reject before publication')
        sample_paths.append(mutable_source)
        mutable_scalar_source = output / 'MutableScalarCapture.rvn'
        mutable_scalar_source.write_text('func Main() -> int {\n    var value = 1\n    let callback: () -> int = () => value\n    value = 2\n    return callback()\n}\n')
        mutable_scalar_output = output / 'MutableScalarCapture.dll'
        failure = run(common + ['--reference', str(library), '-o', str(mutable_scalar_output), str(mutable_scalar_source)], 1)
        if mutable_scalar_output.exists() or 'closure capture requires an immutable reference or supported primitive local' not in failure.stderr:
            raise RuntimeError('Mutable scalar capture did not reject before publication')
        sample_paths.append(mutable_scalar_source)
    query_paths = []
    if args.queries:
        bad_cast_source = output / 'BadCast.rvn'
        bad_cast_source.write_text('func Extract<T>(value: object) -> T => (T)value\nfunc Main() -> int {\n    let boxed: object = "wrong"\n    return Extract<int>(boxed)\n}\n')
        bad_cast = output / 'BadCast.dll'
        run(common + ['--reference', str(library), '-o', str(bad_cast), str(bad_cast_source)])
        failure = run([runtime, 'run', str(bad_cast), '--module', str(library), '--system', str(seed)], 1)
        if 'unbox.any requires the exact boxed value type' not in failure.stderr:
            raise RuntimeError('Incorrect unboxing did not preserve exact-type failure')
        query_paths = [bad_cast_source, bad_cast]
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
             HERE / 'union-seed.neoil', seed_source, consumer_source] + [consumer] + ([invalid_source, invalid] if args.collections else []) + [ROOT / p for p in sources] + extension_paths + query_paths + application_paths + sample_paths + compiler_payloads
    revisions = {}
    for name, directory in [('runtime', ROOT), ('compiler', Path(compiler).parent)]:
        revisions[name] = subprocess.check_output(
            ['git', '-C', str(directory), 'rev-parse', 'HEAD'], text=True).strip()
    evidence = dict(revisions=revisions, scope=('Unchanged application-order-collections compiles against a separate native source-library artifact and executes with exact output and exit 0. Full dual-target gate remains open.' if args.application else 'Native vectors dispatch through explicitly selected source Array<T> backing and independently compiled query/iterator methods; full dual-target gate remains open.' if args.arrays else 'Unchanged query library separately imports and executes OfType, Filter, Map, ToList and Single with value unboxing and shared reference identity. Broad array extension lookup remains open.' if args.queries else 'Native generic extension library and separate consumer execute alongside the HashMap gate. Full query library remains blocked by object-to-generic conversion in OfType.' if args.extensions else 'HashMap separate native import: collisions, growth, replacement, missing keys, callback policies, interface dispatch and shared object identity execute. Full dual-target gate remains open.' if args.hashmap else 'ArrayList separate native import, callbacks, mutation, copying and iteration execute. Full dual-target library/application gate remains open.' if args.collections else 'Native unchanged Option/Result plus iteration contracts; separate native import and execution. Not the full dual-target class-library gate.'),
                    commands=commands, artifacts=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in paths])
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print('PASS unchanged application-order-collections with a separately compiled native library' if args.application else 'PASS separately compiled nominal Array<T> backing and vector query consumer' if args.arrays else 'PASS separately compiled unchanged native query library' if args.queries else 'PASS separately compiled native generic extension library and HashMap' if args.extensions else 'PASS separately compiled native HashMap and comparers' if args.hashmap else 'PASS separately compiled native ArrayList with callback import' if args.collections else 'PASS unchanged source Option/Result native library and separate consumer')


if __name__ == '__main__':
    main()
