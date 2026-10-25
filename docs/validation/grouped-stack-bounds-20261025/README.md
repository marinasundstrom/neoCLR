# Grouped native stack bounds — 2026-10-25

The [current stack profiles](../current-runtime-stacks-20261025/README.md) identify
native stack checks/TLS among shared execution costs. Before changing admission,
inspect generated code: the current macOS ARM64 probe resolves two TLS variables
for the current lower/upper bounds on every check. Grouping the two words behind
one TLS address removes one indirect TLS call, preserving all checks.

## Contract and comparison

`NEOCLR_GROUP_STACK_BOUNDS` is an opt-in layout experiment in the existing macOS
ARM64 native stack probe. It changes no public API or native calling convention.
Both bounds are read afresh, set together by the same owner-thread setter, replaced
on cooperative stack switches, and cleared on teardown. Zero/zero still falls
back to current pthread bounds. It does not cache an OS-thread stack, elide any
check, borrow state across threads, or alter safepoints/GC roots.

This is the implementation-level equivalent of keeping related per-thread runtime
state together; CLR-compatible stack exhaustion semantics remain the target. The
existing explicit neoCLR frame/reserve limits and provisional native-host contract
are unchanged. The benefit is fewer TLS resolutions; costs are a shared internal
layout and platform-specific qualification. Windows stack probes are unaffected.

## Safety and native code

Both layouts pass the focused native fixture: ordinary pthread fallback, valid
explicit bounds, malformed/partial bounds, exhausted reserve, 100 replacements
and resets, and two simultaneous owners each performing 10,000 checks with different
bounds. No bound is cached across calls. Clang -O2 disassembly has two TLS indirect
calls in the original check and one in the grouped check (also two versus one in
the setter). Tests run with assertions enabled; native code compiles with warnings
as errors. This is scoped validation of storage layout, not new arbitrary fiber or
foreign-stack support.

## AOT invocation measurements

Three interleaved rounds validate 98,688 plain/JSON responses with identical objects
and native host options except stack-bound storage. These remain short loopback
cohorts, not saturation tests or framework rankings.

| Handler | GC | Split RPS | Grouped RPS | Invocation CPU change |
| --- | --- | ---: | ---: | ---: |
| Plain | Sync | 7,378 | 7,575 | −2.73% |
| Plain | Concurrent | 5,909 | 6,125 | −3.37% |
| JSON | Sync | 4,048 | 4,199 | −3.07% |
| JSON | Concurrent | 3,539 | 3,458 | −0.96% |

Concurrent JSON has slightly lower CPU but worse wall throughput; retain both
observations rather than claiming uniform improvement. The six standalone
concurrent patterns pass 60 invocations, excluding the first of five rounds from
medians. CPU changes: compute −0.63%, reuse-buffer +1.66%, steady −1.30%, bursts
−1.59%, retained-window −0.83%, mixed-phases −2.33%. Allocation counts and bytes
match across layouts. These small mixed results support a modest shared-runtime
candidate, not a concurrent collector advantage or default promotion.

## JIT follow-up and decision

Twelve concurrent JIT trials pass 49,344 responses (three rounds per layout and
handler) with the same runner, fallback placement and released compilation inputs.
Plaintext median throughput is 5,384 → 5,628 RPS, invocation CPU −1.21%; JSON is
3,022 → 3,331 RPS, CPU −4.53%. Managed allocation counts/bytes match. Compilation is
synchronous and excluded from invocation timing; only collection runs concurrently
with guest execution. These results do not describe background JIT compilation.

The existing native stack test also passes both layouts: recursive exhaustion
before the OS guard page, successful unwind, retained-bound paths and rejection of
a 128 KiB stack. Keep grouping opt-in for now. The shared TLS cost is concrete,
but short standalone/AOT results include regressions, and this is not proof of
broad scheduling or collector superiority. The next useful integration check is
whether this composes with the qualified small-frame validator and whether gains
remain in longer CPU/pause cohorts. Collector and stack-bound defaults are unchanged.

`http-jit.json.gz`, `jit-build.json.gz` and `unwind-tests.json.gz` retain that
follow-up alongside the AOT and focused-test evidence.

## Reproduction

- Compile `matrix/stack-bounds-test.c` with the current native-stack source, with
  and without `-DNEOCLR_GROUP_STACK_BOUNDS`, then run each isolated executable.
- `matrix/prepare-stack-bounds.py` accepts the qualified HTTP build plus a preserved
  original native-stack source whose hash must match the base manifest. It checks
  every other input unchanged and relinks both controls and variants explicitly.
- Run `matrix/http.py` for split/grouped sync/live AOT, three rounds of 4096 requests.
- `matrix/compare-root-validation.py --stack-bounds` reuses qualified standalone
  scalar-store objects with unchanged root validation and current canonical native
  sources. It records input hashes and alternates execution order.
- `matrix/prepare-stack-jit.py` derives matched shared libraries and selections from
  the paired AOT build, with one pinned current runner and fallback JIT placement.

All matrix paths are relative to `benchmarks/gc-suite`. Raw reports, input hashes,
test commands, and disassembly are retained alongside this record. The preserved
original native-stack source allows explicit baseline identity validation despite
the new conditional branch in the current source.
