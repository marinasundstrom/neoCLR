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
TEST_PROJECTS = json.loads((ROOT / 'scripts/runtime-test-projects.json').read_text())
RETENTION_WARNING = ('warning: DAM2004: dynamic-dispatch implementation bodies are not analyzed; '
                     'reflection inside them may require explicit retention\n')


def expected_diagnostics(stderr, mode):
    return stderr == '' or mode == 'jit' and stderr == RETENTION_WARNING


def suite_project(name):
    return ROOT / TEST_PROJECTS[name]


def suite_directory(name):
    return suite_project(name).parent


def discovery_order(output):
    """Successful discovery suites report in ordinal qualified-function order."""
    lines = output.splitlines(keepends=True)
    if not lines or not lines[-1].startswith('Tests: ') or any(not line.startswith('PASS ') for line in lines[:-1]):
        raise ValueError('Expected a successful discovered-test transcript')
    return ''.join(sorted(lines[:-1])) + lines[-1]


ERROR_CONTEXT_NAMES = ['NeoClr.RuntimeTests.Platform and author errors opt into message and absent diagnostics', 'NeoClr.RuntimeTests.Context preserves typed payload and interface cause without capturing', 'NeoClr.RuntimeTests.Successful results skip lazy context and capture', 'NeoClr.RuntimeTests.Failure context evaluates once and propagates the typed wrapper', 'NeoClr.RuntimeTests.Nested context formats each cause once and keeps concise display', 'NeoClr.RuntimeTests.Explicit capture survives collection and later wrappers reuse its snapshot', 'NeoClr.RuntimeTests.Domain causes preserve trace and explicit mapping remains available', 'NeoClr.RuntimeTests.Malformed cyclic cause chains have bounded formatting and lookup', 'NeoClr.RuntimeTests.Unrelated result error types remain unconstrained', 'NeoClr.RuntimeTests.Application capture policy is opt in and explicit overrides win', 'NeoClr.RuntimeTests.Conditional startup configures capture only when DEBUG is defined', 'NeoClr.RuntimeTests.Interface result boundaries preserve runtime errors without automatic context']
ENVIRONMENT_NAMES = ['NeoClr.RuntimeTests.Platform version types retain distinct readable representations', 'NeoClr.RuntimeTests.Every OS case formats through base Object and interpolation', 'NeoClr.RuntimeTests.Distribution identity and release remain independently optional', 'NeoClr.RuntimeTests.Architecture cases preserve readable and unknown names', 'NeoClr.RuntimeTests.OS record equality preserves payload and runtime case identity', 'NeoClr.RuntimeTests.Live discovery returns execution platform architecture and newline', 'NeoClr.RuntimeTests.Discovered platform values survive garbage collection', 'NeoClr.RuntimeTests.Nested OS patterns select characteristics and cover every case']

HTTP_LIFETIME_NAMES = ['NeoClr.RuntimeTests.HTTP client use disposes its owned handler exactly once', 'NeoClr.RuntimeTests.HTTP borrowed handler survives one client and serves another', 'NeoClr.RuntimeTests.HTTP disposal requests cancellation and waits for handler acknowledgement', 'NeoClr.RuntimeTests.HTTP caller cancellation propagates without disposing the handler', 'NeoClr.RuntimeTests.HTTP completed responses survive disposal and retain request association', 'NeoClr.RuntimeTests.HTTP pre-cancellation skips custom handlers', 'NeoClr.RuntimeTests.HTTP socket handler disposal is idempotent through Disposable', 'NeoClr.RuntimeTests.HTTP delegating handlers run outward and unwind inward', 'NeoClr.RuntimeTests.HTTP borrowed delegating pipeline remains reusable', 'NeoClr.RuntimeTests.HTTP borrowed clients cancel only their own pending sends', 'NeoClr.RuntimeTests.HTTP disposed requests are rejected before invoking a custom handler']

MEMORY_LIFETIME_NAMES = [
    'Pending Memory destinations survive collection and preserve surrounding bytes',
    'Pending ReadOnlyMemory sources survive repeated suspension and collection',
    'Cancelled and rejected Memory transfers do not write or resume twice',
    'Default and end-position Memory windows remain empty after suspension',
]

HTTP_CONTENT_COPY_NAMES = ['HTTP content copy awaits writes before reading more and preserves ownership', 'HTTP memory content preserves selected windows and aliases across copies', 'HTTP content copy preserves stream failures and rejects truncated or invalid reads', 'HTTP content copy respects declared lengths and skips empty I/O', 'HTTP content copy cancellation skips pre-cancelled input and stops pending writes', 'HTTP content copy rejects disposed bodies and stops before the next source read', 'HTTP string reader consumes generic streams and preserves owned text and errors', 'HTTP stream creation validates before taking ownership or reading']

HTTP_TEXT_NAMES = [
    'NeoClr.RuntimeTests.HTTP buffered text preserves scalars beyond the one-shot codec quota',
    'NeoClr.RuntimeTests.HTTP buffered text rejects malformed and truncated large bodies',
    'NeoClr.RuntimeTests.HTTP buffered text preserves empty BOM and NUL data',
]
ENCODING_BUFFER_NAMES = [
    'NeoClr.RuntimeTests.UTF-8 decoder quota rejection preserves an incomplete scalar',
    'NeoClr.RuntimeTests.ASCII decoder accepts the maximum input after rejecting an oversized call',
    'NeoClr.RuntimeTests.Encoder drain quotas preserve accepted text and destination bytes',
    'NeoClr.RuntimeTests.UTF-8 decoder maximum input counts new bytes separately from carried bytes',
    'NeoClr.RuntimeTests.Native buffer comprehensions preserve selector indices and range endpoints',
    'NeoClr.RuntimeTests.One-shot decoding enforces byte limits and accepts the maximum',
]

ENCODING_BOUNDARY_NAMES = [
    'NeoClr.RuntimeTests.StreamReader decodes Unicode lines across single-byte reads',
    'NeoClr.RuntimeTests.StreamReader distinguishes empty lines EOF and lone carriage returns',
    'NeoClr.RuntimeTests.StreamReader delimits custom decoded output and bounds expansion',
    'NeoClr.RuntimeTests.StreamWriter reports zero progress and preserves flush failures',
    'NeoClr.RuntimeTests.StreamReader rejects invalid limits before consuming its input',
    'NeoClr.RuntimeTests.Encoding input quotas reject oversized chunks without poisoning sessions',
    'NeoClr.RuntimeTests.StreamWriter accepts its maximum byte count and preflights oversized lines',
    'NeoClr.RuntimeTests.StreamReader preserves input failures and rejects truncated UTF-8 at EOF',
]

ENCODING_STREAM_NAMES = [
    'NeoClr.RuntimeTests.StreamWriter completes short writes and preserves cross-input graphemes',
    'NeoClr.RuntimeTests.ASCII StreamWriter rejects complete lines before output and remains reusable',
    'NeoClr.RuntimeTests.Custom encoder trailers require explicit Finish and are emitted once',
    'NeoClr.RuntimeTests.Closing a writer manages ownership without implicit encoder finalization',
    'NeoClr.RuntimeTests.Partial write and final-output failures are observable without replay',
    'NeoClr.RuntimeTests.Custom encoder final acceptance waits for all trailer bytes to drain',
    'NeoClr.RuntimeTests.Selected ASCII reader recognizes CRLF and preserves borrowed ownership',
    'NeoClr.RuntimeTests.Selected readers reject malformed bytes and enforce decoded text limits',
    'NeoClr.RuntimeTests.StreamReader waits for pending input before decoding and completing',
    'NeoClr.RuntimeTests.StreamWriter waits for pending output before reporting the byte count',
    'NeoClr.RuntimeTests.Cancelled input cancels the pending reader operation',
]

ENCODING_NAMES = [
    'NeoClr.RuntimeTests.Encoding providers preserve UTF-8 and reject non-ASCII text',
    'NeoClr.RuntimeTests.UTF-8 decoding preserves text at every byte split',
    'NeoClr.RuntimeTests.UTF-8 decoders reject malformed scalars and truncated final input',
    'NeoClr.RuntimeTests.Decoder sessions copy pending input and preserve BOM data',
    'NeoClr.RuntimeTests.Decoder windows preserve text and ASCII errors are terminal',
    'NeoClr.RuntimeTests.Encoders preserve accepted input through one-byte drains',
    'NeoClr.RuntimeTests.Encoder chunks preserve scalars across the internal byte boundary',
    'NeoClr.RuntimeTests.ASCII encoder preflight is retryable and factories are independent',
    'NeoClr.RuntimeTests.One-shot decoding preserves Unicode BOM NUL and empty sequences',
    'NeoClr.RuntimeTests.One-shot errors do not poison providers and results are snapshots',
    'NeoClr.RuntimeTests.One-shot decoding borrows only its window and owns its result',
]

