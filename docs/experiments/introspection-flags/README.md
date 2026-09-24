# TypeInfo classifications (development)

The source case verifies all six public flags across open classes, abstract
classes, ordinary leaves, open interfaces, closed class/interface families, the
TypeInfo contract itself, Option/Result nominal unions, BindingFlags, primitives
and arrays. Repeated generic queries run under collection pressure. This verifies
source metadata survives compilation, import and runtime acquisition; the Rust
matrix additionally covers open generic classes, managed references and metadata round trips.
The application importer currently rejects user-defined constructed generic classes;
source generic classification coverage uses the supported Option/Result declarations.

```sh
python3 docs/experiments/introspection-flags/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
cargo test --test raven_reflection
```

Expected output is `TypeInfo flags passed`. The verifier also requires multiple
collections and zero live managed objects on completion. The flags describe retained
metadata; the test does not claim raw-IL permits enforcement. IsClosedHierarchy
means a closed direct family, not necessarily a transitively closed descendant set.

Validation on 2026-09-24: the compiled sample passes, with 390 allocations, seven
collections and zero live managed objects. The public API snapshot check and combined
575-page website build also pass.
