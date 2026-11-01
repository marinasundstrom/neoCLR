# Post-release testing migration — 2026-11-01

## Bounded scope

The author deferred applicable Python-driven behavior-fixture migration until after
Preview 19. The first slice is
[RangeIterationTests](../../../runtime/System.Runtime/test/RangeIterationTests/README.md).
It preserves the [existing compiler contract and .NET comparison](../range-iteration/README.md)
without changing runtime/compiler code, APIs, release assets or execution defaults.

| Original checks | Discovered cases |
| --- | --- |
| Eight signed-limit loops, including overshoot and minimum steps | Four cases per Int32/Int64 width, with bounded iteration guards and count assertions |
| Inclusive bounds and step side effects | Once-only source-order and iteration-count assertions |
| Break at signed maximum | Immediate break and one-iteration assertion |
| Exclusive maximum, empty and reversed bounds | One case per width, preserving yielded-value checks and rejecting any empty-range body execution |
| Exclusive bound side effects | Evaluation-order and bounded-count assertions |
| Continue/break interaction | Exactly one counted body between continue and break |

The standard discovered-test registry and runner replace a handwritten Main and
numeric failure exits. Python retains bundle hash verification before/after the run
and orchestration. The legacy command remains supported; `results.json` now copies
the common runner's `report.json` schema. Existing historical evidence is unchanged.

## Remaining applicability review

- Socket and HTTP memory lifecycles now use explicit async framework adapters.
  Both pass all three modes. The HTTP follow-up below diagnoses its earlier
  interpreter wait-bridge failure and scopes the encoding borrow to a synchronous
  helper; paired original/migrated failure evidence remains available.
- HTTP custom-method wire peers, compiler visibility rejection and terminal-failure
  subprocess probes retain legitimate Python host orchestration. Do not mechanically
  convert these into in-process assertions.
- Published sample output checks and package/file-integrity checks remain distinct
  from framework behavior tests. This slice does not retire them wholesale.

## Validation

[The common runner report](three-mode-report.json.gz) and [transcript](three-mode.log.gz)
pass all 14 tests in interpreter, managed JIT and native AOT: 42 behavior-test
executions on macOS ARM64. Discovery, registration verification, compilation and
bundle-integrity checks pass. Inputs are fingerprinted in the report. The matching
development bundle is `target/http-message-surfaces/inputs9/bundle`; this verifies
the migrated tests against that qualified compiler/library/runtime composition,
not a new release or a claim of freshly rebuilt unified Raven main.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-native-range-iteration.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/range-testing-migration-20261101
```

No website build is needed for this test-only migration. Existing .NET/compiler
qualification and historical negative probes are retained; no compiler contract
or platform-support claim changes.

A [fresh-output negative control](fresh-output-control.json) rejects a reused
validation directory and proves every existing evidence file remains unchanged.
That early wrapper guard was added after the three-mode run; it changes only the
precondition, while the recorded runner and Raven test inputs are unchanged.

## String-backed UTF-8 views

The next bounded migration replaces `StringUtf8Views/Main.rvn`'s standalone Panic
checks with four discovered tests in the same project. The historical native memory
kernel and binding-negative controls remain separate and unchanged. This is test
organization only: the existing interpreter/native byte-view contract, compiler
behavior and public APIs are unchanged. .NET comparison remains the existing
UTF-8 platform/view design; no new semantic divergence is introduced.

- Literal length, embedded NUL and multibyte byte positions.
- Nested owner lifetime after helper return and collection, successful decoding,
  isolated mutable copy, rejected split-scalar decoding and a second collection in
  one sequence. Splitting this sequence into independent tests would weaken coverage.
- Empty-string and end-of-string windows.
- Independently named split-scalar rejection through `EncodingError.InvalidUtf8`.

The original file is retained in Git history; historical reports are not rewritten.
Use `scripts/test-runtime-library.py --suite string-utf8-views` with interpreter,
JIT and native modes. Discovery and the standard runner replace the custom entry
point. [The report](string-utf8-three-mode-report.json.gz) and
[transcript](string-utf8-three-mode.log.gz) pass all four tests in interpreter,
managed JIT and native AOT: 12 test executions on macOS ARM64. Discovery and
registration verification also pass. The same qualified
`target/http-message-surfaces/inputs9/bundle` provides fingerprinted compiler,
library, runtime and helper inputs; this does not claim a fresh compiler rebuild.
The independent split-scalar test and the rejection in the combined lifetime test
both preserve the invalid-UTF-8 negative assertion. Native kernel/binding probes
are unaffected and their existing evidence is reused.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/test-runtime-library.py --suite string-utf8-views \
  --mode interpreter --mode jit --mode native \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/string-utf8-testing-migration-20261101
```

