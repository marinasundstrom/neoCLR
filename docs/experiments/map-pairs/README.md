# Requested Map pair contract

The author requests Map<K,V> : Iterable<KeyValuePair<K,V>>, with a deconstructable
record struct. `raven/Main.rvn` is the reduced native-emission repro using the exact
record/value/deconstruction source shape. With bundle compiler `71cafd353`, the
project driver fails with NEOMETA001: native emission admits supported source
declarations but not this record declaration. This is a known failing compiler
repro, not a passing library sample. Do not replace the requested value pair with
a reference class or silently claim native record support.
