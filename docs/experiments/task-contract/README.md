# Completion and continuation experiment

The newer [generic Task/TCS proof of concept](../../task-contracts.md) builds on
this initial fixed-payload experiment. Run `verify_tasks.py` for the library types;
`verify.py` below preserves the earlier manual state-machine evidence.

Experimental evidence for the provisional async implementation, 2026-09-19.
This is executable Raven application code on neoCLR, not a public Task API and not
compiler-generated async/await support. The model specializes its payload to
Result<int,string>; the verifier also specializes the same completion storage to
Result<unit,string>. This does not establish a generic Task<T> implementation.

## Question and boundary

Can an initially pending operation retain heap-owned resume state after the start
function returns, then resume through an explicit executor without running consumer
code inside the producer's completion call? A working completed-only await would
not answer this question.

The author explicitly selected state machines as a provisional mechanism. Public
completion semantics must not promise that builders, state-machine interfaces,
awaiter registration, boxing or layout exist forever. Compiler adaptation is
expected. Runtime-owned suspension may later replace those implementation contracts.

The model uses one executor within one invocation. Registration and completion are
serialized: there is no multi-thread race or cross-invocation host callback support.
TryComplete accepts the first value only. Consumers use the Awaitable interface;
the experiment's public implementation fields are not a hardened capability boundary.
The final library must hide producer state and expose only the intended capabilities.

A pending state machine registers a bound method on a heap object. Its source and
destination remain reachable through that continuation. Completion stores the value,
posts registrations in order and releases its registration list. Late registration
also posts. Drain processes batches and rejects recursive pumping. Already-completed
operations can finish inline without registering. This makes awaiting different
from an explicit yield; no ambient context capture or ConfigureAwait API is introduced.
These are experimental dispatch choices, not settled UI-affinity or context policies.

The completion storage and executor treat their payload as an opaque value. The
Transform method uses ordinary ? propagation to add a saved number or return an
error. This is example application logic, not a Result-aware lowering rule. The final
Task<T> implementation must work identically for arbitrary T.

Expected failure in this example is a completed Result value. A pending GetResult faults instead of
blocking; duplicate TryComplete returns false. Callback faults terminate execution.
Cancellation, deadlines, ownership of abandoned work, fairness, logical context,
external I/O and cleanup after faults are deliberately unresolved. An endlessly
replenished queue can starve its host. Do not build public I/O APIs on this probe.

Storage uses ArrayList<Result<...>> as an occupied slot because the current bounded
importer rejects direct arrays of this closed Result type. This allocates extra
objects and is not a proposed production Task representation. Public helper/type
visibility accommodates the application importer's current cross-type call boundary.

## Comparison and tradeoffs

Primary sources reviewed on 2026-09-19:

- [.NET 10 TaskCompletionSource<T>](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.taskcompletionsource-1?view=net-10.0)
  separates producer control from consumer Task access and is thread-safe. It also
  supports exception and cancellation states; this probe uses Result values and
  does not claim thread safety. Keeping producer/consumer separation is useful;
  copying the complete .NET state model would conflict with the selected error model.
