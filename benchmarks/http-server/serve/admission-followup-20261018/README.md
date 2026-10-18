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
