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

[Hosted run 38449399768](https://github.com/marinasundstrom/neoCLR/actions/runs/38449399768) passes at `df256c9c63d5ba61a49e79a522aaead45c1785e6`: all 28 GC/root executions, sockets and relocated-package checks on macOS ARM64 and Windows x64, plus macOS profiler attachment. The native fixture exercises strict Windows UTF-16 conversion as well as Unix UTF-8. Reports retain source hashes and command provenance. This is not a fresh managed Windows/JIT/HTTP or Windows ARM64 matrix. This does not enable collecting
file/network/TLS/SQL producers, nor change public managed APIs or general pacing.