- [ConfigureAwait FAQ](https://devblogs.microsoft.com/dotnet/configureawait-faq/)
  distinguishes scheduling/context capture from logical execution-context flow.
  An explicit single executor makes this experiment predictable but does not solve
  affinity, cross-executor work or logical context propagation for the platform.
- [Python asyncio Future](https://docs.python.org/3/library/asyncio-future.html)
  schedules done callbacks through its loop, including callbacks added after
  completion. That offers a comparable queued model. Its cancellation, exception
  and context semantics are not imported into neoCLR.

Inline callbacks would avoid queue overhead but permit unexpected consumer execution
inside completion. A global thread pool adds concurrency and host assumptions before
we have the contracts to manage them. The explicit queue is the smallest way to
measure ordering and retention here; it is not a performance improvement claim.
Library policy owns this queue. Existing runtime heap tracing and ordinary calls
execute it, and manual Raven code stands in for compiler lowering. No suspension
opcode, Raven compiler patch or runtime behavior change is part of this experiment.

## Running

Use a collection-profile saved project and matching development bridge/System library:

```sh
python3 docs/experiments/task-contract/verify.py /path/to/Demo.rvnproj \
  --bridge /path/to/Probe.dll --system /path/to/System.neoil \
  --runtime /path/to/neoclr
```

The driver creates disposable project copies. It compiles and verifies every
scenario before running it, requires exact success output, and checks terminal
faults separately. The pending-state scenario requires the runtime's GC diagnostics
to show actual collections, rather than assuming allocation pressure triggered GC.

It checks immediate and pending completion, multiple consumers, saved state across
GC, repeat reads, error and unit payloads, duplicate completion, registration order,
late registration, nested completion, pending reads, recursive pumping and callback
faults. Passing these checks is evidence for the model, not general async support.

Validation on 2026-09-19: all 10 scenarios passed using Raven SDK
0.1.12-neoclr.15, the development reference/library at neoCLR 7a805cd and the
Preview 8 runtime. The pending-state case observed 46 garbage collections.
The larger initial allocation loop exceeded the runtime's instruction budget;
the bounded 1,000-iteration pressure loop used for validation stays within it.

## Next compiler integration slice

Raven ddf10eca80d599ede28f9da59f145497a21e854e still selects its .NET builder
families in SynthesizedAsyncStateMachineTypeSymbol.DetermineBuilderType and emits
an AsyncDispatchGuard with a SetException path in AsyncLowerer. A neoCLR adapter
must address these instead of admitting or discarding unsupported exception handlers.

1. Introduce the smallest generic completion storage/producer implementation and
   compiler-facing reference contract, with its provisional status explicit.
2. Make the compiler choose the target's builder and completion/error policy. Keep
   completion uniform in T, without Result recognition, and runtime Faults terminal.
   Result handling and ? belong to their ordinary language semantics.
   Omit the generated exception guard itself, not merely SetException: the current
   compiler still creates a catch when that builder method is absent. Audit normal
   and early-Result cleanup separately from unsupported exception unwinding.
3. Ensure pending state is heap-owned and builder copies share completion identity;
   never retain a byref into a returned activation. Validate Task<unit> explicitly.
4. Execute actual generated async code through immediate and pending cases using
   the same observable assertions as this experiment. Inspect emitted metadata/IL,
   and reject unsupported state/awaiter shapes rather than guessing their layout.
5. Keep neoCLR-specific lowering on Raven's neoclr branch. Extract any general
   builder or metadata correction only after independent CLI/.NET validation.

The public Task model should not expose this sequence as permanent implementation
requirements. Replacing state machines later may require rebuilding compiled code;
source API continuity is a separate goal from permanent binary ABI compatibility.

The propagation variant was revalidated after the compiler-policy slice: all 10
neoCLR scenarios still pass, with 46 collections in the pending-state case. The
resumed Transform method uses ?; failure returns from that ordinary method before
the success computation, and the surrounding completion machinery treats its
returned value uniformly. This still uses the manual probe, not generated async.

## Task composition follow-up

`verify_composition.py` exercises Map and Then against the current Task/Promise
library. It covers queued callbacks on pending and terminal inputs, value-changing
flattening, both stages of cancellation, Result.Error payloads, generic unit,
cross-queue progress, GC retention, terminal callback Faults and private dispatcher
access. See the [current Task contracts](../../task-contracts.md#task-composition--2026-09-21).


## Current generated async and editor probes — 2026-09-23

- `verify_async.py`: generated async, Result propagation, immediate/resumed
  cancellation, combined `(await input)?`, GC and unsupported cleanup diagnostics.
- `verify_default_queue.py`: default Promise construction and automatic invocation
  dispatch, worker progress, retained callbacks and execution-budget faults.
- `verify_map_result.py`: explicit Ok mapping, Error/cancellation bypass, async
  composition, source dispatcher retention and terminal callback faults.
- `verify_editor.py <prepared-project-folder>`: launch the server configured by
  that folder's `.vscode/settings.json`; check Task/Promise/MapResult completions
  and the Default property. It does not edit Main.rvn on disk.

The first three take a prepared Demo.rvnproj plus `--bridge`, `--system` and
`--runtime` paths. The [Tasks feature page](../../../website/features/tasks/index.html)
uses complete source samples, also tested through installed `.rvnproj` build/run
commands. Local development toolchains must pair the compiler and language server
with the current reference metadata, System library, importer and runtime.
