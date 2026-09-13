# CLI-style no-result methods

Recorded 2026-09-12. The Raven experiment should primarily improve neoCLR's compatibility,
with compiler changes limited to generally useful facilities. This slice adds a runtime
return convention rather than requiring Raven to synthesize inhabited Void values.

## Runtime contract

Function metadata now has `no_result: true`, paired with `returns: "Void"`. The marker
means that a successful call produces no evaluation-stack value. It is not a new type,
a nullable value, or a function that never returns. Omission defaults to false, preserving
existing artifacts and the inhabited Void convention. Old strict readers reject artifacts
containing the new field instead of silently executing different stack semantics.

In assembly syntax the explicit experimental spelling is:

```text
.module Calls
.entry Main
.function Main() -> Int32
    ldc.i4 42
    call Empty()
    ret
.end
.function Empty() -> noresult
    ret
.end
```

`Main` returns 42: `Empty` leaves the value already on Main's stack untouched. The
existing `call` and `ret` instructions select their behavior from the method definition;
no new instruction is introduced. A no-result `ret` requires an empty evaluation stack.
Both the verifier and interpreter enforce the distinction. Frame cleanup still runs.

The supported subset now includes static/free and ordinary nominal-class instance IL
methods, including class constructors, and static/free generic IL methods. Bodyless
nominal interface contracts also support no-result signatures. Generic class methods,
virtual class/delegate contracts, InternalCall and P/Invoke declarations cannot opt in yet. Delegates with inhabited Void results cannot bind no-result targets.
These bounds are explicit validation rules, not claims that CLR no-result methods have
those restrictions. Neo source-language projection is not implemented by this slice.

The host `Execution` envelope still uses `Value::Void` for a no-result entry point; this
is not pushed onto a guest caller's stack. `LoadedFunction.has_return_value()` and the
reachability report expose the distinction without changing existing `returns()` callers.
A future hosting API can represent absent results directly if needed.

## Why this belongs in neoCLR

CLI methods can return no value; their return transfers no value onto the caller's
stack. See Microsoft's [`ret` contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ret?view=net-10.0)
(consulted 2026-09-12) and ECMA-335 Partitions II/III. This is separate from language-level
Unit values or neoCLR's inhabited Void type.

The earlier experiment proposal inserted Void construction/disposal around every CLI
no-result call and return. Direct runtime support better preserves standard stack behavior
and avoids unnecessary compiler or import rewrites. Its cost is an additional method
contract to carry through validation, dispatch, tooling and hosting. This slice carries
it through static calls, verification and reachability, while rejecting other forms until
they are implemented. No performance improvement is claimed.

Existing System APIs still return inhabited Void. A no-result method can call them and
explicitly `pop` that result. A focused test prints through the real System Console in
this way. The future binary loader must bind library signatures deliberately: it must
not label an existing inhabited-Void implementation as no-result without an appropriate
runtime adapter or API migration. Direct calls between imported no-result IL methods
will no longer require inserted stack operations.

## Validation and remaining work

Focused tests execute nested calls, preserve a caller result, call System Console,
round-trip metadata, and reject bad stack states or inconsistent metadata. Existing
CLR-style binaries are not loaded yet. PE reading, input tokens, generated-helper admission
and real System reference binding remain the next compatibility work. Raven is unchanged.

Validation on 2026-09-12: 45 tests passed across `no_result`, `delegates`,
`function_generics` and `verifier`. The sample sweep
`verifier_accepts_all_samples_without_executing_them` was excluded from the final
focused run after reproducing its existing assembly failure on unchanged commit
`148aad9`: `Snapshot<String>` reports an unknown generic type `System.Clonable`.
The complete test suite was not completed; this is bounded regression evidence.

## Void as a type, and future async conventions

The author clarified that the purpose of inhabited Void is its use in generic types
and other value positions; it does not require changing ordinary CLI call/return behavior.
`Box<Void>` and generic `Identity<Void>` remain valid value operations, independently of
whether a particular method has a no-result signature. Tests cover both alongside the
new mode. The marker added here is internal metadata for the existing interpreter model,
not a proposed new CLI flag or opcode. A future reader can derive it from a standard
CLI void return signature. This must be decided from the declared signature, not
inferred after generic substitution: a generic value-returning method instantiated
with Void still returns its value. Generic Void encoding remains a separate binary-format question;
this slice does not claim that CLR accepts Void in all generic/value signature positions.

The [dotnet/runtime runtime-async design](https://github.com/dotnet/runtime/blob/main/docs/design/specs/runtime-async.md)
(consulted 2026-09-12, document labels itself a draft) illustrates a related separation:
Task/ValueTask remain declared return types, while marked async bodies return an empty
stack or the underlying generic result. It uses explicit metadata and defined suspension
rules; it is not an arbitrary relaxation of return checking. This is a useful future
comparison for neoCLR, not an implemented async contract or a claim of standardized CLI
support. Keep declared type, body return convention and runtime implementation separate
when designing suspension. No async method flag is accepted by this slice.

The author also set later alignment with .NET value-type/reference-type semantics as the
intended direction. The current internal return marker preserves a narrow distinction
while that migration is planned; it is not a commitment to retain the original type model
or export a new CLI return flag. Generic Void remains an intentional extension to examine
at that boundary, not a reason to add general compatibility workarounds.

## Subsequent author clarification (2026-09-12)

Void is a real neoCLR type and targeting compilers must adapt to that contract, even
when it contributes no evaluation-stack value. Type participation and physical stack
representation are distinct. The existing inhabited generic Void representation described
above is current behavior, not a requirement that every Void use occupy a stack slot.
Unifying that representation across generic calls, storage and compiler lowering remains
separate implementation work; the no-result slice has not completed it.

The [class-constructor follow-up](class-semantics.md#constructor-and-field-store-follow-up-2026-09-12)
uses no-result constructor bodies while `newobj` independently yields the allocated object.

## Generic static calls (2026-09-13)

Static generic methods may declare a no-result return independently of their type
parameters. Specialization retains the return convention: `Ignore<Void>(Void)`
consumes its unit argument without producing a caller-stack result, whereas
`Identity<Void>(Void) -> T` still produces a unit. This removes an implementation
restriction and reuses the CLI call/return distinction described above. No new
opcode or CLI metadata flag is needed. Tests cover caller-stack preservation,
multiple specializations and rejection of a value left at a no-result return.
