# Nullable extracted payload qualification

Raven native compiler `2d22055dc` binds an extracted union/property payload directly
when its declared storage type matches the pattern binding. Previously a redundant
Object type test rejected an `Ok(null)` payload, making JSON property mapping fall
through to a non-exhaustive-match fault. Narrower typed patterns retain their test.

Raven main `ffa3d0812` passes three comparison cases (null, string, boxed integer)
without a production compiler change. The native line passes 82 focused CLR tests;
NullablePayload.rvn passes interpreter, JIT and AOT (see validation.json).
This matches ordinary .NET nullable payload binding semantics, with no native
metadata or Runtime Contract configuration change. The temporary portable planner
now matches main's extraction behavior; native metadata/backend replacement is
unchanged. Full JSON integration remains separate from this reduced regression.

The development bundle swaps the compiler core; its libraries retain their earlier
producer. It is not release qualification. A coherent library rebuild is required
before full-consumer validation. Separate exploratory forms involving null-literal
match-result conversion or unit-valued fault arms remain unqualified.
