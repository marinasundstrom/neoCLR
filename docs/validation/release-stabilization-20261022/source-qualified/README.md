# Source qualification and Unix connect readiness — 2026-10-22

Run [39849082702](https://github.com/marinasundstrom/neoCLR/actions/runs/39849082702),
revision `bec78e0b4c730e12a47564105bc5d5af98de7bad`, passes canonical extracted-source
release tests and 25 smoke programs, formatting, strict all-target Clippy, Windows
host/package checks and minimum Rust 1.85 compilation on all three hosts. Compressed
raw reports preserve the canonical pass, Windows host pass and macOS failure.
This is not an overall passing run: macOS fails the refused-connect unit test,
returning generic I/O error payload 11 rather than ConnectionRefused payload 7.

## Repair and contract

Unix `connect_ready` previously returned true unconditionally, allowing SO_ERROR
and peer-address probes before nonblocking connect completion. Use zero-timeout
poll for writability/error/hangup before probing SO_ERROR. Interrupted readiness
checks defer to a later scheduler poll; invalid descriptors remain errors. The
existing Windows select path is unchanged. Deadlines, cancellation and error
mapping remain unchanged; no broader generic-error acceptance is added to tests.
The CI log does not retain the native errno, so it does not prove which premature
probe produced the generic error. Requalification is required to close the failure.

This preserves the distinction between generic socket errors and connection refusal
found in [.NET SocketError](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socketerror?view=net-10.0).
The implementation follows the native [nonblocking connect contract](https://www.man7.org/linux/man-pages/man2/connect.2.html):
wait for write readiness, then inspect SO_ERROR. This is a provider correction,
not a public API divergence. It adds one nonblocking poll syscall per pending
connection inspection and no heap allocation; no throughput improvement is claimed.

The regression checks actual writable/nonwritable socket states, and the refusal
lifecycle check repeats 16 times while retaining its exact error and resource
release assertions. Local socket validation and repaired hosted qualification are
pending. Earlier local type-identity (9) and terminal (10) tests pass.

The change affects the Rust interpreter provider; existing package evidence remains
tied to its original binaries. A release containing the repair must refresh affected
runtime packages and their acceptance. Website/API signatures do not change; the
qualification index records the open gate rather than publishing a new capability.
