# Owned-root boxing — 2026-10-05

Reviewing Raven's source-root emitter guard exposed a metadata dependency: boxing still
pushed the bootstrap Object signature even with an authored native root. The metadata
API now exposes ObjectType and uses it for boxing/value-test result validation. The
writer checks complete local root slots rather than requiring a second System binding.
CoreObjectType keeps its explicit bootstrap meaning; existing signatures are not rewritten.
As with CLR boxing, the result has the target's Object identity. This change supplies
that identity for an explicitly authored native root, without inventing a new opcode.

Validation from neoCLR parent `3d11b894` plus this slice: all 157 C# metadata groups pass.
The focused Rust `metadata_api_authored_root_slots_load_and_execute` test passes, now
also calling API-authored BoxedDisplay (box, value-type isinst, virtual call),
returning "42". Existing default boxing tests
run in the C# suite; incomplete-root and mismatched bootstrap result tests were added.
Runtime code is unchanged from the previous gate. Raven parent `f7a211173` retains the
source-root emission guard; no compiler behavior is changed here. Matching compiler
documentation is committed at `926af0db3`.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --object-root-slots-image tests/fixtures/metadata-container/object-root-slots.pe
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
  cargo test --release --test object_root_identity metadata_api_authored_root_slots_load_and_execute
```

Updated fixture SHA-256: `2dc52b8deee0cb87c2cb190aace81a0914291b46fd97be0952630e922aec7ce7`.
The earlier root-slot evidence records the previous fixture before BoxedDisplay was added.
Regeneration changes the projection MVID; compare decoded semantics rather than PE bytes.

Remaining: local root override signatures, selected imported root identity, compiler
capabilities/emitter integration and host/driver selection. This is not source-built
production Object execution. VS Code/LSP and Tasks/await samples remain required release
evidence. The generated guest API snapshot check still reports its pre-existing stale
inputs; the manual host API reference is updated, with no unrelated website build.