ZONED_TIME_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Zoned parsing preserves UTC and the pinned database version',
    'NeoClr.RuntimeTests.Zoned parsing validates winter and summer offsets',
    'NeoClr.RuntimeTests.Explicit offsets select both DST overlap candidates without losing ticks',
    'NeoClr.RuntimeTests.Zoned parsing rejects skipped local times for either adjacent offset',
    'NeoClr.RuntimeTests.Zoned parsing handles half hour transitions',
    'NeoClr.RuntimeTests.Zoned parsing enforces rule bounds and historical second precision',
    'NeoClr.RuntimeTests.Zoned parsing preserves lexical and civil range errors',
    'NeoClr.RuntimeTests.Named zone lookup remains ordinal and accepts pinned aliases',
]

INSTANT_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Instant parsing normalizes explicit offsets to the same epoch',
    'NeoClr.RuntimeTests.Instant parsing preserves fractional ticks before and after the epoch',
    'NeoClr.RuntimeTests.Instant parsing preserves the wider tick domain at civil boundaries',
    'NeoClr.RuntimeTests.Instant parsing requires a complete invariant timestamp and explicit offset',
    'NeoClr.RuntimeTests.Instant parsing rejects invalid civil fields and offset ranges',
    'NeoClr.RuntimeTests.Parsed instants project back through their explicit fixed offset',
]

DURATION_OFFSET_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Duration parsing preserves signed tick boundaries',
    'NeoClr.RuntimeTests.Duration parsing rejects overflow and invalid clock fields',
    'NeoClr.RuntimeTests.Duration parsing rejects unsupported numeric grammar',
    'NeoClr.RuntimeTests.Duration fractions scale to ticks and accept bounded leading day zeros',
    'NeoClr.RuntimeTests.Offset parsing accepts zero signs seconds and inclusive bounds',
    'NeoClr.RuntimeTests.Offset parsing rejects invalid ranges without normalization',
    'NeoClr.RuntimeTests.Offset parsing requires complete ASCII signed syntax',
    'NeoClr.RuntimeTests.Parsed offsets project instants and parsed durations preserve arithmetic',
]

TIME_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Date parsing accepts bounds and Gregorian leap years',
    'NeoClr.RuntimeTests.Date parsing separates malformed text from invalid civil fields',
    'NeoClr.RuntimeTests.Time parsing scales one through seven fraction digits without rounding',
    'NeoClr.RuntimeTests.Time parsing rejects malformed clocks leap seconds and excess precision',
    'NeoClr.RuntimeTests.Civil parsers round trip all supported numeric culture conventions',
    'NeoClr.RuntimeTests.Explicit cultures reject mismatched date and fraction separators',
    'NeoClr.RuntimeTests.Local date time parsing preserves boundaries and rejects zone suffixes',
    'NeoClr.RuntimeTests.Default civil parsers use the current culture formatting convention',
]

MAP_NAMES = [
    'NeoClr.RuntimeTests.Map materialization handles collisions and empty input',
    'NeoClr.RuntimeTests.HashMap copies sequences and maps independently',
    'NeoClr.RuntimeTests.Map iteration snapshots pairs before mutation',
    'NeoClr.RuntimeTests.ToMap materializes pair queries and selectors',
    'NeoClr.RuntimeTests.HashMap copies preserve reference values and comparer',
]
COLLECTION_NAMES = [
    'NeoClr.RuntimeTests.AnyWithoutCurrent',
    'NeoClr.RuntimeTests.For break disposes its iterator',
    'NeoClr.RuntimeTests.ArrayList copy has independent storage',
    'NeoClr.RuntimeTests.Map materialization handles collisions and empty input',
    'NeoClr.RuntimeTests.HashMap copies sequences and maps independently',
    'NeoClr.RuntimeTests.Map iteration snapshots pairs before mutation',
    'NeoClr.RuntimeTests.ToMap materializes pair queries and selectors',
    'NeoClr.RuntimeTests.HashMap copies preserve reference values and comparer',
    'NeoClr.RuntimeTests.Predicate Any stops at its first match',
    'NeoClr.RuntimeTests.PriorityArrayQueue clear retains capacity and permits reuse',
    'NeoClr.RuntimeTests.PriorityArrayQueue accepts reversed ordering and preserves reference identity',
    'NeoClr.RuntimeTests.PriorityArrayQueue duplicate priorities neither lose nor duplicate entries',
    'NeoClr.RuntimeTests.PriorityArrayQueue interleaved insertions preserve the heap',
    'NeoClr.RuntimeTests.PriorityArrayQueue grows and removes the lowest priority first',
    'NeoClr.RuntimeTests.Queue growth preserves wrapped FIFO contents',
    'NeoClr.RuntimeTests.ArrayQueue preserves FIFO',
    'NeoClr.RuntimeTests.Queue preserves references through clear and reuse',
    'NeoClr.RuntimeTests.Queue iterator captures FIFO contents before mutation',
    'NeoClr.RuntimeTests.HashSet applies its comparer',
    'NeoClr.RuntimeTests.Stack growth preserves LIFO contents',
    'NeoClr.RuntimeTests.ArrayStack preserves LIFO',
    'NeoClr.RuntimeTests.Stack preserves references through clear and reuse',
    'NeoClr.RuntimeTests.Stack iterator captures LIFO contents before mutation',
]
CONSTRUCTION_NAMES = [
    'NeoClr.RuntimeTests.Collection constructors consume and dispose each input once',
    'NeoClr.RuntimeTests.Collection constructors accept empty arrays',
    'NeoClr.RuntimeTests.Queue and stack copies follow source iteration order',
    'NeoClr.RuntimeTests.List construction materializes filtered input independently',
    'NeoClr.RuntimeTests.Collection copies preserve element reference identity',
    'NeoClr.RuntimeTests.Set copy retains values and explicit comparer after source clear',
    'NeoClr.RuntimeTests.Map indexers share comparer lookup and insert-or-replace semantics',
    'NeoClr.RuntimeTests.Set collision chains survive removal growth and slot reuse',
    'NeoClr.RuntimeTests.Set snapshot retains values across clear and reuse',
    'NeoClr.RuntimeTests.Set ordinal string equality preserves case and Unicode',
]
ITERATION_NAMES = [
    'NeoClr.RuntimeTests.For completion disposes its iterator',
    'NeoClr.RuntimeTests.For continue preserves traversal and disposal',
    'NeoClr.RuntimeTests.For disposes an empty iterator',
    'NeoClr.RuntimeTests.Labeled continue disposes each inner iterator',
    'NeoClr.RuntimeTests.For return disposes before returning to its caller',
    'NeoClr.RuntimeTests.All stops at the first failed predicate',
    'NeoClr.RuntimeTests.Count advances without reading Current',
    'NeoClr.RuntimeTests.Any disposes empty input',
    'NeoClr.RuntimeTests.First stops after one value',
    'NeoClr.RuntimeTests.Fold visits every value',
    'NeoClr.RuntimeTests.Last visits the complete input',
    'NeoClr.RuntimeTests.Predicate Single stops at the second match',
    'NeoClr.RuntimeTests.Predicate First disposes input with no match',
    'NeoClr.RuntimeTests.Predicate Count reads every value',
    'NeoClr.RuntimeTests.Predicate First stops at its match',
    'NeoClr.RuntimeTests.Predicate Last retains the final match',
    'NeoClr.RuntimeTests.Predicate Single scans for a unique match',
    'NeoClr.RuntimeTests.Single rejects multiple values without reading the second',
    'NeoClr.RuntimeTests.ToList materializes and disposes its input',
]

