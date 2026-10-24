# Root publication and GC execution-state access

Development evidence, 2026-10-24. A bounded follow-up in the
[native execution-state workstream](../../../docs/design/runtime-bottleneck-investigation.md).
Baseline `87ffcc1ce` already uses binary slot headers. No codegen or managed changes.

## Why and what

A fresh, exactly mapped four-owner profile records 10,916 worker self samples:
`collect_indexed` 1,672, TLS lookup 1,537, `enter_owned` 889, `validated_bases` 404,
`stack_roots` 403, conditional collection 367, and published-boundary entry 177.
These are wall-clock samples, not CPU shares. Normal publication resolved root TLS,
then called a helper that separately resolved GC pacing TLS.

The root and collector translation units now share one **private thread-local**
execution-state record. Hot root, statistics, pacing and cleanup fields precede the
cold legacy string-search hints. A static inline pacing check reuses publication's
resolved state and calls the existing idle/collection slow path only when needed.
The generic native boundary still checks the current head; site-profile builds keep
that checked path because an external callback intervenes. Shared-worker boundaries
still go through their coordination hook. No worker barrier or GC poll is omitted.

Lifetimes stay independent: parking/resuming changes the active roots, never resets
pacing or statistics; cleanup guards and context/collection epochs are unchanged.
There is no whole-state reset, retained state pointer or cross-thread sharing. The
symbol is hidden on ELF/Mach-O to avoid interposition between runtime libraries.
Generated imports, 104-byte frame layout, 32-byte allocation descriptor, allocator and managed
contracts remain unchanged. On the current 64-bit layouts the two old TLS records
sum to 1,120 bytes; the combined record is also 1,120 bytes. No allocation is added.

This uses the existing [.NET/Java/Go owner-local state comparison](../../../docs/design/allocation-metadata-runtime-comparison.md).
It is not a CoreCLR Thread implementation or a general execution-context registry.
Tradeoffs are private cross-translation-unit layout coupling, a larger publication
function, and target/compiler-dependent TLS savings. Other TLS state remains
separate; this does not complete the execution-state redesign or speed metadata loading.

## Clean paired capacity and residual profile

Same checked ten-byte plaintext response, 32 client connections, connection close,
Go GOMAXPROCS=2, two seconds warmup plus ten seconds measured, two trials/configuration
in opposing orders. Application objects are identical between variants. Production
pacing/owned heaps are enabled; four-owner capacity uses isolated heaps, not the
shared-heap stress configuration.

| Configuration | Requests/s baseline → candidate | Change | CPU µs/response baseline → candidate | Reduction |
| --- | ---: | ---: | ---: | ---: |
| AOT one owner | 3,639.9 → 3,769.7 | +3.6% | 271.8 → 261.2 | 3.9% |
| AOT four owners | 9,330.1 → 9,517.8 | +2.0% | 378.0 → 371.1 | 1.8% |
| Owned JIT one owner | 2,081.2 → 2,127.4 | +2.2% | 478.2 → 466.3 | 2.5% |

AOT CPU is server lifetime less client CPU; JIT uses process CPU from readiness to
shutdown, excluding startup/teardown. Do not pool these scopes. All candidate rate
ranges exceed the matched baseline ranges, but these short desktop trials do not
establish confidence intervals or sustained gains. Initial AOT showed little four-owner
benefit. Keep that uncertainty visible: this is a modest local improvement, not a
large throughput step. Historical rates must not be compared across cohorts.

Latency tails are mixed. AOT one-owner p99 is 10.93/10.64 ms baseline versus
9.82/11.88 candidate; four-owner is 4.61/5.21 versus 4.62/4.48. JIT p99 worsens in this
cohort, 17.24/26.70 versus 20.64/27.25 ms. This is no latency improvement claim.

Final disassembly shows one TLS resolver in stack_roots for both variants; baseline
then calls the published-boundary helper with its own resolver. Candidate inlines
the check and removes that call on the normal path. The ordinary checked helper
remains available. Dynamic exports omit the new private state symbol.

