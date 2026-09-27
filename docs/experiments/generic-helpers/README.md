# Closed generic application helpers

Development compatibility slice, after Preview 10. Ordinary unconstrained static
Raven helpers such as `Identity<T>(value: T) -> T` now import with closed arguments.
This closes an importer gap against Raven's normal contract; it is not a compiler
rewrite or a new Runtime Contract setting. Raven source and emitted CLI signatures
are unchanged.

## Contract and limits

`ApplicationSpecialization` builds one checked body per closed method instantiation.
It substitutes method parameters through return/parameter/local types, array and
constructed type arguments, member references and nested generic calls. Type-owned
parameters in member signatures remain owned by their constructed declaring type.
Cache identity includes assembly-qualified type arguments. Private visibility and
source-token debug origins remain intact; helpers receive no invented source token.

The bound remains static methods on nongeneric owners, one to four method type parameters,
no exception handlers, and at most 128 specializations. Arguments must be closed and
supported by ordinary import. Open arguments, byref arguments, instance methods,
generic declaring types and unsupported constraints remain rejected. Existing exact
Number<T> constraint admission and constrained numeric dispatch are preserved.
This is not a claim of arbitrary CLI generic support or reflective construction of
open generic methods.

The consumer covers nested calls with int, string and bool, two method type parameters,
shared application-object and vector identity, a generic ArrayList reader, and existing
Number<int>/Number<double> arithmetic. Metadata checks cover distinct signatures,
cache reuse, same-named arguments from distinct assemblies, retained visibility/debug
origin and unsupported open/byref/constraint/instance shapes. `verify.py` also retains
source checks for additional Number constraints and a class constraint.

The Task.Run [generic capture fixture](../task-run/compiler-gaps/GenericCapture.rvn)
now passes helper specialization and reaches the unsupported constructed application
state-machine type. `verify.py` in that directory checks the specific import failure;
that program still does not execute. The next bounded slice is generic application
type import, starting with a small holder before synthesized state-machine/closure
admission. Raven's independent generic-containing-type async arity bug remains separate.

## .NET comparison and decision

The [.NET generic type/method model](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/generics)
retains generic metadata and runtime type information; the
[CLI standard](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
provides the interoperability baseline. Sources reviewed 2026-09-27. The existing
[numeric design](../../design/numeric-contracts.md) already chose bounded import-time
specialization. This slice extends that implementation to ordinary helpers while
preserving the familiar contract and reference identity.

Alternatives are retaining the numeric-only restriction or implementing general
runtime generic method loading now. Extending checked import admits useful Raven
programs without changing the compiler or native VM. Its cost is a separate body for
each admitted instantiation and finite import limits; it does not reproduce the
CLR's full runtime generic/reflection capabilities. No performance improvement is
claimed, so no benchmark is required for this correctness slice.

## Validation

Run `verify.py --runtime ... --bridge ... --system ... --reference ...` with the
matching development artifacts. It compiles the consumer, performs typed-stack
verification, runs it, and checks the retained constraint rejections. Run
`task-run/verify.py --case generic-capture-import-gap` separately to confirm the
next unsupported contract. No website build or full test suite is required.

Recorded result: all checks above pass with the artifacts pinned in
[validation.json](validation.json). The async import-gap check reaches the recorded
constructed state-machine rejection. The reference DLL is unchanged because this
slice changes application import only; its importer/XML fingerprints are refreshed.