JSON_NAMES = [
    'NeoClr.RuntimeTests.JSON arrays report invalid indices',
    'NeoClr.RuntimeTests.JSON serialization rejects cyclic node graphs',
    'NeoClr.RuntimeTests.JSON nested documents round trip with Unicode and number spelling',
    'NeoClr.RuntimeTests.JSON document round trip removes surrounding whitespace',
    'NeoClr.RuntimeTests.JSON parsing rejects escaped aliases of duplicate names',
    'NeoClr.RuntimeTests.JSON node text supports compact and indented round trips',
    'NeoClr.RuntimeTests.JSON parsing rejects trailing commas and truncated input',
    'NeoClr.RuntimeTests.Empty JSON objects support empty map snapshots and safe lookup',
    'NeoClr.RuntimeTests.JSON map lookup distinguishes null absence and ordinal keys',
    'NeoClr.RuntimeTests.JSON map views keep JSON node serialization precedence',
    'NeoClr.RuntimeTests.JSON map snapshots preserve insertion order and node identity',
    'NeoClr.RuntimeTests.JSON numbers preserve spelling and reject fractional Int32 conversion',
    'NeoClr.RuntimeTests.JSON duplicate insertion preserves the original value',
    'NeoClr.RuntimeTests.JSON objects distinguish present null from missing fields',
    'NeoClr.RuntimeTests.JSON object indexed access preserves insertion order',
    'NeoClr.RuntimeTests.JSON depth defaults to 64 and validates document options',
    'NeoClr.RuntimeTests.JSON parsing supports large containers and long UTF-8 escaped strings',
    'NeoClr.RuntimeTests.JSON document options follow strict defaults and independent relaxed syntax',
    'NeoClr.RuntimeTests.JSON documents support every root node kind',
    'NeoClr.RuntimeTests.JSON scalar nodes expose their typed values',
    'NeoClr.RuntimeTests.JSON sequence indexing retains node identity and explicit null',
    'NeoClr.RuntimeTests.Empty JSON arrays expose empty sequence snapshots',
    'NeoClr.RuntimeTests.JSON sequence views retain node serialization precedence',
    'NeoClr.RuntimeTests.JSON sequence snapshots preserve order and shared child contents',
]
JSON_STREAM_NAMES = [
    'NeoClr.RuntimeTests.JSON HTTP byte input preserves memory windows and aliases and rejects unbuffered content',
    'NeoClr.RuntimeTests.HTTP JSON async reads consume streams once preserve ownership and honor cancellation',
    'NeoClr.RuntimeTests.JSON byte input borrows selected windows and returns owned mapped values',
    'NeoClr.RuntimeTests.JSON byte input accepts UTF-8 scalar boundaries and rejects BOM and raw controls',
    'NeoClr.RuntimeTests.JSON byte input rejects invalid UTF-8 before syntax at any document size',
    'NeoClr.RuntimeTests.JSON UTF-8 byte output preserves escaping formatting and independent ownership',
    'NeoClr.RuntimeTests.JSON borrowed token parsing preserves empty Unicode escaped and scalar values',
    'NeoClr.RuntimeTests.JSON borrowed token parsing rejects malformed escapes controls and numbers',
    'NeoClr.RuntimeTests.JSON stream serialization detects cycles before writing',
    'NeoClr.RuntimeTests.JSON stream output quota includes escaping and validates before writes',
    'NeoClr.RuntimeTests.JSON indented stream output matches node text and preflights quotas',
    'NeoClr.RuntimeTests.JSON stream reads reject split invalid UTF-8',
    'NeoClr.RuntimeTests.JSON memory stream round trip preserves document content',
    'NeoClr.RuntimeTests.JSON long numbers preserve spelling with separate write limits',
    'NeoClr.RuntimeTests.JSON streams retry single-byte reads and writes without taking ownership',
    'NeoClr.RuntimeTests.JSON stream reads preserve the underlying I/O failure',
    'NeoClr.RuntimeTests.JSON streams read beyond the former document quota and preserve ownership',
    'NeoClr.RuntimeTests.JSON stream reads reject a truncated document',
    'NeoClr.RuntimeTests.JSON stream boundary counts UTF-8 bytes rather than characters',
    'NeoClr.RuntimeTests.JSON stream writes preserve partial output and failure cause',
    'NeoClr.RuntimeTests.JSON stream writes reject zero progress without looping',
]
MEMORY_STREAM_NAMES = [
    'NeoClr.RuntimeTests.Omitted interface tokens are non cancellable while explicit tokens still cancel',
    'NeoClr.RuntimeTests.Text and gzip factory defaults retain ownership and explicit encoding choices',
    'NeoClr.RuntimeTests.Watcher and time creation preserve omitted and explicit scalar defaults',
    'NeoClr.RuntimeTests.MemoryStream async cancellation and rejected ranges preserve storage',
    'NeoClr.RuntimeTests.MemoryStream async interfaces preserve complete writes short reads and EOF',
    'NeoClr.RuntimeTests.MemoryStream quota errors preserve length and position',
    'NeoClr.RuntimeTests.MemoryStream closure is shared across interfaces and idempotent',
    'NeoClr.RuntimeTests.MemoryStream gap writes zero-fill while empty writes preserve length',
    'NeoClr.RuntimeTests.MemoryStream rejects invalid buffer and seek ranges without moving',
    'NeoClr.RuntimeTests.MemoryStream overwrites preserve length and untouched bytes',
]
STRING_CONSTRUCTION_NAMES = [
    'NeoClr.RuntimeTests.Character literals preserve graphemes and allow char identifiers',
    'NeoClr.RuntimeTests.String construction copies character storage',
    'NeoClr.RuntimeTests.String construction accepts an empty character sequence',
    'NeoClr.RuntimeTests.String construction preserves and merges grapheme boundaries',
    'NeoClr.RuntimeTests.String named arguments preserve public parameter contracts',
    'NeoClr.RuntimeTests.String exposes a readonly character sequence',
]
UNICODE_CASING_NAMES = [
    'NeoClr.RuntimeTests.Invariant casing preserves embedded NUL and empty strings',
    'NeoClr.RuntimeTests.Invariant casing supports full Unicode expansions',
    'NeoClr.RuntimeTests.CasingNoNormalization',
    'NeoClr.RuntimeTests.CasingNotOrdinalFolding',
    'NeoClr.RuntimeTests.Invariant lowercase respects contextual Greek sigma',
    'NeoClr.RuntimeTests.Invariant uppercase preserves emoji and supplementary letters',
]
INT64_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Int64 parsing accepts both signed boundaries',
    'NeoClr.RuntimeTests.Int64 decimal formatting round trips signed boundaries',
    'NeoClr.RuntimeTests.Int64 parsing rejects noncanonical lexical inputs',
    'NeoClr.RuntimeTests.Int64 parsing distinguishes overflow from invalid trailing text',
    'NeoClr.RuntimeTests.Int64 parsing accepts signs leading zeros and values beyond Int32',
]
GLOBALIZATION_NAMES = [
    'NeoClr.RuntimeTests.UK and US clocks preserve midnight noon and invariant output',
    'NeoClr.RuntimeTests.Culture preferences resolve explicit regions and bounded fallbacks',
    'NeoClr.RuntimeTests.EnglishUnitedKingdom formatting data and calendar isolation',
    'NeoClr.RuntimeTests.EnglishUnitedStates formatting data and calendar isolation',
    'NeoClr.RuntimeTests.HebrewIsrael formatting data and calendar isolation',
    'NeoClr.RuntimeTests.Invariant formatting data and calendar isolation',
    'NeoClr.RuntimeTests.SwedishSweden formatting data and calendar isolation',
]
NUMERIC_FORMATTING_NAMES = [
    'NeoClr.RuntimeTests.Culture parsing covers every numeric primitive and roundtrips formatting',
    'NeoClr.RuntimeTests.Culture parsing rejects malformed groups mixed separators and overflow',
    'NeoClr.RuntimeTests.Floating formatting preserves precision culture and boxed display',
    'NeoClr.RuntimeTests.Integral formatting preserves widths culture and interface dispatch',
]

ENUM_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Enum lists combine names independently of Flags metadata',
    'NeoClr.RuntimeTests.Enum names preserve aliases ordinal case and imported types',
    'NeoClr.RuntimeTests.Enum numbers preserve signed bounds and unnamed values',
    'NeoClr.RuntimeTests.Enum TryParse resets output and rejects malformed input',
    'NeoClr.RuntimeTests.Unicode type member parameter and local identifiers use ordinary binding',
    'NeoClr.RuntimeTests.Enum names preserve Unicode and explicit ordinal identity',
]

