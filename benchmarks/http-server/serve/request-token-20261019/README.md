# Borrow HTTP method bytes and measure text allocations — 2026-10-19

Validate incoming method tokens directly over ReadOnlySlice<byte>, then decode
once into owned text. Internal request construction accepts the validated
HttpMethod rather than validating/encoding its text again. Public HttpMethod
construction remains strict; case and extension-token spelling remain unchanged.
A parser-owned contiguous header buffer replaces the previous list with the same
initial reservation and geometric growth. No receive/header view escapes parsing.

The malformed-token path still decodes before returning its token error, preserving
invalid UTF-8's existing error precedence. This path is not the allocation target.
Lazy decoding is not adopted: it would retain header storage and need an explicit
request-lifetime/cache contract. The resulting method still owns a string.

## Allocation evidence and string investigation

The matched native caller trace runs 200 measured plain HTTP exchanges, 20 warmups
and one stop exchange. Startup/shutdown are included. Both traces reconcile every
allocation and payload byte, with zero dropped records. This compares the adopted
Promise array-slot baseline with the new parser, using the same compiler, guest,
native runtime and heap settings. Instrumented timings are not speedup evidence.

| Counter | Before | After |
| --- | ---: | ---: |
| All allocations | 51,806 | 50,922 |
| Allocated payload bytes | 3,043,939 | 3,021,397 |
| Actual text objects | 889 | 889 |
| Actual text payload bytes | 12,209 | 12,209 |
| UTF-8 encoding arrays | 884 | 442 |
| UTF-8 encoding array payload bytes | 18,784 | 10,386 |
| UTF-8 decoding snapshot arrays | 884 | 884 |
| Decoding snapshot payload bytes | 24,308 | 24,308 |

Exactly two method encoding arrays and two validation iterators disappear per
exchange: 884 fewer allocations (1.71%) and 22,542 fewer payload bytes (0.74%).
Actual strings account for only 1.72% of baseline allocations and 0.40% of payload
bytes in this tiny ASCII workload. Of those, 884 strings come from UTF-8 decoding;
five are console formatting. This does **not** measure typed live retention, prove
string CPU cost is small, or generalize to Unicode/long-text applications. Counters
exclude allocator headers. Native kind 4 describes reference-slot arrays, not
individual strings. `text-allocations.py` makes the distinction reproducible.

The author's next direction is to eliminate unnecessary temporary arrays. The
remaining decoder snapshots are a concrete target: System.Text.Utf8 currently
copies every Sequence<byte> before invoking the runtime's array decoder. A borrowed
decoding boundary can avoid that snapshot while allocating the owned result text.
Actual string representation, grapheme semantics and public codec policy remain
unchanged in this commit. Separate ASCII/non-ASCII/long-text and retention tests
are needed before drawing broader conclusions about string allocation impact.

.NET provides [Encoding.GetString(ReadOnlySpan<byte>)](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.getstring?view=net-10.0#system-text-encoding-getstring(system-readonlyspan((system-byte))))
for decoding directly from borrowed bytes without an intermediate managed byte
array. This is the relevant implementation/API comparison; neoCLR's UTF-8 strings,
strict Result errors and codec quotas remain its existing contracts. The benefit
of borrowing is fewer temporary copies; the cost is lifetime/GC validation across
the runtime boundary, not a reason to retain parser buffers in request objects.

## Correctness

- 46 differential wire cases match the baseline exactly in interpreter, JIT and
  AOT: standard/custom/mixed-case tokens, allowed punctuation, invalid delimiters,
  non-ASCII, malformed/overlong/truncated/surrogate UTF-8, fragmented methods,
  method-specific body rules, error kind/message and recovery through shutdown.
- The existing header-reservation probe passes all three modes: exact and exceeded
  limits at 1, 127, 128, 129, 256 and 512 bytes, including fragmented growth.
- No public signatures changed; the documentation source fingerprint is refreshed.

## Reproduction

Use the same Raven compiler dcf7ac41f and matching Xcode 26.2 SDK. Build five
libraries with scripts/build-native-class-library.py. Build Serve and the two
probes with ../task-completion-20261018/build.py; use Probe.rvn here and
../header-reserve-20261018/Probe.rvn respectively. Run verify.py with the retained
baseline oracle and all three modes; run the header probe's verify.py. The runtime
must include current Socket Memory bindings (the retained TLS-memory bundle does).

Run ../caller-profile-20261018/capture.py on each Serve build, then:

```sh
python3 text-allocations.py TRACE/summary.json --output TEXT_SUMMARY.json
python3 compare.py --baseline-build BASELINE --candidate-build CANDIDATE \
  --rates 400 --duration 30 --output LOAD_OUTPUT
```

All builds and profilers finish before timed trials. Evidence and input hashes are
retained beside this report. This bounded parser change is separate from the
proposed runtime completion bridge and does not select a collector default.

## Matched load outcome

Four 30-second synchronous paced-GC windows at 400 offered requests/sec use the
same 1 MiB heap, 16 lanes and 64 client slots, reversing baseline/candidate order.
All 48,000 requests are admitted and validate; shutdown passes. Two baseline
requests in round two complete during drain, and every other completion is within
the measured window.

| Round / arm | CPU ms per in-window completion | Offered p99 ms | Peak sampled RSS KiB |
| --- | ---: | ---: | ---: |
| 1 baseline | 1.8475 | 3.64 | 12,592 |
| 1 candidate | 1.8742 | 8.80 | 12,848 |
| 2 candidate | 1.8758 | 25.96 | 12,848 |
| 2 baseline | 1.8661 | 31.92 | 12,592 |

The candidate uses about 0.5–1.4% more CPU per in-window completion in these pairs;
tail latency varies. Fewer allocations therefore do not establish a speedup.
This step removes redundant work and provides the contiguous parser buffer needed
for direct decoding; its allocation saving is measured, its throughput benefit is
unproven. No overload or concurrent-GC performance claim is made.
