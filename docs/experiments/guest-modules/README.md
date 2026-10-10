# Guest logical modules (development, 2026-10-10)

The assembly `CoffeePackage` contains the flat module names `Acme.CoffeeMaker`,
`Acme.CoffeeMaker.Empty`, `Acme.CoffeeMaker.Factories` and `Acme.CoffeeMaker.Tests`.
Dotted names are a convention; no parent modules are synthesized. The namespace
need not match the packaging assembly.

The interpreter consumer starts at RuntimeContext.Current.ExecutingAssembly. It
checks ordinal enumeration, the explicit empty module, exact-name type membership,
assembly ownership, and consistent type/method/parameter module descriptions.
Nested-type ownership and cross-assembly scoping also have focused Rust tests.
Metadata name ordering matches the host reader's UTF-16 ordinal ordering; it does
not change the platform's ordinary UTF-8 String comparison contract.

The native consumer checks names from `NominalTypeInfo.Module` for two explicitly
retained type definitions, plus repeated queries, Module.Assembly.Name and agreement
of Assembly.FullName across two different modules in the same assembly. The private reflection-roots
policy uses schema 4 with explicit `moduleCatalogs` and all type execution flags false; obtaining module metadata grants no
construction or invocation capability. An otherwise identical executable retaining
only one of the two types must fault with `native logical module metadata was not
retained`. The factories use the interpreter's metadata recipe and GC-managed
RuntimeModuleInfo/RuntimeAssemblyInfo providers. Module.Assembly validates the exact
retained assembly/module pair; Assembly.Name validates the retained full identity.
The native consumer also enumerates all four declared modules through Assembly.GetModules,
including the explicit empty module and the module containing only functions, and
checks each owning assembly. Dropping only moduleCatalogs must fault with
`native assembly module catalog was not retained`. These catalog names do not retain
contained types or executable bodies. Schema checks reject incomplete legacy
projections and unknown/duplicate identities; an explicit empty table is valid.
The native probe deliberately does not claim support for Module.GetTypes,
Assembly.GetTypes or Object.Equals dispatch.
Guest module function enumeration and general reflective callable invocation remain
open; this is not in-process test discovery yet.

The application now checks RuntimeContext.Current.ExecutingAssembly. A separate
Coffee.Context library queries its own assembly through a helper; the application
calls that helper directly and through a lambda callback, then checks its own
identity again. This second executable uses no reflection-roots policy. The harness
uses the backend's explicit dependency load set and the first build's platform linker
commands: the project driver itself still admits bundle libraries only. It does not
claim new arbitrary-project-dependency support in that driver.

A native query retains only its caller's assembly identity/name. A module catalog
remains a separate requirement. The private lowering preserves callvirt, relocates
branches/source positions, and rejects changed runtime facades or binding the getter
itself as a function. Unit checks also cover foreign same-name caller scopes.
The .NET baseline uses a separate library and a Func callback to Assembly.GetExecutingAssembly;
see [the recorded comparison](dotnet-context-validation.json).

Run with a matching source-built Runtime/Data/Networking/Web bundle and rebuilt
interpreter/AOT tools:

```sh
python3 docs/experiments/guest-modules/verify.py \
  --bundle /absolute/path/to/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --runner tools/aot-poc/target/debug/neoclr \
  --output target/guest-module-check
```

The output directory must be new. On Windows use the `.exe` tool names and an
MSVC developer environment. The Windows native collections action runs this same
gate. [Local validation](validation.json) records actual commands, output, executable
hashes, fixture hashes and the qualified boundary; Windows qualification is pending.

Migration: rebuild consumers after removing ModuleInfo.MetadataToken. Logical
modules do not have physical metadata-row tokens; remaining definition tokens keep
physical scope semantics. The public [API reference](../../../api-docs/introspection.md#logical-modules-development-2026-10-10)
describes this distinction. The reference-only bridge and source-built class libraries
rebuild successfully. The independent legacy CLI bootstrap regeneration still stops
at `Incompatible map contract: Map`; its checked ModuleInfo fragments receive only
a documented mechanical deletion of the removed getter/property, not a claimed clean
regeneration of the legacy library.