## File memory windows

Two discovered [file-memory tests](../../../runtime/System.Runtime/test/FileMemoryTests/README.md)
replace numeric exits and the inline-completion Panic helper. The first preserves
selected write/read windows, empty end windows, invalid ranges, sentinel bytes,
short read/EOF and cursor preservation. The second preserves closed-state versus
range validation and pre-cancellation precedence for both input and output.
Each case has its own file; execution order is immaterial. Explicit task-outcome
observation is intentional: these assertions qualify the current blocking provider's
inline completion, not asynchronous OS I/O. Stream behavior and its existing .NET
comparison are unchanged; this migration changes no production API or implementation.

Python retains isolated temporary-directory ownership and independent file-byte
checks (`2802` and an empty closed-state file), while the common metadata discovery
and registration verifier check the test identity. A small custom entry point passes
the host-selected path to the fixtures and forwards optional selectors to TestRunner.
The wrapper's stale `Tests.rvnproj` reference now selects `FileMemoryTests.rvnproj`.

[The interpreter report](file-memory-report.json.gz) passes both tests on macOS ARM64
using the same fingerprinted development bundle as the previous migrations.
[Discovered identities](file-memory-discovery.json.gz) are verified again against
the final compiled assembly. An [existing-file negative control](file-memory-negative-control.json.gz)
passes: CreateNew's rejected input produces a named test failure, exit code one and
one passing/one failing result, while the preexisting bytes remain unchanged.
This demonstrates failure propagation through the migrated runner.

The [initial compile diagnostic](file-memory-initial-compile-error.log.gz) exposed
that slicing entry arguments produces Slice<string>, while TestRunner expects
string[]. The fixture now calls ToArray on that slice; the successful run uses a
fresh output directory. No compiler change was needed. Historical host-call/rooting
and legacy bootstrap tests remain separate and unaffected. JIT/AOT file-service
coverage is not inferred from this interpreter-only fixture.

```sh
python3 runtime/System.Runtime/test/FileMemoryTests/verify.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/file-memory-testing-migration-20261101-retry
```

## Console memory windows

The [console fixture](../../../runtime/System.Runtime/test/ConsoleMemoryTests/README.md)
now uses one discovered `NeoClr.Testing` case and a quiet TestSuite result path.
Success prints no test transcript into the binary channels; failures use the normal
ConsoleReporter and nonzero exit. Python still validates exact stdout `2800ff`
and guest stderr `00ff` in interpreter, managed JIT and AOT. Only the exact existing
DAM2004 JIT compiler warning may precede the stderr payload; raw bytes and the
separated diagnostic remain recorded.

