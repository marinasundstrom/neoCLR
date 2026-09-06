# Assembler expressiveness

neoIL should have the potential to be at least as expressive as .NET's assembly
language for the concepts neoCLR supports. Its syntax can be different, but must
not make important metadata or IL semantics inaccessible. The current small grammar
and interpreter subset are an incremental implementation, not a capability ceiling.

The default semantic reference is .NET/CLR. A deviation needs a platform reason and
an explicit contract. Missing implementation is not itself a reason to redefine an
otherwise familiar operation. Use [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
as the metadata/instruction baseline, alongside tests of intended runtime behavior.

## Capability target

The assembler should eventually directly express:

- Assembly/module identity, references, exported/imported symbols, visibility,
  attributes, resources, and the metadata needed for tooling.
- Full type signatures: primitives, named and constructed types, generic parameters,
  constraints, arrays/ranks, safe borrows, explicit references, native pointers,
  and function pointers/calling conventions where the platform supports them.
- Type relationships, fields/layout, interfaces and implementations, functions,
  methods/dispatch, properties/events and their associated functions where retained.
- Complete callable signatures: parameter and return types, generic arity, calling
  convention, modifiers, and parameter metadata. The prototype currently selects
  overloads by name and ordered parameter types only; that must not prevent richer
  metadata signatures later. Return-qualified calls may be necessary to express
  signatures that cannot be distinguished by parameter types alone.
- Every supported instruction, prefix, operand kind, branch form, local signature,
  and control-flow construct with precise stack and verification contracts.
- Debug/source information and explicit platform additions such as union layout,
  ownership/lifetime information, Faults, and future suspension semantics.

The assembler should not require a higher-level compiler to secretly supply meaning
that cannot be written in assembly. Source conveniences can lower into ordinary
metadata and IL, while an explicit form should remain available when the distinction
matters. Encoding should follow CLI conventions wherever compatible, as described in
[format direction](format-direction.md).

## Deliberate replacements

Expressiveness does not require resurrecting removed concepts unchanged. Guest
exceptions remain excluded: neoIL must fully express Result-based control flow and
terminal Faults. Likewise, eliminating an inherent value/reference type split needs
explicit ownership and identity operations, rather than hidden class-like rules.
Functions must be representable without an artificial class owner.

## Current surface and next tests

Declarations use `.function Describe(int32) -> string`; calls use
`call Describe(int32)`. Inline signatures share a parser for constructed types and
primitive aliases. Legacy `.param` declarations remain accepted when the header
omits a parameter list. Metadata carries separate typed fields rather than relying
on assembler source strings at execution time.

The System runtime library is an early test of assembly-level sufficiency: implement
real library behavior in neoIL and improve missing platform primitives as needed.
It is not proof of full ilasm parity. Next capability milestones are generic/type
metadata, module references, a full verifier, and a CLI-based binary backend.
Each milestone should add representative source-to-metadata/IL round trips and
execution or inspection tests, documenting unsupported features explicitly rather
than silently dropping them.
