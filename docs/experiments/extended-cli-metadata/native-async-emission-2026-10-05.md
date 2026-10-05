# Native async state-machine emission — 2026-10-05

Raven `ddd154db1` emits nongeneric top-level async functions using its existing heap
AsyncLowerer and portable declaration/body plans. NeoCLR uses the existing metadata
encoding, interface dispatch and queue/task implementation; no runtime changes are needed.
Compared with the existing .NET backend, this shares lowering but materializes compiler
symbols through the native builders instead of Reflection.Emit. Machine owners are internal
top-level metadata types. Generic and class/extension async methods and async entry
completion remain outside this bounded gate.

## Executable evidence

The checked-in `bootstrap/native-async-state.rvn` compiles through ordinary `rvnc neoclr`
with explicit native async provider `Numbers`, then runs through `neoclr`. It asserts
awaitless completion, an already-completed await, a genuinely pending await, retention of
a hoisted local, and cancellation propagation. Its synchronous entry completes/cancels
promises and drains the queue explicitly. Exact stdout is `Native async state checks
passed\n`, exit status zero and empty stderr. This does not prove pending async Main.

The accompanying JSON records commands, source/artifact hashes, compiler/runtime revisions
and outcomes. The runtime is from `fccb3a64`; the NeoCLR changes in this slice are only
sample, tooling and documentation. Explicit primitive bootstrap, retained seed and ownership
manifest remain required. No application reference falls back to a CLI projection.

Reproduce with the recorded dependencies (or equivalent rebuilt artifacts):

```sh
python3 scripts/check-native-poc-samples.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCoreParams.dll \
  --seed /path/to/System.neox --ownership /path/to/ownership.json \
  --reference /path/to/Numbers.dll --reference /path/to/Http.dll \
  --async-library Numbers --case native-async-state \
  --runtime target/release/neoclr --output /tmp/fresh-native-async-emission
```

Inspect `compiled` and `execution.passed`; the inventory tool reports failures without
itself failing the process. Five original async/HTTP cases are also recorded separately
within the evidence; successful binding alone does not count as execution.

## Compiler validation and next step

44 focused .NET async/options and shared portable-plan tests pass. The C# metadata probe
`--native-async-symbols <core> <Numbers> <seed>` verifies selected identity, substitution,
native emission/reader materialization of two state machines, malformed/missing provider
rejection and preservation of destination streams for unsupported class/generic methods.
The selected provider also supplies the public nongeneric IAsyncStateMachine interface.
These nominal special identities must not be treated as primitive storage.

Next connect async entry completion and class methods, then execute the unchanged async
and HTTP samples. Do not substitute full System compilation for the sample-driven POC,
or introduce runtime suspension/green threads. No independently useful .NET fix requires
a main backport from this native adapter slice. No guest public APIs changed, so the
known API snapshot gaps remain tracked separately; no website build is required here.

## Original sample reassessment

The unchanged `library-async-cancellation` also compiles and executes, printing exactly
`Cancelled\n` with exit zero and no stderr. It relies on the existing runtime queue
completion behavior. The inventory runner now checks that output as an executable control.

Remaining observed failures, with no output publication:

- `library-async`: portable body emission reports an undeclared local.
- `library-async-default-queue`: encoded entry must have an Int32/no-result signature,
  exposing the missing async entry completion bridge.
- Both HTTP JSON samples: class async methods hit the explicit native emission guard.

Prioritize the shared async lowering/body and entry integration gaps, then class methods;
retain the new cancellation sample as a regression while extending support.

## Follow-up: closure capture of hoisted locals

The native portable capture path now reads immutable suspension locals from their
state-machine fields. The added CapturedPromise regression schedules a callback over
its Promise, suspends, and proves the callback and await retain the same object identity.
It returns 42 after draining; all prior assertions and exact stdout remain unchanged.
`library-async` now reaches the same entry signature failure as the default-queue sample.
Mutable captures remain unsupported. Reflection.Emit closure storage is unchanged.

[Capture follow-up evidence](native-async-capture-2026-10-05.json) records the compiler
base revision plus the capture fix described here; artifact hashes identify the tested
binaries. The original emission JSON remains the evidence for the prior slice.

The capture fix is Raven `70c111b25`. All 37 focused .NET async/portable-body tests pass.

## Follow-up: class async methods and metadata ownership

Nongeneric static/instance class async methods now emit through the same pipeline. Their
machines are nested under their source class instead of flattened; existing runtime access
rules then permit private receiver storage. The focused AsyncCounter case awaits a pending
promise, mutates its original private field from 40 to 42 and proves receiver identity
through its property. Its static async method also returns 42. No field visibility was
widened; there is no runtime access bypass or metadata format change.

C# reader checks retain the Worker declaring type and reject generic methods/owners
before publication. Extensions and async entry completion remain guarded. Both unchanged
HTTP samples advance to a portable value-block control-flow rejection. The next bounded
work is handling the async propagation block at its owning layer and completing async
entry semantics without discarding result/failure handling.

The class-method implementation is Raven `25cc60977`; 38 focused .NET async/declaration
tests pass. [Final class/sample evidence](native-async-class-2026-10-05.json) records
exact input hashes and command outcomes. No independently useful .NET behavior fix
requires backporting from this native declaration adapter change.
