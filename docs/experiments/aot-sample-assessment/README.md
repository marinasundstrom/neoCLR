# Native sample coverage and release baseline — 2026-10-08

The author asks for executed sample comparisons to prioritize a **stable baseline**,
not universal parity across execution modes. The survey records current first blockers;
these are not all release requirements. The subsequent direction is to fix easy and
urgent native issues. Reflection through IL-free sidecar metadata is a future proposal.

## Results

| Stage | Result |
| --- | --- |
| Corpus | 98 Raven API samples + 5 native async controls + JSON mapping = 104 |
| Raven native emission | 68 emitted, 36 rejected |
| Interpreter of those same 68 artifacts | 63 exit 0; 3 intentional exit/fault controls; 2 introspection failures |
| AOT of those same 68 artifacts | 12 compile/link/run with matching success/output; 56 reject compilation |
| Linking/runtime after successful AOT | All 12 pass; no mismatch observed |
| Additional focused baseline | ResultList, CallbackList, QueuePump and QueuePumpFault pass interpreted, sanitized native and standalone |
| HTTP baseline | Four single-request cases × three modes pass; 32-request persistent consumer also passes |

The intentional interpreter outcomes are native-async-entry-int (23), pending (UserFault)
and cancelled (UserFault). They are not interpreter regressions. The two unexpected
interpreter failures are library-assembly-info and library-introspection-tour, both
inside introspection services with the current source-built library/seed combination.
Those deserve separate diagnosis; their AOT rejection is not the cause of the VM fault.

[Raw survey](survey.json), [focused baseline checks](baseline-controls.json),
[HTTP correctness/cleanup](../../../benchmarks/native-web/gc-index-validation.json).
Twelve matched programs are constructors, ordinary types, division, integer operations,
error values, Option, Option propagation, integer parsing, String slices, UTF-8, value
copies and Void. Matching output is differential evidence, not an independent oracle
for every operation. Focused baseline verifiers additionally assert expected outcomes.

## First native blockers, ranked by observed reach

| First AOT blocker | Samples | Interpretation |
| --- | ---: | --- |
| Primitive arrays | 39 | Int32 30, Void 5, Int64 3, Boolean 1; common to collections, Tasks/await and helpers |
| RuntimeTypeHandle | 7 | Introspection and reflection-driven JSON mapping; metadata/invocation design needed |
| Other storage/dispatch/services | 10 | Interface arrays, virtual dispatch, interface-bound callbacks, Char generics, Double, pointers, environment/files/path/value contracts |

Counts describe the **first rejection only**. Primitive arrays may expose further
callback/dispatch/service gaps; 39 is not a promise that one patch makes 39 samples work.
The 36 Raven emission failures are separate: immutable-capture restrictions, stale API
names/import assumptions and unsupported emitter shapes. Do not label all of them AOT
backend bugs or universal Raven/.NET failures without target controls. No broad sample
rewriting was used to inflate the result.

## Recommended release priorities

1. **Preserve a small acceptance baseline.** Constructors/value copies, Result/Option
   branches, UTF-8 console/text, callback ownership/queue faults and sequential HTTP
   should keep exact behavior, cleanup and controlled failures. Use the focused
   verifiers below as gates; the broad inventory remains informational. Publish the
   supported target/limits rather than claiming all library samples work natively.
2. **Complete the no-bridge compiler-target bootstrap.** This remains the author's
   explicit release gate independently of AOT breadth. Current commands still consume
   CLI Core.dll. The preparation script explicitly generates it; removing the core
   option is rejected by the driver. Substituting the native Runtime library reaches
   its image-size guard, not a qualified replacement. [Diagnostic probes](bootstrap-probes.json)
   record an initial duplicate-reference configuration error and its corrected attempt.
   Do not infer that raising that size guard alone removes the bridge. Compiler hosting
   on .NET remains a separate question from the target metadata contract.