The separate final profile has 10,940 worker self samples: collector 1,739, TLS
1,333, owned entry 970, stack_roots 566 and base validation 385. TLS falls from
1,537/10,916 to 1,333/10,940 in these wall samples. Inlining moves published-boundary
work into stack_roots, so the latter's larger count is not evidence of extra work.
Neither comparison is an individual-helper CPU speedup. Collection, frame entry and
other TLS accesses remain prominent. An earlier candidate profile overlapped binary
inspection; it is retained separately and excluded. The final profile ran alone.

`summary.json` is reproduced by the existing tested paired summary tool. Raw final
capacity, initial/excluded cohorts, profiles, disassembly and visibility checks are
retained beside it. No diagnostic/profile throughput is included in the table.

## Qualification

- 230 native slot/root checks: 28 normal completions and 202 expected aborts,
  optimized/sanitized, forced/paced, worker hooks and probe-only variants.
- Optimized and sanitized pacing tests each exercise the owner thread plus two
  fresh native threads, exact allocation/publication budgets, foreign contexts,
  unrelated heap acquire/release, explicit GC epochs and cleanup reentry. New checks
  park/resume without changing the publication budget and require an abort when
  root publication occurs during cleanup (two expected aborts).
- Existing compiled shared-worker graph/fault stress passes optimized and sanitized:
  32 graph/32 fault invocations each, 24,544/448 coordinated collections, peaks four
  graph/three fault participants. These use the current binary-map object.
- Five existing async objects pass 20 relinked optimized/sanitized and context-reuse
  configurations, 40 invocations, including expected terminal-fault output/exit codes.
- Forced-GC plaintext and JSON each pass two responses in both AOT and JIT. Diagnostic
  work counts are retained as qualification, not capacity. Scheduling can add polls,
  so fixed HTTP request counts need not imply identical collector-work counts.

The first standalone pacing link omitted `text-arena.c`, failing on its allocation
helper symbol before execution. The corrected runner uses the standard validator's
source set; the failed command remains in `pacing-link-failure.json.gz`.
No compiler rebuild, API snapshot, unrelated suite or website build is required.
No new Windows runtime qualification is claimed; the portable pacing fixture remains
part of the standard validator. Native source/header packaging uses recursive copying.

## Evidence and reproduction

Reports preserve commands, binary/object/source identities and failures. Scripts reuse
qualified assemblies, objects and hosts; they are not clean-checkout installers.
Run builds/checks before capacity, keep capacity runs sequential, and use fresh output
directories. Both variants use the same `target/slot-maps/plain.o` and current codegen
binary. Baseline root and collector sources come from Git `87ffcc1ce`.

The initial experiment exported its private state symbol. Its preliminary AOT cohort
shows little four-owner benefit. That variant is superseded by the hidden-symbol
version, for runtime isolation as well as qualification. Preserve its exact header
and report separately. The initial JIT cohort may overlap a three-second disassembly
command and is excluded from final claims. Clean final AOT and JIT cohorts use the
hidden-symbol version with no overlapping builds or diagnostic jobs.

```sh
python3 benchmarks/execution/slot-maps/validate.py --output target/state-new/roots
python3 benchmarks/execution/state-access/validate.py --output target/state-new/consumers
python3 benchmarks/execution/slot-maps/counts.py --sample plain --forced --requests 2 --output target/state-new/plain
python3 benchmarks/execution/slot-maps/counts.py --sample json --forced --requests 2 --output target/state-new/json
python3 benchmarks/execution/state-access/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/slot-maps/plain.o --candidate-object target/slot-maps/plain.o \
  --client target/primitive-comparison-refresh/client --output target/state-new/capacity-final
python3 benchmarks/execution/state-access/jit.py --output target/state-new/jit-final
python3 benchmarks/execution/owned-root-chain/summarize.py \
  target/state-new/capacity-final/measurement.json target/state-new/jit-final/results.json target/state-new/summary.json
python3 benchmarks/execution/state-access/profile.py --directory target/state-new
```

The previous object-tracing prototype remains unintegrated. This does not establish
shared-heap capacity, keep-alive/off-host throughput, interpreter gains or peer parity.
