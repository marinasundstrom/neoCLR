# Value-type async state machines

Development evidence, 2026-09-24. Generated state machines are a transitional route
to building the platform. Runtime-owned suspension remains the intended future
direction; compiler-facing builder APIs may be deprecated or removed.

The development importer supports non-generic application state machines with
neoCLR Task awaiters. Generic `ref` startup runs the state in place. On the first
pending await the reference-type builder retains one boxed state; later awaits
reuse that owner. No continuation retains a reference into the kickoff frame.
Completion and cancellation clear the builder's retained state. Heap states remain
the default; set `RavenHeapAsyncStateMachines=false` after the props import to opt in.

## Validation

Build a matching development bundle and measurement runner, then run:

```sh
cargo build --example measure_async
python3 docs/experiments/value-async/validate.py \
  --toolchain-root /path/to/bundle \
  --runner target/debug/examples/measure_async --output /tmp/value-async-evidence
```

The script compiles identical Release fixtures with each storage policy, imports
and verifies their IL, checks output and runs with a 96-object heap limit. It retains
build/run logs and imported IL. `Matrix.rvn` covers ready, one-pending, two-pending,
ready-cancelled and cancellation after a resume. A reference field is used and
mutated across awaits, with allocation pressure between resumes. `Payloads.rvn`
adds pending unit and propagated Result error completion plus an awaitless result.
All fixtures must reclaim their managed objects after the scalar entry returns.

[Recorded counts](results.json) compare managed objects, including tasks, promises,
continuations and 320 allocation-pressure objects. Ready completion avoids one
state allocation; pending cases have allocation parity with heap states. The
payload fixture also avoids one state allocation for its awaitless method.
This does not measure allocated bytes, host allocations, payload copies or elapsed
time, and does not justify a broad speed claim or a new Release default.

Generic states, arbitrary awaiters, async lambdas and scheduler/runtime suspension
remain outside this gate. The builder remains a class holding a shared Promise;
lazy/fused task-state storage is a separate possible optimization.

## Initial investigation

The earlier probe at Raven d2a583386 / neoCLR b3b3cfc5 emitted a struct but failed
import. Its by-value builder contracts boxed at startup and continuation registration.
That finding motivated the ref projection and ownership protocol above. `probe.py`
retains that smaller diagnostic experiment; `validate.py` is the strict acceptance
check and does not count rejected imports as supported.

An initial scratch fixture captured queue in a nested callback and hit the previously
noted `Missing local builder for 'queue'` issue under both policies. Promise creation
outside the callback isolates state storage; the shared closure candidate remains
open. The six Raven HeapAsyncStateMachineTests check ordinary .NET policy behavior,
not target execution.

See [the assessment](../../async-state-machine-assessment.md#value-state-machine-priority--2026-09-24)
and [builder contracts](../../../api-docs/async-builders.md) for tradeoffs and limits.
