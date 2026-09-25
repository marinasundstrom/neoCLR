# Cancellation foundations probe

Development slice 3 of the networking/web release plan. See the
[contract and .NET comparison](../../cancellation-design.md).

```sh
python3 docs/experiments/cancellation/verify.py \
  --toolchain-root /path/to/matching/neoclr-bundle \
  --runner target/release/examples/measure_async
```

Use a freshly built bridge, reference core and assembled System library containing
`runtime/raven/Cancellation.neoil`. The script builds in a temporary directory. It
checks source/token separation, default tokens, copied/boxed/array tokens, LIFO and
inline callbacks, reentrancy, disposal during cancellation, and a queued cleanup
acknowledgement before Task cancellation. It rejects registration after disposal
and access to internal helpers, and runs an independent .NET 10 callback baseline.
`--guards-only` skips a main sample already validated against the same library.

The main runtime fixture passed on 2026-09-25: 448 allocations, nine collections,
peak 64 live allocations and zero live at invocation teardown (256-byte GC budget).
The disposal/access guards and .NET 10 baseline also passed. The bridge's
`--signatures` fixture checks token layout, public/internal member
access and mismatched static cancellation signatures. No website build is run by
author direction. This is not evidence of thread safety or native I/O cancellation.

A compiler observation remains separate: capturing an outer token array from a
lambda created inside the churn loop failed emission with “Missing local builder
for 'tokens'”. The retained fixture keeps the array live outside that lambda and
still tests captured scalar tokens. This needs an independent Raven/.NET reduction
before classification as a general compiler fix. No Raven emission change is made.
