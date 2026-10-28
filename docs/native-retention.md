# Project retention files

Development tooling, added 2026-10-28. The native project builder automatically
reads `App.retention.json` beside `App.rvnproj` (or the corresponding project stem).
Check this file into source control with the project. No per-type command-line
arguments are needed.

```json
{
  "schemaVersion": 1,
  "types": [
    {
      "name": "Examples.DependencyInjection.Greeter",
      "retain": ["constructors"]
    }
  ]
}
```

The [DI sample](experiments/dependency-injection/sample/README.md) uses this file
and is qualified as a native executable. To select another file, pass
`--retention-file config/native.json`; relative paths resolve against the project
directory, independent of the shell's working directory. An explicit file replaces
the conventional file. An absent conventional file preserves existing behavior;
an explicitly requested missing file is an error.

## Supported declarations

`schemaVersion` must be the integer `1`. `types` contains 1–64 distinct exact
metadata names. Each entry has only `name` and a nonempty `retain` array. Modes
can be combined for the same type or across different types:

| Mode | Retained capability |
| --- | --- |
| `constructors` | Public constructor metadata and supported checked constructor invocation, equivalent to `--reflection-constructor-type`. |
| `properties` | The existing object-mapping policy: construction, property metadata, getters and setters, equivalent to `--reflection-type`. It is not metadata-only retention. |
| `methods` | Declared method inspection, equivalent to `--reflection-method-type`; it does not enable method invocation. |

Unknown keys/modes, duplicate JSON keys, duplicate names/modes, malformed JSON and
unsupported versions fail. The builder rejects combining the selected file with
any legacy reflection-policy option rather than silently ignoring one policy.
Scalar interop/library profiles continue to reject managed reflection retention.

Names are resolved from the final compiled application, with supplied class-library
modules searched when the application has no match, following the existing named
root resolver. Missing or ambiguous matches fail. Application names take precedence;
assembly-qualified selection and project-reference library discovery are not added
by this file format. No wildcards or blanket keep-all mode are supported.

The builder combines declarations into private schema-8 `app.reflection.json` in
its output directory and passes that policy to the backend. Constructor/property
flags are merged by exact definition identity; method metadata remains separate.
The source file and generated policy are hashed in `build.json`, and the normal
input-change check runs before executable publication. Do not hand-maintain compiled
module/revision/index identities in the project file: they can change on rebuild.

## Generics and current boundaries

An exact definition such as `System.Localization.DefaultLocalizer` followed by
its metadata arity suffix `` `1 `` can use `constructors` retention. As with the
existing command-line option, only supported closed shapes observed by the native
specializer are retained. This does not enable arbitrary runtime generic
construction or runtime code generation. Explicit closed-generic instantiation
syntax, field policies, assembly catalogs and individual member selection are
not part of version 1; the private backend policy remains available separately.

This support belongs to `build-native-project.py`, including newly assembled build
kits containing that script. It does not change Raven's project evaluator or make
`neoclr run` discover project files. JIT still accepts the private compiled-identity
`--reflection-roots` file explicitly. No new managed API, compiler mapping or CLI
metadata encoding is introduced. Existing bridge limitations remain documented in
[the CLI bridge](raven-cli-bridge.md).

## .NET comparison and annotation follow-up

.NET supports external [trimmer root descriptors](https://learn.microsoft.com/en-us/dotnet/maui/deployment/trimming?view=net-maui-10.0)
and [DynamicallyAccessedMembers contracts](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/fixing-warnings).
The external file serves a similar purpose here. JSON deliberately reuses neoCLR's
existing retention pipeline and bounded capability policies; it avoids XML/MSBuild
integration for this slice, at the cost of a neoCLR-specific format and less granular
member selection. The compiled backend format stays private.

`DynamicallyAccessedMembers` was not implemented by the initial file-support change.
A subsequent [parameter-contract slice](dynamically-accessed-members.md) adds
constructor/property analysis and selected runtime API annotations; the remaining
work below is not a claim of complete support. Repository
inspection found no existing implementation in the runtime/Raven library. The
[.NET attribute](https://learn.microsoft.com/dotnet/api/system.diagnostics.codeanalysis.dynamicallyaccessedmembersattribute)
communicates member requirements through type-bearing locations and generic
parameters. A class declaration alone would not supply that analysis. A follow-up
must qualify attribute transport, supported member flags, propagation through
parameters/returns/fields/generic arguments, inheritance rules, diagnostics for
unprovable requirements and AOT retention from proven flows. It must include
negative cases and matching managed API documentation. Until that exists, explicit
project retention is the working contract; annotations must not be presented as
protection from trimming. Library-provided retention requirements remain separate.

**Author direction, 2026-10-28:** When the attribute is implemented, annotate the
runtime APIs that benefit from it in the same work. Audit introspection member
queries, DI registration/activation and JSON object mapping, propagating the
narrowest requirements through public generic parameters and type-bearing
parameters/returns into internal helpers. Concrete starting points include
`ConstructorPlan.Prepare(TypeInfo, ...)`, `JsonSerializer.Deserialize<T>` and its
`TypeInfo` overloads. Do not put an ineffective annotation on an arbitrary `Object`
parameter: object-based serialization needs an analyzable contract or an explicit
unsupported-flow diagnostic. Validate callers with and without sufficient
requirements, including that proven flows retain members without a project-file
entry; document each API's contract in its reference entry. This is planned work,
not an annotation implementation in the current change.

## Validation

[Recorded qualification](validation/project-retention-20261028/README.md) covers
file validation, fresh identities, combined policies and a native DI consumer.
