# Typed native pointer foundation — 2026-10-21

Author-directed native interop work. `Ptr<T>` is a non-owning constrained struct,
not an allocation owner or a managed reference. Its `==`, `!=`, typed Equals and
boxed Equals compare addresses for the same closed type. Default value hashing
reads pointer bits directly, without boxing fields or reading native storage.

The public contract is documented in [the manual reference](../../../api-docs/typed-pointers.md).
The [.NET pointer baseline](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/pointer-related-operators#pointer-comparison-operators)
compares addresses. The wrapper adds generic value contracts; it deliberately does
not yet expose arithmetic or implicit conversions because layout/offset validation
and ownership are separate work. No safety or throughput improvement is claimed.

## Evidence

- Raven main `1e2b43ce4`: 35 pointer tests, including emitted .NET pointer-to-void
  and explicit reinterpretation round trip. Carried as `469e6a092`.
- Raven native `922c0f085`: 42 focused pointer tests, including portable casts,
  equality and explicit capability admission for constrained structs.
- Raven native `0da6b1cd1`: imported value types retain ordinary special constraints.
- SDK `--pointer-signatures`: casts, matching-pointer equality, scoped generic
  substitution/import, invalid operands and reference-type constraint rejection pass.
- `cargo test --test value_defaults`: seven tests pass, including null and allocated
  pointer fields with a two-object budget for two explicit wrapper boxes.
- All five native libraries rebuilt with the pointer wrapper.
- `scripts/test-runtime-library.py --suite native-pointers --mode interpreter`:
  all three discovered tests pass, including generic forwarding across the library
  boundary and equality/hash agreement across explicit boxing and GC collection.
  Local evidence: `target/native-pointers-tests-20261021-v5/report.json`.
- Native API rendering, four snapshot tests and the 5,637-page website build pass. The API includes a manual
  operator reference because the renderer currently collides `==` and `!=` URLs.

The focused suite is explicitly selectable and is not added to the default JIT
release gate until that execution path is qualified. No complete release/package
qualification or native import JIT/AOT support is implied. Native layout support,
raw pointer/integer conversions and scoped Slice borrowing remain separate work.
