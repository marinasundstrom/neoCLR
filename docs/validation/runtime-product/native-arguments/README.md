# Rooted native argument construction

Generated native-GC entry calls use a private rooted process-argument adapter. Its
construction frame retains the partial string array before the guest frame exists.
Native argv or Windows malloc-backed conversion bytes stay stable through decoding;
no I/O or argument acquisition is repeated. Old adapters remain noncollecting.

All 28 local sanitized GC/root executions and package checks pass. The fixture
forces retry both before array creation and after the first decoded argument,
checks earlier-element survival, strict Unicode/empty arguments, invalid text/null
arguments, partial exhaustion, unchanged failure outputs and complete reclamation.
Two compiled tests pass: actual macOS entry behavior/failure parity and Windows
object import/ABI selection. The native helper must match generated symbols.

Hosted macOS/Windows qualification is pending. This does not enable collecting
file/network/TLS/SQL producers, nor change public managed APIs or general pacing.
