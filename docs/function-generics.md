# Generic functions and static methods

neoCLR supports function-level generic parameters on concrete free functions and
static IL methods. They are independent of the declaring type's parameters.
This fills a platform gap relative to CLR generic methods; it does not make functions
into objects or introduce delegates.

```text
.function Identity<T>(T value) -> T
    ldarg value
    ret
.end

.function Main() -> Int32
    ldc.i4 42
    call Identity<Int32>(Int32)
    ret
.end
```

The same declaration works as `.method static Identity<T>(T value) -> T` inside a
type, called with `Helpers::Identity<Int32>(Int32)`. Module-level function names can
be namespace-qualified. Explicit call signatures contain substituted parameter types.

## Metadata and execution

Function definitions carry a `generic_parameters` name table; its length is arity.
`MethodTypeParameter(index)`, spelled `!!0` in IL, is separate from owner
`TypeParameter(index)` (`!0`). Named aliases bind in the method scope before the
owner scope. A method alias can shadow an owner alias; indexed signatures remain
unambiguous. Names do not affect identity. Function references carry `generic_arguments`;
method arity participates in overload identity. Definitions and call sites remain
ordinary metadata, with no new invocation opcode.

Substitution replaces both namespaces simultaneously so references to a caller's
parameters cannot be captured by the callee's parameters. Calls specialize signatures,
locals and type operands. Symbolic bodies remain verifiable; concrete operations
still enforce their runtime requirements. Substitution rejects structurally invalid
results such as nested managed references. Type nesting and closed graph limits
remain enforced. Call graphs distinguish instantiations even if the type argument
appears only in the body or return. Host resolution accepts explicit closed arguments;
stack trace targets retain them.

Value arguments copy according to existing storage semantics. `T = Int32&` passes
the reference as a value and preserves its owner and readonly/lifetime checks. `T&`
borrows a T slot. These are different signatures; instantiating `T&` with another
managed reference is invalid. Void remains a valid type argument under neoCLR's
existing generic policy. Generic functions do not extend frame lifetimes or permit
returning a reference to a local slot.

The initial slice excludes generic instance/virtual/interface methods, native imports,
internal calls and generic entry points. Constraints remain future work; type inference belongs to frontends. Reflection has no open method-parameter type identity yet: selecting a generic
method through GetMethods faults explicitly rather than returning an incomplete
MethodInfo. Host resolution and metadata inspection are available; guest
MakeGenericMethod and generic method-definition reflection are separate work.

Existing artifacts omit the new optional tables and remain readable. New generic
function artifacts require this runtime. Rust Function/FunctionRef literals must
supply the new fields; resolved Function generic arguments are execution metadata
and are not serialized back into definitions.

## .NET comparison and validation

Primary sources consulted 2026-09-08:

- [Microsoft's generic methods guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/generic-methods)
  describes method parameters independent of their declaring type, explicit calls,
  compiler inference and overload arity. Inference belongs to C#, not the runtime.
- [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
  Partition II §23.2.1, §23.2.12 and §23.2.15, specifies generic method signatures,
  method instantiations and the VAR/MVAR distinction. neoCLR uses structural JSON
  signatures rather than the CLI's binary encodings.

The pinned [comparison probe](experiments/function-generics-dotnet/Program.cs), SDK
10.0.100/net10.0, checks independent owner/method parameters with reflection and
explicit/inferred C# calls. Run from its directory with `dotnet run --project Probe.csproj`.
It prints `Independent owner/method parameters; explicit and inferred calls passed.`

Adopting distinct runtime method parameters supports reusable APIs and future delegate
binding across languages. Compiler-only cloning would avoid metadata work but hide
generic definitions and duplicate backend policies. Erasing every T into Object would
introduce conversions and conflict with explicit addressing. The selected design costs
specialization work and larger bounded call graphs; it claims no measured speedup.
Unlike ordinary .NET generic arguments, neoCLR accepts managed-reference and Void
arguments under its existing policy; the cost is enforcing valid substituted shapes
and managed lifetimes. The bounded static/free slice leaves generic dispatch and
reflection incomplete, rather than implying CLR parity.

`tests/function_generics.rs` covers artifact round trips, normalization, distinct
parameter scopes, symbolic forwarding, reference lifetime checks, host invocation,
closed graph identity, overload arity and invalid contexts/substitutions. Existing
owner-generic, dispatch and reachability suites remain regression coverage.
