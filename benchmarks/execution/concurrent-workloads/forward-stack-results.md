# Forwarded stack-argument proof audit

2026-10-25. Following the rejected [conditional borrowing experiment](conditional-borrow-results.md),
this targets removal of guard calls rather than their TLS acquisition cost. It is
read-only: no additional stores are unguarded and no runtime speedup is claimed.

## Proof and exclusions

The test-only audit accepts `NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS`. It starts with
existing direct stack-address/extent proofs. A local parameter gains a bound only
when **every** incoming direct call supplies either a bounded stack address or a
forwarded parameter whose bound is already established. Subtract a nonnegative,
checked constant offset, then take the minimum remaining extent across callers.
Iterate until no new parameter bounds appear; facts are never assumed to bootstrap
an unsupported cycle. The tests reverse body order to exercise propagation over
multiple iterations and multiple call levels.

Exported/address-taken parameters, missing/unknown callers, pointer loads,
ambiguous joins, negative offsets and out-of-bounds writes remain unproven.
Forwarding cycles cannot manufacture a proof from their own assumptions. A direct
fresh stack anchor is still valid even inside a recursive caller; the exclusion
is circular evidence, not recursion as a language feature. Native callbacks and
stored/reloaded pointers do not gain a lifetime promise from this analysis.

The existing complete-call-closure requirement is unchanged. Direct local calls
execute while their caller/ancestor stack storage remains alive. Only explicit
SSA forwarding and the already supported unique incoming edge/constant offset
forms are followed. This is a bounded compiler-storage proof, not an inference
that a pointer outside one arena is safe. It adds no context/heap ownership proof
and does not weaken heap barriers, tracing or root publication. Class/struct/record
names and managed allocation metadata are irrelevant to this native-storage fact.

`report` can select the forwarding analysis, but `elide` explicitly invokes the
direct-only analysis. Consequently setting the audit flag cannot silently enable
additional guard removal. Qualifying that transformation is the next task.

## Findings

| Sample | Direct candidates | With forwarding | Additional sites | Affected functions |
| --- | ---: | ---: | ---: | ---: |
| Tasks | 64 | 64 | 0 | 0 |
| Collections | 159 | 195 | 36 | 22 |
| Arrays | 86 | 103 | 17 | 15 |
| Plaintext HTTP | 13,807 | 13,883 | 76 | 37 |
| JSON HTTP | 20,543 | 20,727 | 184 | 121 |

These are static store sites, not dynamic guard-call counts. Plaintext additions
include the header-buffer index/count accessors and array index helpers identified
by [caller profiling](acquisition-sites-results.md). Using that earlier eager-borrow
profile strictly as invocation context, affected functions account for 24.5% of
observed plaintext acquisitions, 19.5% for JSON, 12.8% for collections and 12.1%
for arrays. Those shares are neither CPU percentages nor predicted speedups:
branches/loops and multiple stores per invocation prevent that inference. The
profiles use the same retained selections but different native instrumentation;
there is no new workload execution or performance comparison in this audit.

All five complete AOT selections compile and produce byte-identical objects against
the retained checked-store stack-elision controls. Their existing runtime behavior,
allocation/CPU measurements and correctness evidence remain applicable; there is
no reason to rerun performance on identical objects. The new raw reports retain
commands, environments, reference/object/source/runner hashes and changed functions.

Two new tests cover multi-level propagation/body ordering, exact remaining bounds,
minimum caller extents, negative/overflowing offsets, unknown/exported/address-taken
callers and unsupported cyclic forwarding. All 52 focused mutation/admission tests
pass (31 environment-specific tests remain ignored in that run). Three invalid-data
controls reject changed objects, inconsistent static counts and inconsistent prior
profile counts. Broader generated forwarding execution is not claimed as qualified.

## Next qualification and platform comparison

Add separate explicit admission for guard removal using these facts, then test
real forwarding chains, mixed native/heap callers, unaligned writes and faults.
Preserve existing cleanup and native liveness checks. Compare transformed standalone
and HTTP JIT/AOT against the current checked-store/direct-elision control, including
CPU, throughput and allocation ranges. Shared workers, complete pauses and default
acceptance remain open; production GC remains synchronous.

This extends the existing CLR comparison: compiler-known native storage need not
pay managed-heap mutation checks, while uncertain destinations retain them. It
reuses the ownership/provenance research linked from the prior stack audit rather
than changing .NET-facing semantics. No Runtime Contract, Raven semantic/emission,
CLI representation, managed API, heap ownership or runtime policy changes occur.
There is no Raven integration change to carry; neoCLR owns this internal diagnostic.

## Evidence and reproduction

`audit-forwarded-stack.py --runner <signed-test-runner> --root
 target/concurrent-workloads --output <new-directory>` requires the retained
selection/build/object inputs and SDK. It emits all five objects with the old
stack-elision optimization enabled, reports forwarding facts and asserts byte
identity with the old checked-store objects. Borrowing is explicitly disabled.

`forward-stack-audit.json.gz`, the five `forward-stack-*-audit.json.gz` files and
`forward-stack-tests.log` retain raw evidence. `forward-stack-summary.json` combines
static counts with prior caller observations. Recreate it using
`summarize-forwarded-stack.py --audit <audit>/results.json --profile
<prior-sites>/results.json --output <summary.json>`. The prior profile is retained
as `store-acquisition-sites.json.gz`. `forward-stack-negative.json` records the
three invalid-data controls. These tools report opportunities, not GC acceptance.

Follow-up: [transformed forwarding qualification](forward-elision-results.md) adds a separate removal flag with JIT/native and matched performance evidence. The audit flag remains read-only; the direct-only control remains available.
