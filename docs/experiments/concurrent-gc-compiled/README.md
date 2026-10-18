# AOT callback dispatch with the concurrent GC POC

This isolated experiment links actual AOT-generated code against the qualified
snapshot collector. It is not installed runtime support or a new `--gc` choice.
The input is hand-authored neoIL, not Raven; the existing callback export dispatches
to generated instance, static and faulting bodies. The allocating instance body
increments a counter and publishes a newly allocated child on every call.

## Finding and bounded adapter

The first integration attempt discarded every allocating callback snapshot:
`NEOCLR_NATIVE_GC` root hooks eagerly collect at generated allocation safepoints.
Those synchronous collections correctly invalidate the background result. Earlier
known-C callback fixtures did not exercise this hook.

`run.py` replaces that one call in a generated copy of `root-probe.c` with a test
adapter. It retains root publication, descriptor/fault validation and boundary
checks. When this owner's context has a pending cycle, the experimental mode
defers the eager collection until the host consumes the background result. Outside
that interval it uses the existing synchronous collector. Rooted allocation-pressure
collection is never deferred. The control uses the same adapter but always collects.
No production collector, root-probe or compiler source changes are required.

This is a deliberately limited scheduling policy: one owner, bounded heap and
host-driven cycle completion. It has no autonomous polling, deadline or pressure
heuristic, and no managed async scheduler integration. Deferral can retain garbage
longer; the rooted allocator's synchronous fallback remains the capacity safeguard.
As with CLR background collection, allocation progress still needs a foreground
fallback; this private snapshot implementation is not the CLR's collector design.

## Qualification

[Retained UBSan/bounds evidence](results.json) includes commands, compiler/object
hashes, native inputs and the preceding collector qualification. The compiler was
rebuilt from the current checkout with `cargo build --locked --release -p
neoclr-codegen`. A 128-object host-rooted chain makes the worker's midwalk gate
deterministic; it is fixture work, not generated application work. While paused:

- Invoke 40 generated allocating instance callbacks; check the counter, child value,
  allocation count, published roots, and child survival after subsequent collection.
- Invoke a generated static callback and a divide-by-zero callback; verify fault
  status/frames and rejection of another invocation while faulted.
- Invoke one more allocating callback with the arena full of reclaimable garbage.
  The deferred mode performs exactly one synchronous pressure collection, retries
  successfully, and rejects the stale snapshot.

| Mode | Applied background cycles | Discarded cycles | Deferred eager boundaries |
| --- | ---: | ---: | ---: |
| Eager control | 49 | 41 | 0 |
| Pending-cycle deferral | 89 | 1 | 41 |

Counts include the explicit post-invocation and final drain cycles. Both modes
finish with every allocation reclaimed and no leaked roots. Sanitizers cover the
C runtime/adapter, not the generated machine code. These are deterministic safety
and dispatch checks, not timing or throughput results. The initial assertion that
all eager snapshots would apply failed; the control now explicitly verifies that
they are discarded. The one remaining deferred discard is required for pressure.

Reproduce on the tested macOS ARM64 host (set the Xcode SDK environment as needed):

```sh
cargo build --locked --release -p neoclr-codegen
python3 docs/experiments/concurrent-gc-walker/run.py --output target/gc-qualified-new
python3 docs/experiments/concurrent-gc-compiled/run.py \
  --aot target/release/neoclr-codegen \
  --validated-build target/gc-qualified-new \
  --output target/gc-compiled-new
```

Use fresh output directories. The runner rejects changed qualification inputs.
The [matched compiled-path benchmark](../../../benchmarks/gc-concurrent-poc/README.md#generated-aot-callbacks-with-matched-batch-pacing-2026-10-18)
now compares synchronous and concurrent collection at identical batch boundaries.
Larger-graph repeats gain elapsed time at higher CPU cost; small heaps lose. Next:
reduce overhead and exercise managed async suspension/scheduler dispatch. No HTTP/Go
capacity conclusion follows.
