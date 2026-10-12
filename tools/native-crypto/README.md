# Private native cryptography adapter

This static library exposes the borrowed-buffer C ABI in
[`include/neoclr_crypto.h`](include/neoclr_crypto.h). Its implementation delegates to
`../security-provider`, which is also used by the interpreter. It has no TLS,
certificate or networking dependency. The TLS archive no longer exports these
symbols; source hosts must link this archive when using cryptography services.

Build with `cargo build --locked --release --manifest-path tools/native-crypto/Cargo.toml`.
The output is `target/release/libneoclr_native_crypto.a` on macOS, or
`target/release/neoclr_native_crypto.lib` on Windows, relative to this directory.
Use the target-compatible C toolchain and Rust target. These are source-build
artifacts, not installed runtime bundle capabilities.

The API is private and unchanged by this extraction. See
[semantics and .NET comparison](../../docs/shared-security-provider.md) and the
[executable qualification](../../docs/experiments/native-cryptography/README.md)
for native link commands and interpreter/JIT/AOT coverage. Run
`python3 scripts/test-native-security.py` from the repository root for the shared
provider, C ABI and TLS tests. No public managed API is introduced here.
