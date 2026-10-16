# Constructed generic class bases

Development SDK fixture: `Leaf : Derived<bool, int> : Base<int>`, where
`Derived<Unused, T>` derives from `Base<T>` and inherits `View<T>`. The executable
uses both integer and reference payloads, transitive inherited storage, and
inherited generic-interface dispatch. Successful CLR, interpreter, AOT and
managed JIT execution returns 42. Native cleanup and AOT sanitizers are checked.

```sh
python3 docs/validation/constructed-class-base/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/constructed-class-base-final-20261016
```

The SDK tests also verify the reordered open base argument in the native snapshot,
CLR upcast identity, and rejection of wrong constructor/base arguments, foreign,
sealed, interface, unbound and out-of-scope bases, invalid closed upcasts, missing
initialization and unproven special constraints. Adding the matching child
constraint permits CLR/native metadata writing. Existing generic class/Object-root
and interface-variance/metadata-reader checks also pass. The full SDK contract
runner passes 178/178 tests after these changes.

This adds owned base authoring, not external constructed base layout authoring or
generic reference-class virtual slots. Raven constructed-base emission is the next
integration step. Interface conformance remains explicitly nominal; classes are
invariant. Structural types and Sliceable/Result variance remain deferred.
