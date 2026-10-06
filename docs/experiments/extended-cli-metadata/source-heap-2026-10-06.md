> **Follow-up completed:** The nullable import blocker described below is now fixed.
> Raven `d19c6e4a3` with neoCLR metadata `119d2daf` compiles and executes both consumers,
> including KeepAlive(null). The earlier checkpoint is retained as historical evidence.
> [Successful native-nullability gate](source-heap-nullable-2026-10-06.json).

# Native source GC checkpoint — 2026-10-06

**Partial gate:** the unchanged System.Runtime.GC source compiles with two internal
native adapters. Its artifact-only retention consumer verifies and executes with exit
0 and exact output `Native source heap passed`. The valid KeepAlive(null) consumer
still fails native semantic import; this checkpoint does not complete the public API.
[Commands, revisions and input/artifact hashes](source-heap-2026-10-06.json).

## Implemented and tested

Eight missing RuntimeServices declarations are now supplied by source adapters. The
six Int64 counter services already existed. GCCollect and GCKeepAlive now additionally
accept the no-result return convention emitted by Raven. GCCollect must not push an
inhabited Void onto the caller's stack for that convention. Existing unit-returning
callers retain their behavior; wrong argument/return signatures and no-result counter
services still reject. No metadata format or opcode changes are introduced.

The nine runtime GC tests cover both conventions, roots, heap limits and execution-local
counters. The new no-result test keeps a live heap reference on the operand stack,
collects, calls KeepAlive with a null reference and reads the retained value. The focused
native admission test also passes. The artifact consumer checks all six public counters,
explicit collection, retained contents, reference identity and shared mutation. There
are no library sources in its compilation command. API snapshot validation passes.
The [existing GC/.NET comparison](../../runtime-gc.md) applies: control calls have the
usual no-result call behavior; neoCLR's object counters and collector limits are unchanged.
No performance claim or broad runtime/release requalification is made.

## Open nullable metadata blocker

Native import currently erases the nullable-reference parameter in KeepAlive(object?).
The compiler therefore rejects KeepAlive(null) with RAV1503/RAV1509 and publishes no
consumer assembly. The original valid fixture remains `bootstrap/heap-consumer.rvn`;
`bootstrap/heap-retention-consumer.rvn` isolates the non-null runtime contract.
The harness records and checks this known rejection separately from successful execution;
a zero harness exit is not full GC API acceptance.

Fix the general metadata boundary next: preserve CLI-style nullable-reference annotation
facts in definitions and encoding, expose them through readers/introspection, and project
them into Raven symbols. Cover nullable/non-null parameters and results, nested generic
and array positions, and reference-only round trips before claiming the whole category.
Do not special-case GC, weaken null diagnostics or substitute importer objects in emission.
This metadata gap potentially affects many APIs and takes priority over adding more
isolated service facades. No general Raven code fix was made or backported in this slice;
Raven integration documentation is committed as 7221a99ea.

## Reproduce

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_heap.py \
  --compiler /path/to/rvnc.dll --compiler-revision <revision> \
  --library /path/to/Numbers.dll --ownership runtime/raven/native/poc-ownership.json \
  --seed /path/to/System.neox --core /path/to/Core.dll \
  --runtime target/debug/neoclr --output /tmp/source-heap-gate
cargo test --test runtime_gc
cargo test --lib reflection_signature_tests
python3 scripts/build-api-docs.py --check
```

Use the matching bootstrap and Numbers artifacts in the evidence. The successful run
uses the pinned extracted Preview 12 compiler and a newly built debug runtime; a separate
compiler test process was replacing the integration checkout's build outputs, so those
mutable outputs were not reused. Published Preview 12 runtime artifacts are unchanged.
The full-source audit was not rerun; no new total diagnostic count is claimed.


## Nullable gate closure

The current `verify_source_heap.py` requires both artifact-only consumers to compile,
verify and execute with exact stdout `Native source heap passed` and exit 0. The earlier
expected compiler rejection has become a success assertion; no fixture source was
changed to hide the failure. The fixed compiler snapshot and explicit primitive core,
Numbers, ownership manifest and retained runtime seed are hashed in the new evidence.

C# native symbol checks validate parameter/results, nested array/generic positions and
open/constructed generic method substitution. Nonnullable parameters still reject null.
Seventeen focused .NET checks and all seven native consumer executions pass. General
nullable context/field support and full-System bootstrap remain unfinished. Published
Preview 12 is unchanged.
