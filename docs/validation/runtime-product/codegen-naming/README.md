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
qualification passes all three execution modes in
[run 38469467357](https://github.com/marinasundstrom/neoCLR/actions/runs/38469467357)
at `a980bff44`, using rebuilt runtime/codegen tools and matching compiler/library
inputs retained from run `38464730911`. It also stages a fresh Windows JIT helper
from the async-console recipe. The first renamed-package attempt exposed a helper
recipe-label restriction: staging accepted only HTTP profiles despite the shared
async host. The fix accepts either profile only when that host is present.

The retained Windows validation, preparation and manifest reports preserve input
identities and the installed `bin/neoclr-codegen.exe` role. The artifact bin
inventory confirms the old executable is absent.
The preparation extract replaces command output with SHA-256 hashes and retains
the original report hash, matching the full SDK provenance procedure.
