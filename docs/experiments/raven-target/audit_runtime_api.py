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
    'calendar': ('Date Time LocalDateTime Clock SystemClock', ['library-calendar.rvn', 'library-clock.rvn']),
    'process': ('Environment Console', ['library-environment.rvn', 'library-console.rvn']),
    'files': ('File Path', ['library-files.rvn', 'library-paths.rvn']),
    'math': ('Math', ['library-floating-math.rvn', 'library-math.rvn', 'library-clamp.rvn']),
    'reflection': ('Type TypeInfo Reflection RuntimeTypeHandle', ['library-reflection.rvn', 'library-flags.rvn']),
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
    if file.startswith('runtime/raven/generated/BindingFlags.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-enum-declaration',
                     'samples': ['library-flags.rvn', 'library-reflection.rvn'],
                     'tests': ['tests/enums.rs', 'docs/experiments/raven-target/verify_flags_library.py'],
                     'note': 'A normal Raven enum owns the named Int32 values and Flags attribute. Checked compiler lowering supplies intrinsic enum operations using the existing nominal runtime ABI, preserving unknown bits.'})
        continue
    if file.startswith('runtime/raven/generated/Func.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-delegate-declarations',
                     'samples': ['library-delegates.rvn'],
                     'tests': ['tests/delegates.rs', 'docs/experiments/raven-target/verify_delegate_library.py'],
                     'note': 'All five invariant Func arities are checked against CLI runtime delegate signatures; invocation and closure lifetime remain runtime-owned.'})
        continue
    if file.startswith('runtime/raven/generated/Fault.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-terminal-namespace-function',
                     'tests': ['tests/system_fault.rs', 'docs/experiments/raven-target/verify_fault.py', 'docs/experiments/raven-target/verify_fault_library.py'],
                     'note': 'Computed String diagnostic; terminates guest execution, not the embedding host. No compiler non-return analysis.'})
        continue
    if file == 'runtime/System/Array.neoil':
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'replaced-in-raven-profile',
                     'replacementFiles': ['runtime/raven/Array.neoil', 'runtime/raven/NativeMemory.neoil'],
                     'samples': ['library-managed-array-metadata.rvn', 'library-native-buffer.rvn'],
                     'note': 'The native descriptor API is removed; direct IL covers typed NativeMemory access. Raven native casts remain limited.'})
        continue
    if file == 'runtime/System/Collections/ArrayList.neoil':
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-class-bootstrap',
                     'replacementFiles': ['runtime/raven/ArrayList.neoil', 'runtime/raven/src/System/Collections/ArrayList.rvn'],
                     'samples': ['library-list-filters.rvn', 'library-delegates.rvn', 'library-maps.rvn'],
                     'note': 'The complete class and private iterator are Raven-authored. Direct checked array storage replaces the legacy state wrapper; searches preserve captured buffer/extent and Option outcomes. Historical Neo profile unchanged.'})
        continue
    if file in ('runtime/System/Time.neoil', 'runtime/System/Date.neoil'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-value-bootstrap',
                     'replacementFiles': ['runtime/raven/' + Path(file).name, 'runtime/raven/src/System/' + Path(file).stem + '.rvn'],
                     'samples': ['library-calendar.rvn', 'library-clock.rvn'],
                     'tests': ['tests/raven_calendar.rs'],
                     'note': 'Existing factories, calendar/tick boundaries, comparison and readonly access preserved. Historical Neo profile unchanged.'})
        continue
    if file == 'runtime/System/Collections/List.neoil':
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'replaced-in-raven-profile',
                     'replacementFiles': ['runtime/raven/CollectionContracts.neoil', 'runtime/raven/List.neoil'],
                     'samples': ['library-collection-capabilities.rvn'],
                     'note': 'Count, read indexing and replacement are inherited; List retains Add. All contracts remain invariant.'})
        continue
    if file.startswith('runtime/raven/generated/') and Path(file).name.split('.')[0] in (
            'Boolean', 'SByte', 'Byte', 'Int16', 'UInt16', 'UInt32', 'Int64', 'UInt64', 'Single', 'Double', 'IntPtr', 'UIntPtr'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-primitive-struct',
                     'samples': ['library-primitives.rvn', 'library-booleans.rvn'],
                     'tests': ['tests/common_interfaces.rs', 'docs/experiments/raven-target/verify_primitive_library.py'],
                     'note': 'Matched private backing storage becomes intrinsic loads; no nested runtime field or new primitive constructor.'})
        continue
    if file.startswith(('runtime/raven/generated/Value.', 'runtime/raven/generated/RuntimeTypeHandle.')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-memberless-declaration',
                     'tests': ['tests/erased_inputs.rs', 'tests/value_storage.rs', 'tests/raven_reflection.rs',
                               'docs/experiments/raven-target/verify_declaration_library.py'],
                     'note': 'Source declarations only; erased values and opaque type handles retain intrinsic runtime representations.'})
        continue
    if any(file.startswith('runtime/raven/generated/' + name + '.') for name in
           ('FileReadError', 'FileWriteError', 'ConsoleReadError', 'Utf8SliceError', 'Int32ParseError', 'IntegerDivisionError')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-error-carrier',
                     'samples': ['library-errors.rvn', 'library-string-slices.rvn'],
                     'tests': ['tests/io_errors.rs', 'tests/strings.rs', 'docs/experiments/raven-target/verify_error_carrier_library.py'],
                     'note': 'Raven owns nested empty cases, case constructors/predicates/extractors and formatting. Checked constructor lowering preserves one erased payload and the existing value receiver ABI.'})
        continue
    if any(file.startswith('runtime/raven/generated/' + name + '.') for name in
           ('InvalidRangeError', 'InvalidDateError', 'InvalidTimeError', 'OverflowError', 'EnvironmentError', 'Void')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-empty-value',
                     'samples': ['library-errors.rvn', 'library-void.rvn'],
                     'tests': ['tests/errors.rs', 'tests/arithmetic_errors.rs', 'docs/experiments/raven-target/verify_empty_library.py'],
                     'note': 'Empty values preserve payload-free defaults, existing constructors/formatting and nominal Void. No new constructor on EnvironmentError or Void.'})
        continue
    if file.startswith(('runtime/raven/generated/Option.', 'runtime/raven/generated/Result.')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-generic-union',
                     'samples': ['library-unions.rvn', 'library-generic-unions.rvn', 'library-result-void-propagation.rvn'],
                     'tests': ['tests/propagation.rs', 'tests/union_out.rs', 'docs/experiments/raven-target/verify_generic_union_library.py'],
                     'note': 'Raven owns cases, one erased carrier payload, factories and extraction. Checked constructor/readonly receiver projections and literal Boolean returns retain true-only output assignment. Bootstrap-only LeaveUnassigned is restricted to immediate false returns.'})
        continue
    if file.startswith('runtime/raven/generated/Propagatable.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-conditional-output-contract',
                     'tests': ['tests/propagation.rs', 'docs/experiments/raven-target/verify_propagation_library.py'],
                     'note': 'Exact generic positions and out metadata preserve readonly receivers and true-only output initialization.'})
        continue
    if file.startswith(('runtime/raven/generated/String.', 'runtime/raven/generated/Error.')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-opaque-bootstrap',
                     'samples': ['library-strings.rvn', 'library-string-slices.rvn', 'library-errors.rvn'],
                     'tests': ['tests/strings.rs', 'tests/errors.rs', 'docs/experiments/raven-target/verify_opaque_library.py'],
                     'note': 'Checked intrinsic String storage and fieldless Error bodies retain native ownership and mixed receiver ABI. Compiler operators remain intrinsic; opaque allocation and storage mutation are rejected.'})
        continue
    if file.startswith('runtime/raven/generated/Char.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-char-struct',
                     'tests': ['tests/character_classification.rs', 'docs/experiments/raven-target/verify_scalar_library.py'],
                     'note': 'System/Char.rvn owns comparison and all sixteen predicates; UTF-16 units and native Unicode category behavior are unchanged.'})
        continue
    if file.startswith(('runtime/raven/generated/Instant.', 'runtime/raven/generated/Duration.')):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-time-values',
                     'samples': ['library-instants.rvn'], 'tests': ['tests/instant_clock.rs'],
                     'note': 'Signed 100 ns tick values; Instant uses Unix epoch, Duration has no epoch. System-zone conversion is a bounded demo API.'})
        continue
    if file.startswith('runtime/raven/generated/Int32.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-scalar-bootstrap',
                     'samples': ['library-primitives.rvn', 'library-division.rvn'],
                     'tests': ['tests/character_classification.rs', 'docs/experiments/raven-target/verify_scalar_library.py'],
                     'note': 'Raven owns Parse, Divide, Equals, CompareTo and ToString. Bootstrap intrinsics decode native parse payloads; the scalar formatting receiver is preserved.'})
        continue
    if file.startswith('runtime/raven/generated/File.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-file-write-bootstrap',
                     'tests': ['tests/file_output.rs', 'tests/io_errors.rs'],
                     'note': 'Raven constructs existing read and write Results from native payload/status values. Unknown statuses still fault; public signatures are preserved.'})
        continue
    if file.startswith('runtime/raven/generated/Path.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-path-bootstrap',
                     'tests': ['tests/path.rs'],
                     'note': 'Raven wrappers preserve the existing lexical host services and parameter names.'})
        continue
    if file.startswith('runtime/raven/generated/Math.'):
        rows.append({'file': file, 'declarations': len(entries), 'disposition': 'raven-authored-math-bootstrap',
                     'samples': ['library-math.rvn', 'library-clamp.rvn'],
                     'tests': ['docs/experiments/raven-target/verify_math_library.py', 'tests/math_helpers.rs', 'tests/math_typed.rs'],
                     'note': 'All twenty Math functions are Raven-authored; Double operations retain native services behind checked bootstrap-only bindings. Source authority is runtime/raven/src/System/Math/Functions.rvn.'})
        continue
    group, samples = lookup[Path(file).stem]
    for sample in samples:
        if not (HERE / 'samples' / sample).is_file():
            raise ValueError('Missing evidence sample: ' + sample)
    rows.append({'file': file, 'declarations': len(entries), 'disposition': group, 'samples': samples})
