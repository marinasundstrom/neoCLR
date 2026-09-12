# CLI PE container and method-body reader

Implemented 2026-09-12. `neoclr::pe::Image::parse(bytes, Profile::StaticCallsV1)`
inspects a standard CLI container. `method_body(rva)` extracts code and header information;
`MethodBody::decode()` passes code to the [CIL decoder](cil-decoder.md).
This API does not admit or execute an assembly. It does not discover MethodDef RVAs:
those still need to come from validated metadata.

## Standard format, bounded coverage

The baseline is Microsoft's [PE format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
and ECMA-335 Partition II, section 25 (consulted 2026-09-12). RVAs are mapped through
sections to file offsets; they are not treated as file positions. Tiny and fat method
headers carry the standard code size, maxstack and local-signature information.
No metadata field or opcode is added. The profile is a caller-side opt-in to limited
inspection, not a marker written into the image.

The initial implementation accepts I386 PE32 containers with the standard 72-byte CLI
header, version 2.5, and plain IL-only flags. This container choice does not select native
x86 execution: no native code executes here. PE32+, architecture-required flags, signing,
managed resources, native directories and method extra sections (including exception
regions) are rejected for now. Those are coverage limits, not proposed replacements for
.NET behavior. Ordinary native PE import/relocation sections can be present but are never
executed by this reader.

Images are limited to 16 MiB and 1–96 sections. Reads require complete file-backed ranges;
overlapping sections, truncated ranges, RVA overflow and header overlap are rejected.
Method bodies cannot use section padding or zero-filled virtual tails. Code size remains
limited to 64 KiB. Fat headers must be four-byte aligned, have the standard three-word
size, and contain only supported flags. Local signatures and entry points are checked for
non-nil token kinds, not for existence or semantic correctness.

`metadata()` returns the bounded raw metadata region after checking its signature only.
Tables, heaps, assembly identities, signatures, reachable helpers, dependencies, method
ownership and stack/local initialization behavior still require separate validation.
Successful inspection is not a security or executable-admission guarantee. In particular,
maxstack is exposed but not yet compared against verifier analysis.

A narrow Rust reader keeps the runtime independent of a host .NET process and avoids a
new on-disk format. The cost is maintaining parsing and negative tests; it does not claim
the coverage of .NET's PEReader. A broader reader dependency can still replace the internal
implementation when metadata work demonstrates the need. No speed improvement is claimed.

## Evidence and reproduction

The [probe](experiments/pe-reader/Program.cs) uses .NET 10.0.100's
[PersistedAssemblyBuilder](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.persistedassemblybuilder?view=net-10.0)
to emit a small DLL and Microsoft's PEReader to report its method bodies. It includes
both tiny and fat headers, a local signature, and a static call. The checked-in JSON
contains the image as hexadecimal plus the independently read header/code results.
Timestamps and generated module identity may differ on regeneration.

```sh
cd docs/experiments/pe-reader
dotnet run --project Probe.csproj
```

From the repository root:

```sh
cargo test --test pe --test cil --test no_result
```

All 17 focused tests passed. Four PE tests compare the fixture with .NET's results,
reject every truncated image prefix, mutate headers/sections, and check method-body
limits and unsupported flags. These tests do not execute the PE or establish complete
metadata validation. The previous decoder fixture still tests execution through its
explicit test-only binding.

## Next priority

The author clarified that the milestone is a useful Raven subset on neoCLR. After this
bounded container slice, type-semantics alignment takes priority over further isolated
parser work. The [experiment plan](raven-target-experiment.md#next-milestone-useful-raven-subset)
now centers a class/value program. Continue metadata reading only as needed to run that
scenario, preserving standard metadata and instructions unless a concrete extension is
necessary for an intended platform capability.