3. **Next easy/high-reach AOT slice: primitive arrays.** Start bounded Int32/Boolean/
   integer/Void storage using explicit layouts and initialized-slot rules. Prove default
   versus reserved reads, bounds, full-width values, copies, GC retention and rejection
   of unsupported address semantics. Then rerun selected collections/async consumers.
   Reuse interpreter semantics; do not create an AOT-specific language API.
4. **Reassess Tasks/await after arrays.** Keep the passing fn<Void>/TaskQueue compatibility
   host as the baseline; interface-bound callbacks and result-bearing async samples need
   separate treatment. Runtime Scheduler/green threads are not prerequisites.
5. **Harden the HTTP teaser and qualification.** Keep malformed-input/fault cleanup,
   repeat-request evidence and matched measurements. Retain existing editor/Windows
   gates and eventually run equivalent .NET comparison apps; this survey does not
   qualify those. Address unsupported services only as chosen scenarios need them.

Defer general reflection/invocation, the sidecar ABI, broader dispatch, floating point,
raw native pointers, JIT and green threads unless a selected baseline scenario needs
them. This is a proposed sequencing, not an expanded release promise. Reflection-driven
JSON mapping currently introduces metadata dependencies that the plain HTTP sample
avoids; an explicit JSON handler can be assessed separately before committing to general
reflection support. Two interpreter introspection failures should be triaged even if
introspection is excluded from the first AOT baseline.

.NET remains the ergonomic comparator; reuse the [existing AOT research](../../native-execution-investigation.md)
and [shared-service review](../../runtime-scheduling-design.md). A bounded supported set
is legitimate, but current neoCLR restrictions should not be described as .NET AOT
restrictions or advantages. No external-platform benchmark ranking follows here.

## Reproduction and limitations

```sh
SDKROOT=/path/to/MacOSX.sdk python3 scripts/compare-native-samples.py \
  --compiler /path/to/rvnc.dll --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/native-bundle --output target/native-sample-survey-new
```

The output directory must be new; repeat `--case NAME` for focused follow-ups. Run
`benchmarks/native-web/verify_callbacks.py` with the same tool/bundle arguments and
`--case ResultList --case CallbackList --case QueuePump --case QueuePumpFault` for the
asserted callback baseline. `verify_server.py` drives the peer-required HTTP cases;
add `--persistent --rounds 3` for matched short HTTP measurements when timing is relevant.

The survey uses one artifact per case for both modes, private C adapters, macOS ARM64,
1 MiB native heap and current explicit service bindings. Input is AB followed by EOF;
file samples run in separate empty working directories. It records commands, sources,
tools, library hashes and diagnostics. It uses bounded execution and intentionally
returns a report even when samples fail. It is not CI acceptance by itself. Timings
in this survey include parallel-process contention and must not be used as benchmarks.
Other neoIL/Rust/HTTP integration suites are outside the 104 denominator. Existing
focused evidence covers more native operations than these older API samples exercise.

## First implemented follow-up: scalar arrays

[Bounded scalar-array support](../aot-console/scalar-arrays.md) now passes 77 native/
interpreter mode/type comparisons plus atomic-GC/allocation checks. Ten selected Raven
consumers were rebuilt and rerun; all still run successfully interpreted. `library-arrays`
now also runs natively with matching output. The other nine advance beyond the original
primitive-array rejection to explicit further boundaries:

| Consumer | Next AOT boundary |
| --- | --- |
| library-booleans | Boolean value-member owner |
| library-array-foreach | RuntimeTypeHandle metadata |
| library-array-unified | Jagged Int32 arrays |
| application-order-collections | Unsupported instruction in Main |
| library-generic-collections | Int64 value-member owner |
| library-async | DrainEntryTasks service |
| library-async-cancellation, library-task-result, native-async-state | Interface-bound callback binding (initially misclassified) |

[Raw follow-up](scalar-array-followup.json). This confirms why the original 39 first
blockers were not a 39-sample completion estimate. The initial matrix below remains
historical evidence; this follow-up supersedes its primitive-array implementation gap.
Next inspect primitive wrapper ownership as a potentially small fix; Tasks entry draining
and interface-bound callbacks remain distinct runtime/ABI work.

