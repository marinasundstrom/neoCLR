# Execution modes and HTTP costs — 2026-10-24

This investigation separates common metadata work, native GC policy, and warm
HTTP execution. Production runtime behavior is unchanged. Measurements use
`7d14c15c6` on macOS ARM64; identities, commands and failures are retained below.

## What the modes share

| Layer | Interpreter | Managed JIT | AOT |
| --- | --- | --- | --- |
| Source loading and validation | Rust `LoadedProgram`/metadata services | Rust `LoadedProgram` through closed-world selection | Same selection at build time |
| Execution | Rust VM, 1,024-instruction quanta | Cranelift `values::compile_into`, JITModule | Same lowering, ObjectModule |
| Managed heap | Rust `ManagedHeap`, BTreeMap slots, Arc/Mutex cells | Native C collector supplied by helper library | Native C collector linked into host |
| Coordination | Shared-heap participants, roots, cancellable mutex entry | Host scopes and native scheduler | Host scopes and native scheduler; optimized HTTP host can use isolated owners |

See `src/gc.rs`, `slots.rs`, `shared_heap.rs`, `vm.rs`, `runtime_lookup.rs`,
`runtime/codegen/src/{lib,jit,values,linking}.rs` and
`docs/experiments/jit-managed/host.c`. Source identities are in provenance.json.
The interpreter's contention path parks for 1 ms; this is a candidate to measure,
not evidence that lock contention limits these samples.

Native root publication collects at every boundary unless `NEOCLR_GC_PACED` is
compiled into the host. The optimized HTTP host also acquires an owned heap,
enabling its free-space index. The JIT entry host used here does not acquire that
ownership. These are host configurations, not inherent JIT/AOT restrictions.
Recent native root optimizations therefore reach both native modes with matching
runtime libraries; they do not automatically optimize the Rust interpreter heap.

## Ordinary samples

Two previously admitted task/collection samples use the existing async-Main
wrappers from the transient-handoff corpus. Three uninstrumented invocations per
configuration and one diagnostic invocation all produce expected output and exit
successfully: 40 invocations total. AOT and JIT use the same native entry-host
source, external 1 MiB heap, and explicit forced/paced policy. Diagnostic counters
are excluded from timing summaries.

Median times, in milliseconds:

| Sample / mode | Process wall | Metadata load | Selection + verification | Compile + finalize | Invocation |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tasks / interpreter | 4,957 | — | — | — | — |
| Tasks / JIT paced | 3,383 | 99 | 3,137 | 94 | 0.301 |
| Tasks / AOT paced | 4.43 | build time | build time | build time | 0.203 |
| Collections / interpreter | 5,004 | — | — | — | — |
| Collections / JIT paced | 3,702 | 100 | 3,370 | 202 | 0.433 |
| Collections / AOT paced | 4.74 | build time | build time | build time | 0.272 |

Invocation includes native host setup and teardown, not just guest instructions.
Submillisecond timings are noisy and do not establish a native execution ranking.
Interpreter timing includes loading/verification; it has no separate invocation
measurement here. Its five-second process duration must not be attributed to GC.
Phase medians are independent and need not sum to the process median.

| Sample | Interpreter allocations / collections | Native allocations | Native forced collections | Native paced collections |
| --- | ---: | ---: | ---: | ---: |
| Tasks | 22 / 1 | 28 | 137 | 4 |
| Collections | 51 / 1 | 70 | 513 | 4 |

JIT and AOT native counts agree exactly. Different heap representations make the
interpreter/native allocation counts unsuitable for a direct efficiency ranking.
Diagnostic native counters include final host collections.

## Startup profile: a specific next target

A separate two-second, 1 ms `sample` capture of the collections JIT run starts
200 ms after process launch. It contains 1,358 main-thread samples, 1,345 under
`linking::prepare` → `LoadedProgram::with_modules_and_object_root`. Among these,
1,124 pass through `link_modules_with_object_root`; 470 pass through inheritance
`declared_method`. The collapsed self counts include 481 in SipHasher::write,
150 in memcmp, 106 in VM resolve and 60 in runtime_lookup::candidates. Inclusive
and self counts overlap: do not add them or treat this selected startup window as
whole-process CPU percentages. Profiling was separate from timing trials.

Source inspection explains a plausible source of repeated work: `declared_method`
scans all module functions and resolves each instance method's owner before testing
its name/signature. Scope-local name indexes already exist, but this nested scan
still repeatedly hashes/resolves owners. The next bounded execution experiment
should index methods by resolved owner within an immutable verification scope,
retaining overload, inheritance, generic substitution and access checks. Compare
positive and rejection behavior with the existing path, and measure all three
modes' load/build time. Avoid persistent pointer caches and weaker verification.
This is a proposed optimization, not an implementation or proven speedup.

## Warm plaintext HTTP

