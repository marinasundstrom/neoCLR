#!/usr/bin/env python3
"""Build and run the Raven runtime-library suite and runner contract in both modes."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / 'runtime/raven/tests'
# Explicit admission gaps: these suites still run interpreted; no native pass is claimed.
NATIVE_ADMISSION_GAPS = {
    'string-construction': 'value member requires a local record owner (Char)',
    # Inspection resolves this private index to neoCLR.Runtime.ParseInt64.
    # A changed linked index deliberately requires fresh inspection/qualification.
    'int64-parsing': '$aot_linked_114: unsupported value member contract',
    'primitive-parsing': 'unsupported closed generic argument: Single',
    'path-values': 'virtual calls requiring dispatch need a later specialization profile: System.Object.Equals',
}
MAP_NAMES = [
    'Map materialization handles collisions and empty input',
    'HashMap copies sequences and maps independently',
    'Map iteration snapshots pairs before mutation',
    'ToMap materializes pair queries and selectors',
    'HashMap copies preserve reference values and comparer',
]
COLLECTION_NAMES = [
    'Any does not read Current',
    'For break disposes its iterator',
    'ArrayList copy has independent storage',
    'Map materialization handles collisions and empty input',
    'HashMap copies sequences and maps independently',
    'Map iteration snapshots pairs before mutation',
    'ToMap materializes pair queries and selectors',
    'HashMap copies preserve reference values and comparer',
    'Predicate Any stops at its first match',
    'Queue growth preserves wrapped FIFO contents',
    'ArrayQueue preserves FIFO',
    'Queue preserves references through clear and reuse',
    'Queue iterator captures FIFO contents before mutation',
    'HashSet applies its comparer',
    'Stack growth preserves LIFO contents',
    'ArrayStack preserves LIFO',
    'Stack preserves references through clear and reuse',
    'Stack iterator captures LIFO contents before mutation',
]
CONSTRUCTION_NAMES = [
    'Collection constructors consume and dispose each input once',
    'Collection constructors accept empty arrays',
    'Queue and stack copies follow source iteration order',
    'List construction materializes filtered input independently',
    'Collection copies preserve element reference identity',
    'Set copy retains values and explicit comparer after source clear',
    'Set collision chains survive removal growth and slot reuse',
    'Set snapshot retains values across clear and reuse',
    'Set ordinal string equality preserves case and Unicode',
]
ITERATION_NAMES = [
    'For completion disposes its iterator',
    'For continue preserves traversal and disposal',
    'For disposes an empty iterator',
    'Labeled continue disposes each inner iterator',
    'For return disposes before returning to its caller',
    'All stops at the first failed predicate',
    'Count advances without reading Current',
    'Any disposes empty input',
    'First stops after one value',
    'Fold visits every value',
    'Last visits the complete input',
    'Predicate Single stops at the second match',
    'Predicate First disposes input with no match',
    'Predicate Count reads every value',
    'Predicate First stops at its match',
    'Predicate Last retains the final match',
    'Predicate Single scans for a unique match',
    'Single rejects multiple values without reading the second',
    'ToList materializes and disposes its input',
]

JSON_NAMES = [
    'JSON arrays report invalid indices',
    'JSON serialization rejects cyclic node graphs',
    'JSON nested documents round trip with Unicode and number spelling',
    'JSON document round trip removes surrounding whitespace',
    'JSON parsing rejects escaped aliases of duplicate names',
    'JSON parsing rejects trailing commas and truncated input',
    'JSON numbers preserve spelling and reject fractional Int32 conversion',
    'JSON duplicate insertion preserves the original value',
    'JSON objects distinguish present null from missing fields',
    'JSON object indexed access preserves insertion order',
    'JSON documents support every root node kind',
    'JSON scalar nodes expose their typed values',
]
JSON_STREAM_NAMES = [
    'JSON stream serialization detects cycles before writing',
    'JSON stream output quota includes escaping and validates before writes',
    'JSON stream reads reject split invalid UTF-8',
    'JSON memory stream round trip preserves document content',
    'JSON number document quota preserves long number spelling',
    'JSON streams retry single-byte reads and writes without taking ownership',
    'JSON stream reads preserve the underlying I/O failure',
    'JSON stream input quota counts bytes and preserves ownership',
    'JSON stream reads reject a truncated document',
    'JSON stream boundary counts UTF-8 bytes rather than characters',
    'JSON stream writes preserve partial output and failure cause',
    'JSON stream writes reject zero progress without looping',
]
MEMORY_STREAM_NAMES = [
    'MemoryStream quota errors preserve length and position',
    'MemoryStream closure is shared across interfaces and idempotent',
    'MemoryStream gap writes zero-fill while empty writes preserve length',
    'MemoryStream rejects invalid buffer and seek ranges without moving',
    'MemoryStream overwrites preserve length and untouched bytes',
]
STRING_CONSTRUCTION_NAMES = [
    'String construction copies character storage',
    'String construction accepts an empty character sequence',
    'String construction preserves and merges grapheme boundaries',
    'String named arguments preserve public parameter contracts',
    'String exposes a readonly character sequence',
]
UNICODE_CASING_NAMES = [
    'Invariant casing preserves embedded NUL and empty strings',
    'Invariant casing supports full Unicode expansions',
    'Invariant casing preserves normalization form',
    'Full casing expansion does not redefine ordinal ignore-case comparison',
    'Invariant lowercase respects contextual Greek sigma',
    'Invariant uppercase preserves emoji and supplementary letters',
]
INT64_PARSING_NAMES = [
    'Int64 parsing accepts both signed boundaries',
    'Int64 decimal formatting round trips signed boundaries',
    'Int64 parsing rejects noncanonical lexical inputs',
    'Int64 parsing distinguishes overflow from invalid trailing text',
    'Int64 parsing accepts signs leading zeros and values beyond Int32',
]
PRIMITIVE_PARSING_NAMES = [
    'Boolean parsing accepts mixed case and rejects numeric aliases',
    'Byte parsing preserves values and reports overflow',
    'Byte parsing preserves distinct small values',
    'Double parsing preserves values and reports overflow',
    'Primitive parsers distinguish invalid format from overflow',
    'Int16 parsing preserves values and reports overflow',
    'Int32 parsing preserves values and reports overflow',
    'SByte parsing preserves values and reports overflow',
    'Single parsing preserves values and reports overflow',
    'UInt16 parsing preserves values and reports overflow',
    'UInt32 parsing preserves values and reports overflow',
    'UInt64 parsing preserves values and reports overflow',
]
PATH_VALUES_NAMES = [
    'Path Object equality distinguishes colliding map keys',
    'Equal paths share hashes and virtual display',
    'Path maps replace equal keys and retain lookup through growth',
    'Path equality distinguishes case roots and normalization',
    'Path equality agrees across typed interface and Object dispatch',
]
DISCOVERY_SPEC = importlib.util.spec_from_file_location('test_discovery', ROOT / 'scripts/discover-runtime-tests.py')
DISCOVERY = importlib.util.module_from_spec(DISCOVERY_SPEC)
DISCOVERY_SPEC.loader.exec_module(DISCOVERY)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True, help='Matching native development bundle')
    parser.add_argument('--output', type=Path, required=True, help='Fresh build/evidence directory')
    parser.add_argument('--aot', type=Path, default=ROOT / 'tools/aot-poc/target/debug' / ('neoclr-aot-poc.exe' if os.name == 'nt' else 'neoclr-aot-poc'))
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--suite', action='append', choices=('collections', 'collection-construction', 'collection-iteration', 'json-dom', 'json-streams', 'memory-stream', 'string-construction', 'unicode-casing', 'int64-parsing', 'primitive-parsing', 'path-values', 'discovery-contract', 'runner-contract'),
                        help='Run only this suite (repeatable); default runs all suites')
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    inputs = [Path(__file__).resolve(), ROOT / 'scripts/build-native-project.py', ROOT / 'scripts/discover-runtime-tests.py', ROOT / 'scripts/validate-test-discovery.py',
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.cs')),
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.csproj')),
              *sorted(TESTS.rglob('*.rvn')), *sorted(TESTS.rglob('*.rvnproj'))]
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  bundleManifestSha256=sha(bundle / 'manifest.json'),
                  aotSha256=sha(args.aot.resolve()), interpreterSha256=sha(args.runtime.resolve()))
    validation_spec = importlib.util.spec_from_file_location('discovery_validation', ROOT / 'scripts/validate-test-discovery.py')
    validation = importlib.util.module_from_spec(validation_spec)
    validation_spec.loader.exec_module(validation)
    expected = [
        ('collections', 0, ''.join('PASS ' + n + '\n' for n in COLLECTION_NAMES) + f'Tests: {len(COLLECTION_NAMES)}, passed: {len(COLLECTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('collection-construction', 0, ''.join('PASS ' + n + '\n' for n in CONSTRUCTION_NAMES) + f'Tests: {len(CONSTRUCTION_NAMES)}, passed: {len(CONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('collection-iteration', 0, ''.join('PASS ' + n + '\n' for n in ITERATION_NAMES) + f'Tests: {len(ITERATION_NAMES)}, passed: {len(ITERATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('json-dom', 0, ''.join('PASS ' + n + '\n' for n in JSON_NAMES) + f'Tests: {len(JSON_NAMES)}, passed: {len(JSON_NAMES)}, failed: 0, skipped: 0\n'),
        ('json-streams', 0, ''.join('PASS ' + n + '\n' for n in JSON_STREAM_NAMES) + f'Tests: {len(JSON_STREAM_NAMES)}, passed: {len(JSON_STREAM_NAMES)}, failed: 0, skipped: 0\n'),
        ('memory-stream', 0, ''.join('PASS ' + n + '\n' for n in MEMORY_STREAM_NAMES) + f'Tests: {len(MEMORY_STREAM_NAMES)}, passed: {len(MEMORY_STREAM_NAMES)}, failed: 0, skipped: 0\n'),
        ('string-construction', 0, ''.join('PASS ' + n + '\n' for n in STRING_CONSTRUCTION_NAMES) + f'Tests: {len(STRING_CONSTRUCTION_NAMES)}, passed: {len(STRING_CONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('unicode-casing', 0, ''.join('PASS ' + n + '\n' for n in UNICODE_CASING_NAMES) + f'Tests: {len(UNICODE_CASING_NAMES)}, passed: {len(UNICODE_CASING_NAMES)}, failed: 0, skipped: 0\n'),
        ('int64-parsing', 0, ''.join('PASS ' + n + '\n' for n in INT64_PARSING_NAMES) + f'Tests: {len(INT64_PARSING_NAMES)}, passed: {len(INT64_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('primitive-parsing', 0, ''.join('PASS ' + n + '\n' for n in PRIMITIVE_PARSING_NAMES) + f'Tests: {len(PRIMITIVE_PARSING_NAMES)}, passed: {len(PRIMITIVE_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('path-values', 0, ''.join('PASS ' + n + '\n' for n in PATH_VALUES_NAMES) + f'Tests: {len(PATH_VALUES_NAMES)}, passed: {len(PATH_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('discovery-contract', 1, 'PASS first discovered test\nFAIL discovered failure: Expected 1, actual 2\nPASS after discovered failure\nPASS NeoClr.DiscoveryTests.DWithoutDescription\nPASS manually registered companion\nTests: 5, passed: 4, failed: 1, skipped: 0\n'),
        ('runner-contract', 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nSKIP intentional skip: contract probe\nTests: 4, passed: 2, failed: 1, skipped: 1\n'),
    ]
    try:
        selected = [case for case in expected if not args.suite or case[0] in args.suite]
        report['suites'] = [case[0] for case in selected]
        if not args.suite:
            validation.validate(bundle, out / 'discovery-signatures')
        for name, exit_code, stdout in selected:
            build = out / (name + '-build')
            environment = dict(os.environ)
            if name != 'runner-contract':
                registry = DISCOVERY.discover(TESTS / name / 'Tests.rvnproj', bundle, out / (name + '-discovery'))
                environment['NeoClrTestRegistry'] = str(registry)
            command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                       'windows-console' if os.name == 'nt' else 'console', '--project', TESTS / name / 'Tests.rvnproj',
                       '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build]
            result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment, capture_output=True, timeout=600)
            (out / (name + '-build.stdout.log')).write_bytes(result.stdout)
            (out / (name + '-build.stderr.log')).write_bytes(result.stderr)
            admission_gap = NATIVE_ADMISSION_GAPS.get(name)
            if admission_gap:
                diagnostic = result.stderr.decode('utf-8')
                if not result.returncode or admission_gap not in diagnostic or not (build / 'app.dll').is_file():
                    raise RuntimeError(name + ' native admission changed; qualify support or inspect retained logs')
                report.setdefault('nativeAdmissionGaps', []).append(dict(suite=name, diagnostic=admission_gap))
                print(name + ': native execution unavailable: ' + admission_gap, flush=True)
            elif result.returncode:
                raise RuntimeError(name + ' build failed; see retained logs')
            if name != 'runner-contract':
                DISCOVERY.verify_registration(build / 'app.dll', bundle, out / (name + '-discovery'))
            isolated = out / (name + '-isolated')
            isolated.mkdir()
            exe = isolated / ('app.exe' if os.name == 'nt' else 'app')
            if not admission_gap:
                shutil.copy2(build / exe.name, exe)
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            interpreter = [args.runtime.resolve(), 'run', build / 'app.dll', '--system', lib / catalog['runtimeSeed'],
                           *[arg for a in catalog['assemblyNames'] for arg in ('--module', lib / (a + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
            env = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            executions = [('interpreter', interpreter, ROOT, None)]
            if not admission_gap:
                executions.insert(0, ('native', [exe], isolated, env))
            for mode, command, cwd, environment in executions:
                result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=180)
                record = dict(suite=name, mode=mode, exitCode=result.returncode,
                              stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                              stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                report['cases'].append(record)
                print(name + ' (' + mode + '):\n' + record['stdout'], end='', flush=True)
                if (record['exitCode'], record['stdout'], record['stderr']) != (exit_code, stdout, ''):
                    raise RuntimeError(name + ' ' + mode + ' did not match the runner contract')
            selections = {
                'collections': [(['--filter', 'ArrayQueue'], 0, 'PASS ArrayQueue preserves FIFO\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                (['--filter', 'Map'], 0, ''.join('PASS ' + n + '\n' for n in MAP_NAMES) + f'Tests: {len(MAP_NAMES)}, passed: {len(MAP_NAMES)}, failed: 0, skipped: 0\n')],
                'discovery-contract': [(['--id', 'manual'], 0, 'PASS manually registered companion\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                       (['--filter', 'after discovered'], 0, 'PASS after discovered failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n')],
                'runner-contract': [
                    (['--entry-probe', '', 'two words', 'Räven ☕ 😀', 'quote"slash\\', '--id'], 0, 'PASS entry arguments preserve text\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.after'], 0, 'PASS after failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.failure'], 1, 'FAIL intentional assertion failure: Expected 1, actual 2\nTests: 1, passed: 0, failed: 1, skipped: 0\n'),
                    (['--id', 'runner.skipped'], 0, 'SKIP intentional skip: contract probe\nTests: 1, passed: 0, failed: 0, skipped: 1\n'),
                    (['--filter', 'failure'], 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nTests: 3, passed: 2, failed: 1, skipped: 0\n'),
                    (['--filter', 'RUNNER'], 2, 'ERROR No tests matched the selection\n'),
                    (['--filter'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--filter', ''], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--unknown', 'after'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                    (['--id', 'runner.after', '--filter', 'after'], 2, 'ERROR Expected --filter <text> or --id <id>\n'),
                ],
            }.get(name, [])
            for arguments, selected_exit, selected_stdout in selections:
                for mode, command, cwd, environment in [('native', [exe, *arguments], isolated, env),
                                                        ('interpreter', [*interpreter, '--', *arguments], ROOT, None)]:
                    result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=180)
                    record = dict(suite=name, mode=mode, arguments=arguments, exitCode=result.returncode,
                                  stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                                  stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                    report['cases'].append(record)
                    if (record['exitCode'], record['stdout'], record['stderr']) != (selected_exit, selected_stdout, ''):
                        raise RuntimeError(name + ' ' + mode + ' selection did not match: ' + repr(record))
            if selections:
                print(name + ': filtering checks passed', flush=True)
        for name, digest in report['inputs'].items():
            if sha(ROOT / name) != digest:
                raise RuntimeError('Input changed during validation: ' + name)
        if sha(args.aot.resolve()) != report['aotSha256'] or sha(args.runtime.resolve()) != report['interpreterSha256']:
            raise RuntimeError('Execution tool changed during validation')
        report['passed'] = True
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Runtime library tests: PASS')


if __name__ == '__main__':
    main()
