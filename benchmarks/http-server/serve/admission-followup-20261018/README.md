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
