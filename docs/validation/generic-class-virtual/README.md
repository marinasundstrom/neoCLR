# Generic class virtual dispatch

### Concrete generic class virtual slots (2026-10-16)

The SDK now permits `AddVirtualMethod` and `AddOverride` on concrete generic
reference classes. Methods themselves remain nongeneric. Override parameters,
result and passing modes must match the inherited contract **after substituting
each constructed base edge**. The native metadata reader performs the same
substitution and rejects incompatible results; definition ordinals are not compared
as if they belonged to the same generic owner.

`CallVirtual(ConstructedMethodReference)` and `Emit(Callvirt, ...)` accept virtual
reference-class contracts as well as interfaces. `Call` still invokes the selected
concrete base body directly, so base calls do not redispatch into the override.
This follows CLR nominal virtual-slot semantics. Class constructions remain
invariant, and nothing infers nominal interface implementation from member shape.

AOT and managed JIT specialization retain the exact closed base-slot signature,
then discover overrides among the selected closed reference shapes. Each ancestry
edge substitutes its own owner arguments, including `Derived<Unused,T> : Base<T>`.
Existing closed-world type/function limits and native dispatch thunks remain in
force. There is no native format, ABI, Runtime Contract or boxing change.

The retained `docs/validation/generic-class-virtual` fixture returns 42 in CLR,
interpreter, AOT and managed JIT for integer/reference payloads and direct base
calls. It includes SDK and native-reader rejection of incompatible override
results. A specialization regression verifies an override that never calls its
base; all 14 specialization tests pass. Native cleanup and AOT sanitizers pass.
This is macOS ARM64 development qualification. Generic methods, abstract generic
class authoring, generic Object override qualification and Raven emission/import
of these virtual slots remain separate work; structural types remain parked.

```sh
python3 docs/validation/generic-class-virtual/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/generic-class-virtual-final-20261016
```

`results.json` records exact runtime, code generator, artifact and fixture hashes.
The system bundle is unchanged; the native code generator is freshly rebuilt.

### Abstract generic class authoring (2026-10-16)

`TypeBuilder.SetAbstractClass` and `TypeDefinition.SetAbstractClass` now accept
ordinary top-level generic reference classes. The native reader preserves the
abstract flag alongside owner parameters. Public nongeneric abstract slots use
the existing virtual contract, with inherited signatures substituted through each
base edge, including reordered parameters. This follows CLR abstract-class rules;
there is no new metadata encoding, ABI or Runtime Contract configuration.

The `generic-class-virtual/run.py --abstract` SDK consumer returns 42 in CLR,
interpreter, AOT and managed JIT, with native cleanup and AOT sanitizers. It covers
an abstract `Base<T>`, concrete `Derived<Unused,T>`, abstract and concrete slot
calls, direct concrete base calls, and integer/reference payloads. SDK checks reject
constructing an abstract generic type, leaving an inherited abstract slot
unimplemented on a concrete child, and calling an abstract method directly.
An abstract child may defer implementation. Generic methods, nested abstract
classes and generic closed hierarchies remain outside this slice. Raven admission
and imported abstract contracts still need separate qualification.
