# External constructed class bases

### SDK external constructed base authoring (2026-10-16)

`CompleteClassBaseReference` now accepts an **unconstructed generic definition**
as well as a nongeneric definition. The host supplies the resolved public method
contracts, complete declared instance field count (including private storage),
and nominal base/interface edges. Complete each ancestor definition separately;
methods must belong to that definition. These are trusted compiler facts, not a
substitute for checking the dependency's visibility, extensibility and constraints.
Exact artifact identity remains required when linking.

`TypeBuilder.SetBaseType(ImportedTypeReference baseType)` binds a complete external
construction to a top-level root class before its members. It supports generic
children with declaration-scoped arguments and nongeneric children with closed
arguments. Missing contracts, wrong ownership, unbound or out-of-scope arguments
throw `ArgumentException`; repeated/late binding or unsupported owners throw
`InvalidOperationException`, before mutation. `AddClass` also accepts a closed
external construction registered this way. `BaseTypeSignature` retains arguments.

`Call(ImportedConstructedMethodReference)` accepts a constructor only as the exact
direct-base initializer of a constructor. Protected access and exactly-once current
receiver initialization remain checked. It does not allow arbitrary constructor
calls on existing objects. Dependency methods/interfaces and ancestry substitute
each construction while layout counts remain definition-owned. Generic classes
remain invariant, and interface conformance remains nominal and explicit.

CLI output uses TypeSpec extends entries; native output uses existing constructed
signatures and arity-matched dependency definition bindings. No encoding version,
ABI or Runtime Contract change is needed. Generic reference-class virtual slots
remain unsupported. The SDK fixture in `docs/validation/external-constructed-class-base`
covers a protected generic base constructor, reordered child parameters, private
base and own storage, closed value/reference payloads, transitive upcasts and
inherited generic-interface dispatch. CLR/interpreter/AOT/JIT return 42 with native
cleanup and AOT sanitizers; all 179 SDK checks and API snapshot checks pass.
Raven source qualification follows separately. Structural work remains parked.

```sh
python3 docs/validation/external-constructed-class-base/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/external-constructed-sdk-20261016
```

`results.json` records the commands and exact artifact fingerprints. This is
macOS ARM64 development qualification, not a cross-platform release claim.
