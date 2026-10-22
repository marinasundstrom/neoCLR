#!/usr/bin/env python3
"""Build and run the Raven runtime-library suite and runner contract in selected execution modes."""
import runpy
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
import time

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / 'runtime/raven/tests'
HTTP_LIFETIME_NAMES = ['HTTP client use disposes its owned handler exactly once', 'HTTP borrowed handler survives one client and serves another', 'HTTP disposal requests cancellation and waits for handler acknowledgement', 'HTTP caller cancellation propagates without disposing the handler', 'HTTP completed responses survive disposal and retain request association', 'HTTP pre-cancellation skips custom handlers', 'HTTP socket handler disposal is idempotent through Disposable', 'HTTP delegating handlers run outward and unwind inward', 'HTTP borrowed delegating pipeline remains reusable', 'HTTP borrowed clients cancel only their own pending sends', "HTTP disposed requests are rejected before invoking a custom handler"]

MEMORY_LIFETIME_NAMES = [
    'Pending Memory destinations survive collection and preserve surrounding bytes',
    'Pending ReadOnlyMemory sources survive repeated suspension and collection',
    'Cancelled and rejected Memory transfers do not write or resume twice',
    'Default and end-position Memory windows remain empty after suspension',
]

HTTP_CONTENT_COPY_NAMES = ['HTTP content copy awaits writes before reading more and preserves ownership', 'HTTP memory content preserves selected windows and aliases across copies', 'HTTP content copy preserves stream failures and rejects truncated or invalid reads', 'HTTP content copy respects declared lengths and skips empty I/O', 'HTTP content copy cancellation skips pre-cancelled input and stops pending writes', 'HTTP content copy rejects disposed bodies and stops before the next source read', 'HTTP string reader consumes generic streams and preserves owned text and errors', 'HTTP stream creation validates before taking ownership or reading']

HTTP_TEXT_NAMES = [
    'HTTP buffered text preserves scalars beyond the one-shot codec quota',
    'HTTP buffered text rejects malformed and truncated large bodies',
    'HTTP buffered text preserves empty BOM and NUL data',
]
ENCODING_BUFFER_NAMES = [
    'UTF-8 decoder quota rejection preserves an incomplete scalar',
    'ASCII decoder accepts the maximum input after rejecting an oversized call',
    'Encoder drain quotas preserve accepted text and destination bytes',
    'UTF-8 decoder maximum input counts new bytes separately from carried bytes',
    'Native buffer comprehensions preserve selector indices and range endpoints',
    'One-shot decoding enforces byte limits and accepts the maximum',
]

ENCODING_BOUNDARY_NAMES = [
    'StreamReader decodes Unicode lines across single-byte reads',
    'StreamReader distinguishes empty lines EOF and lone carriage returns',
    'StreamReader delimits custom decoded output and bounds expansion',
    'StreamWriter reports zero progress and preserves flush failures',
    'StreamReader rejects invalid limits before consuming its input',
    'Encoding input quotas reject oversized chunks without poisoning sessions',
    'StreamWriter accepts its maximum byte count and preflights oversized lines',
    'StreamReader preserves input failures and rejects truncated UTF-8 at EOF',
]

ENCODING_STREAM_NAMES = [
    'StreamWriter completes short writes and preserves cross-input graphemes',
    'ASCII StreamWriter rejects complete lines before output and remains reusable',
    'Custom encoder trailers require explicit Finish and are emitted once',
    'Closing a writer manages ownership without implicit encoder finalization',
    'Partial write and final-output failures are observable without replay',
    'Custom encoder final acceptance waits for all trailer bytes to drain',
    'Selected ASCII reader recognizes CRLF and preserves borrowed ownership',
    'Selected readers reject malformed bytes and enforce decoded text limits',
    'StreamReader waits for pending input before decoding and completing',
    'StreamWriter waits for pending output before reporting the byte count',
    'Cancelled input cancels the pending reader operation',
]

ENCODING_NAMES = [
    'Encoding providers preserve UTF-8 and reject non-ASCII text',
    'UTF-8 decoding preserves text at every byte split',
    'UTF-8 decoders reject malformed scalars and truncated final input',
    'Decoder sessions copy pending input and preserve BOM data',
    'Decoder windows preserve text and ASCII errors are terminal',
    'Encoders preserve accepted input through one-byte drains',
    'Encoder chunks preserve scalars across the internal byte boundary',
    'ASCII encoder preflight is retryable and factories are independent',
    'One-shot decoding preserves Unicode BOM NUL and empty sequences',
    'One-shot errors do not poison providers and results are snapshots',
    'One-shot decoding borrows only its window and owns its result',
]

ZONED_TIME_PARSING_NAMES = [
    'Zoned parsing preserves UTC and the pinned database version',
    'Zoned parsing validates winter and summer offsets',
    'Explicit offsets select both DST overlap candidates without losing ticks',
    'Zoned parsing rejects skipped local times for either adjacent offset',
    'Zoned parsing handles half hour transitions',
    'Zoned parsing enforces rule bounds and historical second precision',
    'Zoned parsing preserves lexical and civil range errors',
    'Named zone lookup remains ordinal and accepts pinned aliases',
]

INSTANT_PARSING_NAMES = [
    'Instant parsing normalizes explicit offsets to the same epoch',
    'Instant parsing preserves fractional ticks before and after the epoch',
    'Instant parsing preserves the wider tick domain at civil boundaries',
    'Instant parsing requires a complete invariant timestamp and explicit offset',
    'Instant parsing rejects invalid civil fields and offset ranges',
    'Parsed instants project back through their explicit fixed offset',
]

DURATION_OFFSET_PARSING_NAMES = [
    'Duration parsing preserves signed tick boundaries',
    'Duration parsing rejects overflow and invalid clock fields',
    'Duration parsing rejects unsupported numeric grammar',
    'Duration fractions scale to ticks and accept bounded leading day zeros',
    'Offset parsing accepts zero signs seconds and inclusive bounds',
    'Offset parsing rejects invalid ranges without normalization',
    'Offset parsing requires complete ASCII signed syntax',
    'Parsed offsets project instants and parsed durations preserve arithmetic',
]

