# Runtime obsolete annotations

Development contract: `System.ObsoleteAttribute` follows the basic .NET shape:
three constructors, nullable read-only `Message`, read-only `IsError`, and a
non-repeatable, non-inherited usage policy on types and members (not assemblies,
modules, parameters, return values or generic parameters). See the
[.NET contract](https://learn.microsoft.com/en-us/dotnet/api/system.obsoleteattribute?view=net-10.0).

Raven consumes the metadata during compilation. Default/message-only annotations
produce `RAV0505` warnings; `error: true` makes use an error. An annotation does not
remove a method, change its execution or impose a runtime check. Native AOT and
interpreted consumers execute the same retained implementation. Attribute data is
inspected without invoking its constructor.

`String.GetUtf8ByteCount` and `String.SliceUtf8` warn because byte operations belong
to explicit encoding/buffer APIs. The full replacement surface is still planned;
the messages deliberately do not direct callers to a nonexistent replacement.
Existing runtime consumers are migration debt and continue compiling with warnings.

The modern .NET `DiagnosticId` and `UrlFormat` customization properties are not yet
implemented. This is an API/compiler gap, not different obsolete semantics.

Run `scripts/validate-runtime-obsolete.py` with matching `--compiler`, `--libraries`,
`--runtime`, `--aot` and a fresh `--output`. It verifies constructor properties,
nullable messages, an imported obsolete string member, warning/error emission,
invalid and repeated usage, and interpreted/native macOS ARM64 execution.
Windows execution remains qualification work; metadata has no OS-specific layout.

Compiler prerequisites: shared Int32 flag-folding fix `76fb04c76` and native
nullable attribute-import fix `0a5e145a0`. The native integration branch includes
both. The first prevents Int64 accumulator values from entering Int32 metadata;
the second matches `string?` constructors to physical String attribute payloads.

[Retained validation](validation.json) passes compiler/interpreter checks and
macOS ARM64 AOT execution. The first AOT harness omitted the user-fault binding
needed by SliceUtf8's existing terminal-fault branch; the report reuses unchanged
compiler/interpreter evidence and records the corrected native invocation.
Windows CI runs portable diagnostics/constructor checks with `--skip-native`;
Windows AOT execution is not yet claimed.
