# API-produced Object root slots — 2026-10-05

The independent C# metadata API now supplies concrete new virtual slots on an explicitly
authored Object root. Manual MethodDefinition attributes and AddNativeObjectSlot share
validation. Equals requires the exact owned root, not the bootstrap Object reference.
Reader materialization, introspection and CLI projection preserve Virtual/NewSlot;
native format-5 uses its existing flags. This follows the CLI distinction between slot
declaration and override without changing the platform's existing abstract root shape.

Validation was performed from neoCLR parent `0026f05c` plus this slice. Runtime code is
unchanged; host root admission remains the implementation introduced at `4e9e4045`.
Raven's source binding is `e748b089f`; emitter guards remain active. Matching
compiler integration/release-gate documentation is committed at `f7a211173`. The fixture is
`tests/fixtures/metadata-container/object-root-slots.pe`, SHA-256
`eec97bd687f959f5d73cc610988fa8d47ecce5bf7e8d6af90413ad49bfce752f`.
Its constant method bodies test dispatch; they do not replace production Object services.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --object-root-slots
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
  cargo test --release --test object_root_identity
```

Results: 157/157 C# contract groups (baseline 156/156) and 13/13 runtime identity tests.
The new runtime case loads the C# PE using explicit library root selection and executes
ToString, Equals and GetHashCode on ordinary objects plus boxed Int32 display. It checks
returned values, not instruction sequences. Existing default-load, wrong-identity,
revision, duplicate-root and malformed-slot controls still pass.

A previous test rejected Virtual/NewSlot at detached construction because no supported
owner existed. It now accepts the declaration but explicitly rejects attachment to a
value owner; the old .NET override execution checks still pass. This is deliberate API
expansion, not relaxing ordinary override behavior.

The manual host metadata API reference is updated. The generated guest API snapshot
check continues to fail for the previously recorded stale inputs. No website build.

Next: selected root identity for boxing/signatures/overrides, Raven emitter and host
catalog wiring, then source Object and artifact-only consumers. The broader gate requires
VS Code/language-server editing/build/run and working Tasks/await samples. Native async
emission remains open; runtime suspension and green threads are explicitly deferred.
