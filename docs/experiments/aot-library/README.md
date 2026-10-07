# Explicit AOT value libraries

**2026-10-07 — bounded development implementation on main.** The
[Raven library](library.rvn) and [application](app.rvn) compile separately to neoCLR
PE/#Neo metadata/IL, then into one standalone ARM64 executable. The application calls
an external constructor/member, mutates private storage through that member, and
checks independent value copies. Neither input DLL is needed at execution time.
The executable links only macOS libSystem; no shared managed framework/runtime is used.

## Reproduce

Use the pinned native Raven compiler revision
`70aea9a7e9e424159a48b0227869245a95bf2ec2` and the CoreProbe compiler reference recorded
in [validation.json](validation.json). The full native runtime-owned Core.dll bundle
has a different target contract; it is not interchangeable with this primitive probe.

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/aot-library/verify.py \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --core api-docs/reference/NeoCLR.CoreProbe.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-library-run
```

Select a compatible macOS SDK as in the [Hello World reproduction](../aot-hello/README.md).
The output directory must be new. The script compiles both sources, runs the interpreter,
compares inspection with emission selection, links the executable, checks object imports
and dynamic dependencies, then copies only the executable into an empty directory and
runs it with an empty environment. The report retains tool/source/artifact hashes and
commands. [App.pe](App.pe) and [Values.pe](Values.pe) are the exact tested fixtures.

```sh
neoclr-aot-poc --closed-world App.pe @entry app.o --module Values.pe
neoclr-aot-poc --inspect App.pe @entry --closed-world --module Values.pe
```

## Contract and validation boundary

- Explicit load sets accept one to eight dependency inputs, each without an entry point,
  and a root declared uniquely in the application. PE/#Neo and NEOX are tested. The
  existing standalone neoIL reader is a convenience, not a multi-source assembler;
  use native containers for applications whose field/type fixups require dependencies.
- Each input is limited to 16 MiB; the combined inventory is limited to 4,096 functions
  and 1,024 types. Selected code keeps the existing 128-function/32-type/inline-layout
  limits. Source modules and source assemblies must be distinct. No dependency discovery,
  dynamic loading, custom System/Object root, generic declarations or interface dispatch
  is supported by this first library profile. Single-module generic specialization is
  unchanged. Unknown external calls are rejected, never replaced with name-based intrinsics.
- Before relocation, the ordinary runtime loader and verifier check the **entire original
  load set**, using bundled System. This enforces reference lists/revisions, identities,
  signatures, initialization, borrows and private/internal/type/field accessibility in
  their original scopes. This is deliberately stricter than verifying only selected code:
  invalid unused bodies/dependencies also fail. Supplied source definition identities
  are checked before normalization can replace missing identities.
- Only after those checks does the compiler create its private canonical projection and
  select direct calls. All selected bodies still pass normal AOT admission and verification.
  Original files and source origins stay unchanged. Relocation cannot authorize a previously
  illegal cross-module access. The report maps compiled rows back to original module,
  revision and definition indices; `sourceIndex` is the concatenated inventory index.
  The projection is not serialized or exposed as public metadata, and its symbols are not
  a stable linking ABI.
- Inspection uses the same preparation. Its declaration/opcode inventory covers the
  application input; the selection report includes dependency rows and the explicit load
  set. Inspect a library separately for its complete declaration inventory.

Tests exercise interpreter/native equivalence, public library calls over internal storage,
PE/#Neo and NEOX fixtures, inspection/emission parity, and rejection of wrong revisions,
missing references/dependencies, invalid definition IDs, duplicate modules, dependency
entry points, library roots and cross-module internal method/type/field access. Rejections
create no object. No runtime/public library APIs changed.

## Comparison and next step

This extends the existing [.NET Native AOT comparison](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
and [metadata identity baseline](../../design/extended-cli-metadata.md). Like a closed-world
native publish, required library code is compiled at build time. The experiment uses the
existing neoCLR runtime binder/verifier before projection instead of inventing parallel
access rules or concatenating unchecked declarations. The benefit is reuse of executable
identity/access semantics; the cost is whole-load-set verification, a bundled-System
restriction and a deliberately small nongeneric profile. This does not claim general
.NET-style library compatibility, performance gains or general trimming.

Next: specialize generic values across this same verified boundary, then address the
actual System.Result propagation-interface contract. The [library Result probe](../aot-values/README.md#real-library-result-dependency-boundary-2026-10-07)
still needs generic runtime-library and interface support. UTF-8 console input/lifetimes,
multiple generic instantiations and the HTTP driver remain later work.
