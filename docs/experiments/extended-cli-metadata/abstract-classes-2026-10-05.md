# Ordinary abstract class metadata prerequisite

Date: 2026-10-05. Development branch: `codex/extended-cli-metadata`, based on
`1dd5b090`. Raven `codex/metadata-consumer` remains `2b683df67`; no compiler change.

The unchanged `application-inheritance` baseline is blocked at declaration admission.
This slice repairs the prerequisite metadata shape: the reader previously inferred
static from Abstract alone. As on CLR, static classes require Abstract plus Sealed;
ordinary abstract classes may contain concrete members and instance constructors.
The existing native `is_abstract` / `is_sealed` fields suffice. Closed-family and native
Object-root contracts are unchanged. This reuses the existing CLI flag contract; it
introduces no new semantics or format version.

## Validation

C# `ClassBaseAuthoringChecks` covers builder/manual definition parity, public/protected
constructors, concrete/abstract bases, native PE read/introspection and CLR execution.
A concrete subclass initializes inherited and owned fields and returns 42. Direct
abstract construction fails both writers. Unsupported mutators leave flags unchanged.
Sealed and closed-family controls run alongside the changed reader checks.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0 -- --class-base-checks
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0 -- --class-base-runtime target/release/neoclr /tmp/abstract-class-runtime1005
```

The runtime command verifies and executes four API-produced PE assemblies (concrete
or abstract base, public or protected constructor). Each runs with exit 42 and empty
stderr. Tests create unique assembly identities; no external seed or source library
is required. The CLR controls execute the equivalent authored graph.

## Remaining work

General virtual slots, abstract methods, overrides and inherited interface mappings
must be supported together before Raven admits the original sample. No claim is made
that that sample now compiles or that the editor release gate is complete. Public host
API documentation is in `api-docs/experimental-metadata.md`; no guest reference snapshot
or website capability claim changes are required for this prerequisite.