The former fixture no longer compiled against the current public console surface:
OpenStandardOutput/Error return SyncOutputStream with ReadOnlySlice writes, not
array/range Task APIs. The migration follows the existing
[provider capability decision](../../stream-design.md#text-capabilities-and-provider-dependent-modes--2026-10-20-proposal)
and .NET comparison; it introduces no new semantic divergence. Closed valid/empty
writes and flush are asserted in-process. Negative, oversized and beyond-end empty
windows now fault during Slice construction; three subprocess guards preserve
those negative inputs in every mode. Async console/pre-cancellation assertions are
obsolete under the current contract, not silently counted as passing. Current
asynchronous file/stream cancellation fixtures retain their own contracts.

The wrapper's stale Tests.rvnproj path is corrected. Generated discovery and final
registration verification use the shared tooling.

[The final report](console-memory-report.json.gz) passes all three binary-output
runs and all nine bounds guards on macOS ARM64 with the same fingerprinted
`target/http-message-surfaces/inputs9/bundle`. [Discovered identities](console-memory-discovery.json.gz)
are reverified against the final assembly. The original
[obsolete-API compile failure](console-memory-old-api-compile.log.gz) and
[unseparated JIT warning failure](console-memory-unseparated-warning.json.gz) are
preserved separately. No compiler/runtime/library code changes were made.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 runtime/System.Runtime/test/ConsoleMemoryTests/verify.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/console-memory-testing-migration-20261101-final
```

### Open diagnostic: provider write-error propagation

An additional [read-only-stderr probe](console-memory-readonly-stderr-probe.json.gz)
redirected child stderr to a null device opened read-only, keeping stdout captured.
It is not passing qualification: interpreter returned zero with only the normal
stdout payload; AOT printed the named failed assertion and one-failure summary but
returned two, not the probe's expected one. JIT was not probed because compiler
diagnostics share that stderr descriptor. The probe used the same test/native
assembly as the preceding synchronous run; only host descriptor permissions changed.
This exposes an unresolved provider-error/exit-status difference requiring focused
runtime investigation. Do not infer corrected I/O error propagation from this
fixture migration or silently relabel the diagnostic as passed. Existing file-fixture
failure-control evidence still covers ordinary framework failure propagation in the
interpreter. Native host memory/binding probes remain separate and unaffected.

## Socket memory ownership

The [socket fixture](../../../runtime/System.Networking/test/SocketMemoryTests/README.md)
is now a named async `NeoClr.Testing` case. Its original lifecycle remains combined:
loopback pair creation, a pending selected receive, collection, selected send,
source alias mutation, collection, exact byte/count/sentinel checks, default and
end windows, expired deadline, acknowledged cancellation and unchanged destination,
pre-cancelled empty receive, detached send/receive owners with collections, EOF and
closed receive. Numeric early-return codes become descriptive framework failures.
No production API, runtime/compiler behavior or socket contract changes. The
existing memory-view ownership design and .NET comparison remain authoritative;
this migration does not introduce a new divergence.

The runner's discovery contract is synchronous, so Main explicitly awaits the
whole lifecycle before handing its result to TestSuite. A small reference owner
stores the result and exposes a bound Invoke method. The first direct closure
capture failed with NEOMETA001; this adapter respects the selected compiler's
current restriction without turning it into a permanent platform contract.
OnCompleted remains only for observing cancellation without cancelling Main.

Python retains native-profile building, process execution, exact framework output
and artifact fingerprints. The stale Tests.rvnproj path now selects the existing
SocketMemoryTests.rvnproj. Only the exact known DAM2004 JIT compiler warning is
accepted; other diagnostics still fail validation.

[The final report](socket-memory-three-mode-report.json.gz) passes the complete
lifecycle in interpreter, managed JIT and native AOT on macOS ARM64. Each mode
prints one passing framework case and the standard summary. Source/compiler/runtime
and bundle identities are fingerprinted; inputs use the same qualified
`target/http-message-surfaces/inputs9/bundle` as the preceding migrations.
No fresh compiler rebuild or Windows qualification is implied.

Retained compile attempts identify the adapter repairs:
[direct capture](socket-memory-capture-compile.log.gz) rejects the Result capture;
[a missing System import](socket-memory-import-compile.log.gz) prevents resolving
Result's type; [stored-result analysis](socket-memory-stored-result-compile.log.gz)
requires proving the union is active. Invoke now matches Ok/Failure and returns
the corresponding active case, preserving the failure payload. These are fixture
adaptations; no compiler change or general async discovery feature is implemented.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 runtime/System.Networking/test/SocketMemoryTests/verify.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/socket-memory-testing-migration-20261101-validated
```

The original numeric-exit source remains in Git history. Historical socket/kernel
ownership reports and negative controls remain unchanged. No website build is
needed for this test-only slice.

## HTTP memory lifecycle

The [HTTP memory fixture](../../../runtime/System.Web/test/HttpMemoryTests/README.md)
now uses a named async `NeoClr.Testing` case with the same shared server and
fixed-length, chunked, cancellation and client-response phase order. Each phase
returns descriptive TestFailure values; WithPhase attaches the phase while retaining
expected/actual values. Main awaits the whole lifecycle, then uses the already
qualified bound-reference TestSuite adapter. Automatic async discovery is not added.

Preserved behavior includes selected readonly socket sends with surrounding
sentinels, headers split across sends, pending nonzero destination windows,
collections before body delivery, exact body bytes and unchanged sentinels,
fixed/chunked EOF, empty end windows, invalid destination ranges, acknowledged
cancellation without byte mutation, and client ResponseHeadersRead partial reads
with collections and read-ahead byte order. Failed phases still stop before the
next request, matching the original early-return contract.

The initial nested-await assertion form was rejected by the selected compiler's
native value-block emission. Awaiting into named locals before asserting preserves
the ordering and compiles more clearly. No compiler fix or new platform rule is
claimed. The existing HTTP/stream ownership design and .NET comparison are unchanged;
there are no production library/API/runtime changes.

Python retains HTTP-profile native building, exact framework transcript checks,
artifact/source fingerprints and all three execution modes. It corrects only the
stale Tests.rvnproj path and permits the exact existing DAM2004 JIT diagnostic.
The 300-second process timeout and 100,000,000 interpreter instruction budget are
unchanged. Initial migration qualification was **partial**; the follow-up below
now passes all three modes.

[The three-mode report](http-memory-three-mode-report.json.gz) passes JIT and AOT,
including all four phases and the final framework summary. Interpreter exits one
inside RuntimeAwaitProvider.Bridge.Wait while SendText waits; it produces no test
result. The verifier retains this failure and returns nonzero overall. No mode is
skipped and no timeout/instruction limit is relaxed.

The [original numeric-exit fixture](http-memory-original-interpreter.json.gz),
compiled separately from the pre-migration committed source, fails at the same
wait-bridge location using the same bundle and limits. Its source hash, commands,
output and duration are recorded. [Reproduction script](http-memory-baseline-reproduction.py.gz).
A second paired diagnostic using the existing local `target/release/neoclr` also
fails for [the original](http-memory-current-original.json.gz) and
[the migrated fixture](http-memory-current-migrated.json.gz). That local binary is
identified by hash; it was not freshly rebuilt or claimed as new qualification.
These comparisons establish a preexisting failure in the selected inputs rather
than a passing interpreter migration. At that stage the wait-bridge/provider
interaction remained unresolved; the follow-up below identifies the cause without
changing the runtime or compiler.

The [initial compile failure](http-memory-initial-compile.log.gz) belongs to the
nested-await assertion form, before awaiting into locals. That compile adaptation
is distinct from the interpreter execution failure. Historical evidence is unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 runtime/System.Web/test/HttpMemoryTests/verify.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output /fresh/http-memory-output
```

The current fixture passes this command in all three modes. Earlier failure
reports above retain the pre-fix source identities and are not rewritten.

### HTTP borrowed encoding lifetime follow-up

The author's next request was to investigate and fix the interpreter wait-bridge
failure. Temporary local diagnostics exposed Fault.message and the rejected frame
value at the suspension check. [The diagnostic](http-memory-borrow-diagnostic.log.gz)
identifies `borrowed memory views cannot cross suspension`, with a UTF-8 view of
`!POST /window HTTP/1.1\r\nHost: localhost\r\n!` in SendText's closure. These diagnostic
prints were removed; the CLI diagnostic contract and interpreter are unchanged.
Disassembly of the failing app places `get_Utf8Bytes`, `stloc 0`, `ldloca 0` and
`ToArray` at instructions 6–9 of that closure; the later bridge Wait call is at 107.
The borrowed receiver temporary is still resident when Wait calls AwaitUnit at 13.
The signal-release guard is not the failing operation.

The interpreter deliberately rejects every resident borrowed view at suspension,
including dead locals; see the existing [memory-view contract](../../native-memory-views.md).
EncodeSendWindow now performs the UTF-8 copy in a synchronous helper. Its frame
returns before SendText awaits socket work, leaving only an owned array and
persistent readonly memory windows. The message bytes, sentinels, split writes,
shared-server lifecycle, assertions, timeout and instruction budget are unchanged.
This is a fixture lifetime correction under the current conservative contract,
not a runtime wait-bridge repair or compiler liveness relaxation. General dead-local
liveness remains a separately unqualified compiler/runtime capability.

[The new report](http-memory-borrow-scope-three-mode-report.json.gz) passes the exact
named-test transcript and summary in interpreter, JIT and AOT using the same
`target/http-message-surfaces/inputs9/bundle` as the failures. Source, consumer,
compiler, runtime and manifest hashes are recorded. Reproduce with the command
above and a fresh output directory (the recorded run used
`target/http-memory-borrow-scope-20261101`). Historical failures serve as the
before-fix negative control. Both focused Rust checks also pass:
`cargo test --test memory_views borrowed_views_` verifies synchronous host-call
borrow preservation and rejection across async yield/heap boxing. The existing
unused reflection-helper warning is unrelated. No production API, release asset,
website content or compiler bridge contract changes; no website build is needed.


## Persistent memory lifetimes

The [memory-lifetime fixture](../../../runtime/System.Runtime/test/MemoryLifetimeTests/README.md)
already used descriptive `[Test]` functions and NeoClr.Testing assertions, but Main
called a handwritten Report helper that panicked on a Failure and printed a literal
four-pass summary. This bounded migration completes framework reporting rather
than claiming the assertions were previously in Python.

Main awaits the four existing cases in their original order, retains each completed
Result in a bound reference owner and registers it with TestSuite. ConsoleReporter
prints real outcomes and Main returns report.ExitCode. The adapter reconstructs the
Result cases, following the already qualified socket/HTTP pattern; it avoids direct
value-union closure capture. Recoverable assertion failures now allow the remaining
independent cases to run and contribute to the final report. Terminal runtime faults
are unchanged. Async metadata discovery is still unsupported and is not claimed.

Tests.rvn is unchanged: two pending awaits, creator-frame return, explicit collection,
selected destination/source windows and sentinel checks, cancellation observation,
rejected transfer and default/end-position empties retain their sequencing. The
existing .NET persistent-memory versus borrowed-span comparison still applies.
The Python runner retains native-profile building, exact transcript checks and
bundle integrity checks. Ref-like capture compiler-rejection fixtures remain
separate and unchanged; their existing evidence is reused.

[Three-mode report](memory-lifetimes-three-mode-report.json.gz) and
[transcript](memory-lifetimes-three-mode.log.gz): all four cases pass in interpreter,
JIT and native AOT on macOS ARM64, using the same fingerprinted development bundle
as the preceding migrations. Source hashes are recorded; no freshly rebuilt compiler
or new release qualification is implied.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/test-runtime-library.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/memory-lifetimes-testing-migration-20261101 \
  --suite memory-lifetimes --mode interpreter --mode jit --mode native
```

[An isolated negative control](memory-lifetimes-negative-control.json.gz) changes only
the expected destination count from two to three in a copied Tests.rvn. It runs all
four lifecycles in the interpreter and reports `Expected 3, actual 2`, three passes,
one failure and exit code one, with empty stderr. The repository fixture is not
mutated. [Reproduction script](memory-lifetimes-negative-control.py.gz) retains the
compile/run commands and source hashes; run from the repository root with a fresh
output directory. No public APIs, production behavior or website content change;
no website build is needed.


## HTTP content copy

The next bounded slice completes framework reporting for the eight
[HTTP content-copy cases](../../../runtime/System.Web/test/HttpContentCopyTests/README.md).
They already used NeoClr.Testing assertions and descriptive `[Test]` functions;
the remaining custom entry point panicked on failures and printed a literal
success summary. Main now awaits the cases in their original order and registers
bound completed-result adapters with TestSuite, writes ConsoleReporter output and
returns report.ExitCode. Recoverable assertion failures contribute named failures
while later independent cases still run. Terminal runtime faults remain terminal;
this does not add automatic async discovery.

Tests.rvn is unchanged. Coverage retains pending-write backpressure, collection and
buffer ownership, selected memory windows and aliasing, buffered replay and stream
single consumption, read/write error preservation, truncated/invalid counts,
declared lengths and empty I/O, cancellation, disposal, UTF-8/text-quota checks and
creation validation before ownership transfer. The existing
[.NET API comparison and design](../../design/http-content-streaming.md) still apply.
Python retains process/build orchestration, exact transcripts and bundle integrity
checks; independent wire peers remain outside this in-process fixture migration.

[Report](http-content-copy-three-mode-report.json.gz) and
[transcript](http-content-copy-three-mode.log.gz) pass eight tests in interpreter,
JIT and native AOT: 24 test executions on macOS ARM64 with the same fingerprinted
`target/http-message-surfaces/inputs9/bundle`. Inputs are recorded in the report;
this does not claim a freshly rebuilt compiler or a new release qualification.
The existing interpreter instruction/timeout budgets, including the longer timeout
for the 65,537-byte text-quota boundary, are unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/test-runtime-library.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/http-content-copy-testing-migration-20261101 \
  --suite http-content-copy --mode interpreter --mode jit --mode native
```

[The isolated negative control](http-content-copy-negative-control.json.gz) changes
one copied-count assertion from 2500 to 2501 in a separate Tests.rvn copy. JIT reports
`Expected 2501, actual 2500`, seven passing cases, one failure and exit code one.
Only the existing exact DAM2004 compiler warning is permitted on stderr.
The repository fixture is not mutated. The
[reproduction script](http-content-copy-negative-control.py.gz) runs from the
repository root with a fresh output directory; the report retains commands and
source hashes. No production API, website content or runtime/compiler change is
required, and no website build is needed.
