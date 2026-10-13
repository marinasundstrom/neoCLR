# Concurrent HTTP admission

Development evidence, 2026-10-13, macOS ARM64. Not a published performance claim.

`HttpServer` queues concurrent Accept/ServeOne calls within its existing limit of
16 outstanding scopes. One native socket accept remains pending. After accepting
a socket, it starts the next queued accept before TLS and request decoding.
Cancellation unlinks queued nodes immediately; shutdown cancels all waiting calls.
No native socket, scheduler or collector policy changed.

The retained fixture sends four concurrent connections in four phases: incomplete
headers, fixed-length body, chunked body, and a large fragmented header. Every
other response must finish before the client releases the first request's suffix.
It also checks 140 queued cancellations, admission rejection, and 16 shutdown
cancellations. [Validation](validation-20261013.json) passes 16 responses in each
of interpreter, JIT and AOT. Client phase durations are not isolated parser or CPU
timings. Four connections are used in every mode because seven interpreter peers
exceeded the existing five-second per-transfer ceiling in a preliminary run.
The earlier eight-connection JIT/AOT runs passed, but are not the retained fixture.

[Diagnostic GC](gc-20261013.json) passes the same AOT fixture: 14,737 allocations,
14,737 reclaimed, zero failures, peak live payload 39,093 bytes. This is a
whole-process lifetime check, not an allocation-per-request measurement or proof
of every possible GC interleaving. Windows qualification remains open.

This fixes admission, not body streaming: Accept still waits for the complete
request body. The next slice will deliver headers and expose demand-driven body
reads, retaining bounded read-ahead and explicit ownership/cancellation.

## Reproduce

Build Server.rvnproj with `scripts/build-native-project.py --profile http`, a
matching freshly prepared HTTP bundle, 4 MiB byte arrays and a 32 MiB heap. Then:

```sh
python3 benchmarks/http-concurrency/validate.py --build BUILD --bundle BUNDLE --output validation.json
python3 benchmarks/http-concurrency/profile.py --build BUILD/build.json --output NEW_PROFILE_DIR
```

The reports retain commands and input hashes. The fixture uses callbacks only to
observe cancelled outcomes without awaiting a cancelled task as a successful value.

## Compiler follow-up

A preliminary argument-selected width using Environment.GetCommandLineArgs and
array indexing failed AOT emission with `$aot_linked_16: unsupported value member
contract`. The retained fixed-width fixture compiles. This is an unresolved
compiler investigation, not a new language restriction or a diagnosed root cause.

## Sequential regression measurement

[Five alternating matched rounds](sequential-comparison-20261013.json) validate
1,536 responses with unchanged applications and the same optimized JIT host.
Median candidate/baseline throughput ratios are 1.004 plain JIT, 1.001 plain AOT,
0.997 JSON JIT and 0.988 JSON AOT. Ranges overlap; JSON AOT has a small negative
shift worth watching. This is a concurrency correctness improvement, not evidence
of improved sequential throughput. Do not replace published baselines with it.
