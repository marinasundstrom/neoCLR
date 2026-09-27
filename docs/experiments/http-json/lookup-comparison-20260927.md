# HTTP CPU follow-up — 2026-09-27

The author directed proceeding with the slices from the [CPU investigation](cpu-investigation-20260927.md).
Sample recovery is implemented in 22a1950b. Execution now indexes immutable module
metadata candidates, and a general Raven workspace-discovery fix is integrated.

## Runtime lookup and measurements

An execution-scoped, thread-local index groups type/function definition indices by
name, preserving their original order and duplicate candidates. Existing arity,
receiver, identity, signature, generic substitution and access checks still decide
matches. Outside an active immutable module borrow, lookup uses the original scan.
Nested scopes restore the previous index on return or panic; worker threads build
their own index. No address is dereferenced or retained past the scope. Mutating a
module between executions rebuilds the index. Serialized metadata and public APIs
are unchanged.

This avoids repeatedly scanning the whole module. Costs are owned names and candidate
indices proportional to metadata size per active execution/worker, plus hashing and
thread-local lookup. It is not a persistent loaded-image cache or a cache of resolved
constructed calls. Loading, linking and verification remain outside the scope. Like
the CLR resolution goal discussed in the original investigation, it avoids redundant
metadata work without changing language contracts; no JIT or CLR performance parity
is claimed. A persistent loaded-program index could reduce setup cost later, but
would require an explicit ownership/invalidation contract.

[All measurements and artifact hashes](lookup-comparison-20260927.json) retain two
runs per runner, ordered baseline/candidate/candidate/baseline, using the identical
compiled recovery server and System. Each run checks six alternating GET/POST
responses and a final drain request. Median CPU seconds across six requests of each
method per runner:

| Request | Baseline | Indexed | Reduction |
| --- | ---: | ---: | ---: |
| GET | 0.980 | 0.750 | 23.5% |
| POST | 1.255 | 0.955 | 23.9% |

The baseline ranges are GET 0.89–1.55, POST 1.19–1.65 seconds; candidate ranges are
0.69–0.82 and 0.92–0.99. Background load varied, and the first baseline overlapped
the end of SDK/VSIX packaging. Both candidate runs beat the quieter final baseline;
these are local diagnostics, not a controlled throughput or statistical claim.
Candidate startup still costs 3.81–3.84 CPU seconds; the quieter baseline costs 3.77.
Do not attribute the noisy first baseline startup (5.05) to this optimization.
Candidate two-second idle windows consume 0–0.01 CPU seconds before requests and
0.02 afterward. All runs exit 0. Raw profiles/logs remain under
target/http-lookup-comparison. Reproduce with measure-server-cpu.py --runner to
select each binary, holding the bundle and request fixture constant.

[The indexed runner also passes header recovery](indexed-header-recovery-20260927.json):
128/1,024-byte accepted requests, a rejected 2,049-byte request, then a valid GET;
exit 0 and zero live objects.

Three lookup unit tests and 42 focused accessibility, function-generic,
generic-class, constraint and loaded-program tests pass. Thirteen Raven Task.Run
consumers pass, covering worker captures, generic owners, suspension, cancellation,
unwrap and faults. No full suite or website build was run.

## Quota reassessment and next boundary

The first candidate request profile's collapsed top-of-stack counts include 519 in
pthread_mutex_lock, 203 in unlock, 265 in ManagedHeap::array_usage, 184 in
frame_array_usage and 143 in Slot::array_usage. Module::type_definition now has 45
samples. These are sample counts, not additive timed phases or evidence that every
lock belongs to accounting. The source still visits every heap/frame slot at every
instruction boundary, even when its payload summary is unchanged.

**The next justified optimization is reducing repeated accounting aggregation.**
A bounded design should track dirty slot/heap totals and update them on all writes,
including alias/native writes, while preserving tighter-limit checks, saturating
arithmetic, collection-before-rejection, roots from other participants and worker
synchronization. Existing per-slot payload summaries are insufficient to prove such
a change correct. Keep locks and enforcement frequency until that invariant is
implemented and tested. This turn completes the requested reassessment, not that
larger redesign. The remaining 0.75–0.96-second tiny-request cost is material; it is
not a production-throughput result or a new agreed numeric release gate.

## Raven editor diagnosis and integration

Managed stacks from the old PIDs clarify two separate paths. PID 38069 was in
WorkspaceManager.FindWorkspaceSolutionFiles during watched-file reload; its logs
repeatedly attempted generated projects. PID 37889 was in OmniSharp JSON-RPC
InputHandler.ProcessInputStream. After preserving stacks, both old high-CPU
instances were stopped. A closed-stdin probe of the previously installed SDK stayed
alive for eight seconds but used no steady CPU; it did not reproduce the hot input
loop. That transport diagnosis remains open, as does attribution of the old process's
large memory footprint.

Raven now excludes target/artifacts from recursive workspace discovery and automatic
watched-file reload, including generated project.assets.json. Normal source obj
assets still trigger reload, and explicit project references retain normal behavior.
Two new cases failed before the fix and all eleven focused discovery/watch cases
pass after it. The general fix is 264fcc7d9 on Raven main, individually cherry-picked
as 2a3fbb346 on neoclr. No Runtime Contract settings, semantic rules or emission
behavior change; there is no neoCLR-specific compiler policy in this fix. Raven's
compiler runtime-contract documentation and changelog record the same boundary.
Fresh SDK/VSIX packaging succeeds with the fix. This addresses the reproducible
workspace bug, not every cause of long-lived editor CPU.
