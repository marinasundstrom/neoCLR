# Native introspection compilation prerequisites — 2026-10-05

The production descriptor/JSON sources now bind using native source adapters under
development. This slice admits their nongeneric sealed interfaces, preserving the
existing native closed-family flag and ordinary CLI Interface/Abstract projection.
Like Raven's .NET sealed-hierarchy attributes, closure constrains direct children rather
than forbidding all inheritance. Unlike CLR custom-attribute enforcement, native linking
checks direct relationships against module/revision ownership. No format change is needed.

Definition and builder paths share validation. Metadata introspection enumerates direct
interface children as well as class children. Open branches remain extensible; direct
external implementations and derived interfaces reject. Generic closure remains outside
the authoring profile. Existing class behavior is preserved.

Raven also admits static extension methods through existing lowering and authoring.
The configured primitive core's exact RuntimeServices.TypeHandle<T>() marker lowers to
the existing type-token instruction, including method type parameters. It is a temporary
CLI compiler contract, never an executable default-returning runtime implementation.
Future source-built core ownership must replace this marker. Ordinary .NET paths remain
unchanged. Neither change shares importer objects with emission.

## Reproduction and scope

Use the matching Probe --reference-comparer-storage-core bootstrap, then run:

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_introspection_prerequisites.py \
 --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
 --core /path/to/Core.dll --seed /path/to/System.neox \
 --ownership /path/to/ownership.json --base-library /path/to/Numbers.dll \
 --output /tmp/introspection-prerequisites
```

Both native consumers verify and return 42 with empty stdout. One executes interface
dispatch; the other compares concrete/open-method type tokens and invokes a static
extension. C# definition/builder round trips and runtime ownership checks also pass.
[Executable commands and hashes](introspection-prerequisites-evidence-2026-10-05.json).
149 C# metadata groups and seven focused runtime ownership/constructor tests pass.
The guest snapshot check still reports the previously recorded stale snapshot; this
host-only metadata change does not refresh an incomplete guest bridge.
Guest public API snapshots and website capability claims do not change in this slice;
the host metadata API has its manual reference entry.

The full production build has advanced to class Object overrides. Runtime descriptor
factory materialization is still incomplete. These prerequisites are not a completed
JSON mapping gate.