## Second implemented follow-up: primitive members and wide ordering

Boolean, Int64 and UInt64 ordinary nonvirtual wrappers now use the same verified
borrow-preserving projection as Int32. Wide ordering and comparison branches now admit
matching 64-bit operands; the existing code generator already implements the machine
comparisons. [Evidence](primitive-member-followup.json) records 56 signed/unsigned
comparison cases, mutation through four exact borrowed receiver types, and a Raven
consumer with hard-coded expected outputs in interpreted/sanitized/standalone modes.

`library-booleans` and `library-generic-collections` now pass both modes with matching
output. `library-calendar` advances to unsupported wide arithmetic in Time.get_Hour;
calendar services and broad numeric coverage are not claimed. Together with scalar
arrays, three previously blocked samples now pass. The original 104-case survey is not
rerun wholesale, and its historical counts above are not presented as a fresh census.
Next distinguish small arithmetic gaps from Tasks entry lifecycle/interface-bound
callback work; preserve the release's explicit bootstrap and baseline priorities.

## Third implemented follow-up: wide division and remainder

Signed/unsigned 64-bit division and remainder now use width-correct zero/overflow
checks before native operations. Seventy-two cases across Int32 and Int64 match the
interpreter, including minimum/-1, zero divisors, high-bit unsigned operands and the
valid Int64 case Int32.MinValue/-1. Fault outputs remain atomic and frames unwind.
`library-calendar` now passes both modes; time-contracts and time-zones advance to
other separately recorded blockers. [Evidence](division-followup.json).
This restores existing numeric semantics, not a new public API or performance claim.

## Per-sample matrix

`Emission blocked` means neither mode was run. Interpreter `expected 1/23` identifies
the three intentional outcome controls. AOT rejects are reported verbatim below; they
can precede other missing dependencies.