result = {'purpose': 'Explicit source-by-source API audit. Samples and signature checks are evidence, not a claim of arbitrary generic/compiler support.',
          'declarationCount': sum(r['declarations'] for r in rows), 'sources': rows}
result['targetProfileAdditions'] = [{
    'file': 'runtime/raven/Clock.neoil',
    'disposition': 'raven-authored-interface-contract',
    'samples': ['library-clock.rvn', 'library-instants.rvn'],
    'tests': ['docs/experiments/raven-target/verify_interface_library.py', 'tests/instant_clock.rs'],
    'note': 'Checked nongeneric interface declaration with unchanged Now property. SystemClock remains runtime-backed; Info interface identity migration is separate.'
}, {
    'file': 'runtime/raven/TypeInfo.neoil',
    'disposition': 'raven-authored-type-info',
    'samples': ['library-reflection.rvn', 'library-flags.rvn'],
    'tests': ['docs/experiments/raven-target/verify_type_library.py', 'docs/experiments/raven-target/verify_introspection_namespace.py'],
    'note': 'TypeInfo queries use Raven bodies and checked runtime-service adapters. The handle factory remains internal; invocation is outside the descriptive API.'
}, {
    'file': 'runtime/raven/ParameterInfo.neoil',
    'disposition': 'raven-authored-parameter-descriptor',
    'samples': ['library-reflection.rvn'],
    'tests': ['docs/experiments/raven-target/verify_parameter_info_library.py', 'docs/experiments/raven-target/verify_introspection_namespace.py'],
    'note': 'Six parameter snapshot readers are Raven-authored. The importer validates field order/types; runtime factories still produce snapshots. The inherited member hierarchy and its array copies are Raven-authored in Descriptors.rvn.'
}, {
    'file': 'runtime/raven/Type.neoil',
    'disposition': 'raven-authored-type',
    'samples': ['library-reflection.rvn', 'library-array-unified.rvn'],
    'tests': ['tests/raven_reflection.rs', 'docs/experiments/raven-target/verify_type_library.py'],
    'note': 'Raven Type owns identity and shape. Member, base/interface and enum metadata are queried through the Raven-authored TypeInfo using the same handle. No public described-object construction API.'
}, {
    'file': 'runtime/raven/Map.neoil',
    'disposition': 'experimental-map-contracts-with-raven-implementation',
    'samples': ['library-maps.rvn'],
    'tests': ['tests/raven_collections.rs', 'docs/experiments/raven-target/verify_collection_capabilities.py'],
    'note': 'HashMap algorithms and Map/MutableMap contracts are authored in Raven. Private helpers retain visibility and all bodies are checked. Explicit equality/hash callbacks; no default comparer, removal or pair iteration. See docs/map-contracts.md.'
}]
result['targetProfileAdditions'] += [{
    'file': 'runtime/raven/Descriptors.neoil',
    'disposition': 'raven-authored-inherited-descriptors',
    'samples': ['library-reflection.rvn'],
    'tests': ['tests/raven_reflection.rs', 'docs/experiments/raven-target/verify_descriptor_library.py'],
    'note': 'Exact inherited snapshot storage, parameter-array copies and accessor visibility filtering; native factories retain ownership.'
}, {
    'file': 'runtime/raven/NativeMemory.neoil',
    'disposition': 'raven-authored-native-allocation-helpers',
    'samples': ['library-native-buffer.rvn'],
    'tests': ['tests/native_memory_api.rs', 'docs/experiments/raven-target/verify_native_library.py'],
    'note': 'Overload composition is Raven code; checked native multiplication and allocation/release remain runtime instructions.'
}]
result['targetProfileAdditions'] += [{
    'file': file,
    'disposition': 'query-library-and-terminal-outcomes',
    'samples': ['library-query-terminals.rvn'],
    'tests': ['tests/query_terminals.rs', 'docs/experiments/raven-target/verify_queries.py'],
    'note': 'Operators, SingleError case/formatting bodies and private deferred iterator classes are authored in Raven with generated bootstrap bodies; First/Last return Option and Single returns Result with Empty/Multiple. Checked storage retains generic cached elements. Normal-outcome cleanup only. See docs/raven-query-api.md.'
} for file in ('runtime/raven/Linq.neoil', 'runtime/raven/SingleError.neoil', 'runtime/raven/src/System/Linq/Operators.rvn',
                 'runtime/raven/generated/Linq.methods.neoil', 'runtime/raven/generated/Linq.helpers.neoil')]
