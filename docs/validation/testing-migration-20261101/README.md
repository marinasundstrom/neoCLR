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
- HTTP custom-method and media-type guest assertions now use named framework cases, while their
  independent wire peers remain in Python. Compiler visibility rejection and
  terminal-failure subprocess probes also retain legitimate host orchestration.
  Do not mechanically convert these into in-process assertions.
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


## HTTP header allocations

The [header-allocation fixture](../../../runtime/System.Web/test/HttpHeaderAllocationTests/README.md)
already used descriptive `[Test]` functions and NeoClr.Testing assertions. Its custom
Main still panicked on Failure and printed a literal three-pass summary. It now
runs the cases in the same order, explicitly awaits the parsed-header HTTP lifecycle,
retains completed results in bound reference adapters and uses TestSuite,
ConsoleReporter and report.ExitCode. Recoverable failures now contribute named
results while later independent cases run; terminal faults remain terminal.
Automatic async discovery remains unsupported.

Tests.rvn is unchanged. Adapter construction, registration and reporting are outside
all measured allocation regions. Warmup, loop counts, cached-view identity checks,
parsed request/response checks, zero-allocation lookups/configuration and exactly
one materialized header on positional access remain intact. This is reporting work,
not an allocation optimization or a new .NET divergence; the existing
[header API comparison](../../../api-docs/http-headers.md#net-comparison) applies.

The first run used `target/http-message-surfaces/inputs9/bundle`, which predates the
response-header retention fix. [Its report](http-header-allocations-old-bundle.json.gz)
correctly records two passes and the final case failing with 512 allocations.
That is the same known negative baseline documented in the
[retention qualification](../http-header-noop-20261031/README.md), not a reporting
regression. The first wrong-expectation experiment also used that older bundle and
therefore produced two failures rather than the expected one;
[its failed report](http-header-allocations-initial-negative.json.gz) is retained.

Using the fixture's previously qualified `target/http-header-noop/inputs/bundle`,
[the final report](http-header-allocations-report.json.gz) and
[transcript](http-header-allocations.log.gz) pass all three tests on macOS ARM64.
The report fingerprints the sources and selected inputs; no compiler rebuild or
new release qualification is claimed. The existing runner explicitly rejects
native/JIT modes because their allocation counters are not bound. No execution-mode
coverage or budgets change.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/test-runtime-library.py \
  --bundle target/http-header-noop/inputs/bundle \
  --output target/http-header-allocations-testing-migration-20261101-matched \
  --suite http-header-allocations --mode interpreter
```

[The final isolated negative control](http-header-allocations-negative-control.json.gz)
changes the first case's expected allocation count from zero to one in a separate
Tests.rvn copy. The interpreter reports `Managed allocations: 0`, one named failure,
two passes and exit one, with empty stderr. The repository fixture is not mutated.
[The reproduction script](http-header-allocations-negative-control.py.gz) runs from
the repository root with a fresh output directory; commands and source hashes are
retained. Production APIs, Python orchestration, measurement contracts and website
content are unchanged; no website build is needed.


## CancelAfter

The [CancelAfter fixture](../../experiments/cancel-after/README.md) replaces its
local Check/Panic helper and handwritten PASS lines with four descriptive backtick
`[Test]` functions returning `Task<Result<unit, TestFailure>>`. Assertions now name
which behavior failed. Main explicitly awaits the cases in their original order,
retains completed results in reference adapters, runs TestSuite, prints the actual
ConsoleReporter summary and returns report.ExitCode. Recoverable assertion failures
permit later independent cases to run; terminal runtime failures remain terminal.
Automatic async discovery is not added.

The cases retain countdown reset from maximum duration to zero, deferred dispatch,
cancellation remaining sticky, disabling and reenabling, disposal (including repeated
disposal) suppressing pending cancellation, and manual cancellation notifying once.
Timer-backed Pause, callback registrations, durations and disposal order are preserved.
Callbacks have an intentional timer/cancellation observation role. There is no new
.NET divergence; the existing [API comparison](../../http-client-design.md#shared-native-deadline-checkpoint--2026-09-24)
and [design](../../design/http-client-lifetime.md) apply unchanged.

Faults.rvn and its project remain unchanged. Python continues to run negative and
excessive duration, cancelled-negative, disposed scheduling and disposed disabling
as separate terminal-failure processes. For this migrated suite it now checks the
exact four-test transcript and empty stderr, rather than counting PASS substrings,
and records framework, validator, runtime and codegen hashes alongside fixture and
bundle inputs. The shared timer wrapper now selects `--execution interpreter`
explicitly: [the first run](cancel-after-initial-selector-failure.json.gz) exposed
its stale reliance on the former execution default, rejecting instruction limits
in JIT before guest code ran. Other timer-suite assertions are unchanged.

[The final report](cancel-after-report.json.gz) and [log](cancel-after.log.gz) pass
four behavior tests in each of interpreter and native AOT, plus five fault probes
in each mode (eight behavior executions and ten expected-failure executions) on
macOS ARM64. This preserves the validator's existing two-mode scope; JIT coverage
is not claimed. The matching `target/http-header-noop/inputs/bundle` is identified
by hashes, not claimed as a fresh compiler build or release qualification. Process
and instruction limits are unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-timers.py --suite cancel-after \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/cancel-after-testing-migration-20261101-explicit
```

[An isolated negative control](cancel-after-negative-control.json.gz) expects two
manual-cancellation notifications instead of one in a separate Tests.rvn copy.
Interpreter reports the named exactly-once assertion failure, three passes, one
failure and exit code one with empty stderr. The repository fixture is not mutated.
[Reproduction script](cancel-after-negative-control.py.gz) retains commands and
source hashes and runs from the repository root with a fresh output directory.
No production API or website content changes; no website build is needed.

After validation, one trailing blank line was removed from CancelAfter Tests.rvn
to satisfy the staged whitespace check; the reports retain the tested source hash.
No executable text changed.


## Timers

The [Timer fixture](../../experiments/timers/README.md) replaces generic Check/Panic
assertions and handwritten PASS lines with six named NeoClr.Testing cases. The
original ten checkpoints are retained: the five synchronous phases remain combined
because they share a timer's one-shot/stop/dispose history and a repeat/reschedule
sequence. The other five cases preserve async repeating, restart, active-callback
disposal, already-cancelled callback and pending cancellation behavior.

Main explicitly awaits the six lifecycles in their original order and registers
completed-result reference adapters with TestSuite. ConsoleReporter and report.ExitCode
replace manual success reporting. Recoverable assertion failures allow later
independent cases to run; terminal failures remain terminal. Async discovery is
not added. Existing durations, pauses, promise barriers and explicit disposal order
are retained. Timer callbacks intentionally exercise notification/serialization;
there is no new scheduler or .NET API divergence. The existing
[timer design and comparison](../../design/http-client-lifetime.md) apply.

The three callback helper types now keep sticky validity flags. Repeating and
rescheduling callbacks record excessive invocation counts; async callbacks record
any overlap at callback entry. Tests assert those flags after their existing
completion barriers, including every async lifecycle that uses the helper. A
callback violation therefore becomes a named framework failure rather than a generic
Panic. Invariants are not inferred solely from final counts.

Python retains native building, instruction/process budgets and five separate
terminal-failure probes (negative/excessive duration, zero/negative interval and
starting a disposed timer). Faults.rvn and its project are unchanged. The behavior
run now requires the exact six-test transcript and empty stderr, and the report
includes framework, validator, runtime and codegen hashes as in CancelAfter.

[The report](timers-report.json.gz) and [log](timers.log.gz) pass all six tests in
interpreter and native AOT plus all five guards in each mode: twelve behavior-test
executions and ten expected-failure processes on macOS ARM64. The existing two-mode
scope remains; JIT coverage is not claimed. Inputs use the fingerprinted
`target/http-header-noop/inputs/bundle`, without claiming a fresh compiler build
or a new release qualification. Historical validation.json in the experiment is
unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-timers.py --suite timers \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/timers-testing-migration-20261101
```

[The isolated negative control](timers-negative-control.json.gz) changes the repeating
callback's accepted count from `Count <= 3` to `Count < 3` in a copied Tests.rvn.
The third delivery records a violation, then the combined synchronous test reports
`No callback exceeds three deliveries`; all five async cases pass. Interpreter exits
one with one named failure and empty stderr. This directly checks callback-to-test
failure propagation rather than just a failure in Main. The repository fixture is
not mutated. [Reproduction script](timers-negative-control.py.gz) retains commands
and source hashes; run from the repository root with a fresh output directory.
No production API or website content changes; no website build is needed.


## HTTP timeout

The [HTTP-timeout fixture](../../experiments/http-timeout/README.md) replaces generic
Check/Panic behavior assertions and handwritten PASS lines with nine named
NeoClr.Testing cases plus the separately invoked long-request case. The three
cancellation races now have distinct backtick names: caller cancellation, client
disposal and caller cancellation after a timeout request. Their shared parameterized
helper still executes the original ordering and returns a framework Result.

Assertions preserve the default timeout, handler acknowledgement before settlement,
sticky caller/disposal cancellation, disabled timeout completion, successful
completion removing its countdown, independent concurrent requests, 96 completed
sends releasing timer capacity, and a successful handler result after a cancellation
request. Async ExpectSuccess/ExpectTimeout helpers now propagate TestFailure rather
than Panic. Invalid fixed fixture-URL construction remains a terminal setup check.
Cancellation observation deliberately uses OnCompleted to avoid cancelling the test.

Main explicitly awaits the same lifecycle order and registers completed-result
reference adapters with TestSuite. ConsoleReporter writes actual outcomes and Main
returns report.ExitCode. Recoverable failures allow later independent cases to run;
terminal faults remain terminal. Async discovery is not added. The one-argument
invocation still runs only the real sixteen-second request under the default timeout.
Python now requires exact framework transcripts and empty stderr for both ordinary
and long-request runs. Framework/validator/runtime/codegen hashes are recorded.

Faults.rvn, its project and the six invalid-operation subprocess checks remain
unchanged: zero/negative/excessive timeout, changes after disposal, after a send
starts and after a pre-cancelled send. Independent Network.rvn/host-peer checks and
historical reports are unchanged. The existing
[HTTP lifetime/timeout design and .NET comparison](../../design/http-client-lifetime.md)
still apply; no production timeout, cancellation, API or execution-mode change is
introduced.

[The final report](http-timeout-report.json.gz) and [log](http-timeout.log.gz) pass
nine ordinary tests in interpreter and native AOT, the single long-request test in
AOT, and six fault probes in each mode: nineteen behavior-test executions and twelve
expected-failure processes on macOS ARM64. The long request retains its sixteen-second
timer and thirty-second process budget, continuing to guard the former fifteen-second
native host limit. Other process/instruction budgets are unchanged. JIT coverage is
not claimed. The matching `target/http-header-noop/inputs/bundle` is fingerprinted;
this does not claim a fresh compiler build or new release qualification.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-timers.py --suite http-timeout \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/http-timeout-testing-migration-20261101
```

[The isolated negative control](http-timeout-negative-control.json.gz) changes the
expected default timeout from 1,000,000,000 to 1,000,000,001 ticks in a copied Tests.rvn.
Interpreter reports the named default-timeout assertion, eight passes, one failure
and exit one with empty stderr. The repository fixture is not mutated.
[Reproduction script](http-timeout-negative-control.py.gz) records commands and source
hashes; run from the repository root with a fresh output directory. No website
content changes or website build is needed.


## Custom HTTP methods

The [custom-method fixture](../../experiments/http-method/README.md) moves client,
CONNECT rejection and server behavior checks into three named NeoClr.Testing cases.
The client checks request construction, successful response, status and length;
CONNECT still must report HttpError.Unsupported. The server handler records its
method-spelling and length assertions in a reference owner, echoes the response,
and exposes the saved Result after ServeOne completes and the listener closes.
This lets a handler assertion produce a named failure instead of terminating with
Panic. Listener/port setup failures remain terminal.

A shared completed-result adapter uses TestSuite, ConsoleReporter and report.ExitCode.
Main explicitly awaits the selected lifecycle; no automatic async/parameterized
discovery is claimed. Python still supplies the URL, consumes the server's initial
port line, and independently validates request spelling `m-SEARCH`, Content-Length
and body bytes, plus response X-Method and body bytes. Those wire checks are not
replaced by in-process assertions. The three invalid-constructor strings remain
terminal-failure subprocess probes. Successful guest runs now require exact named
framework output and empty stderr. Interpreter selection is explicit, preserving
its instruction budget under the runtime's JIT default. Native/profile and process
limits are unchanged.

[The report](http-method-report.json.gz) and [log](http-method.log.gz) pass all three
behavior cases in interpreter and native AOT, with six expected constructor failures
and independent client/server wire checks. This is macOS ARM64 qualification using
the fingerprinted `target/http-header-noop/inputs/bundle`; no fresh compiler build,
JIT/Windows run or new release qualification is claimed. Existing
[HTTP method design and .NET comparison](../../http-client-design.md#httpmethod-implementation--development-2026-10-11)
remain applicable. Historical validation.json and method-value suite evidence are
unchanged; production HTTP APIs and behavior are unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-http-method.py \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/http-method-testing-migration-20261101-distinct
```

[The isolated negative control](http-method-negative-control.json.gz) changes only
the server handler's expected spelling to `M-SEARCH` in a copied Server.rvn. The
peer sends `m-SEARCH` and independently validates the echoed X-Method/body. Interpreter
then reports `Expected 'M-SEARCH', actual 'm-SEARCH'`, one named failure and exit one
with empty stderr. This proves callback failures survive the adapter even when wire
checks pass. [Reproduction script](http-method-negative-control.py.gz) records commands
and source hashes; run from the repository root with a fresh output directory.
The repository fixture is not mutated.

[Initial compilation](http-method-initial-compile.log.gz) rejected repeated nested
and outer `result` locals with RAV0174; distinct outcome names compile and qualify
without a compiler change. The [first negative receipt](http-method-initial-negative.json.gz)
correctly failed the guest but rejected its transcript because the control omitted
quotes around string values. The final control corrects that expected formatting.
No website content changes or website build is needed.


## Media-type wire fixtures

The [media-type wire fixture](../../experiments/media-types/README.md) now reports
three named NeoClr.Testing cases: invalid content rejected before DNS, client response
type/length and server quoted-boundary case/value. The invalid-content test still
runs once per input in a fresh process. Assertions preserve request construction,
InvalidRequest classification, successful response, parsed problem+json identity,
two-byte response length and the exact `AbC; 123` boundary. The server handler stores
its assertion Result and reports it after ServeOne settles, so a request mismatch
becomes a named failure while the host can finish its independent response checks.

Main explicitly awaits the selected lifecycle and uses the shared completed-result
reference adapter with TestSuite, ConsoleReporter and report.ExitCode. Automatic
async/parameterized discovery is not added. Listener/port setup failures and the
conflicting-charset probe remain terminal. No production API, media-type parsing
semantics or .NET divergence is introduced; the existing
[structured-media-type comparison](../../http-client-design.md#structured-media-types--development-2026-10-23)
continues to apply.

Python retains all seven invalid inputs (malformed type, duplicate parameter names,
wildcard type, invalid spacing, unterminated quote, embedded CR/LF and q parameter),
the charset-conflict process and independent peers. Request checks still cover POST,
Content-Length, the quoted `profile="A; B"` and `{}` body; response checks retain
problem+json and exact body bytes. The initial server port line remains host protocol.
Successful runs now require exact framework transcripts, exit zero and empty stderr.
Framework/validator/runtime/codegen hashes accompany the fixture/bundle identities.
Mode selection and process/instruction budgets are unchanged.

[The report](media-types-report.json.gz) and [log](media-types.log.gz) pass all twenty
process cases on macOS ARM64: seven invalid-content variants plus client/server
behavior in each of interpreter and native AOT (eighteen framework executions), and
the charset fault in each mode. Independent wire checks pass in both modes.
The matching `target/http-header-noop/inputs/bundle` is fingerprinted; no new compiler
build, JIT/Windows run or release qualification is claimed. Existing contract and
allocation suites and historical reports remain unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-media-types.py \
  --bundle target/http-header-noop/inputs/bundle \
  --output target/media-types-testing-migration-20261101
```

[The isolated negative control](media-types-negative-control.json.gz) changes only
the server's expected boundary to `abc; 123` in a copied Server.rvn. The peer still
sends `AbC; 123` and independently validates the response content type/body.
Interpreter reports `Expected 'abc; 123', actual 'AbC; 123'`, one named failure and
exit one with empty stderr. This verifies callback-to-framework failure propagation
independently of wire correctness. The repository fixture is not mutated.
[Reproduction script](media-types-negative-control.py.gz) retains commands and source
hashes; run from the repository root with a fresh output directory. No website
content changes or website build is needed.


## Filesystem watcher

The [filesystem-watcher fixture](../../../runtime/System.Runtime/test/FileSystemWatcherTests/README.md)
already used three named NeoClr.Testing functions and assertions. This slice removes
its handwritten Panic/PASS reporter and moves Main into its own file. Main retains
the same order, explicitly awaits the physical lifecycle and registers completed
Results through reference adapters with TestSuite. ConsoleReporter prints actual
outcomes and Main returns report.ExitCode. Recoverable assertion failures allow
remaining independent cases to run; terminal faults remain terminal. Automatic
async/parameterized discovery remains unsupported.

The three assertion bodies are unchanged: disabled defaults and permanent disposal,
invalid/missing directory errors, and physical notification filtering across create,
atomic replacement, stop, restart and disposal. Subscription removal, descendant
paths, callback payloads, two-second settling and existing RescanRequired acceptance
remain intact. This adds no OS rename guarantee, provider semantics or new .NET
divergence; the existing [watcher design](../../file-system-watcher.md) applies.

Python retains the READY-triggered file writes, OBS diagnostics, isolated directories
and unchanged native invocation-ownership/teardown harness. It now compares the
exact framework transcript after excluding READY/OBS protocol lines and still
requires the original phase list. Guest stderr must be empty; only the exact
existing DAM2004 JIT compiler warning is allowed and recorded separately. JIT now
uses the explicit matching module list, helper and compiler, instead of relying on
ambient defaults. The instruction/process budgets and three-mode scope are unchanged.

[The report](filewatch-report.json.gz) and [transcript](filewatch.log.gz) pass three
cases in interpreter, JIT and AOT on macOS ARM64 (nine framework executions), plus
the native ownership/teardown harness. Both existing validator harness-command unit
tests pass via `python3 -m unittest discover -s scripts -p test_filewatch_validator.py`.
The matching `target/http-header-noop/inputs/bundle` and selected binaries are
fingerprinted. This does not claim a fresh compiler build, Windows run or expanded
provider qualification; historical watcher evidence remains unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-filewatch.py \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/filewatch-testing-migration-20261101
```

[The isolated negative control](filewatch-negative-control.json.gz) requires the
initially disabled watcher to be enabled in a copied Tests.rvn. Interpreter reports
`Starts disabled` as one named failure, then passes setup errors and the complete
physical lifecycle, yielding two passes, one failure and exit one with empty stderr.
[The reproduction script](filewatch-negative-control.py.gz) reuses the real filesystem
driver in an isolated in-memory copy, changing only its expected failure transcript
and exit status. The repository fixture and validator are not mutated; all five
READY-triggered write phases still execute and are checked. Run from the repository
root with a fresh output directory. No production API or website content changes;
no website build is needed.


## Configuration reload

The [configuration-reload fixture](../../../extensions/Neo.Configuration/test/Neo.Configuration.Reload.Tests/README.md)
already used a named NeoClr.Testing case. This slice replaces its handwritten
Panic/PASS entry point with a separate Main that explicitly awaits the complete
lifecycle, registers a completed-result reference adapter with TestSuite, prints
ConsoleReporter output and returns report.ExitCode. The assertion body is unchanged;
a recoverable failure now produces a named framework result. Setup and terminal
runtime failures remain terminal. Async/parameterized discovery is not added.

All seven READY phases remain in order: changed, malformed, replaced, deleted,
optional-created, optional-deleted and disposed. Python still performs real file
writes, atomic replacement and deletion in isolated directories. The test retains
live DI/section reads, non-reloading configuration staying fixed, malformed/required
missing files retaining valid data with observable errors, successful reload clearing
errors, optional-file creation/deletion, repeated disposal and rejected loads after
disposal. Provider setup, two-second settling intervals and disposal order are
unchanged. Existing [configuration semantics and .NET comparison](../../configuration.md)
apply; this adds no production API or provider contract.

The validator checks exact framework output after excluding READY protocol lines,
requires the original phase list and empty guest stderr, and separates only the
exact existing DAM2004 JIT compiler warning. JIT modules/helper/codegen are explicit
matching inputs rather than ambient defaults. Process/instruction budgets and the
three-mode scope are unchanged. Framework, validator and selected binary/bundle
hashes are recorded.

[The report](configuration-reload-report.json.gz) and
[transcript](configuration-reload.log.gz) pass the full lifecycle in interpreter,
JIT and AOT on macOS ARM64 using `target/http-header-noop/inputs/bundle`. This is
three framework executions, each completing all seven physical phases. It does not
claim a fresh compiler build, Windows execution or broader filesystem qualification.
Historical configuration-reload evidence remains unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-configuration-reload.py \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/configuration-reload-testing-migration-20261101
```

[The isolated negative control](configuration-reload-negative-control.json.gz)
requires the final disposed-provider TryLoad to succeed in a copied ReloadTests.rvn.
Interpreter completes all seven phases, then reports `Disposed provider cannot load`
as one named failure and exits one with empty stderr. The
[reproduction script](configuration-reload-negative-control.py.gz) reuses the real
filesystem driver in an isolated in-memory copy, changing only its expected failure
transcript and exit code. Repository fixture and validator files are not mutated.
Run from the repository root with a fresh output directory. No website content
changes or website build is needed.


## Real HTTP timeout sockets

The separate [network timeout fixture](../../experiments/http-timeout/README.md#real-socket-timeout-fixture)
now contains two named NeoClr.Testing cases: Network timeout reports TimedOut, and
Disabled timeout permits a delayed network response. These replace conditional
Panic assertions and handwritten PASS lines. NetworkMain explicitly awaits the
selected URL-based case, registers a completed-result reference adapter with
TestSuite, writes ConsoleReporter output and returns report.ExitCode. Automatic
async/parameterized discovery is not added. Client disposal remains before the
outcome/status assertions; public timeout semantics and the existing
[.NET comparison](../../design/http-client-lifetime.md) are unchanged.

Python preserves the independent loopback peer: it reads complete request headers,
withholds a response on timeout or waits 350 ms before sending HTTP 200 and `{}`,
then requires EOF from the client. The two-second guest timeout, timeout disabling,
peer limits, process timeout and interpreter/AOT scope are unchanged. Interpreter
selection is now explicit rather than depending on the runtime's former default.
Success requires the exact named framework transcript and empty stderr as well as
peer-confirmed socket closure; source/framework/validator/runtime/codegen hashes are
recorded alongside the bundle identity. Socket closure is checked independently;
this migration does not add finer-grained acknowledgement timing instrumentation.

[The report](http-timeout-network-report.json.gz) and
[log](http-timeout-network.log.gz) pass both tests in interpreter and native AOT:
four framework executions and four peer-confirmed socket closures on macOS ARM64.
Inputs use the matching `target/http-header-noop/inputs/bundle`; no fresh compiler
build, JIT/Windows run or release qualification is claimed. The controlled-handler
timeout suite and historical network-validation.json remain unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-http-timeout-network.py \
  --bundle target/http-header-noop/inputs/bundle \
  --runtime target/http-header-noop/inputs/bundle/bin/neoclr \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/http-timeout-network-testing-migration-20261101
```

[The isolated negative control](http-timeout-network-negative-control.json.gz)
expects status 201 instead of 200 in a copied Network.rvn. The independent peer
still delays its HTTP 200 response and confirms socket closure. Interpreter reports
`Expected 201, actual 200`, one named failure and exit one with empty stderr. The
repository fixture is not mutated. [Reproduction script](http-timeout-network-negative-control.py.gz)
records commands and source hashes; run from the repository root with a fresh output
directory. No production API or website content changes; no website build is needed.

## Incoming HTTP content copy

The [wire probe](../../experiments/http-content-copy/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle, Incoming HTTP body copy preserves
framing bytes and single consumption. It retains one shared client and all five
ordered fixed-length, chunked, EOF-delimited, truncated and malformed exchanges.
Tests.rvn replaces generic Panic checks with phase-specific assertions; Main awaits
the outcome explicitly, adapts it through a reference owner and reports TestSuite's
actual result and exit code. No async/parameterized discovery is added.

Python still withholds each body until HEADERS, supplies independent framing bytes,
checks peer EOF and waits for CASE. It now also requires the exact named framework
transcript, exit zero and empty stderr. The 2049 per-byte reads, destination sentinels,
rewind/length checks, invalid-framing errors and replay rejection remain intact.
Production contracts and the existing .NET comparison are unchanged.

[Report](http-body-copy-report.json.gz) and [log](http-body-copy.log.gz) record
one passing lifecycle in each of interpreter and native AOT: ten peer exchanges
on macOS ARM64. Source/framework/build fingerprints and the bundle manifest hash
identify the reused matching `target/http-header-noop/inputs/bundle`. Existing
instruction/process limits are retained; no JIT, Windows, TLS, performance or release
qualification is added. Historical evidence is unchanged.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile http \
  --project docs/experiments/http-content-copy/Probe.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/http-body-copy-testing-migration-20261101
python3 docs/experiments/http-content-copy/validate.py \
  --bundle target/http-header-noop/inputs/bundle \
  --build target/http-body-copy-testing-migration-20261101 \
  --output target/http-body-copy-testing-migration-20261101/validation.json
```

[Negative control](http-body-copy-negative-control.json.gz) changes expected length
2049 to 2050 in an isolated Tests.rvn copy. Interpreter reports the named body-length
failure and exit one with empty stderr after the first delivered body, without CASE
or DONE; the peer observes EOF. Remaining exchanges intentionally do not run after
that failure. The repository fixture is unchanged. The
[reproduction script](http-body-copy-negative-control.py.gz) records commands and
source hashes; run it from the repository root with a fresh output directory.
No public API or website content changes are required; no website build was run.

## HTTP response metadata

The [reason-phrase fixture](../../experiments/http-message-metadata/README.md#framework-migration--2026-11-01)
now reports one named NeoClr.Testing lifecycle, HTTP response metadata preserves
status version and reason phrases. Main explicitly awaits the outcome, adapts it
through a reference owner, runs TestSuite and returns its report exit code. This
adds no async/parameterized discovery. Assertions preserve status-based success,
HTTP/1.1, repeated phrase access, exact tabs/trailing spaces, empty/OK/custom phrases,
informational/final responses, error status, disposal and delayed streaming content.

The first build exposed a stale `HttpClientResponse.Respond` call, removed by the
client/server type split. The fixture now constructs separate 200 and 201 responses
for default phrase checks. It does not claim client status mutation coverage or
change production semantics. [The rejected build](http-reason-stale-build.log.gz)
is retained; the seven incoming-response scenarios are unchanged. Existing HTTP
metadata design/.NET comparisons remain applicable without a new divergence.

Python retains its independent raw HTTP peer and withholds the final body until
HEADERS_READY. It requires the exact framework transcript and empty guest stderr,
allowing only the exact shared DAM2004 compiler diagnostic for JIT. JIT execution,
module context, compiler and helper now explicitly come from the same bundle.
[Report](http-reason-report.json.gz) and [log](http-reason.log.gz) record one passing
lifecycle in each of AOT, JIT and interpreter: 21 response exchanges on macOS ARM64.
Source/project/framework, executable/image and manifest hashes identify the inputs.
The matching existing `target/http-header-noop/inputs/bundle` is reused; process
limits and historical evidence remain unchanged. No Windows/TLS/performance or
release qualification is added.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile http \
  --project docs/experiments/http-message-metadata/ReasonPhrase.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/http-reason-testing-migration-fixed-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/http-message-metadata/validate_reason.py \
  --bundle target/http-header-noop/inputs/bundle \
  --build target/http-reason-testing-migration-fixed-20261101 \
  --output target/http-reason-testing-migration-fixed-20261101/validation.json
```

[Negative control](http-reason-negative-control.json.gz) changes the final streamed
body expectation from x to y in an isolated source copy. The same seven-response
peer completes, including the headers-before-body handshake. Interpreter reports
`Expected 'y', actual 'x'`, one named failure and exit one with empty stderr.
[Reproduction script](http-reason-negative-control.py.gz) reuses the validator peer
with exact failure transcript/exit expectations; run from the repository root with
a fresh output directory. An initial control harness omitted the quotes in its
expected failure message; [that rejected run](http-reason-negative-harness.log.gz)
is retained separately. The repository fixture was not mutated. No public API or
website content changes are required; no website build was run.

## String line-ending behavior

The [string-lines fixture](../../experiments/string-lines/README.md#framework-migration--2026-11-01)
now uses three descriptive NeoClr.Testing cases with explicit TestSuite registration:
recognized line endings normalize without rescanning replacements; other text bytes
survive replacement; unchanged line endings retain the original string. Main reports
actual outcomes through ConsoleReporter and returns report.ExitCode. No automatic
discovery coverage is claimed.

All original assertions remain: CRLF/CR/LF/FF/NEL/LS/PS, adjacent sequences, default
LF and custom replacements, deletion and empty input, replacements containing
newlines, composed/decomposed text, emoji, retained VT/NUL/NBSP and reference identity
for plain text, LF and matching CRLF. Existing
[.NET comparison and design](../../design/string-api-review-20261019.md#replacelineendings-implementation-2026-10-19)
remain applicable, including the deliberate platform-wide LF default. No API or
runtime behavior changes are introduced.

[Report](string-lines-report.json.gz) and [log](string-lines.log.gz) pass all three
tests in interpreter, JIT and native AOT (nine test executions) on macOS ARM64.
Python requires exact framework stdout, exit zero and empty stderr, permitting only
the exact shared DAM2004 compiler warning for JIT. Source/project/framework,
application and manifest fingerprints identify the reused matching
`target/http-header-noop/inputs/bundle`. Existing process/instruction budgets are
unchanged. No Windows, fresh compiler build or release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/string-lines/Main.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/string-lines-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/string-lines/check.py \
  target/http-header-noop/inputs/bundle target/string-lines-testing-20261101 \
  --output target/string-lines-testing-20261101/validation.json
```

[Negative control](string-lines-negative-control.json.gz) changes expected deleted
text from abcdefgh to wrong in an isolated source copy. Interpreter reports one named
failure, continues the two independent cases and exits one with empty stderr.
[Reproduction script](string-lines-negative-control.py.gz) records commands and
source hashes; run from the repository root with a fresh output directory.
The native kernel boundary checks and allocation diagnostic remain unchanged and
are not rerun: no primitive implementation or measured region changed. Historical
reports are preserved. Website content requires no change and no build was run.

## Integer-formatting behavior

The [integer-format consumer](../../experiments/integer-formats/README.md#framework-migration--2026-11-01)
now uses four descriptive NeoClr.Testing cases with explicit TestSuite registration:
culture/precision formats, zero precision boundaries, signed extremes and
boxed/composite dispatch across collection. Main writes ConsoleReporter output and
returns report.ExitCode. No automatic discovery coverage is claimed.

All original assertions remain: default/lowercase D/G/F/N formats, all five cultures,
Swedish NBSP grouping, precision zero and 99, Int32 minimum and Int64 extremes,
Formattable and composite formatting, current-culture overload and post-GC use of
the same boxed owner. Existing
[.NET comparison](../../experiments/integer-formats/README.md#contract-and-implementation-comparison)
remains applicable; this test migration changes no API or formatting semantics.

[Report](integer-formats-report.json.gz) and [log](integer-formats.log.gz) pass all
four tests in interpreter, JIT and native AOT (12 test executions) on macOS ARM64.
Python requires exact framework output, exit zero and empty stderr, allowing only
the exact shared DAM2004 compiler warning for JIT. Source/project/framework and
compiler/runtime/library/application fingerprints accompany the manifest hash.
The matching existing `target/http-header-noop/inputs/bundle` is reused; process and
instruction budgets remain unchanged. No Windows, fresh compiler build or release
qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/integer-formats/IntegerFormats.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/integer-formats-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/integer-formats/check.py \
  target/http-header-noop/inputs/bundle target/integer-formats-testing-20261101 \
  --output target/integer-formats-testing-20261101/validation.json
```

The separate Invalid.rvn/Invalid.rvnproj remain unchanged. Build that project with
the same console command into `target/integer-formats-fault-testing-20261101`, then
run check.py against it with `--fault G3`, `--fault D-1`, `--fault Q` or `--fault N100`
and a distinct output path. All 12 subprocess guards pass: [general](integer-formats-fault-general.json.gz),
[malformed](integer-formats-fault-malformed.json.gz), [unknown](integer-formats-fault-unknown.json.gz),
[precision](integer-formats-fault-precision.json.gz), [log](integer-formats-faults.log.gz).
They retain nonzero exit, empty stdout and the existing unsupported-integer diagnostic
check; terminal errors are not converted into recoverable framework assertions.

[Negative control](integer-formats-negative-control.json.gz) changes expected F99
length from 101 to 102 in an isolated source copy. Interpreter reports the named
maximum-fixed-precision failure, continues the other three cases and exits one
with empty stderr. [Reproduction script](integer-formats-negative-control.py.gz)
records commands and source hashes; run from the repository root with a fresh
output directory. Historical evidence is preserved. Website content requires no
change and no website build was run.

## Composite-formatting behavior

The [composite-formatting consumer](../../experiments/composite-formatting/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle with explicit TestSuite registration.
ConsoleReporter replaces the handwritten success marker and Main returns its actual
exit code. The former async entry point had no awaits; the current synchronous
Console.Write contract permits a synchronous framework case. No automatic-discovery
or async behavior coverage is claimed.

The assertion helper preserves String.CompareOrdinal rather than using canonical
string equality. All original assertions and console writes remain: escaped braces,
repeated/reordered arguments, fixed/params overloads, nulls, grapheme alignment, embedded NUL, custom
Formattable class/struct dispatch and ToString fallback. The same boxed payload is
formatted after an unrelated allocation. The initial build exposed the obsolete
`Char` type name; `Character.FromString` now preserves the flag-grapheme check.
[The rejected build](composite-stale-build.log.gz) is retained. Existing
[.NET comparisons and limits](../../experiments/composite-formatting/README.md#comparison-and-limits)
remain applicable with the later culture-interface update; no production API or
formatting semantics change is introduced.

[Behavior report](composite-report.json.gz), [log](composite.log.gz) and
[fault report](composite-fault-report.json.gz) pass the lifecycle and all twelve
malformed-format probes in interpreter, JIT and native AOT: 39 executions
on macOS ARM64. Python retains the exact five console lines and requires the named
framework PASS and summary, exit zero and empty stderr, allowing only the exact shared
DAM2004 compiler warning for JIT. The terminal probes retain their existing diagnostic,
nonzero exit and partial-output checks. Source/project/framework/application and
manifest hashes identify the reused matching `target/http-header-noop/inputs/bundle`.
Instruction/process budgets remain unchanged; no Windows, fresh compiler build or
release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/composite-formatting/Main.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/composite-testing-ordinal-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/composite-formatting/Invalid.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/composite-fault-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/composite-formatting/check.py \
  target/http-header-noop/inputs/bundle target/composite-testing-ordinal-20261101 \
  --invalid target/composite-fault-testing-20261101 \
  --output target/composite-testing-ordinal-20261101/validation.json
```

[Negative control](composite-negative-control.json.gz) expects wrong instead of
retained:tag for the boxed payload in an isolated copy. Interpreter reports the named
failure and exits one with empty stderr; the later assertions and console writes
intentionally do not run after failure. [Reproduction script](composite-negative-control.py.gz)
records commands and source hashes; run from the repository root with a fresh output
directory. Fault receipts are extracted from the initial validation run, excluding
its behavior rows after the ordinal-helper correction; Invalid.rvn, its executable
and the validator fault branch were unchanged. Final behavior evidence uses the
rebuilt ordinal helper. Allocation diagnostics and historical reports remain
unchanged and are not rerun; no implementation or measured region changed. Website content requires
no change and no website build was run.

## Culture-aware formatting behavior

The [culture-formatting consumer](../../experiments/formattable-culture/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle with explicit TestSuite registration.
ConsoleReporter replaces the handwritten success marker and Main returns report.ExitCode.
No automatic discovery coverage is claimed. The custom CultureSnapshot formatter
records sticky reference-identity mismatches; an assertion immediately after the
same repeated-placeholder formatting call observes that result. It preserves the
original identity check without forcing TestFailure into Formattable's string result.

All original behavior checks remain: duration zero/general/constant formats,
Int64 tick extrema and minimum roundtrip, Object/Formattable/composite dispatch,
default culture identity, explicit culture propagation across fixed and params
arities, Gregorian roundtrip date, Swedish time precision, sortable/roundtrip local
values and boxed civil dispatch. Parse/date/time setup errors produce named framework
failures. Existing [.NET comparison and tradeoffs](../../experiments/formattable-culture/README.md#comparison-and-tradeoffs)
remain applicable; no production formatting semantics or public APIs change.

[Report](culture-report.json.gz) and [log](culture.log.gz) pass the lifecycle in
interpreter, JIT and native AOT on macOS ARM64. Python requires the exact framework
transcript, exit zero and empty stderr, allowing only the exact shared DAM2004 compiler
warning for JIT. Source/project/framework and application/compiler/runtime/library
hashes accompany the manifest fingerprint for the reused matching
`target/http-header-noop/inputs/bundle`. Existing instruction/process limits are
unchanged. No Windows, fresh compiler build or release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/formattable-culture/FormattableCulture.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/culture-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/formattable-culture/check.py \
  target/http-header-noop/inputs/bundle target/culture-testing-20261101 \
  --output target/culture-testing-20261101/validation.json
```

Invalid.rvn and its project remain unchanged. Build Invalid.rvnproj with the same
console command into `target/culture-fault-testing-20261101`, then run check.py against
that directory with `--fault duration`, `date`, `time` or `local` and distinct output
paths. All twelve terminal-fault checks pass: [duration](culture-fault-duration.json.gz),
[date](culture-fault-date.json.gz), [time](culture-fault-time.json.gz),
[local](culture-fault-local.json.gz), [log](culture-faults.log.gz). They retain the
existing nonzero exit, empty stdout and unsupported-format diagnostic checks.
Together with the behavior lifecycle this covers 15 executions across three modes.

[Negative control](culture-negative-control.json.gz) inverts the reference-identity
observation in an isolated source copy. Interpreter reports the named `one culture
snapshot` failure and exit one with empty stderr, confirming that the observation
reaches framework reporting. Later lifecycle checks intentionally do not run after
failure. [Reproduction script](culture-negative-control.py.gz) records commands and
source hashes; run from the repository root with a fresh output directory. Historical
evidence is preserved. The already migrated composite consumer is unaffected and
is not rerun. Website content requires no change and no website build was run.

## Synchronous text-wrapper behavior

The [sync-text fixture](../../experiments/sync-text/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle with explicit TestSuite registration.
ConsoleReporter replaces the handwritten success marker and Main returns its report
exit code. Setup failures and Text/Character/End helpers propagate TestFailure;
ordinal String.CompareOrdinal checks are preserved rather than using canonical
string equality. The first build exposed obsolete `char` helper signatures; these
now use `System.Character`. [Rejected build](sync-text-stale-build.log.gz) is retained.

The same lifecycle retains input chunks 1/2/7/256, decomposed and multi-scalar
graphemes, peek reuse, mixed character/count/line cursors, EOF and closed state,
zero/invalid counts without input consumption and byte-limit retry. It preserves
all six delimiters, interface properties, exact UTF-8 matching, overlapping prefixes,
changed/empty delimiters, partial EOF prefixes and CR preservation under LF.
Writer checks retain UTF-8 counts, newline configuration, Flush versus Finish,
repeated Finish, write-after-finish/close errors, leaveOpen and owned stream closure,
and ASCII encoding rejection. Existing [.NET comparison and tradeoffs](../../experiments/sync-text/README.md)
remain applicable; no public API or stream behavior changes are introduced.

[Report](sync-text-report.json.gz) and [log](sync-text.log.gz) pass the lifecycle in
interpreter, JIT and native AOT on macOS ARM64. Python retains process budgets, explicit
matching execution inputs and supplied stdin; the current fixture uses in-memory
streams, so this does not establish console-read coverage. Exact framework stdout,
exit zero and empty stderr are required, allowing only the exact shared DAM2004
compiler warning for JIT. Source/project/framework/application and manifest hashes
identify the reused matching `target/http-header-noop/inputs/bundle`. No automatic
discovery, Windows, fresh compiler build or release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/sync-text/Main.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/sync-text-testing-fixed-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/sync-text/check.py \
  target/http-header-noop/inputs/bundle target/sync-text-testing-fixed-20261101 \
  --output target/sync-text-testing-fixed-20261101/validation.json
```

[Negative control](sync-text-negative-control.json.gz) changes the expected composed
accent in the final memory roundtrip to its canonically equivalent decomposed form
in an isolated source copy. Interpreter reports the named Text bytes failure and
exit one with empty stderr; later ownership/ASCII checks do not run after failure.
[Reproduction script](sync-text-negative-control.py.gz) records commands and source
hashes; run from the repository root with a fresh output directory. Historical
reports and separate async/native-provider tests are unchanged and not rerun.
Website content requires no change and no website build was run.

## Synchronous stream-capability behavior

The [sync-streams fixture](../../experiments/sync-streams/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle. Main explicitly awaits its result,
retains it in a reference adapter and reports TestSuite's actual outcome through
ConsoleReporter and report.ExitCode. Tests.rvn replaces generic Panic checks with
recoverable count, write/flush and phase-specific assertions. No automatic async
discovery is claimed.

The original sequence remains: write a borrowed subwindow through SyncOutputStream,
flush, perform a zero-length borrowed read, seek, read asynchronously into a guarded
window, continue through SyncInputStream, verify EOF, flush asynchronously, seek to
the quota, reject a further write, close through the input view, reject output flush,
repeat close and read EOF from a synchronous-only provider. Borrowed slices remain
inside synchronous helper calls and never cross await. The existing
[.NET comparison and stream design](../../stream-design.md) remain applicable; no
provider contract, public API, runtime or compiler behavior changes are introduced.

[Report](sync-streams-report.json.gz) and [log](sync-streams.log.gz) pass the complete
lifecycle in interpreter, JIT and native AOT on macOS ARM64. Python keeps its existing
budgets and matching execution context, requiring exact framework stdout, exit zero
and empty stderr, permitting only the exact shared DAM2004 compiler warning for JIT.
Source/project/framework/application and manifest hashes identify the reused matching
`target/http-header-noop/inputs/bundle`. No Windows, fresh compiler build or release
qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/sync-streams/Main.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/sync-streams-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/sync-streams/check.py \
  target/http-header-noop/inputs/bundle target/sync-streams-testing-20261101 \
  --output target/sync-streams-testing-20261101/validation.json
```

[Negative control](sync-streams-negative-control.json.gz) changes the final
synchronous-only provider's expected EOF count from zero to one in an isolated copy.
All preceding shared-stream checks complete. Interpreter reports the named
`Expected 1, actual 0` failure and exits one with empty stderr.
[Reproduction script](sync-streams-negative-control.py.gz) records commands and source
hashes; run from the repository root with a fresh output directory. Historical
reports and separate compiler visibility/metadata and backend dispatch checks remain
unchanged and are not rerun. Website content requires no change and no website build
was run.

## Persistent-memory Object contracts

The [memory-object fixture](../../experiments/memory-object/README.md#framework-migration--2026-11-01)
now uses one named NeoClr.Testing lifecycle with explicit TestSuite registration.
The equality helper propagates framework failures; Main writes ConsoleReporter output
and returns report.ExitCode. No automatic discovery coverage is claimed.

The original order and assertions remain: mutable/readonly region equality and
matching hashes, wrong types, diagnostic display, distinct windows/owners, mutation
without hash changes, ownerless defaults, empty offsets, dictionary identity,
collection and exact unboxing. The same box retains the sole backing array after
Retain returns and GC runs. The existing [.NET comparison](../../experiments/memory-object/README.md)
remains applicable; no memory semantics, boxing rules, compiler or public APIs change.
Native coverage remains the existing byte-view profile; this adds no allocation or
throughput claim.

[Report](memory-object-report.json.gz) and [log](memory-object.log.gz) pass the full
lifecycle in interpreter, JIT and native AOT on macOS ARM64. Python keeps its matching
execution inputs and budgets, requiring exact framework stdout, exit zero and empty
stderr, allowing only the exact shared DAM2004 compiler warning for JIT. Source,
project, framework, application, compiler, runtime and library hashes accompany the
manifest fingerprint for the reused matching `target/http-header-noop/inputs/bundle`.
No Windows, fresh compiler build or release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/memory-object/MemoryObject.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/memory-object-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/memory-object/check.py \
  target/http-header-noop/inputs/bundle target/memory-object-testing-20261101 \
  --output target/memory-object-testing-20261101/validation.json
```

[Negative control](memory-object-negative-control.json.gz) changes the first expected
sole-owner byte from two to five in an isolated source copy. Interpreter reaches the
post-collection assertion, reports `Box retains sole owner across GC` and exits one
with empty stderr. The final nominal-unboxing assertion does not run after failure.
[Reproduction script](memory-object-negative-control.py.gz) records commands and source
hashes; run from the repository root with a fresh output directory. The unchanged
Borrowed.rvn compiler-rejection probe remains separate and is not rerun; historical
reports are preserved. Website content requires no change and no website build was run.

## Record-formatting behavior

The [record-formatting fixture](../../experiments/record-formatting/README.md#framework-migration--2026-11-01)
now uses three descriptive NeoClr.Testing cases with explicit TestSuite registration:
scalar/nested structure, quoted Unicode text and boxed synthesized display. The helper
returns failures from ordinal equality, preserving byte-sensitive expectations; Main
writes ConsoleReporter output and returns report.ExitCode. No automatic discovery
coverage is claimed. The obsolete `char` field annotation now uses `System.Character`.

All original assertions remain: positive and minimum Int32, nested records, null and
quoted strings, Boolean/Int64 display, escaped backslashes and quotes, composed and
decomposed character/text spelling and boxed record dispatch. Existing
[.NET comparison and implementation scope](../../experiments/record-formatting/README.md#contract-and-implementation)
remain applicable. This changes no compiler synthesis, public APIs or record semantics.

[Report](record-format-report.json.gz) and [log](record-format.log.gz) pass all three
tests in interpreter, JIT and native AOT (nine test executions) on macOS ARM64, plus
the existing CopyConstructorCollision compiler rejection with RAV0111. The unchanged
compiler probe remains separate from behavior assertions. Python keeps its matching
execution context and budgets, requiring exact framework stdout, exit zero and empty
stderr, allowing only the exact shared DAM2004 compiler warning for JIT. Source,
project, framework, application, compiler, runtime and library hashes accompany the
manifest fingerprint for the reused matching `target/http-header-noop/inputs/bundle`.
No Windows, fresh compiler build or release qualification is claimed.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/record-formatting/RecordFormatting.rvnproj \
  --bundle target/http-header-noop/inputs/bundle \
  --aot target/http-header-noop/inputs/bundle/bin/neoclr-codegen \
  --output target/record-format-testing-20261101
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 docs/experiments/record-formatting/check.py \
  target/http-header-noop/inputs/bundle target/record-format-testing-20261101 \
  --output target/record-format-testing-20261101/validation.json
```

[Negative control](record-format-negative-control.json.gz) changes only the expected
record Name from a decomposed accent to its canonically equivalent composed form in
an isolated copy. Interpreter reports the named Record display bytes failure, passes
the other two cases and exits one with empty stderr. This confirms that canonical
equality does not weaken the original ordinal assertion.
[Reproduction script](record-format-negative-control.py.gz) records commands and source
hashes; run from the repository root with a fresh output directory. Historical reports,
allocation diagnostics and other record consumers are unchanged and not rerun.
Website content requires no change and no website build was run.
