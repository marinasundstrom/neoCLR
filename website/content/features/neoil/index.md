# neoIL: the assembler representation

neoIL is a **textual assembler representation of neoCLR metadata and bytecode
instructions**. The text shown here is not the binary or bytecode encoding. It
makes types, signatures and instruction bodies readable, and can be assembled
into a module for a program or library.

There are two routes to an assembly: tools and developers can produce this
assembler text, or a compiler such as Raven can emit the native module directly.
Raven does not need to generate and reparse this text to produce bytecode.
Both routes describe the runtime's metadata and execution contracts.

Reading the representation helps explain what survives compilation: a function's
structural shape, a generic parameter, a reference to storage, or a call with no
result. Raven remains the application language; assembler text is useful for
understanding the platform, compiler development and small runtime tests.

The relationship is similar to textual .NET IL and CLR assemblies. neoIL itself
produces **neoCLR assemblies**, not .NET CLR assemblies; Raven's separate .NET
target supplies the latter. The familiar stack-based mnemonics do not imply
ECMA-335 binary compatibility.

## A module and a function

This walkthrough applies a callback to 40 and returns 42. The opening declares the
module, its entry point and an ordinary function:

```neoil
{{NEOIL_TOUR}}
```

Directives begin with a dot. `.end` closes a declaration. Parameters use
`Type name`; `->` introduces the return type. Type keywords such as `int32`,
`string` and `void` are aliases for System types. Fully qualified names such as
`System.Int32` can also be used.

Instructions operate on an evaluation stack. `ldarg value` loads the argument,
`ldc.i4 2` loads an integer constant, `add` consumes both and pushes their sum,
and `ret` returns that result. Local slots use `.local`, `stloc` and `ldloc`.
Branches target labels written as `Name:`. Whole-line comments start with `;`;
strings use JSON quoting and escapes.

## Structural function types

Here is the entry point from the same executable sample:

```neoil
{{NEOIL_FUNCTION}}
```

`fn<int32,int32>` describes one integer parameter and an integer return. The last
type is the result; preceding types are ordered parameters. No named delegate
class is declared. The signature determines the type, so another function with
the same contract can be stored in `transform` too.

`function.bind` creates a callable object bound to `AddTwo`. The runtime supplies
its typed `Invoke` contract. The call consumes the function object and its argument
and pushes the result. Binding an instance method also retains its receiver;
that lifetime is part of the callable contract, not just a raw code address.

Parameter modes belong to the shape as well: `fn<readonly int32&,void>` takes a
read-only managed reference and returns no result. An `out` reference has a
different contract. Names of parameters or target methods do not change shape
identity; component type identities and reference-passing modes do.

This is the central distinction from a nominal type: two unrelated named classes
do not become the same type merely because their members look alike. Structural
Function identity comes from the signature itself. It does not imply arbitrary
object “duck typing.” See [Function types](../functions/) for equality, receiver
lifetime and reflection behavior.

## Void is a type; a direct return has no result

```neoil
{{NEOIL_VOID}}
```

`void` aliases `System.Void`. Both can be used for a local or a generic argument.
`ldvoid` materializes the logical value when an expression needs one. This does
not require a heap object or a payload of data bytes.

The original generic `Identity<T>` returns a value through its generic contract,
even when instantiated with `void`; `stloc result` consumes that logical value.
By contrast, `Complete() -> void` returns with an empty evaluation stack, and
calling it pushes nothing. `fn<void>` describes that no-result callable shape.
The type's identity and the call's result convention are related but distinct.

The current assembler gives `void` and `System.Void` the same direct-return
semantics. Older fixtures still use bare `Void` with a legacy return convention;
that spelling is migration debt and should not be used in new IL. Removing it
requires coordinated conversion of their bodies and callers.

## Other concepts in the encoding

| Concept | Encoding and meaning |
| --- | --- |
| Named types | `.type`, `.field` and nested `.method` declarations describe nominal storage and behavior. `.type class` selects reference-type representation. |
| Generic types and methods | `<T>` declares a parameter; a closed use supplies its type argument. Generic substitution preserves the original member contract. |
| Arrays | `int32[]` describes an array of integers. Its element type is part of type identity. |
| Managed references | `int32&` describes a reference to managed storage; lifetime and escape rules remain runtime/verifier responsibilities. |
| Native pointers | `int32*` describes a native pointer. It does not acquire managed ownership or make arbitrary native access safe. |
| Interfaces | `.interface` declares a nominal contract and `.implements` records implementation; instance dispatch uses the declared signature. |
| Runtime services | `.methodimpl InternalCall` selects an exact runtime-provided binding rather than an IL body. |
| C imports | `.foreign` records the native library, entry point and calling convention. Execution support and ownership restrictions depend on the native interop profile. |

[Slice and Memory](../memory/) are library types with runtime-recognized contracts;
they are not alternate spellings for raw pointers. [Native interop](../native-interop/)
explains that boundary. Structural function shapes, arrays and references likewise
retain their component types in metadata rather than becoming untyped addresses.

## From text to execution

The assembler resolves the text into a module containing type and function metadata
plus instruction bodies. The runtime consumes that encoded module, not the source
notation. neoCLR has a readable JSON module representation as well as its native
binary format; this walkthrough tests the JSON serialization path. Verification checks those contracts before execution;
interpreting, JIT-compiling or native-compiling a module does not make every
backend support every operation. See [native compilation](../native-compilation/)
for the current execution profiles and [metadata](../../metadata/) for the format.

The [complete walkthrough source](https://github.com/marinasundstrom/neoCLR/blob/main/samples/neoil-tour/main.neoil)
is assembled, verified, serialized, loaded and executed by a repository test,
which checks the result is 42. The [assembler reference](https://github.com/marinasundstrom/neoCLR/blob/main/docs/neoil.md)
contains the detailed instruction contracts.
