# JSON byte output — 2026-10-19

DocumentWriter now accumulates bounded UTF-8 bytes, borrowing String.Utf8Bytes for
number/string payloads and escaping directly into the destination. It no longer
concatenates a growing document string or creates quoted-token strings. Text APIs
decode the final window once; SerializeToUtf8Bytes returns a newly owned array.
HTTP JsonContent and OutputStream serialization consume byte output directly.
Arrays are appropriate for these owned results. The pending stream write retains
its array; no borrowed slice survives an await. Shared stream Memory signatures
and byte-input parsing remain separate work.

This follows System.Text.Json's
[SerializeToUtf8Bytes](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonserializer.serializetoutf8bytes)
contract rather than adding As-style adapters. Existing typed InputStream and
OutputStream overloads already provide asynchronous stream use. Dedicated incremental
Utf8JsonReader/Utf8JsonWriter contracts are not introduced. The author clarified the
System.Text.Json direction after considering Newtonsoft-style TextReader/TextWriter
overloads; those text overloads are not added.

The prerequisite native String.Utf8Bytes gap is fixed separately in cc972be22.
The initial native attempt failed on ViewUtf8 and an intermediate token-encoding
fallback was not adopted. The final writer uses borrowed string bytes. Native
kernel, exact-binding and three-mode consumer evidence is in
[runtime string views](../../../runtime/raven/tests/string-utf8-views/README.md).
No Raven source compiler changes or string representation changes are required.

## Allocation result

The same Probe.rvn parses three fixtures once (numeric array, repeated unescaped
strings, escaped Unicode/NUL), then performs 100 HTTP content creations and 100
MemoryStream serializations for each. Each run validates success and byte lengths.
These are 600 outputs, not 600 network requests. Counts include startup, parsing,
three expected text results, memory streams, validation and shutdown. Native caller
traces reconcile all allocations and payload bytes, with zero dropped records.

| Counter | Baseline | Candidate |
| --- | ---: | ---: |
| All allocations | 28,396 | 8,883 |
| Allocated payload bytes | 1,352,491 | 622,266 |
| Actual text allocations | 7,757 | 373 |
| Text payload bytes | 165,328 | 12,682 |
| Byte-array allocations | 8,855 | 4,238 |
| Byte-array payload bytes | 453,623 | 253,804 |

That is 68.7% fewer allocations and 54.0% fewer allocated payload bytes. Text
allocation count falls 95.2%. Remaining storage includes owned outputs, growth
buffers, parser/mapping state and MemoryStream backing. This is not zero-allocation
serialization, retained-memory measurement or GC-pause attribution. The change
includes removal of the StreamWriter/encoder adapter on JSON byte-stream output.

## Paired uninstrumented timing

Four paired runs alternate baseline/candidate and candidate/baseline. Builds,
profiling and test runs were finished before timing. Every process validates.

| Round | Baseline CPU seconds | Candidate CPU seconds | Baseline wall seconds | Candidate wall seconds |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.5351 | 0.3820 | 0.7227 | 0.6282 |
| 2 | 0.5266 | 0.3858 | 0.5283 | 0.3893 |
| 3 | 0.5273 | 0.3740 | 0.5293 | 0.3759 |
| 4 | 0.5302 | 0.3730 | 0.5321 | 0.3749 |

Mean whole-process CPU is 0.5298 versus 0.3787 seconds (28.5% lower). Startup and
scheduling particularly affect the first wall-time pair; all samples are retained.
This small fixed workload does not establish HTTP throughput, maximum capacity,
near-limit document performance or a synchronous/concurrent-GC comparison. The
baseline is the previous decoder integration; the candidate includes native
string-view support and the JSON writer together. No production GC policy changes.

## Correctness and documentation

All 14 JSON stream cases pass in interpreter, JIT and AOT (42 executions), including
byte-result ownership, compact/indented exact output, Unicode/control escaping,
exact and exceeded quotas, cycles/depth, no writes on invalid documents, short writes,
zero-progress errors, partial-output failures and leaving streams open/unflushed.
Input-side stream regression cases also pass; their implementation is unchanged.
A stale fixture using Decode(Sequence<byte>) was corrected to explicitly materialize
a window. The earlier failed fixture/native runs are retained for transparency.

Native API rendering and the legacy CLI reference include both new overloads;
fingerprint and four native snapshot checks pass. Website descriptions and API
ownership/error contracts are updated; no full website build or publication occurs.

## Reproduction

Build the same Probe.rvnproj with scripts/build-native-project.py, HTTP host profile,
against each matching class-library bundle. The committed build evidence retains
full commands and input hashes. Baseline: target/json-byte-baseline; candidate:
target/json-byte-final-candidate with target/json-byte-final-libraries and
 target/json-byte-final-bundle. Raven revision dcf7ac41f is unchanged. Library and
native source changes were built before their commits; artifact hashes identify
the tested inputs. Xcode 26.2/MacOSX26.2 SDK are used.

```sh
python3 benchmarks/json-byte-output/20261019/capture.py --build target/json-byte-baseline --output target/json-byte-baseline-callers
python3 benchmarks/json-byte-output/20261019/capture.py --build target/json-byte-final-candidate --output target/json-byte-final-callers
python3 benchmarks/json-byte-output/20261019/compare.py --baseline target/json-byte-baseline/app --candidate target/json-byte-final-candidate/app --output target/json-byte-timing.json
```

capture.py reuses the existing native caller instrumentation and reports payload
bytes, not allocation headers or live retention. Its diagnostic timing is excluded
from comparison. evidence/manifest.json hashes the compressed retained reports.
