# Unified code-generation component naming — 2026-10-13

The shared JIT/AOT backend is `neoclr-codegen` in `runtime/codegen`. Active code,
SDK/package builders and workflows use this identity. Historical reports keep
recorded names. No old-name executable alias is supplied.

Local checks pass: one hosting-driver test, 24 scalar interpreter/JIT/AOT checks,
six Windows COFF/root/stack admission checks, installed default-JIT discovery,
notice packaging and two archive checks. The fresh SDK cleanup consumer passes
in interpreter, JIT and AOT. Its installed manifest selects `bin/neoclr-codegen`,
and its bin directory contains only the runtime, codegen executable and JIT helper.
The previous compiler role is removed when staging the replacement.

Reports retain input hashes and invocations; logs retain focused test outcomes.
The runtime-await carrier/functionality evidence is maintained separately in
[the SDK integration record](../runtime-await-sdk/). Windows naming/package
qualification will use the same cleanup consumer with rebuilt execution tools.
