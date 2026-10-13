# Native allocation traversal

The native allocator formerly validated all heap headers, then searched headers
again for the first suitable free block. It now remembers first-fit during the
validation walk. It still validates trailing headers before mutation; corrupt
heaps retain RuntimeError precedence over an oversized request, and failed calls
preserve output and heap state. This reduces work without a persistent free list,
heap ABI change, relaxed validation or collection-policy change.

This is a bounded refinement of the current allocator, not CLR's allocation
context/bump-pointer fast path. Full validation remains linear per allocation;
large batches remain quadratic. See the existing [GC comparison](../../docs/runtime-gc.md).
A persistent trusted index or allocation context requires a separate integrity and
lifecycle design. This change preserves the current first-fit fragmentation policy.

`benchmark.py --baseline 13edfa189 --output NEW_DIRECTORY` compares copied baseline
and current collectors with identical helpers. Five alternating pairs follow one
warmup per variant. Only allocation batches are timed; heap construction for the
holes case, graph checks and final collection are excluded. Objects are 16-byte
payloads; string-sized atomic blocks are 128 bytes (this is not encoding or string
API throughput); buffers are 8192 bytes. Holes retain alternating allocations and
refill freed spans. Every process verifies retained payloads and final reclamation.
Separate diagnostic copies count allocation header visits, not timed speedups.

Local macOS ARM64 results in `macos-20261013.json` show 1.66–1.89× object allocation,
1.87× string-sized allocation, 1.52× buffer allocation and 1.49× fragmented refill.
These short microbenchmarks exclude collection time and do not predict application
speedups. All rounds, source hashes and commands are retained. The new corruption
regression and existing GC/root contracts pass locally (19 executions).

## Platform qualification

[Run 38420279943](https://github.com/marinasundstrom/neoCLR/actions/runs/38420279943)
passes Windows GC/root contracts and macOS/Windows socket contracts. The matched
allocation matrix improves 1.18–1.60× on hosted macOS ARM64 and 1.32–1.84× on
Windows x64. Separate local macOS results above use their own paired baseline;
ratios do not rank platforms. All raw rounds and contract reports are retained.

The subsequent rooted-helper integration passes 20 local GC/root checks, and
compiled concat/join tests in both GC and non-GC modes preserve interpreter
results and faults. Fresh backend compilation and host relinking validate 128
plain/JSON HTTP responses across JIT/AOT (`rooted-helper-http-20261013.json`).
The direct JIT tool emits one expected `JIT_RESULT` diagnostic; the validation
adapter parses it and requires guest exit zero, rejecting other stderr. An initial
controller run rejected this successful diagnostic as unexpected stderr; the
retained passing rerun uses that explicit adapter. This is functional qualification,
not a matched application speedup claim.

At `4ba0a60d`, Windows passes all 20 native GC/root executions, including borrowed
concat/join inputs under allocation pressure, and the socket contracts (retained
`rooted-helper-windows-*-38422680168.json`). The local phase-profiler hook check
also passes two 32-response plain HTTP runs after its source marker was aligned
with the audited-helper collection contract (`rooted-helper-phase-check-20261013.json`).
That check uses an existing guest fixture with current native helpers; it establishes
tool compatibility, not an application speedup.
