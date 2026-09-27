# Raven entry results (development)

Author-selected integration, 2026-09-27; not part of Preview 10.

The collection profile accepts static, nongeneric `Main`, either parameterless or
with one `string[]` parameter, returning `()`, `int`, `Result<int,E>`,
`Result<(),E>`, `Task<int>`, `Task<()>`, `Task<Result<int,E>>` or
`Task<Result<(),E>>`. The target's unit task is spelled `Task<()>` (or
`Task<unit>`); it has no separate nongeneric .NET `Task` class. Both async methods
and ordinary methods returning a Task use the same startup contract.

`int` and `Ok(int)` become the process exit status. Unit success becomes zero.
`Error(E)` writes the payload's Object display to stderr and returns one. An invalid
uninitialized Result faults. Cancelled tasks and tasks that remain pending after
registered work drains fault through the existing GetResult contract. Ordinary
Task.GetResult stays nonblocking. Native faults still produce the normal runtime
diagnostic and failure status.

The importer emits a parameterless Int32 startup adapter. Async startup drives the
existing default task queue and registered host operations before reading the
returned task. Suspended startup frames stay in the usual GC roots and share the
invocation's instruction, cancellation and resource limits. This retains neoCLR's
existing drain-to-quiescence policy, including unrelated registered work; it does
not create a CLR thread-pool synchronization context. Explicit private task queues
still require their owner to drive them. Recursive entry dispatch is rejected.

## Comparison and decision

[C# Main](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/program-structure/main-command-line)
(retrieved 2026-09-27) supports void/int/Task/Task<int>, with optional string[];
integer results determine process status. Raven additionally defines Result success
and stderr/error-status behavior in its
[entry-point specification](https://github.com/marinasundstrom/raven/blob/neoclr/docs/lang/spec/top-level-code-and-entry-points.md).
Those are language/startup policies, not native CLI Result return signatures.

Raven's heap-async target (branch `neoclr`, commit `d6f04b751`) now retains the selected Main and its target return type
in the intermediate PE. The importer owns adaptation, as it already did for entry
arguments. With `TargetCoreAssemblyName` and `UseHeapAsyncStateMachines` selected,
Raven recognizes the target Task and omits its host CLR bridge. Other .NET targets
keep their existing bridge. Runtime Contract configuration is unchanged: named
System.Void, heap async state machines, cancellation propagation and no async
exception capture. These target images require neoCLR import; they are not CLR
executables. No compiler-host Console/TextWriter methods enter the target image.

Keeping the old no-result restriction would force every app to implement startup
completion manually. Reusing the CLR bridge would require .NET Task blocking and
TextWriter contracts that this UTF-8 platform does not implement. Target-owned
adaptation reuses managed Result extraction, Object display and TextWriter output
at the cost of startup adapter code and a bounded dispatcher intrinsic. No speedup
is claimed. A future general target entry contract may replace this existing
heap-async target policy; no wholesale Raven experimental-branch integration is
implied.

Compatibility: `neoclr run` now returns integer entry values as process status,
also for Neo and neoIL. Scripts that formerly treated a nonzero integer result as
successful execution must return zero or explicitly inspect that status. Host OS
exit-status truncation rules apply. Embedding still receives `Execution.value`.
No public System API signatures change.

## Validation

```sh
python3 docs/experiments/entry-results/verify.py \
  --runtime /path/to/neoclr --bridge /path/to/Probe.dll \
  --system /path/to/System.neoil --reference /path/to/NeoCLR.CoreProbe.dll \
  --evidence docs/experiments/entry-results/validation.json
```

The executable matrix checks each return family, success/error process status,
separate stdout/stderr, argument forwarding, pending default-queue completion,
non-async Task-returning Main, and rejected signatures. Native tests in
`tests/entry_results.rs` cover process status, startup resumption and recursive
entry dispatch rejection. Raven's focused entry-point suites cover target selection
and preservation of existing .NET behavior.

The updated `library-async.rvn` and `library-async-default-queue.rvn` samples are
part of the executable matrix. `verify_http.py` builds the updated HTTP client
with its handler fixture and drives a local HTTP peer; pass the same artifact
arguments plus `--runner /path/to/measure_async`. Like the existing HTTP verifier,
it uses 512 heap objects and 100,000,000 instructions, checks output order and
requires zero retained heap objects. The CLI's default budget is intentionally
smaller and is not raised by this integration.

Two independently reproduced compiler observations remain outside entry adaptation;
see [minimal sources and outcomes](compiler-gaps/README.md). They also occur with
synchronous Main. They are deferred general candidates, not claimed fixes.

Validation on 2026-09-27: [25 executable/source cases](validation.json) pass: 19
successful runtime cases, two expected runtime faults and four source rejections.
The source rejections were checked in a focused follow-up after correcting the
inactive-default Result expectation to its compiler diagnostic. The matching
artifacts were unchanged. 22 focused metadata signature checks, 33 existing Raven
entry-point tests and the new target-selection test pass. Native startup/CLI tests
and existing Task/worker checks pass, including retained state, cancellation and
instruction-budget teardown. The regenerated reference assembly is unchanged;
its source fingerprint snapshot passes.

The two async samples and HTTP client build and run with matching development
artifacts. Website tests pass (18 cases); the changed Raven/Tasks pages pass
rendered-sample, local-link and anchor checks. The attempted full website build
stops at an existing, unrelated API-coverage mismatch:
`System.LocalTimeMapping.Unique.Deconstruct(System.ZonedDateTime@)`. No full-site
build success or website publication is claimed.