PRIMITIVE_PARSING_NAMES = [
    'NeoClr.RuntimeTests.Boolean parsing accepts mixed case and rejects numeric aliases',
    'NeoClr.RuntimeTests.Byte parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.Byte parsing preserves distinct small values',
    'NeoClr.RuntimeTests.Double parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.Primitive parsers distinguish invalid format from overflow',
    'NeoClr.RuntimeTests.Int16 parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.Int32 parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.SByte parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.Single parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.UInt16 parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.UInt32 parsing preserves values and reports overflow',
    'NeoClr.RuntimeTests.UInt64 parsing preserves values and reports overflow',
]
PATH_VALUES_NAMES = [
    'NeoClr.RuntimeTests.Path Object equality distinguishes colliding map keys',
    'NeoClr.RuntimeTests.Equal paths share hashes and virtual display',
    'NeoClr.RuntimeTests.Path maps replace equal keys and retain lookup through growth',
    'NeoClr.RuntimeTests.Path equality distinguishes case roots and normalization',
    'NeoClr.RuntimeTests.Path equality agrees across typed interface and Object dispatch',
]
DEPENDENCY_INJECTION_NAMES = [
    'NeoClr.RuntimeTests.DI selects the greediest available public constructor',
    'NeoClr.RuntimeTests.DI activates typed localizer dependencies through a retained plan',
    'NeoClr.RuntimeTests.DI rejects ambiguous missing and inaccessible constructors',
    'NeoClr.RuntimeTests.DI builds plans without executing constructors or factories',
    'NeoClr.RuntimeTests.DI registry snapshots registrations and injects typed localizers',
    'NeoClr.RuntimeTests.DI isolates scopes and shares only singleton instances',
    'NeoClr.RuntimeTests.DI rejects cycles and captive scopes without poisoning resolution state',
    'NeoClr.RuntimeTests.DI disposes owned services in reverse order and leaves instances caller owned',
    'NeoClr.RuntimeTests.DI validates factories and registrations and resets failed singleton factories',
    'NeoClr.RuntimeTests.DI injects the current provider and root disposal invalidates live scopes',
    'NeoClr.RuntimeTests.DI conventional lookups distinguish missing services from activation errors',
    'NeoClr.RuntimeTests.DI generic registrations preserve factory failures and plural singleton slots',
    'NeoClr.RuntimeTests.DI cleans up factory results when disposal reenters resolution',
    'NeoClr.RuntimeTests.DI keyed lookups isolate registrations and preserve ordinary key equality',
    'NeoClr.RuntimeTests.DI keyed factories and scopes preserve independent lifetime slots',
    'NeoClr.RuntimeTests.DI retrieves keyed bound function objects without invoking them as factories',
    'NeoClr.RuntimeTests.DI TryAdd preserves existing type and key registrations without activating factories',
    'NeoClr.RuntimeTests.DI diamond graphs preserve lifetimes and independent scope ownership',
    'NeoClr.RuntimeTests.DI closes open generic registrations and isolates scoped caches by type',
    'NeoClr.RuntimeTests.DI open generic singletons preserve exact overrides keyed lookup and plural order',
    'NeoClr.RuntimeTests.DI website sample resolves a scoped constructor graph and disposes scopes',
]

CONFIGURATION_NAMES = [
    'NeoClr.RuntimeTests.Configuration precedence distinguishes absent null and empty values',
    'NeoClr.RuntimeTests.Configuration sections merge numeric children and preserve Unicode paths',
    'NeoClr.RuntimeTests.Configuration builds snapshot inputs and retain live sections',
    'NeoClr.RuntimeTests.Configuration loads reloads and disposes provider registrations',
    'NeoClr.RuntimeTests.Configuration DI preserves overrides and caller ownership',
    'NeoClr.RuntimeTests.JSON configuration flattens objects arrays and scalar values',
    'NeoClr.RuntimeTests.JSON configuration rejects ambiguous malformed and oversized input atomically',
    'NeoClr.RuntimeTests.JSON configuration reads required optional files and reloads through the root',
    'NeoClr.RuntimeTests.JSON configuration accepts realistic commented settings and preserves scalar text',
    'NeoClr.RuntimeTests.JSON configuration loads large files and preserves data on file size failure',
]

LOGGING_NAMES = [
    'NeoClr.RuntimeTests.Logging levels and event identity follow the Microsoft baseline',
    'NeoClr.RuntimeTests.Null logging skips formatting and returns reusable scope handles',
    'NeoClr.RuntimeTests.Default logging filters before evaluating formatters',
    'NeoClr.RuntimeTests.Logging forwards structured state event and error to every enabled provider',
    'NeoClr.RuntimeTests.Logger factories cache ordinal categories and update existing loggers',
    'NeoClr.RuntimeTests.Logging scopes preserve provider state and dispose each handle once',
    'NeoClr.RuntimeTests.Factory disposal owns provider registrations and disables cached loggers',
    'NeoClr.RuntimeTests.Console logging renders UTF8 messages and explicit diagnostic errors',
    'NeoClr.RuntimeTests.Typed logging creates a category from the declared type',
    'NeoClr.RuntimeTests.Logging DI resolves singleton typed categories and preserves supplied factory ownership',
    'NeoClr.RuntimeTests.Logging DI owns the default factory and preserves explicit logger overrides',
]

LOCALIZATION_NAMES = [
    'NeoClr.RuntimeTests.Localizer preserves strings and distinguishes missing from empty',
    'NeoClr.RuntimeTests.Localizer uses ordinal keys and decodes JSON escapes',
    'NeoClr.RuntimeTests.Localizer rejects malformed and non-string catalogues',
    'NeoClr.RuntimeTests.Localizer supports catalogues beyond provisional DOM limits',
    'NeoClr.RuntimeTests.Localizer reads borrowed streams and explicit files',
    'NeoClr.RuntimeTests.Localizer diagnostics report only missing display lookups',
    'NeoClr.RuntimeTests.Localizer scopes and cultures fall back per key',
    'NeoClr.RuntimeTests.Localizer invariant fallback is configurable and typed scope is explicit',
    'NeoClr.RuntimeTests.Localizer resolves templates before formatting arguments',
    'NeoClr.RuntimeTests.Localizer integrates typed JSON resources with scoped constructor injection',
]

DECONSTRUCTION_NAMES = [
    'NeoClr.RuntimeTests.Value deconstruction binds named and unnamed locals',
    'NeoClr.RuntimeTests.Reference deconstruction binds named and unnamed locals',
    'NeoClr.RuntimeTests.Positional patterns narrow derived and object references',
    'NeoClr.RuntimeTests.Qualified currency patterns match values and reject mismatches',
]

CURRENCY_NAMES = ['NeoClr.RuntimeTests.Currency identity is independent of culture and amount', 'NeoClr.RuntimeTests.Currency formatting follows all five culture profiles', 'NeoClr.RuntimeTests.Explicit currency formatting rounds display without changing the amount', 'NeoClr.RuntimeTests.Currency amount parsing validates identity grouping spacing and direction', 'NeoClr.RuntimeTests.Currency data and language direction are explicit immutable policies', 'NeoClr.RuntimeTests.Money records preserve identity precision and deconstruction']

DECIMAL_NAMES = ['NeoClr.RuntimeTests.Decimal arithmetic preserves exact base ten values', 'NeoClr.RuntimeTests.Decimal equality normalizes scale without changing storage', 'NeoClr.RuntimeTests.Decimal parsing formatting and rounding preserve precision', 'NeoClr.RuntimeTests.Decimal syntax uses ordinary value method contracts', 'NeoClr.RuntimeTests.Decimal boundaries reject overflow and round midpoint ties', 'NeoClr.RuntimeTests.Decimal arithmetic agrees with CLR boundary vectors', 'NeoClr.RuntimeTests.Decimal culture formatting and representation use existing contracts']

