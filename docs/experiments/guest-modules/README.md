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
policy uses schema 1 with `construct: false`; obtaining module metadata grants no
construction or invocation capability. An otherwise identical executable retaining
only one of the two types must fault with `native logical module metadata was not
retained`. The factories use the interpreter's metadata recipe and GC-managed
RuntimeModuleInfo/RuntimeAssemblyInfo providers. Module.Assembly validates the exact
retained assembly/module pair; Assembly.Name validates the retained full identity.
These descriptive ownership facts do not retain an assembly-wide declaration catalog.
The native probe deliberately does not claim support for Module.GetTypes,
assembly-wide traversal or Object.Equals dispatch.
Guest module function enumeration and general reflective callable invocation remain
open; this is not in-process test discovery yet.

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