Eight trials run configurations forward then reverse, with a one-second warmup
and five-second measurement, 32 connections and two Go client processors. All
responses are checked by the existing 10-byte-body client. Servers shut down
through their control socket and report successful guest status. Same application
assembly, native lowering options and entry-host source; AOT uses the retained
root-map object whose compilation flags match this JIT invocation.

| Mode / policy | Median requests/s | Server CPU µs/response |
| --- | ---: | ---: |
| AOT forced | 32.6 | 30,327 |
| JIT forced | 32.3 | 30,698 |
| AOT paced | 1,268.1 | 789 |
| JIT paced | 994.6 | 1,004 |

CPU is the process `ps time` delta from READY to completion of both client runs,
divided by warmup + measured completed/drained responses. It excludes startup and
shutdown, includes intervening host work, and has approximately 10 ms resolution.
Throughput uses only the five-second measured run. This is deliberately separate
from earlier worker-host CPU windows. Two repeats on a desktop establish an
investigation lead, not precise capacity or tail-latency estimates.

JIT paced throughput is 21.6% below AOT here, with 27.3% higher CPU/response.
Both policies use the same collector source; this does not identify a GC cause.
AOT uses PIC object code and static linking; JIT uses non-PIC code, non-colocated
libcalls and a dynamic host. Import calls, helper-library TLS and code layout are
hypotheses requiring disassembly and matched profiles. Neither path explicitly
sets a different Cranelift optimization level. JIT process-to-READY is about 7.35 s
under pacing; startup is excluded from the throughput table. AOT readiness varies
from 5–271 ms, so this cohort should not support a startup-latency claim for AOT.

These hosts use one owner and an external heap. They are not the optimized
isolated-worker server, not a Kestrel comparison, and not a regression from prior
capacity numbers. Interpreter HTTP capacity and JSON mode comparisons remain open.

## Priorities and comparison with .NET

Reuse the [CoreCLR/Java/Go design research](../../../docs/design/allocation-metadata-runtime-comparison.md):
shared execution metadata and allocation contexts motivate avoiding repeated
resolution, not copying any platform's object layout. Preserve .NET-like API
semantics and source access verification. This slice introduces no divergence,
public API, object-header change or compiler bridge contract.

1. For common loading costs, qualify an immutable owner/method index against the
   measured inheritance-resolution path. It can help interpreter/JIT startup and
   AOT compilation, but cannot improve an already running AOT server by itself.
2. For HTTP execution, compare paced owned-heap JIT/AOT hosts, then profile native
   import/TLS and allocation paths. Preserve the external-heap and forced-GC checks;
   changing host ownership requires lifetime/teardown tests, not just a build flag.
3. For the optimized worker host, continue storage/transient metadata and root
   publication work using its existing plain/JSON counters and capacity controls.
4. Instrument interpreter execution and contention separately before changing
   BTreeMap bookkeeping, slot locks or the quantum. Generational GC remains open;
   these measurements do not isolate survivor distributions or establish its gain.

## Evidence and reproduction

- `ordinary.json.gz`, `http.json.gz`: successful raw commands, measurements, outputs,
  counters and input hashes. `summary.json` is generated by `summarize.py`.
- `startup.sample.txt.gz`, `startup-profile.json.gz`: separate raw stack capture,
  invocation command and successful guest output/status.
- `preparation-failure.json.gz`: earlier harness attempt. Guest execution passed,
  but the harness expected JSON from `--jit-run`. Corrected to `--jit-managed` and
  stderr `JIT_RESULT`; all final runs use a fresh output directory.
- `provenance.json`: source revision, platform, dependency and source identities.

This is a local experiment harness, dependent on retained build recipes/artifacts:
`target/primitive-comparison-build/build.json`, `target/root-map/candidate.o`,
`target/primitive-comparison-refresh/client`, the bundle and wrapped corpus paths
recorded in raw commands. It is not a clean-checkout benchmark installer. Rebuild
matching artifacts using the linked HTTP/corpus recipes before reproducing; never
silently substitute another bundle. Set Xcode `DEVELOPER_DIR` and macOS SDKROOT.
Run sequentially without competing builds or load tests:

```sh
python3 benchmarks/execution/gc-mode-comparison/run.py --output target/modes-new
python3 benchmarks/execution/gc-mode-comparison/http.py --ordinary target/modes-new --output target/modes-http-new
python3 benchmarks/execution/gc-mode-comparison/summarize.py target/modes-new/results.json target/modes-http-new/results.json
python3 -m unittest discover -s benchmarks/execution/gc-mode-comparison -p 'test_*.py'
```

Four aggregation tests pass: startup/execution separation, diagnostic exclusion,
incomplete-cohort rejection and CPU units/response denominators. No production
runtime test suite was rerun for this measurement-only change. Website/API review:
no public behavior changed, and no new headline capacity claim is appropriate.
