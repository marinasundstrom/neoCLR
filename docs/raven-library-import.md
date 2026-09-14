# Separate Raven libraries: static-method checkpoint

Source experiment, 2026-09-14. Ordinary Raven compilation can now produce a library
DLL and a separate consumer DLL whose reachable nongeneric static library methods
are imported into one neoCLR program. The System-library migration remains paused.

## Build and import

Use the target configuration in [the compilation workflow](raven-target-compilation.md)
for both projects. Set the library project's `OutputType` to `Library`, and set its
`AssemblyName` explicitly. Give the consumer a normal metadata reference to that DLL:

```xml
<Reference Include="ArithmeticLibrary">
  <HintPath>/path/to/ArithmeticLibrary.dll</HintPath>
</Reference>
```

Compile each project using ordinary `rvnc`; build dependencies first. Supply the
executable DLLs explicitly when importing the consumer:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --import /path/to/Consumer.dll /path/to/NeoCLR.CoreProbe.dll /path/to/new-output \
  /path/to/ArithmeticLibrary.dll /path/to/FacadeLibrary.dll
```

Dependency order does not select resolution priority. Exact assembly identities must
be unique and every referenced assembly must be supplied. There is no host-framework
or sibling-directory search. The existing metadata closure audit checks member
resolution; the importer then validates reachable supported signatures and bodies.
Each image is limited to 16 MiB, with at most eight additional library images and
128 reachable imported methods in total. Multi-module assemblies and type forwarding
remain unsupported.

Verify and execute `new-output/App.neoil` against the matching System library as in
the compilation workflow. The importer still produces a combined textual program;
this is not dynamic DLL loading by the runtime. Each dependency passes through the
existing temporary Void projection before auditing/importing. Failed imports can
leave projected intermediates, but do not produce `App.neoil` on validation failure.
Use a fresh output directory for each attempt.

## Supported boundary

- Nongeneric static methods, including calls between libraries, recursion and normal
  calls to supported core-library APIs.
- Cross-assembly calls require public methods and publicly visible enclosing types.
  Existing same-type private calls remain possible. This is not a complete CLI
  accessibility implementation (`internal`, friend assemblies and protected access
  are not added).
- A library's own calls to Raven namespace functions use their existing emitted CLI
  container. Identity comes from metadata; no guessed container name or new
  namespace-function ABI is used. Direct cross-assembly wildcard import is not yet
  working in this probe: `import Workflows.*` followed by `Answer()` produced RAV0103
  in the consumer. A public static facade calling Answer inside the library works.
  This is a Raven metadata-discovery follow-up, not an importer workaround.
- Existing supported scalar/closed signatures and instruction rules apply. Static
  initializers, exception handlers and unsupported instructions continue to fail.

Library-owned instance types, constructors, generic method/type bodies and library
method-group/delegate targets remain outside this slice. Passing existing core generic
values is not evidence of generic-body importing. Reference-only libraries with no
executable method body are not implementations; reachable missing bodies are rejected.

## Identity and debugging

The [assembly/signature symbol encoding](raven-import-identities.md) is unchanged.
The importer now tracks method definitions from their individual assemblies instead
of treating a row token as globally unique. Output labels use import-local method IDs.
Sidecar method and instruction mappings include assembly identity plus metadata token;
`OutputLabel` identifies the translated instruction label directly. Dependency hashes
refer to the projected input images copied into the output directory.

This changes the source-map schema: `ReachableMethods` entries now contain assembly
identity and token, rather than bare tokens. Consumers must use `OutputLabel` instead
of reconstructing a label from the token. Rebuild generated IL/maps together. The
source bridge and verification scripts are updated; installed SDK/VSIX artifacts are
unchanged.

## Comparison, decision and evidence

This follows the CLI distinction between assemblies as identity scopes and tokens as
module-local references. See the [ECMA-335 comparison](raven-import-identities.md#problem-and-baseline).
Keeping Raven's ordinary metadata references and emission avoids a compiler-side
per-library catalog. Explicit closure resolution remains more restrictive than .NET
loading: no probing, unification, forwarding or runtime loading is claimed.

Admitting only static bodies is a provisional capability slice, not the desired final
library model. It gives separate-compilation evidence without simultaneously changing
instance type ownership, generics and dispatch. A full general importer is the next
step; adding individual API translation entries would not establish that capability.
The cost is a bounded import/link stage and projected intermediate DLLs.

Run the source regression suite with a generated target project:

```sh
python3 docs/experiments/raven-target/verify_library_import.py /path/to/Demo.rvnproj \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime /path/to/neoclr --system /path/to/System.Collections.neoil
```

It independently compiles two libraries and a consumer. The positive program combines
an internal namespace-function call through a public static facade and recursive calls, checks reused method tokens
across assemblies, executes with reversed input order, and checks source-map labels.
Negative cases cover missing/duplicate dependencies, an implementation with a private
method, a changed signature and unsupported library instance construction. Existing
application and normal compiler/import suites cover the retained single-program path.

Checkpoint results: seven library checks, 15 application checks and five normal
compiler/import checks passed. Raven source and installed tools were not changed.
