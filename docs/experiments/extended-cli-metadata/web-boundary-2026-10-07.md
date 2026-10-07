# Separate Web boundary — 2026-10-07

The nine System.Web sources were compiled against independently emitted System.Runtime,
System.Data and System.Networking, with no dependency source files in the invocation.
Data and Networking compile; Web fails binding and publishes no artifact. This is a
reproducible compilation frontier, not Web execution acceptance.

```sh
python3 scripts/audit-optional-libraries.py \
  --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 \
  --core /tmp/failure1006b/Core.dll \
  --runtime-library-directory /tmp/array-runtime1007/runtime-owned \
  --output /tmp/web-frontier-retry --include-web
```

Optionally add `--runtime target/debug/neoclr` to execute the Networking control even
when the Web compile fails. This audit intentionally records nonzero compiler statuses;
its process exit is not a claim that every library compiled. Inspect each case's
exitCode/outputPublished. [Recorded evidence](web-boundary-2026-10-07.json) retains
commands, source/dependency/compiler hashes and exact diagnostics.

## The shared deadline boundary

| HTTP dependency | Current owner/access | Required contract |
| --- | --- | --- |
| SocketDeadlineAfter / SocketDeadlineExpired | Internal RuntimeServices in Networking | Create/check a shared monotonic network budget |
| Dns.GetHostAddressesUntil | Internal Networking method | Resolve within the same remaining budget |
| Socket.ConnectUntil | Internal Networking method | Preserve budget across addresses and connect |
| Socket.SendUntil / ReceiveUntil | Internal Networking methods | Preserve budget across partial transfers |

HTTP creates a 15-second exchange budget and passes its absolute monotonic stamp across
DNS, connect and transfer callbacks. The stamp is currently a private long, relative to
a host process clock origin. Creation accepts 1–60,000 milliseconds; it is not a date,
UTC timestamp, serialized value or portable cross-process identity. Replacing each call
with a fresh duration would extend the total budget and change behavior.

The next bounded implementation is an opaque supported deadline/budget contract and
matching Networking overloads, followed by HTTP migration. Keep raw stamps and service
adapters internal. Preserve cancellation versus timeout outcomes, partial-transfer state,
cleanup and one deadline per exchange. A general clock mechanism, if extracted, belongs
in Runtime without references to Networking/Web; a network-specific policy can stay in
Networking. Public names, general duration range and default-value semantics require
explicit definition in that slice; this audit does not publish such an API.

.NET's [TimeProvider](https://learn.microsoft.com/dotnet/api/system.timeprovider) exposes
timestamp/elapsed-time mechanisms, while
[CancellationTokenSource](https://learn.microsoft.com/dotnet/api/system.threading.cancellationtokensource)
can schedule cancellation. These are useful separations, not an obligation to copy the
API. Keeping the current deadline mechanism behind an opaque contract is a bounded
alternative with fewer scheduling changes; timed cancellation is more general but needs
race/lifetime/error-mapping validation. Promoting the raw long methods is smaller code
but permanently exposes an internal clock representation. Neither alternative is claimed
as implemented or faster.

After compilation passes, execute unchanged HTTP base/headers/JSON client consumers
against the four emitted assemblies, then loopback cancellation/status/upload tests.
Keep Runtime's one-way dependency rule and the unified namespace/type API reference.
No class-library API or current timeout behavior changed in this audit slice.