| Sample | Interpreter | Native | First blocker |
| --- | --- | --- | --- |
| application-constructor-arguments | exit 0 | matched success | — |
| application-delegates | not run | emission blocked | error NEOMETA001: Native emission does not support closure capture requires an immutable reference or supported primitive local or by-value parameter. |
| application-extensions | not run | emission blocked | error NEOMETA001: Native emission does not support closure capture requires an immutable reference or supported primitive local or by-value parameter. |
| application-inheritance | exit 0 | AOT rejected | virtual calls requiring dispatch need a later specialization profile |
| application-interfaces | exit 0 | AOT rejected | nominal native arrays require value records or reference classes |
| application-iterable | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| application-order-collections | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| application-orders | not run | emission blocked | error NEOMETA001: Native emission does not support unsupported pattern BoundConstantPattern from () to (). |
| application-types | exit 0 | matched success | — |
| json-object-mapping | exit 0 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-array-callbacks | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-array-foreach | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-array-queries | not run | emission blocked | error NEOMETA001: Native emission does not support closure capture requires an immutable reference or supported primitive local or by-value parameter. |
| library-array-shapes | not run | emission blocked | error RAV0103: 'Name' is not in scope. |
| library-array-tour | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-array-unified | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-arrays | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-assembly-info | exit 1 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-async | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Void) |
| library-async-cancellation | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-async-default-queue | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Void) |
| library-basics | not run | emission blocked | error NEOMETA001: Native emission does not support dependency method contract unavailable or ambiguous: static func Min(left: int, right: int) -> int (native type missing or ambiguous: System.Math). |
| library-booleans | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Boolean) |
| library-calendar | exit 0 | AOT rejected | $aot_linked_34: value member requires a local record owner (Int64) |
| library-case-payloads | not run | emission blocked | error RAV0103: 'Ok' is not in scope. |
| library-clamp | not run | emission blocked | error RAV0117: 'Math' has no member 'Clamp'. |
| library-clock | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-collection-aliases | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-collection-capabilities | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-comparers | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-console | not run | emission blocked | error RAV0103: 'IO.ConsoleReadError' is not in scope. |
| library-date-formatting | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-delegates | not run | emission blocked | error RAV0103: 'IsNone' is not in scope. |
| library-division | exit 0 | matched success | — |
| library-environment | exit 0 | AOT rejected | $aot_linked_7: unsupported value member contract |
| library-errors | exit 0 | matched success | — |
| library-files | exit 0 | AOT rejected | $aot_linked_10: unsupported value member contract |
| library-flags | exit 0 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-floating-math | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-foreach | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-generic-collections | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int64) |
| library-globalization | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-grapheme-strings | exit 0 | AOT rejected | unsupported closed generic argument: Char |
| library-instants | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-integers | exit 0 | matched success | — |
| library-interfaces | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-introspection-interfaces | exit 0 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-introspection-tour | exit 1 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-list-filters | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-managed-array-metadata | not run | emission blocked | error RAV0103: 'Name' is not in scope. |
| library-maps | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-match | not run | emission blocked | error RAV1503: Cannot convert from 'object' to 'int' |
| library-match-void | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-math | not run | emission blocked | error NEOMETA001: Native emission does not support dependency method contract unavailable or ambiguous: static func Min(left: int, right: int) -> int (native type missing or ambiguous: System.Math). |
| library-native-buffer | exit 0 | AOT rejected | specialization requires closed reference-free local value types: Ptr(Void) |
| library-nested-type-info | exit 0 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-numeric-operators | not run | emission blocked | error NEOMETA001: Native emission does not support value receiver requires addressable storage or a supported value result. |
| library-numeric-widening | exit 0 | AOT rejected | specialization requires closed reference-free local value types: Double |
| library-oftype | not run | emission blocked | error RAV1503: Cannot convert from 'TypeInfo' to 'MemberInfo' |
| library-option | exit 0 | matched success | — |
| library-option-propagation | exit 0 | matched success | — |
| library-outcome-operator-checks | not run | emission blocked | error RAV1503: Cannot convert from 'Option<U>' to 'Option<int>' |
| library-outcome-operators | not run | emission blocked | error RAV1503: Cannot convert from 'Result<U, string>' to 'Result<int, string>' |
| library-parsing | exit 0 | matched success | — |
| library-paths | exit 0 | AOT rejected | $aot_linked_2: unsupported value member contract |
| library-patterns | not run | emission blocked | error RAV1503: Cannot convert from 'object' to 'int' |
| library-primitives | not run | emission blocked | error RAV0117: 'IntPtr' has no member 'CompareTo'. |
| library-propagation | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-propagation-workflow | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-queries | not run | emission blocked | error NEOMETA001: Native emission does not support closure capture requires an immutable reference or supported primitive local or by-value parameter. |
| library-query-basics | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-query-names | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-query-terminals | not run | emission blocked | error RAV0103: 'Name' is not in scope. |
| library-reference-payloads | not run | emission blocked | error RAV0103: 'Name' is not in scope. |
| library-reflection | not run | emission blocked | error NEOMETA003: Native metadata encoding failed: .Main: local loaded before store on some path. |
| library-result | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-result-void-propagation | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| library-string-boundaries | not run | emission blocked | error NEOMETA001: Native emission does not support dependency method contract unavailable or ambiguous: static func Sign(value: int) -> int (native type missing or ambiguous: System.Math). |
| library-string-comparison | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-string-slices | exit 0 | matched success | — |
| library-strings | not run | emission blocked | error NEOMETA001: Native emission does not support dependency method contract unavailable or ambiguous: static func Sign(value: int) -> int (native type missing or ambiguous: System.Math). |
| library-task-composition | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-task-outcomes | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-task-producer | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-task-propagation | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Void) |
| library-task-result | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Void) |
| library-tasks | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| library-time-contracts | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int64) |
| library-time-zones | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int64) |
| library-type-acquisition | not run | emission blocked | error RAV1504: Cannot assign 'int' to 'ComparableTo<int>' |
| library-type-preview | exit 0 | AOT rejected | specialization requires closed reference-free local value types: RuntimeTypeHandle |
| library-union-contract | not run | emission blocked | error RAV0103: 'IUnion' is not in scope. |
| library-unions | not run | emission blocked | error RAV0103: 'IsOk' is not in scope. |
| library-utf8 | exit 0 | matched success | — |
| library-value-copy | exit 0 | matched success | — |
| library-value-interfaces | not run | emission blocked | error RAV1504: Cannot assign 'int' to 'ComparableTo<int>' |
| library-void | exit 0 | matched success | — |
| library-workers | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Void) |
| library-workflow | not run | emission blocked | error RAV0117: 'Math' has no member 'Abs'. |
| native-async-entry-cancelled | expected 1 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| native-async-entry-int | expected 23 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| native-async-entry-pending | expected 1 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |
| native-async-propagation | exit 0 | AOT rejected | native callback binding requires a static function or nonvirtual heap class receiver |
| native-async-state | exit 0 | AOT rejected | specialization requires closed reference-free local value types: ArrayRef(Int32) |

