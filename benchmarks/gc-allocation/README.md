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