TIME_PARSING_NAMES = [
    'Date parsing accepts bounds and Gregorian leap years',
    'Date parsing separates malformed text from invalid civil fields',
    'Time parsing scales one through seven fraction digits without rounding',
    'Time parsing rejects malformed clocks leap seconds and excess precision',
    'Civil parsers round trip all supported numeric culture conventions',
    'Explicit cultures reject mismatched date and fraction separators',
    'Local date time parsing preserves boundaries and rejects zone suffixes',
    'Default civil parsers use the current culture formatting convention',
]

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
    'Map indexers share comparer lookup and insert-or-replace semantics',
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
    'JSON node text supports compact and indented round trips',
    'JSON parsing rejects trailing commas and truncated input',
    'Empty JSON objects support empty map snapshots and safe lookup',
    'JSON map lookup distinguishes null absence and ordinal keys',
    'JSON map views keep JSON node serialization precedence',
    'JSON map snapshots preserve insertion order and node identity',
    'JSON numbers preserve spelling and reject fractional Int32 conversion',
    'JSON duplicate insertion preserves the original value',
    'JSON objects distinguish present null from missing fields',
    'JSON object indexed access preserves insertion order',
    'JSON documents support every root node kind',
    'JSON scalar nodes expose their typed values',
    'JSON sequence indexing retains node identity and explicit null',
    'Empty JSON arrays expose empty sequence snapshots',
    'JSON sequence views retain node serialization precedence',
    'JSON sequence snapshots preserve order and shared child contents',
]
JSON_STREAM_NAMES = [
    'JSON HTTP byte input preserves memory windows and aliases and rejects unbuffered content',
    'HTTP JSON async reads consume streams once preserve ownership and honor cancellation',
    'JSON byte input borrows selected windows and returns owned mapped values',
    'JSON byte input accepts UTF-8 scalar boundaries and rejects BOM and raw controls',
    'JSON byte input rejects invalid UTF-8 before syntax and preserves quotas',
    'JSON UTF-8 byte output preserves escaping formatting and independent ownership',
    'JSON borrowed token parsing preserves empty Unicode escaped and scalar values',
    'JSON borrowed token parsing rejects malformed escapes controls and numbers',
    'JSON stream serialization detects cycles before writing',
    'JSON stream output quota includes escaping and validates before writes',
    'JSON indented stream output matches node text and preflights quotas',
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
    'MemoryStream async cancellation and rejected ranges preserve storage',
    'MemoryStream async interfaces preserve complete writes short reads and EOF',
    'MemoryStream quota errors preserve length and position',
    'MemoryStream closure is shared across interfaces and idempotent',
    'MemoryStream gap writes zero-fill while empty writes preserve length',
    'MemoryStream rejects invalid buffer and seek ranges without moving',
    'MemoryStream overwrites preserve length and untouched bytes',
]
STRING_CONSTRUCTION_NAMES = [
    'Character literals preserve graphemes and allow char identifiers',
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
GLOBALIZATION_NAMES = [
    'UK and US clocks preserve midnight noon and invariant output',
    'Culture preferences resolve explicit regions and bounded fallbacks',
    'EnglishUnitedKingdom formatting data and calendar isolation',
    'EnglishUnitedStates formatting data and calendar isolation',
    'HebrewIsrael formatting data and calendar isolation',
    'Invariant formatting data and calendar isolation',
    'SwedishSweden formatting data and calendar isolation',
]
NUMERIC_FORMATTING_NAMES = [
    'Culture parsing covers every numeric primitive and roundtrips formatting',
    'Culture parsing rejects malformed groups mixed separators and overflow',
    'Floating formatting preserves precision culture and boxed display',
    'Integral formatting preserves widths culture and interface dispatch',
]

ENUM_PARSING_NAMES = [
    'Enum lists combine names independently of Flags metadata',
    'Enum names preserve aliases ordinal case and imported types',
    'Enum numbers preserve signed bounds and unnamed values',
    'Enum TryParse resets output and rejects malformed input',
    'Unicode type member parameter and local identifiers use ordinary binding',
    'Enum names preserve Unicode and explicit ordinal identity',
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
DEPENDENCY_INJECTION_NAMES = [
    'DI selects the greediest available public constructor',
    'DI activates typed localizer dependencies through a retained plan',
    'DI rejects ambiguous missing and inaccessible constructors',
    'DI builds plans without executing constructors or factories',
    'DI registry snapshots registrations and injects typed localizers',
    'DI isolates scopes and shares only singleton instances',
    'DI rejects cycles and captive scopes without poisoning resolution state',
    'DI disposes owned services in reverse order and leaves instances caller owned',
    'DI validates factories and registrations and resets failed singleton factories',
    'DI injects the current provider and root disposal invalidates live scopes',
    'DI conventional lookups distinguish missing services from activation errors',
    'DI generic registrations preserve factory failures and plural singleton slots',
    'DI cleans up factory results when disposal reenters resolution',
    'DI keyed lookups isolate registrations and preserve ordinary key equality',
    'DI keyed factories and scopes preserve independent lifetime slots',
    'DI retrieves keyed bound function objects without invoking them as factories',
    'DI TryAdd preserves existing type and key registrations without activating factories',
    'DI diamond graphs preserve lifetimes and independent scope ownership',
    'DI closes open generic registrations and isolates scoped caches by type',
    'DI open generic singletons preserve exact overrides keyed lookup and plural order',
    'DI website sample resolves a scoped constructor graph and disposes scopes',
]

LOCALIZATION_NAMES = [
    'Localizer preserves strings and distinguishes missing from empty',
    'Localizer uses ordinal keys and decodes JSON escapes',
    'Localizer rejects malformed and non-string catalogues',
    'Localizer supports catalogues beyond provisional DOM limits',
    'Localizer reads borrowed streams and explicit files',
    'Localizer diagnostics report only missing display lookups',
    'Localizer scopes and cultures fall back per key',
    'Localizer invariant fallback is configurable and typed scope is explicit',
    'Localizer resolves templates before formatting arguments',
    'Localizer integrates typed JSON resources with scoped constructor injection',
]

DECONSTRUCTION_NAMES = [
    'Value deconstruction binds named and unnamed locals',
    'Reference deconstruction binds named and unnamed locals',
    'Positional patterns narrow derived and object references',
    'Qualified currency patterns match values and reject mismatches',
]

CURRENCY_NAMES = ['Currency identity is independent of culture and amount', 'Currency formatting follows all five culture profiles', 'Explicit currency formatting rounds display without changing the amount', 'Currency amount parsing validates identity grouping spacing and direction', 'Currency data and language direction are explicit immutable policies', 'Money records preserve identity precision and deconstruction']

DECIMAL_NAMES = ['Decimal arithmetic preserves exact base ten values', 'Decimal equality normalizes scale without changing storage', 'Decimal parsing formatting and rounding preserve precision', 'Decimal syntax uses ordinary value method contracts', 'Decimal boundaries reject overflow and round midpoint ties', 'Decimal arithmetic agrees with CLR boundary vectors', 'Decimal culture formatting and representation use existing contracts']

FLOATING_SPECIAL_NAMES = [
    'Floating named constants preserve IEEE and value equality contracts',
    'Floating classification covers both widths signs and finite values',
    'Floating special values format and parse through existing contracts',
]
RECORD_FORMATTING_NAMES = [
    'Record enum payload display survives boxed dispatch and collection',
    'Record enum display preserves names flags and numeric fallback',
    'Floating record contracts agree across boxed interface and nested calls',
    'Floating records use value equality for NaNs zeros and infinities',
    'Floating record display uses primitive text including negative zero',
    'Generic record display uses nested and boxed value dispatch',
    'Generic record display preserves text null and primitive contracts',
    'Records use imported value formatting across typed and boxed calls',
    'Inherited record equality observes runtime type and derived state',
    'Inherited record display includes base members through virtual calls',
    'Nested records and unions retain imported value display contracts',
    'Recursive record shapes display finite reference chains',
    'Root record classes preserve value equality and reference identity',
    'Record class display preserves nested and absent references',
    'Record scalar formatting preserves null and original Unicode text',
]
INDEX_RANGE_NAMES = [
    'Index preserves direction and resolves end boundaries',
    'Index and Range typed equality agrees with hashing',
    'Range resolution accepts empty and full boundaries',
    'Array range syntax supports omitted and end-relative boundaries',
    'Array indexes support values and from-end assignment',
    'Array slices copy storage and preserve reference elements',
    'Array indexing and slicing evaluate operands once in source order',
    'Read-only value fields use defensive copies for method calls',
    'Array safe access returns options and results for invalid bounds',
    'String ranges preserve graphemes and half-open boundaries',
    'String safe access rejects invalid grapheme bounds',
    'String slicing evaluates receiver and bounds once',
    'ArrayList supports directional mutation and independent range copies',
    'Sequence slicing consumes one iterator without indexed reads',
    'Sequence invalid and empty slices do not acquire iterators',
    'Sequence slicing disposes when enumeration ends before Count',
    'Sequence safe indexing and slicing work through array aliases',
    'Sequence extension indexers provide default access and independent slices',
    'Sequence slicing delegates to the list range contract',
    'Native extension indexers support writes and compound assignment',
]
HTTP_METHOD_VALUES_NAMES = [
    'HTTP coffee requests retain BREW and unnamed status codes',
    'HTTP custom method construction and parsing preserve spelling',
    'HTTP method equality and hashes ignore ASCII case',
    'HTTP methods retain case-insensitive identity in a set',
    'HTTP method validation admits token punctuation and rejects separators',
    'HTTP method parsing canonicalizes every known name',
    'HTTP predefined method values use canonical tokens',
    'HTTP request factories retain predefined and custom methods',
    'HTTP URI request factories support custom methods and empty content',
]
HTTP_HEADER_VALUES_NAMES = [
    'HTTP header lookup distinguishes empty values from absent fields',
    'HTTP header lookup rejects invalid names',
    'HTTP header lookup preserves repeated values without splitting commas',
    'HTTP request lookup exposes content headers without inventing provider headers',
    'HTTP header lookup snapshots remain independent of later additions',
    'HTTP messages expose HTTP 1.1 and dispose owned content without reading',
    'HTTP response constructors retain memory content and replace only content type metadata',
    'HTTP byte producers allocate runtime sized arrays with ordered single endpoint evaluation',
    'HTTP response construction never reads or changes stream ownership',
]
HTTP_ROUTE_MATCHING_NAMES = [
    'Route matching preserves case segment and trailing-slash distinctions',
    'Route byte quotas admit the boundary and reject one extra byte',
    'Route matching separates query text and converts captured integers',
    'Route captures decode UTF-8 once and preserve literal plus',
    'Route integer conversion distinguishes path matching from conversion failure',
    'Route parsing rejects malformed and unsupported pattern syntax',
    'Route matching validates malformed targets even when literals mismatch',
    'Route segment quotas admit the boundary and reject one extra segment',
]
IP_ADDRESS_VALUES_NAMES = [
    'IPAddress values survive subsequent parse allocations',
    'IPv4 parsing rejects malformed and unsupported corpus forms',
    'IPv6 parsing rejects malformed and unsupported corpus forms',
    'IPAddress Object dispatch preserves value equality hashes and family distinctions',
    'IPAddress parsing distinguishes unsupported scope from invalid format',
    'IPv4 parsing and canonical formatting round trip',
    'IPv6 parsing and canonical formatting round trip',
]
URI_RESOLUTION_NAMES = [
    'URI resolution handles authority and path bases and rejects relative bases',
    'URI byte parsing preserves ASCII boundaries and encoded Unicode',
    'URI classification and equality preserve literal identity',
    'URI parsing rejects every invalid grammar corpus input',
    'URI parsing and resolution enforce the byte quota',
    'URI reference resolution corpus group 1',
    'URI reference resolution corpus group 2',
    'URI reference resolution corpus group 3',
    'URI reference resolution corpus group 4',
    'URI reference resolution corpus group 5',
    'URI reference resolution corpus group 6',
    'URI parsing reports unsupported bracketed authorities',
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
    parser.add_argument('--aot', type=Path, default=ROOT / 'target/debug' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--suite', action='append', choices=('native-callbacks', 'native-buffers', 'native-pointers', 'method-state', 'type-relations', 'collections', 'collection-construction', 'collection-iteration', 'json-dom', 'json-streams', 'memory-stream', 'memory-lifetimes', 'string-construction', 'unicode-casing', 'encoding', 'encoding-streams', 'cryptography', 'gzip', 'encoding-boundaries', 'encoding-buffer-limits', 'int64-parsing', 'primitive-parsing', 'numeric-formatting', 'globalization', 'enum-parsing', 'record-formatting', 'record-allocations', 'floating-special-values', 'decimal', 'currency', 'deconstruction', 'localization', 'dependency-injection', 'path-values', 'http-header-values', 'http-method-values', 'index-range', 'http-text', 'http-content-copy', 'http-lifetime', 'http-route-matching', 'ip-address-values', 'uri-resolution', 'time-parsing', 'duration-offset-parsing', 'instant-parsing', 'zoned-time-parsing', 'discovery-contract', 'runner-contract'),
                        help='Run only this suite (repeatable); default runs qualified suites')
    parser.add_argument("--execution-repetitions", type=int, default=1, help="Repeat the full suite process in each mode for timing; guards/selectors still run once")
    parser.add_argument('--mode', action='append', choices=('native', 'interpreter', 'jit'),
                        help='Execution modes to qualify (repeatable); defaults to JIT')
    parser.add_argument('--jit-host', type=Path, help='Matching trusted native helper library for JIT')
    args = parser.parse_args()
    modes = args.mode or ['jit']
    if args.suite and 'record-allocations' in args.suite and set(modes) != {'interpreter'}:
        parser.error('record-allocations requires --mode interpreter; native/JIT GC counters are not bound')
    bundled_bin = args.bundle.resolve() / 'bin'
    if (bundled_bin / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib')).is_file():
        if args.runtime == parser.get_default('runtime'):
            args.runtime = bundled_bin / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
        if args.aot == parser.get_default('aot'):
            args.aot = bundled_bin / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen')
    if 'jit' in modes and not args.jit_host:
        args.jit_host = bundled_bin / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib')
    if 'jit' in modes and not args.jit_host.is_file():
        parser.error('JIT requires a prepared JIT bundle or a matching --jit-host; select --mode interpreter for interpreter-only testing')
    if args.execution_repetitions < 1:
        parser.error("--execution-repetitions must be positive")
    run_started = time.perf_counter()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    inputs = [Path(__file__).resolve(), ROOT / 'scripts/build-native-project.py', ROOT / 'scripts/run-native-sample.py', ROOT / 'scripts/discover-runtime-tests.py', ROOT / 'scripts/validate-test-discovery.py',
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.cs')),
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.csproj')),
              *sorted(TESTS.rglob('*.rvn')), *sorted(TESTS.rglob('*.rvnproj')),
              *sorted((TESTS / 'localization').rglob('*.json')),
              *sorted((ROOT / 'docs/experiments/dependency-injection/sample').glob('*.rvn')),
              *sorted((ROOT / 'runtime/raven/src/System/DependencyInjection').glob('*.rvn'))]
    report = dict(passed=False, modes=modes, jitHostSha256=sha(args.jit_host.resolve()) if args.jit_host else None, platform=platform.platform(), cases=[], phases=[], executionRepetitions=args.execution_repetitions,
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  bundleManifestSha256=sha(bundle / 'manifest.json'),
                  aotSha256=sha(args.aot.resolve()) if any(mode in modes for mode in ('native', 'jit')) else None, interpreterSha256=sha(args.runtime.resolve()))
    validation_spec = importlib.util.spec_from_file_location('discovery_validation', ROOT / 'scripts/validate-test-discovery.py')
    validation = importlib.util.module_from_spec(validation_spec)
    validation_spec.loader.exec_module(validation)
    relationship_names = ['Arrays preserve element identity', 'Closed generic types remain distinct', 'Type identity and assignment direction', 'Boxing and instance checks reject null', 'Interface assignment includes inherited contracts', 'Subclass checks are strict and exclude interfaces']
    expected = [
        ('method-state', 0, 'PASS Method discovery filters visibility and scope on value types\nPASS Method generic state separates owner and method parameters\nTests: 2, passed: 2, failed: 0, skipped: 0\n'),
        ('type-relations', 0, ''.join('PASS ' + n + '\n' for n in relationship_names) + f'Tests: {len(relationship_names)}, passed: {len(relationship_names)}, failed: 0, skipped: 0\n'),
        ('http-lifetime', 0, ''.join('PASS ' + n + '\n' for n in HTTP_LIFETIME_NAMES) + f'Tests: {len(HTTP_LIFETIME_NAMES)}, passed: {len(HTTP_LIFETIME_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-content-copy', 0, ''.join('PASS ' + n + '\n' for n in HTTP_CONTENT_COPY_NAMES) + f'Tests: {len(HTTP_CONTENT_COPY_NAMES)}, passed: {len(HTTP_CONTENT_COPY_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-text', 0, ''.join('PASS ' + n + '\n' for n in HTTP_TEXT_NAMES) + f'Tests: {len(HTTP_TEXT_NAMES)}, passed: {len(HTTP_TEXT_NAMES)}, failed: 0, skipped: 0\n'),
        ('collections', 0, ''.join('PASS ' + n + '\n' for n in COLLECTION_NAMES) + f'Tests: {len(COLLECTION_NAMES)}, passed: {len(COLLECTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('collection-construction', 0, ''.join('PASS ' + n + '\n' for n in CONSTRUCTION_NAMES) + f'Tests: {len(CONSTRUCTION_NAMES)}, passed: {len(CONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('collection-iteration', 0, ''.join('PASS ' + n + '\n' for n in ITERATION_NAMES) + f'Tests: {len(ITERATION_NAMES)}, passed: {len(ITERATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('json-dom', 0, ''.join('PASS ' + n + '\n' for n in JSON_NAMES) + f'Tests: {len(JSON_NAMES)}, passed: {len(JSON_NAMES)}, failed: 0, skipped: 0\n'),
        ('json-streams', 0, ''.join('PASS ' + n + '\n' for n in JSON_STREAM_NAMES) + f'Tests: {len(JSON_STREAM_NAMES)}, passed: {len(JSON_STREAM_NAMES)}, failed: 0, skipped: 0\n'),
        ('memory-lifetimes', 0, ''.join('PASS ' + n + '\n' for n in MEMORY_LIFETIME_NAMES) + f'Tests: {len(MEMORY_LIFETIME_NAMES)}, passed: {len(MEMORY_LIFETIME_NAMES)}, failed: 0, skipped: 0\n'),
        ('memory-stream', 0, ''.join('PASS ' + n + '\n' for n in MEMORY_STREAM_NAMES) + f'Tests: {len(MEMORY_STREAM_NAMES)}, passed: {len(MEMORY_STREAM_NAMES)}, failed: 0, skipped: 0\n'),
        ('string-construction', 0, ''.join('PASS ' + n + '\n' for n in STRING_CONSTRUCTION_NAMES) + f'Tests: {len(STRING_CONSTRUCTION_NAMES)}, passed: {len(STRING_CONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('encoding-buffer-limits', 0, ''.join('PASS ' + n + '\n' for n in ENCODING_BUFFER_NAMES) + f'Tests: {len(ENCODING_BUFFER_NAMES)}, passed: {len(ENCODING_BUFFER_NAMES)}, failed: 0, skipped: 0\n'),
        ('encoding-boundaries', 0, ''.join('PASS ' + n + '\n' for n in ENCODING_BOUNDARY_NAMES) + f'Tests: {len(ENCODING_BOUNDARY_NAMES)}, passed: {len(ENCODING_BOUNDARY_NAMES)}, failed: 0, skipped: 0\n'),
        ('encoding-streams', 0, ''.join('PASS ' + n + '\n' for n in ENCODING_STREAM_NAMES) + f'Tests: {len(ENCODING_STREAM_NAMES)}, passed: {len(ENCODING_STREAM_NAMES)}, failed: 0, skipped: 0\n'),
        ('gzip', 0, 'PASS Gzip decodes independent stored fixed dynamic and optional-header members\nPASS Gzip validates checksums truncation headers and quotas before output\nPASS Gzip incrementally compresses repeated writes and finishes once\nPASS Gzip empty finish ownership ranges modes and cancellation are explicit\nPASS Gzip awaits output and poisons interrupted streams without replay\nPASS Gzip handles pending fragmented input and preserves text-reader errors\nPASS Gzip borrows only the selected source window across compression blocks\nTests: 7, passed: 7, failed: 0, skipped: 0\n'),
        ('cryptography', 0, 'PASS SHA256 preserves aliases tails and short destinations\nPASS HMACSHA256 verifies known answers and rejects altered tags\nPASS RandomNumberGenerator accepts empty and writable buffers\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
        ('encoding', 0, ''.join('PASS ' + n + '\n' for n in ENCODING_NAMES) + f'Tests: {len(ENCODING_NAMES)}, passed: {len(ENCODING_NAMES)}, failed: 0, skipped: 0\n'),
        ('unicode-casing', 0, ''.join('PASS ' + n + '\n' for n in UNICODE_CASING_NAMES) + f'Tests: {len(UNICODE_CASING_NAMES)}, passed: {len(UNICODE_CASING_NAMES)}, failed: 0, skipped: 0\n'),
        ('int64-parsing', 0, ''.join('PASS ' + n + '\n' for n in INT64_PARSING_NAMES) + f'Tests: {len(INT64_PARSING_NAMES)}, passed: {len(INT64_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('dependency-injection', 0, ''.join('PASS ' + n + '\n' for n in DEPENDENCY_INJECTION_NAMES) + f'Tests: {len(DEPENDENCY_INJECTION_NAMES)}, passed: {len(DEPENDENCY_INJECTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('localization', 0, ''.join('PASS ' + n + '\n' for n in LOCALIZATION_NAMES) + f'Tests: {len(LOCALIZATION_NAMES)}, passed: {len(LOCALIZATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('deconstruction', 0, ''.join('PASS ' + n + '\n' for n in DECONSTRUCTION_NAMES) + f'Tests: {len(DECONSTRUCTION_NAMES)}, passed: {len(DECONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('currency', 0, ''.join('PASS ' + n + '\n' for n in CURRENCY_NAMES) + f'Tests: {len(CURRENCY_NAMES)}, passed: {len(CURRENCY_NAMES)}, failed: 0, skipped: 0\n'),
        ('decimal', 0, ''.join('PASS ' + n + '\n' for n in DECIMAL_NAMES) + f'Tests: {len(DECIMAL_NAMES)}, passed: {len(DECIMAL_NAMES)}, failed: 0, skipped: 0\n'),
        ('floating-special-values', 0, ''.join('PASS ' + n + '\n' for n in FLOATING_SPECIAL_NAMES) + f'Tests: {len(FLOATING_SPECIAL_NAMES)}, passed: {len(FLOATING_SPECIAL_NAMES)}, failed: 0, skipped: 0\n'),
        ('record-allocations', 0, 'PASS Record class equality avoids reflection descriptor allocations\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
        ('record-formatting', 0, ''.join('PASS ' + n + '\n' for n in RECORD_FORMATTING_NAMES) + f'Tests: {len(RECORD_FORMATTING_NAMES)}, passed: {len(RECORD_FORMATTING_NAMES)}, failed: 0, skipped: 0\n'),
        ('native-callbacks', 0, 'PASS Native callbacks preserve captures across repeated calls\nPASS Native callbacks preserve generic unit value returns\nPASS Native callbacks accept Raven module functions\nPASS Native unit callbacks discard the logical Void result\nPASS Native callbacks retain bound receivers for the call\nTests: 5, passed: 5, failed: 0, skipped: 0\n'),
        ('native-buffers', 0, 'PASS Repeated native byte copies allocate no managed transfer buffers\nPASS Empty native byte copies never dereference null pointers\nPASS Native buffer copies bridge Memory windows through a C import\nPASS Native byte copies accept borrowed UTF8 without a temporary array\nTests: 4, passed: 4, failed: 0, skipped: 0\n'),
        ('native-pointers', 0, 'PASS Default typed native pointers preserve null and generic forwarding\nPASS Typed native pointers compare addresses and preserve borrowed storage\nPASS Typed native pointer hashes match aliases without reading storage\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
        ('enum-parsing', 0, ''.join('PASS ' + n + '\n' for n in ENUM_PARSING_NAMES) + f'Tests: {len(ENUM_PARSING_NAMES)}, passed: {len(ENUM_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('globalization', 0, ''.join('PASS ' + n + '\n' for n in GLOBALIZATION_NAMES) + f'Tests: {len(GLOBALIZATION_NAMES)}, passed: {len(GLOBALIZATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('numeric-formatting', 0, ''.join('PASS ' + n + '\n' for n in NUMERIC_FORMATTING_NAMES) + f'Tests: {len(NUMERIC_FORMATTING_NAMES)}, passed: {len(NUMERIC_FORMATTING_NAMES)}, failed: 0, skipped: 0\n'),
        ('primitive-parsing', 0, ''.join('PASS ' + n + '\n' for n in PRIMITIVE_PARSING_NAMES) + f'Tests: {len(PRIMITIVE_PARSING_NAMES)}, passed: {len(PRIMITIVE_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('path-values', 0, ''.join('PASS ' + n + '\n' for n in PATH_VALUES_NAMES) + f'Tests: {len(PATH_VALUES_NAMES)}, passed: {len(PATH_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('index-range', 0, ''.join('PASS ' + n + '\n' for n in INDEX_RANGE_NAMES) + f'Tests: {len(INDEX_RANGE_NAMES)}, passed: {len(INDEX_RANGE_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-method-values', 0, ''.join('PASS ' + n + '\n' for n in HTTP_METHOD_VALUES_NAMES) + f'Tests: {len(HTTP_METHOD_VALUES_NAMES)}, passed: {len(HTTP_METHOD_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-header-values', 0, ''.join('PASS ' + n + '\n' for n in HTTP_HEADER_VALUES_NAMES) + f'Tests: {len(HTTP_HEADER_VALUES_NAMES)}, passed: {len(HTTP_HEADER_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-route-matching', 0, ''.join('PASS ' + n + '\n' for n in HTTP_ROUTE_MATCHING_NAMES) + f'Tests: {len(HTTP_ROUTE_MATCHING_NAMES)}, passed: {len(HTTP_ROUTE_MATCHING_NAMES)}, failed: 0, skipped: 0\n'),
        ('ip-address-values', 0, ''.join('PASS ' + n + '\n' for n in IP_ADDRESS_VALUES_NAMES) + f'Tests: {len(IP_ADDRESS_VALUES_NAMES)}, passed: {len(IP_ADDRESS_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('uri-resolution', 0, ''.join('PASS ' + n + '\n' for n in URI_RESOLUTION_NAMES) + f'Tests: {len(URI_RESOLUTION_NAMES)}, passed: {len(URI_RESOLUTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('zoned-time-parsing', 0, ''.join('PASS ' + n + '\n' for n in ZONED_TIME_PARSING_NAMES) + f'Tests: {len(ZONED_TIME_PARSING_NAMES)}, passed: {len(ZONED_TIME_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('instant-parsing', 0, ''.join('PASS ' + n + '\n' for n in INSTANT_PARSING_NAMES) + f'Tests: {len(INSTANT_PARSING_NAMES)}, passed: {len(INSTANT_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('duration-offset-parsing', 0, ''.join('PASS ' + n + '\n' for n in DURATION_OFFSET_PARSING_NAMES) + f'Tests: {len(DURATION_OFFSET_PARSING_NAMES)}, passed: {len(DURATION_OFFSET_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('time-parsing', 0, ''.join('PASS ' + n + '\n' for n in TIME_PARSING_NAMES) + f'Tests: {len(TIME_PARSING_NAMES)}, passed: {len(TIME_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('discovery-contract', 1, 'PASS first discovered test\nFAIL discovered failure: Expected 1, actual 2\nPASS after discovered failure\nPASS NeoClr.DiscoveryTests.DWithoutDescription\nPASS NeoClr.DiscoveryTests.EInline(2, 3, 5)\nPASS NeoClr.DiscoveryTests.EInline(5, 6, 11)\nPASS NeoClr.DiscoveryTests.FSource/NeoClr.DiscoveryTests.Inputs[0]\nPASS NeoClr.DiscoveryTests.FSource/NeoClr.DiscoveryTests.Inputs[1]\nPASS manually registered companion\nTests: 9, passed: 8, failed: 1, skipped: 0\n'),
        ('runner-contract', 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nSKIP intentional skip: contract probe\nTests: 4, passed: 2, failed: 1, skipped: 1\n'),
    ]
    try:
        if 'native' not in modes:
            # Preserve the native builder's bundle-integrity check when bypassing
            # native compilation. A manifest alone does not validate its files.
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            if catalog.get('version') != 1 or catalog.get('objectAssembly') != 'System.Runtime':
                raise ValueError('Requires the split native class-library bundle v1')
            for relative, expected_hash in catalog['files'].items():
                path = (lib / relative).resolve()
                if Path(relative).is_absolute() or lib not in path.parents or not path.is_file() or sha(path) != expected_hash:
                    raise ValueError('Bundle file mismatch: ' + relative)
            report['libraryCatalogSha256'] = sha(lib / 'bundle.json')
        # Only explicitly qualified suites enter the default release gate.
        selected = [case for case in expected
                    if (not args.suite and case[0] not in ('record-allocations', 'native-pointers', 'native-buffers', 'native-callbacks')) or (args.suite and case[0] in args.suite)]
        report['suites'] = [case[0] for case in selected]
        if not args.suite:
            validation.validate(bundle, out / 'discovery-signatures')
        for name, exit_code, stdout in selected:
            build = out / (name + '-build')
            environment = dict(os.environ)
            fixture_environment = {}
            if name == 'native-callbacks':
                if modes != ['interpreter'] or os.name == 'nt':
                    raise ValueError('native-callbacks currently requires the Unix interpreter profile')
                fixture = out / ('native-callback-fixture.dylib' if sys.platform == 'darwin' else 'native-callback-fixture.so')
                subprocess.run([os.environ.get('CC', 'cc'), '-dynamiclib' if sys.platform == 'darwin' else '-shared',
                                '-fPIC', str(TESTS / name / 'fixture.c'), '-o', str(fixture)], check=True)
                imports = out / 'NativeCallbackImports.rvn'
                template = TESTS / name / 'Imports.rvn'
                imports.write_text(template.read_text().replace('"native-callback-fixture"', json.dumps(str(fixture.resolve()))))
                for source in (TESTS / name).iterdir():
                    if source.is_file():
                        report['inputs'][str(source.relative_to(ROOT))] = sha(source)
                fixture_environment['NeoClrNativeCallbackImports'] = str(imports.resolve())
                environment.update(fixture_environment)
                report['nativeCallbackFixtureSha256'] = sha(fixture)
            if name == 'native-buffers':
                if modes != ['interpreter'] or os.name == 'nt':
                    raise ValueError('native-buffers currently requires the Unix interpreter profile')
                fixture = out / ('native-buffer-fixture.dylib' if sys.platform == 'darwin' else 'native-buffer-fixture.so')
                subprocess.run([os.environ.get('CC', 'cc'), '-dynamiclib' if sys.platform == 'darwin' else '-shared',
                                '-fPIC', str(TESTS / name / 'fixture.c'), '-o', str(fixture)], check=True)
                imports = out / 'NativeBufferImports.rvn'
                imports.write_text('import System.Runtime.InteropServices.*\n[NativeImport(' + json.dumps(str(fixture.resolve())) +
                                   ', EntryPoint: "native_buffer_increment")]\nunsafe extern func Increment(buffer: *byte, count: int) -> System.Void;\n')
                fixture_environment['NeoClrNativeBufferImports'] = str(imports.resolve())
                environment.update(fixture_environment)
                report['nativeBufferFixtureSha256'] = sha(fixture)
            if name not in ('runner-contract', 'memory-lifetimes', 'http-content-copy'):
                started = time.perf_counter()
                registry = DISCOVERY.discover(TESTS / name / 'Tests.rvnproj', bundle, out / (name + '-discovery'), fixture_environment)
                report['phases'].append(dict(suite=name, phase='discovery', elapsedSeconds=round(time.perf_counter() - started, 6), details=json.loads((out / (name + '-discovery') / 'timings.json').read_text())))
                environment['NeoClrTestRegistry'] = str(registry)
            # Async memory tests need lifecycle bindings; JSON DOM fixtures also select HTTP content/socket handlers.
            profile = 'http' if name in ('http-text', 'http-content-copy', 'http-lifetime', 'memory-stream', 'memory-lifetimes', 'gzip', 'json-dom', 'json-streams', 'localization', 'encoding-streams', 'encoding-boundaries') else 'console'
            if os.name == 'nt':
                profile = 'windows-' + profile
            if name in ('dependency-injection', 'method-state', 'localization'):
                # Derive exact identities from the discovered application, never from
                # hard-coded metadata ordinals. Only public test service fixtures
                # receive constructor metadata and executable retention.
                compiled = (out / (name + '-discovery') / 'compile.stdout.log').read_text()
                assembly = next(line.removeprefix('Native build output: ') for line in compiled.splitlines()
                                if line.startswith('Native build output: '))
                listing = out / (name + '-reflection-module.neoil')
                subprocess.run([str(args.runtime.resolve()), 'disassemble', assembly, str(listing)], check=True, capture_output=True)
                text = listing.read_text()
                metadata, _ = json.JSONDecoder().raw_decode(text[text.index('{'):])
                reflection_names = [declaration['origin']['name'] for declaration in metadata['types']
                                    if declaration.get('origin', {}).get('name', '').startswith(('NeoClr.RuntimeTests.', 'Examples.DependencyInjection.'))
                                    and (declaration.get('origin', {}).get('publicly_visible') if name == 'dependency-injection'
                                         else declaration['origin']['name'].endswith('.LocalizedGreeting') if name == 'localization'
                                         else any(marker in declaration['origin']['name'] for marker in ('MethodOwner', 'MethodValue')))]
                if name == 'dependency-injection':
                    di_types = reflection_names
                elif name == 'localization':
                    localization_types = reflection_names + ["System.Localization.DefaultLocalizer`1"]
                else:
                    method_types = reflection_names
            if name == "json-streams":
                # Scalar ObjectMapper paths select reflection services. Retain only the
                # runtime's first type identity, without constructors or members.
                listing = out / 'json-reflection-module.neoil'
                subprocess.run([str(args.runtime.resolve()), 'disassemble',
                                str(bundle / 'lib/System.Runtime.dll'), str(listing)], check=True)
                listing_text = listing.read_text()
                metadata, _ = json.JSONDecoder().raw_decode(listing_text[listing_text.index('{'):])
                json_roots = out / 'json-reflection-roots.json'
                json_roots.write_text(json.dumps(dict(schemaVersion=2, types=[dict(
                    definition=dict(module=metadata['name'], revision=metadata['revision'], index=0),
                    construct=False, properties=False, getters=False, setters=False)]), indent=2) + '\n')
            if 'native' in modes:
                command = [sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                           profile, '--project', TESTS / name / 'Tests.rvnproj',
                           '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build]
                if name in ("encoding-buffer-limits", "http-text", "localization"):
                    command += ["--byte-array-limit", "131072"]
                if name == "gzip":
                    command += ["--http-heap-bytes", "33554432"]
                if name == "json-streams":
                    # Scalar mapping needs a reflection scope, but no retained model constructors.
                    command += ["--reflection-roots", json_roots, "--command-timeout", "600"]
                if name == "localization":
                    command += [arg for type_name in localization_types for arg in ("--reflection-constructor-type", type_name)]
                    command += ["--command-timeout", "600"]
                if name == "method-state":
                    command += [arg for type_name in method_types for arg in ("--reflection-method-type", type_name)]
                    command += ["--command-timeout", "600"]
                if name == "dependency-injection":
                    command += [arg for type_name in di_types for arg in ("--reflection-constructor-type", type_name)]
                    command += ["--command-timeout", "600"]
                phase = 'native-project-build'
            else:
                # Raven emits the same metadata assembly directly. JIT/interpreter
                # runs do not require an unused native object, linker or executable.
                build.mkdir()
                environment['NeoClrBundleRoot'] = str(bundle)
                command = ['dotnet', bundle / 'sdk/tools/rvnc/rvnc.dll', 'neoclr',
                           '--project', TESTS / name / 'Tests.rvnproj']
                phase = 'source-compilation'
            started = time.perf_counter()
            result = subprocess.run(list(map(str, command)), cwd=ROOT, env=environment, capture_output=True, timeout=600)
            report['phases'].append(dict(suite=name, phase=phase, elapsedSeconds=round(time.perf_counter() - started, 6)))
            (out / (name + '-build.stdout.log')).write_bytes(result.stdout)
            (out / (name + '-build.stderr.log')).write_bytes(result.stderr)
            if result.returncode:
                raise RuntimeError(name + ' build failed; see retained logs')
            if 'native' in modes:
                report['phases'][-1]['commands'] = [{k: c[k] for k in ('phase', 'elapsedSeconds')} for c in json.loads((build / 'build.json').read_text())['commands']]
            else:
                paths = [line.removeprefix('Native build output: ') for line in result.stdout.decode('utf-8').splitlines()
                         if line.startswith('Native build output: ')]
                if not paths or not Path(paths[-1]).is_absolute() or not Path(paths[-1]).is_file():
                    raise RuntimeError('Raven did not report a valid project assembly')
                shutil.copy2(paths[-1], build / 'app.dll')
                report['phases'][-1].update(command=list(map(str,command)), assemblySha256=sha(build / 'app.dll'))
            if name not in ('runner-contract', 'memory-lifetimes', 'http-content-copy'):
                started = time.perf_counter()
                DISCOVERY.verify_registration(build / 'app.dll', bundle, out / (name + '-discovery'))
                report['phases'].append(dict(suite=name, phase='registration-verification', elapsedSeconds=round(time.perf_counter() - started, 6)))
            if name == 'dependency-injection':
                di_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', di_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), constructors=True)[1]
            if name == 'localization':
                localization_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', localization_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), constructors=True, modules=[bundle / path for path in json.loads((bundle / "manifest.json").read_text())["modules"]])[1]
            if name == 'method-state':
                method_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', method_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), methods=True)[1]
            isolated = out / (name + '-isolated')
            isolated.mkdir()
            if name == 'localization':
                for fixture in ('catalogue.json', 'invalid.json'):
                    shutil.copyfile(ROOT / 'runtime/raven/tests/localization' / fixture, isolated / fixture)
                shutil.copytree(ROOT / 'runtime/raven/tests/localization/resources', isolated / 'resources')
            exe = isolated / ('app.exe' if os.name == 'nt' else 'app')
            if 'native' in modes:
                shutil.copy2(build / exe.name, exe)
            lib = bundle / 'lib'
            catalog = json.loads((lib / 'bundle.json').read_text())
            interpreter = [args.runtime.resolve(), 'run', build / 'app.dll', '--system', lib / catalog['runtimeSeed'],
                           *[arg for a in catalog['assemblyNames'] for arg in ('--module', lib / (a + '.dll'))],
                           '--object-root', lib / 'System.Runtime.dll', '--instructions', '100000000']
            if name in ("encoding-buffer-limits", "http-text", "gzip", "localization"):
                interpreter += ["--array-elements", "1048576", "--array-bytes", "268435456"]
                report["gzipBufferBudgets" if name == "gzip" else "encodingBufferBudgets"] = dict(nativeByteArrayElements=65536 if name == "gzip" else 131072, interpretedLiveArrayElements=1048576, interpretedLogicalArrayBytes=268435456)
            env = {k: v for k, v in os.environ.items() if k.upper() in ('SYSTEMROOT', 'WINDIR', 'TEMP', 'TMP')}
            executions = []
            for mode in modes:
                if mode == 'native':
                    executions.append((mode, [exe], isolated, env))
                elif mode == 'interpreter':
                    executions.append((mode, interpreter + ['--execution', 'interpreter'], isolated if name == 'localization' else ROOT, None))
                else:
                    jit = interpreter[:interpreter.index('--instructions')] + [
                        '--jit-host', args.jit_host.resolve(),
                        '--jit-compiler', args.aot.resolve()]
                    if name == "json-streams":
                        jit += ["--reflection-roots", json_roots]
                    if name == "dependency-injection":
                        jit += ["--reflection-roots", di_roots]
                    if name == "method-state":
                        jit += ["--reflection-roots", method_roots]
                    if name == "localization":
                        jit += ["--reflection-roots", localization_roots]
                    executions.append((mode, jit, isolated if name == 'localization' else ROOT, None))
            repeated_executions = [(repetition, *execution) for repetition in range(1, args.execution_repetitions + 1) for execution in executions]
            for repetition, mode, command, cwd, environment in repeated_executions:
                started = time.perf_counter()
                result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=600 if name in ("http-text", "gzip") else 180)
                record = dict(suite=name, mode=mode, repetition=repetition, exitCode=result.returncode,
                              elapsedSeconds=round(time.perf_counter() - started, 3),
                              stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                              stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                report['cases'].append(record)
                print(name + ' (' + mode + '):\n' + record['stdout'], end='', flush=True)
                if (record['exitCode'], record['stdout'], record['stderr']) != (exit_code, stdout, ''):
                    raise RuntimeError(name + ' ' + mode + ' did not match the runner contract')
            if name == 'index-range':
                guards = {
                    'negative-index': 'Index value must be nonnegative',
                    'negative-conversion': 'Index value must be nonnegative',
                    'negative-length': 'Range length must be nonnegative',
                    'list-capacity': 'Range is outside the value',
                    'list-end': 'ArrayList index out of range',
                    'string-reversed': 'Range is outside the value',
                    'string-past-end': 'Range is outside the value',
                    'reversed': 'Range is outside the value',
                    'past-end': 'Range is outside the value',
                    'before-start': 'Range is outside the value',
                }
                for guard, message in guards.items():
                    for mode, command, cwd, environment in executions:
                        arguments = [*command, *(['--'] if mode in ('interpreter', 'jit') else []), '--guard', guard]
                        started = time.perf_counter()
                        result = subprocess.run(list(map(str, arguments)), cwd=cwd, env=environment, capture_output=True, timeout=30)
                        record = dict(suite=name, mode=mode, guard=guard, exitCode=result.returncode, elapsedSeconds=round(time.perf_counter() - started, 6),
                                      stdout=result.stdout.decode('utf-8'), stderr=result.stderr.decode('utf-8'))
                        report['cases'].append(record)
                        if not record['exitCode'] or message not in record['stderr'] or record['stdout']:
                            raise RuntimeError('Index/Range guard did not fault as expected: ' + repr(record))
                print('index-range: bounds guards passed in selected modes', flush=True)
            selections = {
                'collections': [(['--filter', 'ArrayQueue'], 0, 'PASS ArrayQueue preserves FIFO\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                (['--filter', 'Map'], 0, ''.join('PASS ' + n + '\n' for n in MAP_NAMES) + f'Tests: {len(MAP_NAMES)}, passed: {len(MAP_NAMES)}, failed: 0, skipped: 0\n')],
                'discovery-contract': [(['--category', 'fast'], 0, 'PASS first discovered test\nPASS NeoClr.DiscoveryTests.EInline(2, 3, 5)\nPASS NeoClr.DiscoveryTests.EInline(5, 6, 11)\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
                                       (['--category', 'data'], 0, 'PASS NeoClr.DiscoveryTests.FSource/NeoClr.DiscoveryTests.Inputs[0]\nPASS NeoClr.DiscoveryTests.FSource/NeoClr.DiscoveryTests.Inputs[1]\nTests: 2, passed: 2, failed: 0, skipped: 0\n'),
                                       (['--category', 'FAST'], 2, 'ERROR No tests matched the selection\n'),
                                       (['--category', ''], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'), (['--filter', 'Inputs[1]'], 0, 'PASS NeoClr.DiscoveryTests.FSource/NeoClr.DiscoveryTests.Inputs[1]\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--filter', 'EInline(2, 3, 5)'], 0, 'PASS NeoClr.DiscoveryTests.EInline(2, 3, 5)\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--id', 'manual'], 0, 'PASS manually registered companion\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                       (['--filter', 'after discovered'], 0, 'PASS after discovered failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n')],
                'runner-contract': [
                    (['--entry-probe', '', 'two words', 'Räven ☕ 😀', 'quote"slash\\', '--id'], 0, 'PASS entry arguments preserve text\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.after'], 0, 'PASS after failure\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                    (['--id', 'runner.failure'], 1, 'FAIL intentional assertion failure: Expected 1, actual 2\nTests: 1, passed: 0, failed: 1, skipped: 0\n'),
                    (['--id', 'runner.skipped'], 0, 'SKIP intentional skip: contract probe\nTests: 1, passed: 0, failed: 0, skipped: 1\n'),
                    (['--filter', 'failure'], 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nTests: 3, passed: 2, failed: 1, skipped: 0\n'),
                    (['--filter', 'RUNNER'], 2, 'ERROR No tests matched the selection\n'),
                    (['--filter'], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'),
                    (['--filter', ''], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'),
                    (['--unknown', 'after'], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'),
                    (['--jit-managed', '--jit-run', '--target', '--jit-entry-argument'], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'),
                    (['--id', 'runner.after', '--filter', 'after'], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'),
                ],
            }.get(name, [])
            for arguments, selected_exit, selected_stdout in selections:
                for mode, base_command, cwd, environment in executions:
                    command = [*base_command, *(['--'] if mode in ('interpreter', 'jit') else []), *arguments]
                    started = time.perf_counter()
                    result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=600 if name in ("http-text", "gzip") else 180)
                    record = dict(suite=name, mode=mode, arguments=arguments, exitCode=result.returncode, elapsedSeconds=round(time.perf_counter() - started, 6),
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
        if (report['aotSha256'] and sha(args.aot.resolve()) != report['aotSha256']) or sha(args.runtime.resolve()) != report['interpreterSha256']:
            raise RuntimeError('Execution tool changed during validation')
        if args.jit_host and sha(args.jit_host.resolve()) != report['jitHostSha256']:
            raise RuntimeError('JIT helper library changed during validation')
        report['passed'] = True
    finally:
        report['totalElapsedSeconds'] = round(time.perf_counter() - run_started, 6)
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Runtime library tests: PASS')


if __name__ == '__main__':
    main()
