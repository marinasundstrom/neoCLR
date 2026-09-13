"""Check every inventoried source has an explicit POC disposition and evidence.

This checks audit completeness, not execution: run the linked validation separately.
"""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
source = json.loads((HERE / 'runtime-api-inventory.json').read_text())
groups = {
    'primitives': ('Boolean String SByte Byte Int16 UInt16 Char UInt32 Int64 UInt64 Single Double IntPtr UIntPtr Int32', ['library-primitives.rvn', 'library-integers.rvn', 'library-booleans.rvn', 'library-strings.rvn', 'library-parsing.rvn', 'library-division.rvn']),
    'errors': ('Error InvalidRangeError Int32ParseError OverflowError IntegerDivisionError Utf8SliceError ConsoleReadError FileReadError FileWriteError InvalidDateError InvalidTimeError EnvironmentError', ['library-errors.rvn']),
    'unions': ('Option Result Propagatable', ['library-case-payloads.rvn', 'library-unions.rvn', 'library-result-void-propagation.rvn', 'library-reference-payloads.rvn']),
    'interfaces': ('Equatable Comparable Disposable', ['library-value-interfaces.rvn', 'library-interfaces.rvn']),
    'unimplemented-contracts': ('Clonable Closable', []),
    'collections': ('Array ArrayList List Iterable Iterator', ['library-managed-array-metadata.rvn', 'library-array-shapes.rvn', 'library-reference-payloads.rvn', 'library-generic-collections.rvn']),
    'delegates': ('Func', ['library-delegates.rvn', 'library-array-callbacks.rvn']),
    'calendar': ('Date Time LocalDateTime Clock', ['library-calendar.rvn', 'library-clock.rvn']),
    'process': ('Environment Console', ['library-environment.rvn', 'library-console.rvn']),
    'files': ('File Path', ['library-files.rvn', 'library-paths.rvn']),
    'math': ('Math', ['library-floating-math.rvn', 'library-math.rvn', 'library-clamp.rvn']),
    'reflection': ('Type Reflection RuntimeTypeHandle', ['library-reflection.rvn', 'library-flags.rvn']),
    'signature-markers': ('Value Void UnionAttribute', ['library-void.rvn']),
}
lookup = {name: (group, samples) for group, (names, samples) in groups.items() for name in names.split()}
rows = []
for file in source['sourceFiles']:
    entries = [d for d in source['declarations'] if d['file'] == file]
    if not entries:
        continue
    if '/neoCLR/' in file:
        callers = []
        for entry in entries:
            name = entry['declaration'].split('(')[0].split()[-1]
            for candidate in source['sourceFiles']:
                if candidate == file:
                    continue
                for line, text in enumerate((ROOT / candidate).read_text().splitlines(), 1):
                    if re.search(r'\bcall ' + re.escape(name) + r'\(', text):
                        callers.append({'service': name, 'file': candidate, 'line': line})
            if not any(c['service'] == name for c in callers):
                raise ValueError('Service without a reviewed library caller: ' + name)
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'implementation-service', 'callers': callers})
        continue
    if file == 'runtime/System/Array.neoil':
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'replaced-in-raven-profile',
                     'replacementFiles': ['runtime/raven/Array.neoil', 'runtime/raven/NativeMemory.neoil'],
                     'samples': ['library-managed-array-metadata.rvn', 'library-native-buffer.rvn'],
                     'note': 'The native descriptor API is removed; direct IL covers typed NativeMemory access. Raven native casts remain limited.'})
        continue
    if file == 'runtime/System/Collections/List.neoil':
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'replaced-in-raven-profile',
                     'replacementFiles': ['runtime/raven/CollectionContracts.neoil', 'runtime/raven/List.neoil'],
                     'samples': ['library-collection-capabilities.rvn'],
                     'note': 'Count, read indexing and replacement are inherited; List retains Add. All contracts remain invariant.'})
        continue
    group, samples = lookup[Path(file).stem]
    for sample in samples:
        if not (HERE / 'samples' / sample).is_file():
            raise ValueError('Missing evidence sample: ' + sample)
    rows.append({'file': file, 'declarations': len(entries), 'disposition': group, 'samples': samples})
result = {'purpose': 'Explicit source-by-source API audit. Samples and signature checks are evidence, not a claim of arbitrary generic/compiler support.',
          'declarationCount': sum(r['declarations'] for r in rows), 'sources': rows}
result['targetProfileAdditions'] = [{
    'file': 'runtime/raven/Map.neoil',
    'disposition': 'experimental-map-contracts-and-implementation',
    'samples': ['library-maps.rvn'],
    'tests': ['tests/raven_collections.rs', 'docs/experiments/raven-target/verify_collection_capabilities.py'],
    'note': 'Explicit equality/hash callbacks; no default comparer, removal or pair iteration. See docs/map-contracts.md.'
}]
result['targetProfileAdditions'] += [{
    'file': file,
    'disposition': 'query-library-and-terminal-outcomes',
    'samples': ['library-query-terminals.rvn'],
    'tests': ['tests/query_terminals.rs', 'docs/experiments/raven-target/verify_queries.py'],
    'note': 'First/Last return Option; Single returns Result with Empty/Multiple. Normal-outcome cleanup only. See docs/raven-query-api.md.'
} for file in ('runtime/raven/Linq.neoil', 'runtime/raven/SingleError.neoil')]
for addition in result['targetProfileAdditions']:
    assert (ROOT / addition['file']).is_file()
    for sample in addition['samples']:
        assert (HERE / 'samples' / sample).is_file()
    for test in addition['tests']:
        assert (ROOT / test).is_file()
text = json.dumps(result, indent=2) + '\n'
path = HERE / 'runtime-api-coverage.json'
if args.check:
    if not path.is_file() or path.read_text() != text:
        raise SystemExit('API audit is stale; regenerate and review.')
else:
    path.write_text(text)
print(f"Reviewed disposition for {len(rows)} sources and {result['declarationCount']} declaration candidates")
