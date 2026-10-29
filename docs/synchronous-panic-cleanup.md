# Synchronous panic cleanup: contract and implementation plan

Status: author-directed design, 2026-10-29; **not implemented**. The public rename
is complete, but [current Panic behavior](system-fault.md) still provides no guest
scope-cleanup guarantee. The withdrawn cleanup.push/pop/end prototype is not the
chosen design. This document specifies the bounded next slice, not a new release.

## Scope and contract

neoCLR is not based around exceptions. Expected failures remain ordinary Result
values. Panic terminates the current synchronous guest invocation after releasing
its active scoped resources. There is no guest catch, filter, throw, rethrow,
exception hierarchy, recovery callback, or resumption at the failing call.
Unwinding means executing lifetime obligations while abandoning calls; it does
not require a platform exception unwinder.

The contract for this slice is:

1. An explicit scope acquisition creates a cleanup obligation only after it
   succeeds. Merely implementing Disposable does not register an object. If a
   later acquisition panics, earlier completed acquisitions are still cleaned up.
   Resources acquired internally by a failing constructor are the constructor's
   own obligations; the caller must not Dispose a partially constructed result.
2. Leaving a scope normally or through Panic invokes each active obligation once,
   in reverse acquisition order, from the innermost scope/call outward. Ordinary
   return, break and outward transfers preserve existing cleanup behavior. An
   ordinary Result.Failure does not initiate unwinding; normal control flow still
   determines scope exit.
3. Mark an obligation consumed before calling its cleanup. A manual Dispose call
   does not cancel the compiler-owned obligation; Dispose remains idempotent.
   Multiple explicit scopes over the same object remain separate obligations.
4. Cleanup calls the existing synchronous Disposable.Dispose contract. Closable
   completion/flush/commit is never inferred from scope exit or Panic. An API that
   needs fallible completion must expose it explicitly before disposal.
5. Cleanup may make synchronous calls and allocate. Its receiver, required locals,
   managed references/byrefs and original panic diagnostic remain alive and rooted
   until their final use. Value receivers use the established receiver semantics;
   no hidden ownership transfer or new copy/destructor semantics are introduced.
6. Capture the original panic message and origin/trace before frames are removed.
   Preserve them across cleanup and propagation. Do not publish a failed call's
   result or execute its ordinary continuation. Cleanup is not rollback of shared
   state or proof that partially mutated objects are safe to reuse.
7. A panic during normal-exit cleanup becomes the primary panic: that obligation
   is already consumed and remaining outer obligations unwind. A second panic or
   terminal runtime failure during panic cleanup stops guest cleanup immediately.
   Preserve the primary diagnostic, attach the secondary failure when feasible,
   and terminate the invocation without retrying cleanup. This is fatal to that
   invocation, not a new unconditional host-process termination API. Runtime
   corruption remains subject to existing runtime safety/termination policy.
8. Cleanup cannot suspend in this slice. Existing execution budgets remain in
   force; exhaustion or external hard termination can prevent completion. This is
   not a promise to complete cleanup after corruption, process kill or arbitrary
   native failure. AsyncDisposable and async suspension boundaries are deferred.
9. Use the existing synchronous outer boundary: the CLI reports an unsuccessful
   invocation; an embedding host receives its Fault outcome. Neither boundary
   resumes failed guest code. No thread/task supervision or new process-lifetime
   behavior is added here, including where the synchronous code runs on a worker.

The initial trigger is System.Panic and the existing guest UserFault path used by
it. Do not silently reinterpret all FaultCode categories as panics. Other terminal
fault categories retain current behavior pending explicit classification. Cleanup
failure must nevertheless detect any terminal failure while cleanup is executing.

GC collection does not invoke Dispose or Close. Guest finalizers, destructors,
value-type finalizers, Rust-style automatic drop, async cleanup, task supervision,
Environment termination methods and abort/unwind configuration are outside this
slice. No general user-defined finally facility is introduced.

## Existing implementation and gaps

Inspection on 2026-10-29 found these reusable pieces:

- Raven's `Targets/NeoClrCliProfile.cs` selects RuntimeDisposalContract with
  UseExceptionHandling false. The native scope rewriter in
  `NativeRuntimeAsyncLowerer.cs` preserves ordinary-exit cleanup but flattens
  cleanup-bearing BoundTryStatement nodes and rejects catch clauses. Reusing a
  frontend node internally is not a runtime exception contract; native emission
  must preserve lifetime obligations before this information is flattened away.
