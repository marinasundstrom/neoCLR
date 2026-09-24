# One budget across native I/O phases

Development checkpoint, 2026-09-24. This is a native ownership probe, not a new Raven
HTTP capability. The subsequent HTTP bridge now uses this path with a 15-second exchange budget,
retaining shorter phase bounds.

The combined loopback case creates one absolute monotonic deadline, resolves a
numeric loopback address, connects to a listener and receives the first byte of a
body. The second receive retains the original deadline. Advancing the owner clock
to that deadline yields TimedOut, preserves the already-received byte and releases
the pending transfer storage. The caller then closes the connection.

Run from the repository root:

```sh
cargo test --lib socket_io::tests::lookup_connect_and_body_progress_share_one_absolute_budget
cargo test --lib socket_io::tests
cargo test --lib name_resolution::tests
```

The native implementation accepts an optional absolute deadline on internal lookup,
connect and transfer submission paths. Its effective deadline is the earlier of the
existing phase deadline and the supplied request deadline. Successful short reads
and address fallback do not renew it. A valid submission whose shared deadline has
already expired returns TimedOut before starting native work or reserving its storage.
Normal argument validation and admission limits retain their error precedence.

Additional deterministic tests cover a peer making repeated small progress, expired
admission, retaining the shorter phase bound, no fallback after expiry and DNS worker
ownership. Timed-out DNS cannot interrupt libc: host work retains its permit until
it returns, while late results are detached from the completed operation. These tests
use controlled clocks for expiry rather than sleeping through the request budget.
The combined success path uses only numeric loopback lookup and a local listener.

Local validation passes 32 socket tests and nine resolver tests. No platform adapter,
managed signature, compiler reference or runtime service ABI changed. This is the
native foundation used by the subsequent Raven HTTP integration, which carries the
same deadline across its pending Tasks and checks expiry before reporting success. Accept waiting and application handler cancellation
need separate ownership decisions. No public Timeout/Deadline type is selected.