## Interface-bound callbacks and interface names (2026-10-08)

The earlier generic callback diagnostic was incorrectly described as a value-receiver
restriction for the async samples. Exact target inspection identifies the abstract
`AsyncStateMachine.MoveNext` interface method (previously `IAsyncStateMachine`).
The bounded native profile now binds closed class-interface thunks while retaining
the actual object. No borrowed receiver capture or state-machine copy is introduced.

The [four-case follow-up](interface-followup.json) recompiles against the renamed
native Runtime/Data/Networking/Web bundle. `native-async-state`,
`library-async-cancellation` and `library-task-result` match interpreter output and
exit status. That follow-up still rejected `library-async` at
`neoCLR.Runtime.DrainEntryTasks`; the subsequent queue-only entry slice is recorded below. These are sequential coverage checks,
not benchmark measurements. The original 104-case survey is historical and has not
been rerun wholesale. The CLI core bootstrap still exists.

The [interface protocol validation](interface-protocol-validation.json) records the
completed naming audit, matching reference refresh, legacy union/time/async consumers
and Raven compiler controls. `UnionValue` replaces the final handwritten `IUnion`
protocol. Regenerated legacy union outputs now clear on failed extraction, following
Raven's already-shared body contract. This corrects a stale bridge snapshot, not a
new native capability. The general source-lookup cache fix is independently tested
on Raven main as `d0a115dcf`.

## Queue-only async entry lifecycle (2026-10-08)

The native profile now implements exact `DrainEntryTasks` for queued work while the
startup frame remains rooted. It drains the source queue before the generated entry
wrapper reads the task result. Socket completion services combined with this entry
service remain explicitly rejected: the existing quiescent HTTP host pump cannot be
used as an in-guest wait. See the [private lifecycle contract](../aot-console/task-queue.md#queue-only-async-entry-drain-2026-10-08).

A new callback-fault consumer initially used `_ = await result.Task` and reached
Raven's `value block cannot exit its enclosing expression` native-emission diagnostic.
Changing that statement to a named awaited local followed by `WriteLine` emitted
successfully with the same compiler/bundle. This is a deferred Raven lowering/emission
candidate, not an AOT runtime failure; no fix or general .NET comparison is claimed.
The callback faults before the write, so the final consumer isolates runtime fault
propagation without relying on the unsupported discarded-await shape.

[Five-case executable evidence](../../../benchmarks/native-web/async-entry-validation.json)
records exact exit/output/fault parity for async success, callback fault, pending and
cancelled entries, plus the existing post-entry queue fault. All pass sanitized and
standalone native execution with libSystem-only linkage and cleanup checks. The HTTP
server still passes compiler admission; its request/throughput evidence is reused,
not rerun or remeasured by this slice. The original 104-case survey remains historical.