- Raven's portable `LinearMethodBody.cs` emits instructions and locals. Reachability
  currently follows ordinary control flow. Cleanup-only code and live locals must
  survive lowering, reachability, specialization and separate compilation.
- [Interpreter](../src/vm.rs) guest frames retain args, locals and evaluation
  stacks; terminal Fault returns currently bypass guest cleanup. Host calls can
  complete outside the per-instruction dispatcher. Both routes need coherent
  primary-panic state and cleanup execution before frame release.
- [Scalar native codegen](../runtime/codegen/src/compiler.rs) already passes call
  results separately from status and uses nonzero status returns for faults.
  `return_if_detailed` currently returns directly; [value codegen](../runtime/codegen/src/values.rs)
  uses that mechanism too. This is an implementation starting point, not proof
  that every provider, dispatch or export path has compatible cleanup behavior.
- [Native frame cleanup](../runtime/codegen/src/frame_cleanup.rs) releases compiler
  GC-frame registrations on return. It does not call guest Dispose. Guest cleanup
  must precede releasing the roots/storage it needs.
- [Metadata](../src/metadata.rs) and [verifier](../src/verifier.rs) do not currently
  describe or validate these panic cleanup obligations.

Raven paths above are relative to its `src` directory on `codex/neoclr`. These are
source observations, not a validated cleanup-capable compiler/runtime bundle.

## Alternatives and recommendation

| Approach | Benefit | Cost or missing requirement | Disposition |
| --- | --- | --- | --- |
| Existing failure-status propagation plus compiler-owned lifetime descriptions and generated cleanup calls | Reuses normal calls/returns; no platform unwinder or public status API; native code can branch to cleanup before returning failure | Needs preserved scope identity, active acquisition state, interpreter support, verifier rules and complete backend coverage | Recommended design to qualify first |
| Pure source rewriting using only today's instructions | Smallest apparent format change | A source branch cannot observe the hidden terminal status of a nested call today; querying it after a non-returning Panic is unreachable. Exposing status/recovery to guest code changes the contract | Insufficient without compiler/runtime support; do not disguise that support as a library trick |
| Runtime registration through InternalCall | Could avoid new instruction names and keep obligations dynamically ordered | Per-acquisition runtime work, retained captures, reentrancy, root and byref lifetime constraints; a service ABI is still a new mechanism | Compare if static descriptions cannot cover supported lifetimes; not the default |
| New cleanup push/pop/end instructions | Makes dynamic activation explicit to interpreter | Expands instruction/verification/backend contracts and needs equivalent native support; the removed prototype did not establish that necessity | Not justified now |
| General exception regions and platform exception unwinding | Familiar CLR implementation precedent | Adds exception-oriented control flow, personalities/foreign-unwind requirements or handler machinery beyond this contract | Excluded by author direction |

Recommended direction: retain compiler-owned scope/lifetime information and reuse
existing runtime failure propagation. Native backends generate ordinary cleanup
calls on panic exits before propagating status. The interpreter consumes the same
lifetime contract to execute cleanup before removing frames. Guest code never
receives a panic status to inspect or clear.

A lifetime description would associate completed acquisitions, lexical exits and
cleanup receivers/bodies with their owning frame. It must handle repeated loop
entries, conditional acquisition, nested calls and initialization state. This is
not a settled serialized schema: static instruction ranges alone may be insufficient.
Evaluate explicit active-state locals versus a compact per-frame runtime state.
Do not add arbitrary handler targets, catch types, filters or guest continuation
selection. Metadata is not automatically simpler than opcodes; compare verifier
complexity, normal-path work, code size and compatibility before choosing either.

Native cleanup helpers must return normally while the primary panic remains
pending. Distinguish a cleanup call's own completion from the stored primary panic;
otherwise checking an already-failed context would abort cleanup immediately.
Specify this internal protocol before reusing fault-context helpers. Do not unwind
Rust/C host frames, use host-language panics, or cross arbitrary FFI frames. Audit
managed callbacks and provider exits; reject unsupported compositions explicitly.

## Bounded implementation sequence