result['targetProfileAdditions'] += [{
    'file': 'runtime/raven/' + name + '.neoil',
    'disposition': 'raven-authored-existing-contract',
    'samples': samples,
    'tests': tests,
    'note': 'Existing API preserved; generated bootstrap declarations and bodies are validated with executable consumers.'
} for name, samples, tests in [
    *[(name, ['library-value-interfaces.rvn'], ['docs/experiments/raven-target/verify_foundation_library.py'])
      for name in ('Equatable', 'Comparable', 'Clonable', 'Closable', 'Disposable')],
    *[(name, ['library-collection-capabilities.rvn', 'library-maps.rvn'], ['tests/raven_collections.rs', 'docs/experiments/raven-target/verify_foundation_library.py'])
      for name in ('Collection', 'Sequence', 'MutableSequence', 'List', 'Iterable', 'Iterator', 'MutableMap')],
    *[(name, ['library-clock.rvn', 'library-instants.rvn'], ['tests/raven_calendar.rs', 'docs/experiments/raven-target/verify_interface_library.py'])
      for name in ('SystemClock', 'LocalDateTime')],
    *[(name, ['library-environment.rvn', 'library-console.rvn'], ['docs/experiments/raven-target/verify_process.py'])
      for name in ('Console', 'Environment')],
]]
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
