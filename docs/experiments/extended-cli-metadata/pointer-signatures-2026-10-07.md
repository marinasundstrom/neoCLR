# Bounded unmanaged pointer metadata — 2026-10-07

The metadata library now authors and reads scalar/Void pointer callable signatures
using CLI PTR and the existing native Ptr representation. No format fork or runtime
instruction change is needed. Pointer views retain exact element identity without
loading runtime types. See the [API contract](../../../api-docs/experimental-metadata.md#unmanaged-pointer-signatures-development-2026-10-07).

Validation: **162/162 C# metadata groups pass**. The new group covers manual definitions
and builder parity, CLI reflection of void*, CLI/native round trips, imported signatures,
canonical pointer views, sixteen-level nesting, invalid target/Function/vector/generic
shapes and rejection of a mismatched pointer return.

Reproduce from the repository root:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
cargo build --bin neoclr
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --pointer-allocation /tmp/PointerAllocation.neox
```

Create `/tmp/PointerSystem.neoil` containing `.module System` and a trailing newline.
Then execute:

```sh
target/debug/neoclr verify /tmp/PointerAllocation.neox --system /tmp/PointerSystem.neoil
target/debug/neoclr run /tmp/PointerAllocation.neox --system /tmp/PointerSystem.neoil
```

Verification exits 0. Execution exits **42**, with no stdout/stderr: the API-authored
assembly multiplies 6 × 7 with the checked native service, allocates that byte count,
round-trips the void pointer through a local, and frees it. The empty System seed
makes the runtime dependency explicit. [Recorded artifact/runtime hashes](pointer-signatures-2026-10-07.json).

This proves the metadata writer/runtime boundary only. The previous five allocation
service tests cover initialization, limits, overflow and cross-instruction ownership.
Raven pointer mapping and source NativeAllocation adapters remain next; the current
full-System frontier is still four binding errors, not a successfully emitted System.
