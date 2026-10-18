# Admission and idle-wait investigation — 2026-10-18

Follow-up to the direct incoming-request experiment. Reuse its exact fresh
pre-URI-change baseline, direct-construction object and Go control.

## Diagnosis, before any runtime change

Individual records from both longer candidate confirmations contain maximum
dispatch gaps of 81.8/102.2 ms near the admission misses, despite sub-5 ms overall
p99 lag. A low percentile does not rule out a brief client scheduling stall.
A fresh 60-second phase capture completes 17,839/18,000 arrivals: the longest
request connects/sends in under 1 ms but takes 743 ms to first byte, overlapping
a 473 ms generator scheduling gap. This is not connection setup latency.

The process snapshot exposes an orphan Raven language server using a full CPU
core for almost four hours. No VS Code editor/Helper remained (only crashpad and
orphan language servers), consistent with the author's prior request to close
VS Code. Only the busy orphan PID 14420 was terminated with SIGTERM; active apps
were not stopped. This is a real environment confounder, not proof it caused
every miss. The activity window and OS descheduling cause are not fully known.

A separate 60-second GC/acquire/wait trace completes all 18,000 arrivals with
zero dropped events and 1 microsecond endpoint clock drift. Long stalls do not
reproduce: maximum whole-process GC interval is 2.042 ms. It cannot establish
GC as the cause of the earlier 743 ms delay. The measured window does contain
17,710 blind async-pump sleeps totaling 26.346 seconds. That includes legitimate
idle time, not CPU work or 26 seconds of removable request latency.

Unlike the older saturated trace, this faster guest frequently reaches the idle
pump. On macOS it sleeps for 1 ms rather than waiting for incoming socket
readiness. The shared scheduler already has a bounded Windows readiness wait,
but the async pump bypasses it on both platforms. The targeted follow-up is to
use that wait in the pump and provide POSIX poll readiness, preserving the 1 ms
cap for DNS/timers and keeping outcome/root transfer in the subsequent poll.
This addresses an identified avoidable delay, not proven universal stall causation.

Evidence retains phase requests, traced requests, all intervals, source/link
hashes and environment observation. Instrumented runs are diagnostic only.
Reproduce with the existing serve/diagnose.py and admission-stalls-20261018
capture.py/analyze.py against the direct-construction candidate.

## Readiness change and safety

The native async pump now calls the shared scheduler wait only when it has no
ready continuation. POSIX uses poll with a 64-entry stack array and a 1 ms cap;
Windows retains its existing select implementation. Legacy/no-watchable-I/O
fallback remains bounded sleep. The existing deadline scan avoids waiting for
already-expired operations. EINTR returns to the owner poll; provider failure
falls back to sleep rather than spinning.

No callback runs and no socket data/result/root is consumed inside this wait.
The next ordinary poll still arbitrates and settles operations; collection and
conditional-root behavior are unchanged. This is an internal readiness mechanism
behind the existing .NET-style Task/Socket APIs, not a new managed API or adoption
of CLR scheduling policy. poll avoids select's descriptor-number limit on POSIX.
This remains a bounded cooperative scheduler, not a new epoll/kqueue reactor.

UBSan/bounds passes eight native network contracts, the retained-owner and idle
pump contracts, plus explicit wait checks for descriptor 1024+, read/write/connect
directions, unconsumed readable bytes, unchanged collection/delivery state, EINTR,
expired deadlines, queued work and pending delivery. The old implementation fails
the new readiness regression as expected. All 50 public HTTP wire cases pass
against the retained pre-URI-change oracle using the relinked AOT probe.
Windows execution was not rerun locally; its wait algorithm is unchanged, but
the async pump now reaches it. No managed/compiler changes require a new
interpreter library cohort or public API reference.

The diagnostic capture tool now wraps the shared wait once, including async pump
waits; older evidence retains the earlier separate pause hook. Use build.py to
relink a retained guest without recompiling managed code and verify.py for the
focused native contracts. The ordinary native-network validator now includes the
POSIX wait regression. Performance measurements follow separately.

## Sustained-load comparison

