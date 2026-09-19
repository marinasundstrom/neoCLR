# Separate Raven libraries

Source experiment, 2026-09-14. Ordinary Raven compilation can now produce a library
DLL and a separate consumer DLL whose nongeneric library types and supported method
bodies are imported into one neoCLR program. After Preview 7, the bounded
[shared System project and Math migration](raven-system-library.md) resumes ordinary
library authoring; the broader generic-body boundary still applies.

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
- Constructors, class/value fields, properties through accessors, abstract bases,
  inheritance, virtual/interface dispatch and static/class method-group delegates
  use the same bounded rules as application-owned types.
- Cross-assembly calls require public methods and publicly visible enclosing types.
  Existing same-type private calls remain possible. This is not a complete CLI
  accessibility implementation (`internal`, friend assemblies and protected access
  are not added).
- Raven namespace functions use their existing emitted CLI container and target-owned
  `TopLevelAttribute` marker. Direct cross-assembly wildcard imports now work with the
  refreshed core and Raven `neoclr` compiler. No guessed
  container name or new namespace-function ABI is used. The earlier RAV0103 probe
  exposed both a missing core marker and Raven's host-only marker lookup; these are
  fixed in source. Existing installed tools still require an update.
- Existing supported scalar/closed signatures and instruction rules apply. Static
  initializers, exception handlers and unsupported instructions continue to fail.

Generic method/type bodies, explicit/default interface implementations, value-type
interface implementations and value-type instance delegate targets remain unsupported. Passing existing core generic
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

The initial static-only slice established separate-compilation evidence. The current
instance-type slice reuses existing runtime layouts and dispatch; it does not add
per-library API translations. Generic bodies are the next capability to investigate.
The importer remains bounded and is not yet a general CLI loader.
The cost is a bounded import/link stage and projected intermediate DLLs.

Run the source regression suite with a generated target project:

```sh
python3 docs/experiments/raven-target/verify_library_import.py /path/to/Demo.rvnproj \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime /path/to/neoclr --system /path/to/System.Collections.neoil
```

It independently compiles two libraries and a consumer. The positive program combines
a direct imported namespace-function call and recursive static calls, checks reused method tokens
across assemblies, executes with reversed input order, and checks source-map labels.
Negative cases cover missing/duplicate dependencies, an implementation with a private
method and a changed signature. The later instance checks below cover the expanded boundary. Existing
application and normal compiler/import suites cover the retained single-program path.

Checkpoint results: seven library checks, 15 application checks and five normal
compiler/import checks passed. This records the initial static-library checkpoint;
the namespace follow-up below changes Raven source. Installed tools remain unchanged.

## Namespace metadata follow-up

The namespace marker is a compiler metadata declaration, not an executable neoCLR
attribute API. Regenerate the core, rebuild libraries and consumers, and use the
matching new Raven compiler and bridge. Adding the marker alone was insufficient:
Raven now resolves it from supplied metadata, and completion accepts imported members
without source declarations. The namespace probe covers marker scope, closure audit,
overloads, completion, inaccessible functions, disabled imports and unmarked lookalikes:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --namespace-members /path/to/new-namespace-probe
```

This follows Raven's existing CLI custom-attribute contract rather than inventing a
neoCLR-specific naming rule. The cost is requiring a truthful marker declaration in
the reference pack and matching compiler tools. Missing-marker diagnostics remain a
Raven follow-up; the emitter currently omits an unavailable marker. This does not
resume the System-library migration or expand the generic/instance-body boundary.

Namespace follow-up validation: 47 focused Raven tests, 54 imports-and-namespaces
tests, eight target namespace checks, seven separate-library checks and five normal
compiler/import checks passed. The test sets overlap; these are separate suite totals.

## Library-owned instance types

The importer now resolves application and library types from the explicit assembly
set. Assembly-qualified type identity distinguishes equal namespace/type names in
different libraries. Delegate helper identities also include their assembly/signature
scope. `TypeIdentities` in the source-map sidecar now includes `AssemblyIdentity`.

Cross-assembly type visibility is checked on method signatures, locals, instruction
operands and inherited contracts. Field/method/constructor access retains the existing
public-or-same-declaring-type rule; this does not add general protected/internal access.
Checks operate on metadata before execution, including when an implementation DLL has
been replaced after compiling the consumer. Core reference metadata remains distinct
from executable guest libraries.

This follows the CLI separation of assembly ownership from type layout and dispatch,
using the existing [identity comparison](raven-import-identities.md#problem-and-baseline).
There is no new runtime opcode or Raven compiler change in this slice. The benefit is
reuse of normal separately compiled class/value APIs. The cost is validating visibility
and traversing definitions across the supplied assembly set; eager import of all
instance methods of an admitted type can still reject unused unsupported bodies.

Validation covers a library abstract base/interface and a derived class in a second
library, constructor chaining, inherited property access, virtual/interface calls,
method-group delegates surviving GC, struct copies and collection references. Another
case imports two libraries that independently define `Shared.Item` and verifies their
different behavior. Reversed input order passes. Replaced implementations with a
hidden base type or private base constructor fail, and generic bodies remain rejected.

Current results: 13 library checks, 15 application checks and five normal
compiler/import checks passed. Rebuild generated IL/maps with this source bridge;
installed tools and the paused System-library migration are unchanged.


## MSBuild application/library workflow

The [standalone MSBuild build](raven-msbuild.md#a-referenced-library) now automates a
bounded version of this procedure. The application can name one library `.rvnproj`
with `ProjectReference`; MSBuild builds it first, Raven consumes its declared DLL,
and the importer receives the explicit dependency. The shipped two-project demo
includes a class call and a namespace function. Multi-level project graphs and
package restore remain outside this slice; the importer limits above still apply.
