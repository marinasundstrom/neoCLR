# Current plain-text peer comparison — 2026-10-23

**neoCLR has not reached comparable performance.** The author explicitly reiterates
that this is the target for HttpServer. Relative improvements against older neoCLR
builds are useful evidence, but are not completion of that objective.

This refresh uses the generated-owned-storage runtime at a67aa5a0a and its validated
application object, with the owned heap/base cache and 256/1,024 GC pacing. The
subsequent GC thread-state grouping experiment was reverted before this comparison.
No runtime optimization is introduced by this refresh.

## Like-for-like baseline

Same host; checked GET /, status 200, ten x bytes, HTTP/1.1 Connection: close,
loopback TCP, no TLS/compression, 32 concurrent connections. The same compiled EOF
client drives each server with GOMAXPROCS=2. Seven configurations run sequentially
in forward/reverse order, two seconds warmup and six measured. Every response is
validated and every server exits successfully. Build/test/profiling work does not
overlap capacity trials. This exercises minimal server APIs without routing/MVC or
artificial scheduling hops; it is not a language or full-framework ranking.

neoCLR uses one/four isolated owners sharing a listener, 1 MiB heaps and 16 Serve
lanes per owner. Go uses GOMAXPROCS=1/4, GOGC=100; Kestrel uses processor count 1/4;
Node uses one JS loop. Those are runtime settings, not CPU affinity/equal hard
thread limits. Tool versions are Go 1.26.3, Node 22.21.0 and the installed .NET 10
runtime selected by the existing net10.0 fixture. All installed runtime versions,
source/object/binary hashes and commands are retained in comparison.json.

CPU per response is whole server lifetime CPU divided by all warmup/window/drain
responses, subtracting the separate client CPU. It includes startup/shutdown and
is not handler-only CPU. Latency includes connection establishment through EOF;
closed-loop percentiles are not offered-load SLA measurements. Desktop load and
scheduling vary, so these short runs do not establish sustained capacity.

| Configuration | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- |
| neoclr-1 | 1,454.7, 1,620.3 | 673.6, 607.2 | 37.92, 45.38 |
| go-1 | 24,411.3, 20,865.0 | 30.9, 32.1 | 2.82, 3.50 |
| node-1 | 19,565.7, 19,529.7 | 39.9, 39.4 | 4.82, 4.36 |
| kestrel-1 | 19,143.8, 17,042.7 | 69.8, 59.7 | 4.95, 3.89 |
| neoclr-4 | 4,078.0, 4,285.5 | 841.9, 797.2 | 16.27, 23.52 |
| go-4 | 20,989.8, 19,237.8 | 64.7, 68.4 | 3.18, 4.78 |
| kestrel-4 | 15,382.0, 13,987.2 | 151.4, 155.6 | 4.23, 3.78 |

## Gap and priorities

Mean one-owner neoCLR throughput is 1,537.5 requests/s versus 18,093.3 Kestrel,
19,547.7 Node and 22,638.2 Go: approximately 12–15 times lower. At four owners,
neoCLR averages 4,181.8 versus 14,684.6 Kestrel and 20,113.8 Go: about 3.5–4.8 times
lower. Mean CPU/response is 640 µs at one neoCLR owner versus 31–65 µs for the
references, and 820 µs at four owners versus 67–154 µs. More workers improve
neoCLR's throughput but do not remove its per-request CPU disadvantage.

The old [historical baseline](../README.md) remains unchanged. These are fresh
same-host comparisons, not an accumulated percentage claim across experiments.
The differences in desktop conditions mean old and new absolute rates should not
be treated as a controlled paired speedup measurement.

A separate instrumented main-owner run records approximately 208 allocations,
16,003 allocated payload bytes, 3,677 root publications and 3.6 collection attempts
per completed response, with 65.5 µs measured GC time per response and no collection
failures. Startup/shutdown are included; this is a work inventory, not capacity or
a direct CPU decomposition of the worker trials. See profile.json and profile.py.
It uses the same application object, runtime, ownership and pacing but runs on the
main owner so thread-local diagnostic statistics can be read at exit.

The next bounded investigation should attribute those publications and generated
execution costs to methods/sites in parsing, header validation and response
encoding. Use that evidence to select a safe reduction in repeated work or
unnecessary safepoints. This is a proposed investigation, not a proven attribution
to a specific parser method. GC time alone does not explain the total CPU gap.
Adding workers or polishing small checks is insufficient evidence of parity.

Persistent connections remain an essential separate support/comparison gap:
neoCLR currently closes each response. This refresh uses close for every server,
so it neither measures nor claims keep-alive parity. Shared-heap asynchronous HTTP
and larger JSON handlers remain separate, later qualification work. Preserve the
server-primitives scope and ordinary HTTP/Task semantics while improving execution.

## Reproduce

The comparison tool now accepts an explicitly supplied validated object, a flag
to relink current native runtime sources, and opt-in heap ownership. Default source
hash validation remains strict. Refresh mode permits only native runtime sources,
the known HTTP host/native backend inputs and codegen binary to differ from the
original build; it records both hashes. It does not waive managed-library or
application input mismatches. The override object's hash is recorded and matches
the object validated in the owned-root-storage experiment.

```sh
python3 benchmarks/http-server/primitive-comparison/compare.py \
  --neoclr-build target/primitive-comparison-build/build.json \
  --neoclr-object target/plain-owned-root/candidate.o \
  --refresh-native-runtime --owned-heap \
  --output target/primitive-comparison-refresh
python3 benchmarks/http-server/primitive-comparison/peer-refresh/profile.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client \
  --output target/primitive-refresh-profile
```

Use the previously recorded Xcode SDK and matching Raven bundle. Run diagnostics
only after timed comparisons finish. The full 14-trial run validates the updated
comparison controller; the diagnostic run checks response contents, clean exit and
zero collection failures. Private benchmark/tooling changes need no API snapshot
or website build. No new throughput capability is advertised on the website.