FLOATING_SPECIAL_NAMES = [
    'NeoClr.RuntimeTests.Floating named constants preserve IEEE and value equality contracts',
    'NeoClr.RuntimeTests.Floating classification covers both widths signs and finite values',
    'NeoClr.RuntimeTests.Floating special values format and parse through existing contracts',
]
RECORD_FORMATTING_NAMES = [
    'NeoClr.RuntimeTests.Record enum payload display survives boxed dispatch and collection',
    'NeoClr.RuntimeTests.Record enum display preserves names flags and numeric fallback',
    'NeoClr.RuntimeTests.Floating record contracts agree across boxed interface and nested calls',
    'NeoClr.RuntimeTests.Floating records use value equality for NaNs zeros and infinities',
    'NeoClr.RuntimeTests.Floating record display uses primitive text including negative zero',
    'NeoClr.RuntimeTests.Generic record display uses nested and boxed value dispatch',
    'NeoClr.RuntimeTests.Generic record display preserves text null and primitive contracts',
    'NeoClr.RuntimeTests.Records use imported value formatting across typed and boxed calls',
    'NeoClr.RuntimeTests.Inherited record equality observes runtime type and derived state',
    'NeoClr.RuntimeTests.Inherited record display includes base members through virtual calls',
    'NeoClr.RuntimeTests.Nested records and unions retain imported value display contracts',
    'NeoClr.RuntimeTests.Recursive record shapes display finite reference chains',
    'NeoClr.RuntimeTests.Root record classes preserve value equality and reference identity',
    'NeoClr.RuntimeTests.Record class display preserves nested and absent references',
    'NeoClr.RuntimeTests.Record scalar formatting preserves null and original Unicode text',
]
INDEX_RANGE_NAMES = [
    'NeoClr.RuntimeTests.Index preserves direction and resolves end boundaries',
    'NeoClr.RuntimeTests.Index and Range typed equality agrees with hashing',
    'NeoClr.RuntimeTests.Range resolution accepts empty and full boundaries',
    'NeoClr.RuntimeTests.Array range syntax supports omitted and end-relative boundaries',
    'NeoClr.RuntimeTests.Array indexes support values and from-end assignment',
    'NeoClr.RuntimeTests.Array slices copy storage and preserve reference elements',
    'NeoClr.RuntimeTests.Array indexing and slicing evaluate operands once in source order',
    'NeoClr.RuntimeTests.Read-only value fields use defensive copies for method calls',
    'NeoClr.RuntimeTests.Array safe access returns options and results for invalid bounds',
    'NeoClr.RuntimeTests.String ranges preserve graphemes and half-open boundaries',
    'NeoClr.RuntimeTests.String safe access rejects invalid grapheme bounds',
    'NeoClr.RuntimeTests.String slicing evaluates receiver and bounds once',
    'NeoClr.RuntimeTests.ArrayList supports directional mutation and independent range copies',
    'NeoClr.RuntimeTests.Sequence slicing consumes one iterator without indexed reads',
    'NeoClr.RuntimeTests.Sequence invalid and empty slices do not acquire iterators',
    'NeoClr.RuntimeTests.Sequence slicing disposes when enumeration ends before Count',
    'NeoClr.RuntimeTests.Sequence safe indexing and slicing work through array aliases',
    'NeoClr.RuntimeTests.Sequence extension indexers provide default access and independent slices',
    'NeoClr.RuntimeTests.Sequence slicing delegates to the list range contract',
    'NeoClr.RuntimeTests.Native extension indexers support writes and compound assignment',
]
MEDIA_TYPE_VALUES_NAMES = [
    'NeoClr.RuntimeTests.Media type equality ignores order and syntax but preserves boundary case',
    'NeoClr.RuntimeTests.Media type parameter views cannot mutate the value',
    'NeoClr.RuntimeTests.Media type parser rejects invalid syntax and ambiguous parameters',
    'NeoClr.RuntimeTests.Legacy content parsing caches values and distinguishes absence from errors',
    'NeoClr.RuntimeTests.Media type limits and parameter updates return errors without faults',
    'NeoClr.RuntimeTests.HTTP messages expose actual Content-Type and reject duplicate fields',
    'NeoClr.RuntimeTests.Media type presets reuse values and formatted strings',
    'NeoClr.RuntimeTests.Media type quoted parameters round trip escaped separators',
    'NeoClr.RuntimeTests.Media types separate concrete content types from Accept ranges',
    'NeoClr.RuntimeTests.Media type parsing retains structure and custom types',
    'NeoClr.RuntimeTests.Typed HTTP content retains media types without reparsing',
    'NeoClr.RuntimeTests.Media types accept optional whitespace and empty parameter segments',
]
HTTP_METHOD_VALUES_NAMES = [
    'NeoClr.RuntimeTests.HTTP coffee requests retain BREW and unnamed status codes',
    'NeoClr.RuntimeTests.HTTP custom method construction and parsing preserve spelling',
    'NeoClr.RuntimeTests.HTTP method equality and hashes ignore ASCII case',
    'NeoClr.RuntimeTests.HTTP methods retain case-insensitive identity in a set',
    'NeoClr.RuntimeTests.HTTP method validation admits token punctuation and rejects separators',
    'NeoClr.RuntimeTests.HTTP method parsing canonicalizes every known name',
    'NeoClr.RuntimeTests.HTTP predefined method values use canonical tokens',
    'NeoClr.RuntimeTests.HTTP request factories retain predefined and custom methods',
    'NeoClr.RuntimeTests.HTTP URI request factories support custom methods and empty content',
]
HTTP_HEADER_VALUES_NAMES = [
    'NeoClr.RuntimeTests.HTTP header lookup preserves repeated values without splitting commas',
    'NeoClr.RuntimeTests.HTTP header lookup distinguishes empty values from absent fields',
    'NeoClr.RuntimeTests.HTTP header lookup rejects invalid names',
    'NeoClr.RuntimeTests.HTTP header lookup snapshots remain independent of later additions',
    'NeoClr.RuntimeTests.HTTP request lookup exposes content headers without inventing provider headers',
    'NeoClr.RuntimeTests.HTTP response constructors retain memory content and replace only content type metadata',
    'NeoClr.RuntimeTests.HTTP byte producers allocate runtime sized arrays with ordered single endpoint evaluation',
    'NeoClr.RuntimeTests.HTTP response construction never reads or changes stream ownership',
    'NeoClr.RuntimeTests.HTTP messages expose HTTP 1.1 and dispose owned content without reading',
    'NeoClr.RuntimeTests.Header collections expose named snapshots and ordered fields',
    'NeoClr.RuntimeTests.Single header access distinguishes absence empty duplicate and invalid names',
    'NeoClr.RuntimeTests.Outgoing header views remain attached across message mutations',
    'NeoClr.RuntimeTests.Empty header collections are reusable read only message inputs',
    'NeoClr.RuntimeTests.Mutable outgoing headers preserve repeated values and validate atomically',
    'NeoClr.RuntimeTests.Client request headers enforce transport ownership and derived request independence',
    'NeoClr.RuntimeTests.Incoming response views and Empty never expose outgoing mutation',
]
HTTP_ROUTE_MATCHING_NAMES = [
    'NeoClr.RuntimeTests.Route matching preserves case segment and trailing-slash distinctions',
    'NeoClr.RuntimeTests.Route byte quotas admit the boundary and reject one extra byte',
    'NeoClr.RuntimeTests.Route matching separates query text and converts captured integers',
    'NeoClr.RuntimeTests.Route captures decode UTF-8 once and preserve literal plus',
    'NeoClr.RuntimeTests.Route integer conversion distinguishes path matching from conversion failure',
    'NeoClr.RuntimeTests.Route parsing rejects malformed and unsupported pattern syntax',
    'NeoClr.RuntimeTests.Route matching validates malformed targets even when literals mismatch',
    'NeoClr.RuntimeTests.Route segment quotas admit the boundary and reject one extra segment',
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
    'NeoClr.RuntimeTests.URI resolution handles authority and path bases and rejects relative bases',
    'NeoClr.RuntimeTests.URI byte parsing preserves ASCII boundaries and encoded Unicode',
    'NeoClr.RuntimeTests.URI classification and equality preserve literal identity',
    'NeoClr.RuntimeTests.URI parsing rejects every invalid grammar corpus input',
    'NeoClr.RuntimeTests.URI parsing and resolution enforce the byte quota',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 1',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 2',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 3',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 4',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 5',
    'NeoClr.RuntimeTests.URI reference resolution corpus group 6',
    'NeoClr.RuntimeTests.URI parsing reports unsupported bracketed authorities',
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
    parser.add_argument('--suite', action='append', choices=('error-context', 'environment-discovery', 'native-callbacks', 'native-buffers', 'native-pointers', 'method-state', 'type-relations', 'collections', 'collection-construction', 'collection-iteration', 'json-dom', 'json-streams', 'memory-stream', 'memory-lifetimes', 'string-construction', 'unicode-casing', 'encoding', 'encoding-streams', 'cryptography', 'gzip', 'encoding-boundaries', 'encoding-buffer-limits', 'int64-parsing', 'primitive-parsing', 'numeric-formatting', 'globalization', 'enum-parsing', 'record-formatting', 'record-allocations', 'floating-special-values', 'decimal', 'currency', 'deconstruction', 'configuration', 'logging', 'localization', 'dependency-injection', 'path-values', 'http-header-values', 'http-header-allocations', 'http-method-values', 'media-type-values', 'media-type-allocations', 'uri-path-allocations', 'index-range', 'http-text', 'http-content-copy', 'http-lifetime', 'http-route-matching', 'ip-address-values', 'uri-resolution', 'time-parsing', 'duration-offset-parsing', 'instant-parsing', 'zoned-time-parsing', 'discovery-contract', 'runner-contract'),
                        help='Run only this suite (repeatable); default runs qualified suites')
    parser.add_argument("--execution-repetitions", type=int, default=1, help="Repeat the full suite process in each mode for timing; guards/selectors still run once")
    parser.add_argument('--mode', action='append', choices=('native', 'interpreter', 'jit'),
                        help='Execution modes to qualify (repeatable); defaults to JIT')
    parser.add_argument('--jit-host', type=Path, help='Matching trusted native helper library for JIT')
    args = parser.parse_args()
    modes = args.mode or ['jit']
    if args.suite and any(name in args.suite for name in ('record-allocations', 'media-type-allocations', 'uri-path-allocations', 'http-header-allocations')) and set(modes) != {'interpreter'}:
        parser.error('allocation suites require --mode interpreter; native/JIT GC counters are not bound')
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
              ROOT / 'tools/discovery/RavenNames.cs',
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.cs')),
              *sorted((ROOT / 'tools/testing/NeoCLR.TestDiscovery').glob('*.csproj')),
              ROOT / 'scripts/runtime-test-projects.json',
              *[p for name in TEST_PROJECTS for p in suite_directory(name).rglob('*')
                if p.suffix in ('.rvn', '.rvnproj') and not {'bin', 'obj'}.intersection(p.parts)],
              *sorted((ROOT / 'tools/testing/NeoCLR.Testing').rglob('*.rvn')),
              *sorted(suite_directory('localization').rglob('*.json')),
              *sorted((ROOT / 'docs/experiments/dependency-injection/sample').glob('*.rvn')),
              *sorted((ROOT / 'extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo/DependencyInjection').glob('*.rvn'))]
    report = dict(passed=False, modes=modes, jitHostSha256=sha(args.jit_host.resolve()) if args.jit_host else None, platform=platform.platform(), cases=[], phases=[], executionRepetitions=args.execution_repetitions,
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  bundleManifestSha256=sha(bundle / 'manifest.json'),
                  aotSha256=sha(args.aot.resolve()) if any(mode in modes for mode in ('native', 'jit')) else None, interpreterSha256=sha(args.runtime.resolve()))
    validation_spec = importlib.util.spec_from_file_location('discovery_validation', ROOT / 'scripts/validate-test-discovery.py')
    validation = importlib.util.module_from_spec(validation_spec)
    validation_spec.loader.exec_module(validation)
    relationship_names = ['NeoClr.RuntimeTests.Arrays preserve element identity', 'NeoClr.RuntimeTests.Closed generic types remain distinct', 'NeoClr.RuntimeTests.Type identity and assignment direction', 'NeoClr.RuntimeTests.Boxing and instance checks reject null', 'NeoClr.RuntimeTests.Interface assignment includes inherited contracts', 'NeoClr.RuntimeTests.Subclass checks are strict and exclude interfaces']
    expected = [
        ('method-state', 0, 'PASS NeoClr.RuntimeTests.Method discovery filters visibility and scope on value types\nPASS NeoClr.RuntimeTests.Method generic state separates owner and method parameters\nTests: 2, passed: 2, failed: 0, skipped: 0\n'),
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
        ('gzip', 0, 'PASS NeoClr.RuntimeTests.Gzip decodes independent stored fixed dynamic and optional-header members\nPASS NeoClr.RuntimeTests.Gzip validates checksums truncation headers and quotas before output\nPASS NeoClr.RuntimeTests.Gzip incrementally compresses repeated writes and finishes once\nPASS NeoClr.RuntimeTests.Gzip empty finish ownership ranges modes and cancellation are explicit\nPASS NeoClr.RuntimeTests.Gzip awaits output and poisons interrupted streams without replay\nPASS NeoClr.RuntimeTests.Gzip handles pending fragmented input and preserves text-reader errors\nPASS NeoClr.RuntimeTests.Gzip borrows only the selected source window across compression blocks\nTests: 7, passed: 7, failed: 0, skipped: 0\n'),
        ('cryptography', 0, 'PASS NeoClr.RuntimeTests.SHA256 preserves aliases tails and short destinations\nPASS NeoClr.RuntimeTests.HMACSHA256 verifies known answers and rejects altered tags\nPASS NeoClr.RuntimeTests.RandomNumberGenerator accepts empty and writable buffers\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
        ('encoding', 0, ''.join('PASS ' + n + '\n' for n in ENCODING_NAMES) + f'Tests: {len(ENCODING_NAMES)}, passed: {len(ENCODING_NAMES)}, failed: 0, skipped: 0\n'),
        ('unicode-casing', 0, ''.join('PASS ' + n + '\n' for n in UNICODE_CASING_NAMES) + f'Tests: {len(UNICODE_CASING_NAMES)}, passed: {len(UNICODE_CASING_NAMES)}, failed: 0, skipped: 0\n'),
        ('int64-parsing', 0, ''.join('PASS ' + n + '\n' for n in INT64_PARSING_NAMES) + f'Tests: {len(INT64_PARSING_NAMES)}, passed: {len(INT64_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('dependency-injection', 0, ''.join('PASS ' + n + '\n' for n in DEPENDENCY_INJECTION_NAMES) + f'Tests: {len(DEPENDENCY_INJECTION_NAMES)}, passed: {len(DEPENDENCY_INJECTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('configuration', 0, ''.join('PASS ' + n + '\n' for n in CONFIGURATION_NAMES) + f'Tests: {len(CONFIGURATION_NAMES)}, passed: {len(CONFIGURATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('logging', 0, ''.join('PASS ' + n + '\n' for n in LOGGING_NAMES) + f'Tests: {len(LOGGING_NAMES)}, passed: {len(LOGGING_NAMES)}, failed: 0, skipped: 0\n'),
        ('localization', 0, ''.join('PASS ' + n + '\n' for n in LOCALIZATION_NAMES) + f'Tests: {len(LOCALIZATION_NAMES)}, passed: {len(LOCALIZATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('deconstruction', 0, ''.join('PASS ' + n + '\n' for n in DECONSTRUCTION_NAMES) + f'Tests: {len(DECONSTRUCTION_NAMES)}, passed: {len(DECONSTRUCTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('currency', 0, ''.join('PASS ' + n + '\n' for n in CURRENCY_NAMES) + f'Tests: {len(CURRENCY_NAMES)}, passed: {len(CURRENCY_NAMES)}, failed: 0, skipped: 0\n'),
        ('decimal', 0, ''.join('PASS ' + n + '\n' for n in DECIMAL_NAMES) + f'Tests: {len(DECIMAL_NAMES)}, passed: {len(DECIMAL_NAMES)}, failed: 0, skipped: 0\n'),
        ('floating-special-values', 0, ''.join('PASS ' + n + '\n' for n in FLOATING_SPECIAL_NAMES) + f'Tests: {len(FLOATING_SPECIAL_NAMES)}, passed: {len(FLOATING_SPECIAL_NAMES)}, failed: 0, skipped: 0\n'),
        ('record-allocations', 0, 'PASS NeoClr.RuntimeTests.Record class equality avoids reflection descriptor allocations\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
        ('error-context', 0, ''.join('PASS ' + n + '\n' for n in ERROR_CONTEXT_NAMES) + f'Tests: {len(ERROR_CONTEXT_NAMES)}, passed: {len(ERROR_CONTEXT_NAMES)}, failed: 0, skipped: 0\n'),
        ('environment-discovery', 0, ''.join('PASS ' + n + '\n' for n in ENVIRONMENT_NAMES) + f'Tests: {len(ENVIRONMENT_NAMES)}, passed: {len(ENVIRONMENT_NAMES)}, failed: 0, skipped: 0\n'),
        ('record-formatting', 0, ''.join('PASS ' + n + '\n' for n in RECORD_FORMATTING_NAMES) + f'Tests: {len(RECORD_FORMATTING_NAMES)}, passed: {len(RECORD_FORMATTING_NAMES)}, failed: 0, skipped: 0\n'),
        ('native-callbacks', 0, 'PASS Native callbacks preserve captures across repeated calls\nPASS Native callbacks preserve generic unit value returns\nPASS Native callbacks accept Raven module functions\nPASS Native unit callbacks discard the logical Void result\nPASS Native callbacks retain bound receivers for the call\nPASS Native returned functions remain callable across collection\nPASS Native returned void functions produce logical unit values\nTests: 7, passed: 7, failed: 0, skipped: 0\n'),
        ('native-buffers', 0, 'PASS Repeated native byte copies allocate no managed transfer buffers\nPASS Empty native byte copies never dereference null pointers\nPASS Native buffer copies bridge Memory windows through a C import\nPASS Native byte copies accept borrowed UTF8 without a temporary array\nTests: 4, passed: 4, failed: 0, skipped: 0\n'),
        ('native-pointers', 0, 'PASS Default typed native pointers preserve null and generic forwarding\nPASS Typed native pointers compare addresses and preserve borrowed storage\nPASS Typed native pointer hashes match aliases without reading storage\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
        ('enum-parsing', 0, ''.join('PASS ' + n + '\n' for n in ENUM_PARSING_NAMES) + f'Tests: {len(ENUM_PARSING_NAMES)}, passed: {len(ENUM_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('globalization', 0, ''.join('PASS ' + n + '\n' for n in GLOBALIZATION_NAMES) + f'Tests: {len(GLOBALIZATION_NAMES)}, passed: {len(GLOBALIZATION_NAMES)}, failed: 0, skipped: 0\n'),
        ('numeric-formatting', 0, ''.join('PASS ' + n + '\n' for n in NUMERIC_FORMATTING_NAMES) + f'Tests: {len(NUMERIC_FORMATTING_NAMES)}, passed: {len(NUMERIC_FORMATTING_NAMES)}, failed: 0, skipped: 0\n'),
        ('primitive-parsing', 0, ''.join('PASS ' + n + '\n' for n in PRIMITIVE_PARSING_NAMES) + f'Tests: {len(PRIMITIVE_PARSING_NAMES)}, passed: {len(PRIMITIVE_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('path-values', 0, ''.join('PASS ' + n + '\n' for n in PATH_VALUES_NAMES) + f'Tests: {len(PATH_VALUES_NAMES)}, passed: {len(PATH_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('index-range', 0, ''.join('PASS ' + n + '\n' for n in INDEX_RANGE_NAMES) + f'Tests: {len(INDEX_RANGE_NAMES)}, passed: {len(INDEX_RANGE_NAMES)}, failed: 0, skipped: 0\n'),
        ('uri-path-allocations', 0, 'PASS NeoClr.RuntimeTests.Path parsing allocates only the requested value\nPASS NeoClr.RuntimeTests.URI parsing allocates only the requested value\nPASS NeoClr.RuntimeTests.URI string resolution avoids encoded buffers and reparsing\nPASS NeoClr.RuntimeTests.URI value resolution allocates only the result value\nTests: 4, passed: 4, failed: 0, skipped: 0\n'),
        ('media-type-values', 0, ''.join('PASS ' + n + '\n' for n in MEDIA_TYPE_VALUES_NAMES) + f'Tests: {len(MEDIA_TYPE_VALUES_NAMES)}, passed: {len(MEDIA_TYPE_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('media-type-allocations', 0, 'PASS NeoClr.RuntimeTests.Warm media type access performs no managed allocations\nPASS NeoClr.RuntimeTests.Raw media type validation performs no managed allocations\nTests: 2, passed: 2, failed: 0, skipped: 0\n'),
        ('http-method-values', 0, ''.join('PASS ' + n + '\n' for n in HTTP_METHOD_VALUES_NAMES) + f'Tests: {len(HTTP_METHOD_VALUES_NAMES)}, passed: {len(HTTP_METHOD_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-header-allocations', 0, 'PASS Single header lookup and cached views allocate no managed objects\nPASS Parsed HTTP fields materialize entries only for positional access\nPASS Respond without content type retains headers without managed allocations\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
        ('http-header-values', 0, ''.join('PASS ' + n + '\n' for n in HTTP_HEADER_VALUES_NAMES) + f'Tests: {len(HTTP_HEADER_VALUES_NAMES)}, passed: {len(HTTP_HEADER_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('http-route-matching', 0, ''.join('PASS ' + n + '\n' for n in HTTP_ROUTE_MATCHING_NAMES) + f'Tests: {len(HTTP_ROUTE_MATCHING_NAMES)}, passed: {len(HTTP_ROUTE_MATCHING_NAMES)}, failed: 0, skipped: 0\n'),
        ('ip-address-values', 0, ''.join('PASS ' + n + '\n' for n in sorted('NeoClr.RuntimeTests.' + name for name in IP_ADDRESS_VALUES_NAMES)) + f'Tests: {len(IP_ADDRESS_VALUES_NAMES)}, passed: {len(IP_ADDRESS_VALUES_NAMES)}, failed: 0, skipped: 0\n'),
        ('uri-resolution', 0, ''.join('PASS ' + n + '\n' for n in URI_RESOLUTION_NAMES) + f'Tests: {len(URI_RESOLUTION_NAMES)}, passed: {len(URI_RESOLUTION_NAMES)}, failed: 0, skipped: 0\n'),
        ('zoned-time-parsing', 0, ''.join('PASS ' + n + '\n' for n in ZONED_TIME_PARSING_NAMES) + f'Tests: {len(ZONED_TIME_PARSING_NAMES)}, passed: {len(ZONED_TIME_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('instant-parsing', 0, ''.join('PASS ' + n + '\n' for n in INSTANT_PARSING_NAMES) + f'Tests: {len(INSTANT_PARSING_NAMES)}, passed: {len(INSTANT_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('duration-offset-parsing', 0, ''.join('PASS ' + n + '\n' for n in DURATION_OFFSET_PARSING_NAMES) + f'Tests: {len(DURATION_OFFSET_PARSING_NAMES)}, passed: {len(DURATION_OFFSET_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('time-parsing', 0, ''.join('PASS ' + n + '\n' for n in TIME_PARSING_NAMES) + f'Tests: {len(TIME_PARSING_NAMES)}, passed: {len(TIME_PARSING_NAMES)}, failed: 0, skipped: 0\n'),
        ('discovery-contract', 1, 'PASS first discovered test\nFAIL discovered failure: Expected 1, actual 2\nPASS after discovered failure\nPASS NeoClr.DiscoveryTests.D preserves café. 世界 😀\nPASS NeoClr.DiscoveryTests.E adds integers(2, 3, 5)\nPASS NeoClr.DiscoveryTests.E adds integers(5, 6, 11)\nPASS NeoClr.DiscoveryTests.F accepts data rows/NeoClr.DiscoveryTests.Inputs with spaces. λ[0]\nPASS NeoClr.DiscoveryTests.F accepts data rows/NeoClr.DiscoveryTests.Inputs with spaces. λ[1]\nPASS manually registered companion\nTests: 9, passed: 8, failed: 1, skipped: 0\n'),
        ('runner-contract', 1, 'PASS before failure\nFAIL intentional assertion failure: Expected 1, actual 2\nPASS after failure\nSKIP intentional skip: contract probe\nTests: 4, passed: 2, failed: 1, skipped: 1\n'),
    ]
    # Manual async entry points and intentional-failure contracts retain their
    # explicit registration/report order. All ordinary suites use discovery.
    expected = [(name, code, output if name in ('memory-lifetimes', 'http-content-copy', 'http-header-allocations', 'discovery-contract', 'runner-contract')
                 else discovery_order(output)) for name, code, output in expected]
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
                    if (not args.suite and case[0] not in ('record-allocations', 'media-type-allocations', 'uri-path-allocations', 'http-header-allocations', 'native-pointers', 'native-buffers', 'native-callbacks')) or (args.suite and case[0] in args.suite)]
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
                                '-fPIC', str(suite_directory(name) / 'fixture.c'), '-o', str(fixture)], check=True)
                imports = out / 'NativeCallbackImports.rvn'
                template = suite_directory(name) / 'Imports.rvn'
                imports.write_text(template.read_text().replace('"native-callback-fixture"', json.dumps(str(fixture.resolve()))))
                for source in (suite_directory(name)).iterdir():
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
                                '-fPIC', str(suite_directory(name) / 'fixture.c'), '-o', str(fixture)], check=True)
                imports = out / 'NativeBufferImports.rvn'
                imports.write_text('import System.Runtime.InteropServices.*\n[NativeImport(' + json.dumps(str(fixture.resolve())) +
                                   ', EntryPoint: "native_buffer_increment")]\nunsafe extern func Increment(buffer: *byte, count: int) -> System.Void;\n')
                fixture_environment['NeoClrNativeBufferImports'] = str(imports.resolve())
                environment.update(fixture_environment)
                report['nativeBufferFixtureSha256'] = sha(fixture)
            if name not in ('runner-contract', 'memory-lifetimes', 'http-content-copy', 'http-header-allocations'):
                started = time.perf_counter()
                registry = DISCOVERY.discover(suite_project(name), bundle, out / (name + '-discovery'), fixture_environment)
                report['phases'].append(dict(suite=name, phase='discovery', elapsedSeconds=round(time.perf_counter() - started, 6), details=json.loads((out / (name + '-discovery') / 'timings.json').read_text())))
                environment['NeoClrTestRegistry'] = str(registry)
            # Async memory tests need lifecycle bindings; JSON DOM fixtures also select HTTP content/socket handlers.
            profile = 'http' if name in ('http-text', 'http-content-copy', 'http-lifetime', 'memory-stream', 'memory-lifetimes', 'gzip', 'json-dom', 'json-streams', 'configuration', 'localization', 'encoding-streams', 'encoding-boundaries') else 'console'
            if os.name == 'nt':
                profile = 'windows-' + profile
            if name in ('dependency-injection', 'method-state', 'localization', 'logging'):
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
                                    and (declaration.get('origin', {}).get('publicly_visible') if name in ('dependency-injection', 'logging')
                                         else declaration['origin']['name'].endswith('.LocalizedGreeting') if name == 'localization'
                                         else any(marker in declaration['origin']['name'] for marker in ('MethodOwner', 'MethodValue')))]
                if name == 'dependency-injection':
                    di_types = reflection_names
                elif name in ('localization', 'logging'):
                    service_adapter_types = reflection_names + (["Neo.Localization.DefaultLocalizer`1"] if name == 'localization' else ["Neo.Logging.DefaultLogger`1", "Neo.Logging.DefaultLoggerFactory"])
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
                           profile, '--project', suite_project(name),
                           '--bundle', bundle, '--aot', args.aot.resolve(), '--output', build]
                if name in ("encoding-buffer-limits", "http-text", "localization"):
                    command += ["--byte-array-limit", "131072"]
                if name in ("gzip", "configuration", "json-dom"):
                    # Large JSON input and the configuration 1 MiB rejection boundary
                    # need a host budget larger than the document itself.
                    command += ["--http-heap-bytes", "33554432"]
                if name == "json-streams":
                    # Scalar mapping needs a reflection scope, but no retained model constructors.
                    command += ["--reflection-roots", json_roots, "--command-timeout", "600"]
                if name in ("localization", "logging"):
                    command += [arg for type_name in service_adapter_types for arg in ("--reflection-constructor-type", type_name)]
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
                           '--project', suite_project(name)]
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
            if name not in ('runner-contract', 'memory-lifetimes', 'http-content-copy', 'http-header-allocations'):
                started = time.perf_counter()
                DISCOVERY.verify_registration(build / 'app.dll', bundle, out / (name + '-discovery'))
                report['phases'].append(dict(suite=name, phase='registration-verification', elapsedSeconds=round(time.perf_counter() - started, 6)))
            if name == 'dependency-injection':
                di_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', di_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), constructors=True)[1]
            if name in ('localization', 'logging'):
                service_adapter_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', service_adapter_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), constructors=True, modules=[bundle / path for path in json.loads((bundle / "manifest.json").read_text())["modules"]])[1]
            if name == 'method-state':
                method_roots = runpy.run_path(str(ROOT / 'scripts/run-native-sample.py'))['reflection_roots'](
                    args.runtime.resolve(), build / 'app.dll', method_types,
                    lambda command: subprocess.run(list(map(str, command)), check=True, capture_output=True), methods=True)[1]
            isolated = out / (name + '-isolated')
            isolated.mkdir()
            if name == 'localization':
                for fixture in ('catalogue.json', 'invalid.json'):
                    shutil.copyfile(ROOT / 'extensions/Neo.Localization/test/Neo.Localization.Tests' / fixture, isolated / fixture)
                shutil.copytree(ROOT / 'extensions/Neo.Localization/test/Neo.Localization.Tests/resources', isolated / 'resources')
            if name == 'configuration':
                for fixture in ('appsettings.json', 'invalid-appsettings.json'):
                    shutil.copyfile(suite_project(name).parent / fixture, isolated / fixture)
                # Exercise file reading independently of guest file-write support.
                (isolated / 'large-appsettings.json').write_text(
                    '{/* file */"Long":"' + 'é雪😀' * 8192 + '",}', encoding='utf-8')
                (isolated / 'oversized-appsettings.json').write_bytes(b' ' * (1048576 + 1))
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
                    executions.append((mode, interpreter + ['--execution', 'interpreter'], isolated if name in ('localization', 'configuration') else ROOT, None))
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
                    if name in ("localization", "logging"):
                        jit += ["--reflection-roots", service_adapter_roots]
                    executions.append((mode, jit, isolated if name in ('localization', 'configuration') else ROOT, None))
            repeated_executions = [(repetition, *execution) for repetition in range(1, args.execution_repetitions + 1) for execution in executions]
            for repetition, mode, command, cwd, environment in repeated_executions:
                started = time.perf_counter()
                # Content-copy includes the same 65,537-byte text-quota boundary
                # as the large text suite; interpreter byte loops need its budget.
                timeout = 600 if (name in ("http-text", "gzip")
                                  or (name == "http-content-copy" and mode == "interpreter")) else 180
                result = subprocess.run(list(map(str, command)), cwd=cwd, env=environment, capture_output=True, timeout=timeout)
                record = dict(suite=name, mode=mode, repetition=repetition, exitCode=result.returncode,
                              elapsedSeconds=round(time.perf_counter() - started, 3),
                              stdout=result.stdout.decode('utf-8').replace('\r\n', '\n'),
                              stderr=result.stderr.decode('utf-8').replace('\r\n', '\n'))
                report['cases'].append(record)
                print(name + ' (' + mode + '):\n' + record['stdout'], end='', flush=True)
                if (record['exitCode'], record['stdout']) != (exit_code, stdout) or not expected_diagnostics(record['stderr'], mode):
                    raise RuntimeError(name + ' ' + mode + ' did not match the runner contract')
            if name in ('index-range', 'collections'):
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
                if name == 'collections':
                    guards = {'priority-capacity': 'PriorityArrayQueue capacity must be non-negative',
                              'priority-reentry': 'PriorityArrayQueue callbacks must not reenter the queue'}
                for guard, message in guards.items():
                    for mode, command, cwd, environment in executions:
                        arguments = [*command, *(['--'] if mode in ('interpreter', 'jit') else []), '--guard', guard]
                        started = time.perf_counter()
                        result = subprocess.run(list(map(str, arguments)), cwd=cwd, env=environment, capture_output=True, timeout=30)
                        record = dict(suite=name, mode=mode, guard=guard, exitCode=result.returncode, elapsedSeconds=round(time.perf_counter() - started, 6),
                                      stdout=result.stdout.decode('utf-8'), stderr=result.stderr.decode('utf-8'))
                        report['cases'].append(record)
                        if not record['exitCode'] or message not in record['stderr'] or record['stdout']:
                            raise RuntimeError(name + ' guard did not fault as expected: ' + repr(record))
                print(name + ': guards passed in selected modes', flush=True)
            selections = {
                'collections': [(['--filter', 'NeoClr.RuntimeTests.ArrayQueue preserves FIFO'], 0, 'PASS NeoClr.RuntimeTests.ArrayQueue preserves FIFO\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
                                (['--filter', 'Map'], 0, ''.join('PASS ' + n + '\n' for n in MAP_NAMES) + f'Tests: {len(MAP_NAMES)}, passed: {len(MAP_NAMES)}, failed: 0, skipped: 0\n')],
                'discovery-contract': [(['--id', '21:TestDiscoveryContract21:NeoClr.DiscoveryTests45:NeoClr.DiscoveryTests.D preserves café. 世界 😀()'], 0, 'PASS NeoClr.DiscoveryTests.D preserves café. 世界 😀\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--filter', 'café. 世界 😀'], 0, 'PASS NeoClr.DiscoveryTests.D preserves café. 世界 😀\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--category', 'fast'], 0, 'PASS first discovered test\nPASS NeoClr.DiscoveryTests.E adds integers(2, 3, 5)\nPASS NeoClr.DiscoveryTests.E adds integers(5, 6, 11)\nTests: 3, passed: 3, failed: 0, skipped: 0\n'),
                                       (['--category', 'data'], 0, 'PASS NeoClr.DiscoveryTests.F accepts data rows/NeoClr.DiscoveryTests.Inputs with spaces. λ[0]\nPASS NeoClr.DiscoveryTests.F accepts data rows/NeoClr.DiscoveryTests.Inputs with spaces. λ[1]\nTests: 2, passed: 2, failed: 0, skipped: 0\n'),
                                       (['--category', 'FAST'], 2, 'ERROR No tests matched the selection\n'),
                                       (['--category', ''], 2, 'ERROR Expected --filter <text>, --id <id> or --category <name>\n'), (['--filter', 'Inputs with spaces. λ[1]'], 0, 'PASS NeoClr.DiscoveryTests.F accepts data rows/NeoClr.DiscoveryTests.Inputs with spaces. λ[1]\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--filter', 'E adds integers(2, 3, 5)'], 0, 'PASS NeoClr.DiscoveryTests.E adds integers(2, 3, 5)\nTests: 1, passed: 1, failed: 0, skipped: 0\n'), (['--id', 'manual'], 0, 'PASS manually registered companion\nTests: 1, passed: 1, failed: 0, skipped: 0\n'),
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
                if name == 'collections':
                    selected_stdout = discovery_order(selected_stdout)
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
