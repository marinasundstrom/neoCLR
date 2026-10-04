# Source-declared native runtime services — 2026-10-05

Raven now compiles explicitly marked internal service declarations directly to the
[metadata InternalCall contract](internal-call-authoring-2026-10-05.md). The runtime
library owns `runtime/raven/native/RuntimeHandleServices.rvn`; its TypeArgumentCount
and ObjectTypeHandle declarations execute through a separate native consumer.
No descriptor duplicates or new runtime type representation are needed for this gate.

## Boundaries and comparison

The adapter accepts internal nongeneric bodyless functions in `neoCLR.Runtime` with
exactly the configured core's MethodImpl(InternalCall) attribute. It consumes the
bound marker into implementation flags and exact service names. Local calls use the
normal semantic symbols and output-owned definitions. There is no importer reuse.

This reuses the .NET MethodImpl attribute/flag shape described in the preceding slice.
Unlike arbitrary desktop CLR assemblies, NeoCLR explicitly binds these declarations
to its known runtime service table. Unknown names/signatures still reject; declarations
do not install implementations. The .NET backend and extern/PInvoke defaults are
unchanged. Public services, type-owned services and generic services remain unsupported
in this compiler slice. Public source wrappers are imported normally by consumers.

The temporary CLI core contains compiler-facing MethodImplAttribute and the used
MethodImplOptions constants. Its constructor is metadata-only, never an executable
runtime dependency. These markers must eventually belong to the source-built core;
this does not move runtime descriptor ownership into bootstrap. The native source
file is separate from the legacy bridge inputs. No guest public API or API snapshot
changes; website production claims remain unchanged.

## Validation

Build the matching bootstrap with the existing Probe `--reference-comparer-storage-core`
command, then run:

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_internal_calls.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /path/to/InternalCallCore.dll --ownership /path/to/ownership.json \
  --base-library /path/to/Numbers.dll --output /tmp/native-service-gate
```

The tool uses an explicit executable Object/RuntimeTypeHandle seed. Numbers supplies
compiler primitive/iteration identities through the selected ownership manifest;
this small program does not call its methods and does not need it at execution.
The provider allocates an object, calls the runtime's ObjectTypeHandle and
TypeArgumentCount implementations, and returns 42. The separately compiled consumer
uses only its artifact. Verification returns 0, execution 42, stdout is empty.
Unmarked externs, public declarations, wrong namespaces, generic services, wrong
implementation flags and extern bodies reject without output. Unknown service names
reject at runtime verification. Four ordinary .NET extern/PInvoke tests pass.
[Commands, revisions and hashes](source-internal-calls-evidence-2026-10-05.json).

## Next dependency boundary

Production reflection services need signatures containing source-owned descriptor
interfaces and their factory implementations. Snapshot/value-vector representation
and Object.GetType/RuntimeContext ownership remain open. Moving an internal service
out of the seed also requires moving its seed callers; retaining those calls yields
an unresolved cross-assembly call. The focused minimal seed makes ownership explicit,
not a workaround to claim that production JSON mapping already works.

Compiler integration commit: `185146e41` on `codex/metadata-consumer`; metadata API
prerequisite: `aeb1cbb5` on `codex/extended-cli-metadata`. No shared compiler/main
backport is required for this target-adapter-only slice.
