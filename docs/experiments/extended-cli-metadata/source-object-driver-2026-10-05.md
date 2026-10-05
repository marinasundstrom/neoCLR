# Source Object root through compiler and runtime commands — 2026-10-05

Raven `ac702d7c0` emits the checked-in [source](bootstrap/source-object-root.rvn) with
`--library --source-object-root --core-reference <core> --bootstrap-intrinsics`. NeoCLR
loads the resulting PE with an empty explicit System seed and selects the root using
`--object-root <library>`. A neoIL caller invokes Item.Display, constructs Item and returns
`source root override`. This establishes ordinary producer and runtime commands; the
caller is not yet Raven and imported-root semantic selection remains open.

Build Raven's compiler and probe with `-p:NeoClrMetadataProject=<absolute path to
NeoCLR.Metadata.Experimental.csproj> -p:WarningLevel=0`; build the compiler with
`-f net10.0 -p:UseRavenCoreReference=false`. Build NeoCLR with `cargo build --locked --release`
(on the current macOS host use the Xcode MacOSX26.2.sdk SDKROOT).

```sh
# From Raven; use absolute argument paths and a fresh evidence directory:
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --source-object-root-driver /path/to/IntrospectionCoreParams.dll \
  /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  /path/to/neoclr/target/release/neoclr \
  /path/to/neoclr/docs/experiments/extended-cli-metadata/bootstrap/source-object-root.rvn \
  /tmp/source-root-driver-evidence
```

The C# acceptance runner invokes the actual processes and saves commands, exit statuses,
stdout/stderr, input/artifact hashes and the disassembly. It checks six invalid compilation
cases, existing-output preservation and the exact runtime result. Runtime CLI regression:

```sh
cargo test --locked --release --test cli_object_root --test cli --test cli_modules --test object_root_identity
```

The [command evidence](source-object-driver-2026-10-05.json) records tested artifacts and revisions.
All 30 runtime/CLI checks pass; 16 existing Raven source-root tests pass. The explicit
bootstrap hash is `64aae0e5ef83fe113a38d21c74cabe1bfabbd9de3526d941ea2236b630686bbc`;
regeneration currently requires that supplied declaration bootstrap. Root source methods
are fixtures, not production System implementations. No fallback CLI projection is used
for the root artifact. No general .NET compiler behavior fix was introduced.

Remaining: artifact-only Raven consumer root selection, generic local bases where samples
require them, and normal sample compilation/execution. Complete System is not a POC
prerequisite following the author's scope clarification. Editor and native Tasks/await
work remain visible; prioritize the actual sample inventory before widening support.