All eight uninstrumented points run after terminating the busy orphan. No builds
or profilers run during timing. Each fresh process gets two seconds of warmup,
then 60 seconds at 300 offered requests/sec with a 64-request client limit.
Round one orders baseline, previous, candidate, Go; round two reverses that order.
The 16-lane Serve fixture returns ten bytes after queued Task work, closes each
connection, and uses the same 32 MiB paced synchronous heap. Go uses GOMAXPROCS=1,
GOGC=100 and no GOMEMLIMIT. Client and server share this macOS ARM64 machine.

Baseline is the retained freshly rebuilt managed cohort before URI removal;
previous is URI removal alone. Candidate relinks that exact previous managed
object with readiness waits. This isolates the native change without introducing
another managed rebuild. Build identities and start/end input hashes are retained.

| Arm | Round | Successful arrivals | CPU ms/completion | Offered p50 ms | Offered p99 ms | Peak sampled CPU cores |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fresh baseline | 1 | 18,000/18,000 | 2.197 | 3.233 | 9.759 | 0.792 |
| Fresh baseline | 2 | 18,000/18,000 | 2.015 | 3.280 | 6.981 | 0.673 |
| URI removal only | 1 | 18,000/18,000 | 2.054 | 3.101 | 6.302 | 0.654 |
| URI removal only | 2 | 18,000/18,000 | 1.862 | 3.280 | 7.029 | 0.634 |
| Readiness candidate | 1 | 18,000/18,000 | 1.902 | 2.325 | 5.573 | 0.674 |
| Readiness candidate | 2 | 18,000/18,000 | 1.904 | 2.331 | 5.671 | 0.673 |
| Go | 1 | 18,000/18,000 | 0.288 | 1.981 | 5.505 | 0.099 |
| Go | 2 | 18,000/18,000 | 0.257 | 1.907 | 4.826 | 0.099 |

All 144,000 arrivals succeed with clean shutdown and exact server/client accounting.
Within-window completion rates are 299.97–300/sec; the remaining one/two requests
finish during drain. All points pass the provisional 100% success, offered p99
under 250 ms and generator p99 under 5 ms criterion. These are repeated local
60-second qualifications of 300/sec, not maximum capacity or long-run stability.

Against URI removal alone, readiness reduces p50 by 25.0%/28.9% and p99 by
11.6%/19.3%. CPU changes by -7.4%/+2.2%: no repeatable CPU reduction. Coarse peaks
increase slightly relative to URI removal alone; candidate RSS is 12,880 KiB
versus 12,816 KiB. CPU peaks are half-second process samples, not instantaneous
spikes. Against the fresh pre-URI baseline, combined CPU reductions are about
13.4%/5.5%; those include both changes and cannot be attributed solely to waiting.
Go still needs roughly 6.6–7.4 times less CPU/completion. Similar fixed-rate latency
does not establish equal throughput capacity.

The unchanged controls also pass now. Admission recovery cannot be credited to
the readiness change alone: removal of the environment confounder applies to all
arms. The earlier long stall's exact cause remains unproved; this slice fixes an
independently observed blind wait and shows repeatable latency improvement.
Even this matrix contains a 45.5 ms maximum generator gap in baseline round one,
despite 0.75 ms p99. Raw request records remain necessary for short stalls.

[Summary and paired deltas](evidence/load-summary.json),
[execution order and identities](evidence/comparison.json), and eight compact
`*-300.json` reports retain the measurements; adjacent `.json.gz` archives retain
all request/resource records with checksums. Earlier diagnostic failures remain
alongside them. Reproduce using compare.py with the retained baseline, previous,
candidate and Go build paths; use the load/archive.py helper for lossless archives.
Next increase offered load in a separate matched matrix and profile CPU at the
first sustained-capacity boundary, keeping client scheduling checks explicit.

For the readiness contract, Apple's primary [poll documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/poll.2.html)
describes descriptor arrays, readiness events, millisecond timeouts and EINTR.
This internal POSIX choice preserves managed .NET-style semantics; it does not
claim equivalence to the CLR's platform-specific thread-pool implementation.
