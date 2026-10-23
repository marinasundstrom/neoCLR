# Retained declaring types — 2026-10-12

Native `TypeDeclaringType` now materializes the interpreter's descriptor recipe for
explicitly retained, nongeneric type declarations. The Raven consumer verifies the
actual enclosing type identity and absent ownership for a top-level type. This follows
.NET `Type.DeclaringType` semantics, projected as `Optional<TypeInfo>` rather than null.
It does not grant constructor or method invocation rights. Missing retention faults;
it must never fabricate `None` for an unretained nested type.

[Validation](validation.json) covers explicit interpreter execution, macOS ARM64
AOT compilation/link/execution, managed JIT execution, and JIT missing-retention
failure. Two focused Rust tests cover distinct same-named nested declarations,
unknown identities, malformed reserved contracts and empty retention. This is not
Windows qualification or generic-declaration retention support.

```sh
python3 docs/experiments/declaring-type/verify.py \
  --bundle target/generic-cleanup-gc-bundle/bundle \
  --compiler target/generic-cleanup-project-compiler/rvnc.dll \
  --backend tools/aot-poc/target/release/neoclr-aot-poc \
  --output target/declaring-type-check
```

The development harness reuses the pinned macOS shared-GC build recipe, including
its absolute SDK/library paths. It is not a portable SDK command. The CLI does not
yet project reflection-roots configuration: the JIT check uses the backend directly.

The full module-function consumer now reaches a later interface-to-object `Equals`
dispatch mismatch in both [JIT](discovery-jit.json) and [AOT](discovery-aot.json).
These are retained failures, not passing discovery claims. Module-function retention,
ownerless attribute queries and registration remain tracked in
[issue #27](https://github.com/marinasundstrom/neoCLR/issues/27).

Follow-up: [interface-to-Object dispatch now passes](../guest-functions/interface-object-dispatch/README.md).
The reports above preserve the earlier failure; current discovery starts under JIT
and faults on missing module-catalog retention.
