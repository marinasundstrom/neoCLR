# Borrowed JSON token parsing — 2026-10-19

DocumentReader and MessageReader now receive ReadOnlySlice<byte> parameters and
retain only cursor/count state. String input borrows its UTF-8 storage. Unescaped
names/values decode directly into owned strings; escaped tokens still use scratch
storage. Boolean/null tokens avoid temporary strings. Number grammar borrows the
owned lexeme, retaining spelling without encoding another array.

This is the internal foundation for direct UTF-8 input, comparable to
[System.Text.Json's byte input](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.deserialize).
Public signatures are unchanged. HTTP and InputStream deserialization still decode
the whole document first. Before removing that step, byte validation, quotas,
read consumption and I/O failure ordering must be preserved. Views never become
parser fields or survive an await; returned DOM strings remain owned.

## Allocation measurement

The identical probe executes 200 parse/serialize/reparse/serialize cycles for each
of three fixtures: a numeric array, repeated unescaped strings, and escaped Unicode
with NUL. Each process performs 1,200 parses and 1,200 serializations and checks
success throughout. Correctness/content assertions are in the separate contracts.
This measures a round-trip workload, not isolated parser time or network requests.

| Counter | Baseline | Candidate |
| --- | ---: | ---: |
| All allocations | 46,816 | 31,616 |
| Allocated payload bytes | 1,480,065 | 1,050,065 |
| Actual text allocations | 9,200 | 6,800 |
| Text payload bytes | 148,800 | 106,400 |
| Byte-array allocations | 14,801 | 4,001 |
| Byte-array payload bytes | 551,633 | 266,433 |

Allocations fall 32.5% and allocated payload bytes 29.1%. Both instrumented traces
reconcile with zero dropped records. Counts include setup, serialization,
validation and process startup/shutdown. They do not measure retained memory,
GC pauses, HTTP throughput or differences between collector policies. Remaining
allocations include owned DOM values/lexemes, escaped-token buffers and outputs.
The escaped fallback adds an initial scan before decoding, a CPU tradeoff to check.

## Reproduction and provenance

Build Probe.rvnproj using scripts/build-native-project.py with the HTTP host
profile and matching bundles. Baseline libraries are from 138f47886; candidate
libraries differ only in JSON parsing. Raven codex/neoclr revision dcf7ac41f,
target/release/neoclr and target/debug/neoclr-codegen are shared. Native ViewUtf8
support and the byte-output writer are present in both builds.

Builds: target/json-input-baseline and target/json-input-candidate.
Candidate library/bundle: target/json-input-libraries and target/json-input-bundle.
capture.py relinks each app object with diagnostic GC caller/size instrumentation.
compare.py alternates four paired uninstrumented process runs. Do not run timing
while compilation, profiling or contract suites are active.

Compressed evidence retains build commands/hashes, allocation traces, timings
and contract reports; MANIFEST.json records checksums of the stored files.


## Correctness coverage

The existing 21 DOM contracts pass in interpreter and JIT. The expanded stream
suite covers empty/unescaped Unicode, combining text, surrogate escapes, embedded
NUL, number spelling, booleans/null, malformed escapes/control bytes/numbers and
escaped aliases of duplicate names, alongside existing stream/quota/error checks.
The initial new fixture incorrectly expected a short newline escape; both baseline
and candidate emitted the existing canonical \u000a spelling. The fixture was
corrected without changing production behavior; initial logs are retained.
The DOM suite's separate synchronous AOT host-link limitation remains known;
native parser coverage comes from the async-entry stream suite and process probe.

The matching native API rendering and legacy reference fingerprints are refreshed;
four native snapshot checks pass. Public signatures and Raven compiler are unchanged.

## Timing and unresolved CPU regression

All four paired uninstrumented runs validate. Builds and tests had finished before
timing. This is a material regression, not an HTTP performance improvement.

| Round | Baseline CPU seconds | Candidate CPU seconds | Baseline wall seconds | Candidate wall seconds |
| --- | ---: | ---: | ---: | ---: |
| 1 | 1.4302 | 1.8737 | 1.6315 | 2.0538 |
| 2 | 1.4338 | 1.8788 | 1.4381 | 1.8809 |
| 3 | 1.4290 | 1.9022 | 1.4317 | 1.9203 |
| 4 | 1.4274 | 1.8737 | 1.4297 | 1.8760 |

Mean CPU rises from 1.4301 to 1.8821 seconds
(31.6% higher), despite the allocation reduction. The borrowed parser is retained
as an allocation/lifetime foundation, with this CPU cost explicitly unresolved.
No general speedup, HTTP capacity or collector-policy result is claimed.

A separate one-second /usr/bin/sample capture at 1 ms intervals for each executable
shows GC/root processing dominating both samples: collect_indexed, mark_frames,
neoclr_gc_stack_roots_v1 and neoclr_gc_collect_conditional_v1 lead the leaf counts.
In the candidate these account for 226, 181, 145 and 96 samples respectively
(out of 742 main-thread samples). Byte-view validation appears but is not the
dominant sampled leaf. These short, phase-dependent captures do not prove a cause
or quantify a per-function regression. Source inspection also shows every checked
view access validates storage; increasing borrowed arguments changes frame/root
work. GC boundary/root costs and redundant view validation are investigation
targets before claiming that fewer arrays make HTTP faster. No checks were removed.

Final validation: 16 stream cases pass in AOT, interpreter and JIT (48 executions),
and 21 DOM cases pass in interpreter and JIT (42 executions). The baseline also
passes the corrected 16-case native suite.
