# Concurrent snapshot investigation — rejected prototype

2026-10-18. This is an isolated **counterexample**, not a selectable or production
collector. It leaves runtime sources unchanged. The patch is tied to the source hash
checked by `run.py`; rebasing requires review. The author requested continuing the
concurrent investigation after adding named GC selection.

The prototype copies a bounded heap at a safe point and traces it with the existing
walker on a persistent POSIX worker. The owner can mutate and allocate while that job
is outstanding. Existing allocated blocks are not reused until owner-thread publication;
new allocations are retained. Explicit collections discard/join the worker and use the
synchronous collector. No snapshot worker accesses the live application heap.

## Executable result

`python3 docs/experiments/concurrent-gc-snapshot/run.py --output NEW_DIRECTORY`
(with the project's Xcode SDK environment) reproduces:

1. Hold the worker after publication. Delete an edge, move its target into a root,
   allocate another object pointing to that target, and verify owner progress while
   the cycle remains outstanding. Release the worker; the old reachable target and
   new allocation survive, while pre-existing garbage is reclaimed.
2. Create an owned host root whose owner is otherwise unreachable. Three consecutive
   snapshot cycles retain that owner and its value because the prototype promoted
   conditional roots to strong roots. A synchronous collection immediately expires
   that handle, runs its resource cleanup exactly once on the owner thread, and reclaims the heap.

This second result rejects the algorithm as implemented: repeated conservative
promotion can retain abandoned asynchronous state indefinitely. Finite floating garbage
is different from retaining the same unreachable owner on every collection. A passing
reproducer means it reproduced this flaw, not that the collector passed qualification.
The first scenario demonstrates progress with a deliberately delayed worker; it is
not a throughput benchmark or proof of safe handling of all mutation patterns.

UBSan/bounds execution passes and raw commands/output are retained. ThreadSanitizer
builds, but its executable exits with SIGSEGV before output on this host. This attempt
predates the added resource-cleanup assertion; the retained input hashes identify
that version. An empty
`main` compiled with the same sanitizer fails identically. Retain both reports; race
validation is **unavailable**, not passed. macOS evidence does not qualify Windows.

## Constraints exposed by the investigation

- Preserve conditional reachability and its fixed point in the snapshot, including
  parked frames and dependent host roots. Do not keep abandoned owners alive merely
  to simplify root capture.
- Define what happens when a host reads/replaces a conditional handle or resumes
  parked work before reclamation is published. The current synchronous model cannot
  simply be assumed to cover that interval; cancellation, barriers or revalidation
  must make the operation safe.
- Generated multi-lane stores in `runtime/codegen/src/values.rs::write`, direct array
  and reference stores, fault/static roots and native helpers are separate mutation
  surfaces. Barrier-based tracing must cover all of them and obey the C memory model.
- Interpreter graph operations use `shared_heap::Access`; moving collection to a
  thread that takes the same exclusive lock merely moves the pause. Actual concurrency
  needs immutable snapshots or a changed access protocol.
- Keep snapshot-copy pause, worker CPU, scratch memory and reclamation publication
  separate. Reducing one pause by adding more total work is not automatically a gain.

The [alternatives](../../design/concurrent-gc-evaluation.md) remain open. The author
subsequently clarified that this is a practical proof of concept, not a request for a
complete collector framework. The next decision should use phase costs to establish
which work is worth moving off-thread. Continuing this snapshot candidate would need
conditional-root/mutation checks against the synchronous oracle. Do not expose
this prototype through `--gc`, benchmark it as a qualified server, or promote it by
adding periodic full collections that conceal the lifetime flaw.
