# TechEmpower comparison boundary — 2026-10-25

The author supplied [Round 23](https://www.techempower.com/benchmarks/#section=data-r23)
as an external comparison reference during concurrent-GC qualification. The ranking
page is dynamically rendered; no Round 23 throughput figures were extracted or
used as measured neoCLR peers in this investigation.

**Review update, 2026-10-26:** The sections below preserve the original protocol
boundary. Internal `ServePersistent` has since been implemented and qualified;
public serving and the older close-per-request measurements remain distinct.
The [current workload review](workload-review-20261026.md) checks the official
specification, database/load realism, required reruns and remaining compatibility
gaps. The official repository is now archived; this does not invalidate its
preserved workload definitions or make our local measurements comparable scores.

## Primary specification

The [official test specification](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview)
was retrieved on 2026-10-25 (page identifies its last edit as 2024-04-04). Plaintext
uses a small text response and pipelined HTTP requests. JSON requires a newly
created message object and serialization within each request, without cached
serialized output. Routing and Server/Date response headers are required. Keep-alive
is encouraged, and plaintext assumes pipelining with graceful handling by servers
that lack it. These are specification requirements, not independently verified
Round 23 runner settings or hardware details.

## Current neoCLR evidence

Our fixed-work GC harness opens a connection per request and reads through EOF.
The owner benchmark does the same with 32 Go client lanes. These are intentional
within-runtime controls. `HttpServer.Encode` unconditionally calls
`AppendConnectionClose`; `HttpContext.Close` disposes the exchange and closes its
connection. This is an implemented protocol limitation, not only a load-tool choice.

The [existing local peer comparison](../http-server/primitive-comparison/README.md)
uses minimal .NET 10 Kestrel, Go and Node handlers under the same close-per-request
policy. Kestrel's terminal delegate is the relevant .NET primitive/API baseline;
a full MVC/ORM stack would obscure the current runtime question. Existing peer
results remain historical until rebuilt and measured with current neoCLR artifacts.

Do not divide our current throughput by a TechEmpower score: machine, transport,
client placement, concurrency, request/response shape, implementation level and
execution configuration are not matched. Likewise, GC improvements measured here
do not establish persistent-connection performance.

## Comparison plan and ownership boundaries

1. Retain the current plaintext/JSON and standalone suite for GC regressions.
2. Add a separate persistent-connection workload once server support is implemented.
   Distinguish sequential reuse from pipelining. Check ordered responses, buffered
   unread bytes, body framing, explicit close, cancellation, timeouts, malformed
   requests, and shutdown before measuring throughput.
3. Keep connection lifetime separate from exchange lifetime. Each request's content,
   response and task roots must retire correctly while its transport survives.
   This is a server primitive, not a web-framework layer or benchmark-only shortcut.
4. Refresh the local Kestrel/Go baseline with matched protocol and resource settings,
   then evaluate a TechEmpower-compatible harness. Record exact runner revisions,
   hardware, client saturation and settings before comparing external results.
5. Run sync and concurrent policies independently, measure CPU per response, pause
   distributions, retained memory and errors alongside throughput. Keep standalone
   allocation patterns because connection reuse can hide costs without removing them.

Persistent connections may amortize socket/Task/allocation work. That is a testable
hypothesis, not a claimed gain or a reason to abandon concurrent-GC optimization.
Separately, current GC phase evidence makes repeated final-collection index/seed
processing a concrete target. Neither direction authorizes bypassing safe root or
ownership contracts. No new protocol behavior is implemented by this research note.