1. **Inventory and representation decision.** Trace one synchronous Raven `use`
   scope through lowering, native metadata, interpreter, scalar/value JIT and AOT.
   Inventory direct/virtual/interface calls, host providers, callbacks, exports,
   constructors and inlining. Specify primary/secondary panic state, activation,
   cleanup completion, diagnostic ownership, rooting and foreign boundaries.
   Compare a concrete lifetime-description encoding with runtime registration;
   retain the existing opcode set unless a documented missing capability justifies
   a change. Record format/version and mixed-bundle rejection requirements.
2. **Preserve and verify lifetime obligations.** Raven emits the chosen target-owned
   representation before flattening cleanup. Keep ordinary .NET lowering unchanged.
   Verification rejects uninitialized captures, invalid scope nesting/activation,
   branches bypassing cleanup, invalid byref lifetime and unsupported suspension.
   Separate compilation and reachability must retain cleanup bodies and callees.
3. **Implement synchronous propagation in both execution families.** Interpreter
   handles guest and host-call fault routes. Native code routes the relevant status
   exits through generated cleanup and only then tears down GC frames/returns the
   panic. Preserve the original trace and suppress failed output publication. Do
   not claim the contract on any backend that silently skips obligations; reject
   unsupported shapes until implemented, with limitations documented explicitly.
4. **Qualify an end-to-end consumer.** Use separately compiled Raven resources and
   nested synchronous calls, then exercise interpreter, JIT and AOT. Keep tests
   focused on observable lifetime and diagnostic behavior (matrix below). Record
   exact compiler/runtime revisions and unsupported cases.
5. **Integrate documentation and commit the implementation slice.** Update current
   Panic/disposal docs, relevant website/API limitations, both repositories'
   compiler integration docs and changelogs. Refresh affected API snapshots only
   if public reference inputs change. Run focused validation and staged diff checks.

This document completes planning only. No new API, Runtime Contract option, metadata
encoding, opcode or cleanup behavior is implemented by it. Raven owns frontend
scope preservation/emission; neoCLR owns metadata verification and all execution
backends. Future compiler-affecting changes must document temporary CLI encoding,
information loss and eventual native representation in both integration guides;
this plan does not make CLI exception tables the native contract.

## Acceptance evidence

- Nested scopes and a nested call: reverse order, exactly one attempt per obligation,
  original computed UTF-8 message/trace, no ordinary continuation after Panic.
- Acquisition failure, conditional scopes and repeated loop entries: only completed,
  still-active acquisitions clean up; no stale obligation from a prior iteration.
- Normal return/break/outward transfer and ordinary Result failure: existing ordering
  retained; successful Dispose does not accidentally start panic propagation.
- Panic from normal cleanup and secondary failure during panic cleanup: correct
  primary diagnostic, no retry, no further guest cleanup after secondary failure.
- Cleanup calling ordinary helpers/host services and allocating under GC pressure:
  captures and original diagnostic remain rooted; byref/value receiver identity is
  preserved; managed result outputs remain unpublished on failure.
- Malformed lifetime metadata and unsupported suspension/foreign boundaries reject
  explicitly. Execution-budget exhaustion cannot hang an unbounded cleanup loop.
- Direct and dispatched separately compiled calls behave consistently in interpreter,
  JIT and AOT. Test optimized/inlined paths where admitted. Inspect no-cleanup code
  and a focused scope-heavy loop for new per-call/per-scope costs before claiming
  negligible overhead; no broad platform benchmark is required for the design doc.

## Comparison and sources

Reviewed 2026-10-29. .NET `using` gives deterministic disposal, including reverse
order for multiple resources and cleanup on early exit, implemented via try/finally.
neoCLR keeps the resource-lifetime objective but deliberately excludes the exception
programming model. Benefit: expected failures remain explicit and panic cannot be
caught in ordinary guest code. Cost: neoCLR must specify and validate its own
cleanup propagation/interop contract rather than inherit CLR exception machinery.
See [Microsoft's using reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/using).

.NET distinguishes deterministic disposal from GC reclamation and recommends
idempotent Dispose. That separation fits the existing neoCLR interfaces; GC
finalization is not needed to implement this slice. See
[Microsoft's dispose guidance](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/implementing-dispose).

Rust demonstrates that panic can abandon calls with cleanup and that abort/unwind
strategies carry target and linking constraints. This is comparison evidence,
not a decision to adopt Rust Drop, catch_unwind, its native unwinder or a strategy
switch. See [the Rust panic reference](https://doc.rust-lang.org/reference/panic.html).
