# Native cryptography service qualification

Development, 2026-10-12. The explicit `--bind-cryptography` compiler option lowers
four exact byte-array InternalCall contracts through the same host adapter in JIT
and AOT. It requires `--compile-system`, `--reference-arena` and a host exporting
`neoclr_cryptography_v1`. Ordinary managed method bodies are never replaced. Missing
binding permission fails compilation; missing JIT host exports fail before execution.

The contracts are in `runtime/neoCLR/Runtime/Cryptography.neoil`: SHA-256,
HMAC-SHA-256, verification and secure random fill. The adapter does not collect,
call guest code, retain buffers or run a separate scheduler. It passes byte payloads
to the existing ring-backed provider after checking array headers and complete input
initialization, and publishes reserved output slots only after provider completion.
A short digest destination returns -1 unchanged; uninitialized input is a terminal
RuntimeError, not an operation error. Successful hash/MAC writes affect exactly 32
bytes and can overlap input. Random-provider failure clears output and returns -2.
Verification returns a Boolean. The helper return status remains reserved for
terminal faults; operation results use a separate output slot.

This follows the synchronous computation layer behind .NET's HashData and
RandomNumberGenerator APIs, while using neoCLR's explicit runtime-service and Fault
contracts. The byte-array/header ABI is private and experimental; it is not a public
Span/Memory substitute. See [comparison and provider limits](../../shared-security-provider.md).

## Reproduce

Build the runtime and native compiler, then run:

```sh
cargo build --locked --bin neoclr
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/native-cryptography/validate.py \
  --runtime target/debug/neoclr \
  --compiler tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/native-cryptography
```

On Windows x64, run from an MSVC x64 developer environment and use `.exe` for the
two tool paths. The same script is wired into the Windows/macOS security workflow.
The script builds the provider archive and a focused GC/stack host, assembles the
same IL for every mode, and checks real process results. Detailed commands and
output go to the target directory's `validation.json`. The checked-in
[checkpoint](validation.json) records the local case outcomes and input hashes.

Twelve cases pass locally in interpreter, JIT and AOT (36 executions): empty/short/
multiblock SHA input, aliased SHA input/output, RFC HMAC output and verification,
tampered MAC rejection, zero/nonzero random fill, unchanged short destination,
reserved output, and uninitialized input at one byte and above 64 KiB. The script
also checks that compilation without explicit binding fails without publishing an
object. A focused compiler unit test rejects changed signatures, generic contracts
and ordinary managed method bodies.

## Remaining gates

Windows execution is pending; adding CI is not platform qualification. This is a
private bytecode/source-host checkpoint, not the public Raven cryptography API or
an installed bundle capability. The qualification host currently links the existing
TLS archive for its raw C exports, though the cryptographic implementation itself
is in the TLS-independent provider crate. A standalone native cryptography archive,
normal project/JIT-host packaging, public managed projection and matching API
reference snapshots remain next. Managed SslStream and certificate/trust APIs are
also still open.
