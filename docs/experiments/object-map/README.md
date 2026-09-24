# Mixed Object map keys and values

Development, 2026-09-24. With a matching development bundle:

```sh
cargo build --example measure_async
python3 docs/experiments/object-map/verify.py \
  --toolchain-root /path/to/bundle --runner target/debug/examples/measure_async
```

The importer previously rejected HashMap<Object, int> while accepting Object in
ordinary signatures. Generic payload mapping now recognizes the same CLI intrinsic
Object marker (or the target core Object definition). No runtime/library algorithms,
public signatures or Raven compiler configuration changed.

The sample uses HashMap<Object, Object> with explicit Object equality/hash callbacks.
It checks separately parsed Path keys, fresh type descriptors, separately boxed
integers, Boolean versus integer keys, and ordinary class allocation identity. Object
values pass through Option<Object>; reference aliases survive replacement, table growth
and collection. A second map deliberately returns hash zero to check collision handling.
It does not use string-to-Object conversion, nullable keys, a default comparer or any
unsupported boxed primitive behavior.

Recorded run: 955 allocated/reclaimed objects, peak 202, eleven collections and zero
retained objects under a 256-object limit. These are correctness measurements, not a
performance comparison with .NET. The original Path probe also passed as an Object-keyed
map after reproducing the importer rejection before the fix.
