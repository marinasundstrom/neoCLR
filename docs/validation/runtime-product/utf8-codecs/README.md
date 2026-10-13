# Rooted UTF-8 codec qualification

Generated native-GC encode/decode calls protect the complete source String/array
through the sole allocation retry, then copy and publish without reentry. Strict
UTF-8 and reserved-element validation happen first. Invalid UTF-8 does not collect;
uninitialized reserved elements retain their earlier failure precedence. Existing
native entry points and raw native-buffer decoding remain noncollecting.

The local sanitized gate passes all 24 GC/root executions. The new fixture omits
sources from caller roots, forces retry, and checks copied bytes and retained
sources. It covers multibyte scalars, embedded NUL, empty data, ordinary/reserved
arrays, live exhaustion, unchanged failure outputs, missing/foreign boundaries,
validation precedence and complete final reclamation. Package checks also pass.

Fourteen focused compiled tests pass on macOS ARM64. Encoding/snapshot consumers
and the strict decoding matrix run in GC and non-GC modes against interpreter
results/faults; the GC objects import the new rooted symbols. Decoding covers every
single byte, valid scalar boundaries, overlong forms, surrogates, invalid
continuations, truncation, null input and uninitialized elements. GC hosts verify
fault-message retention and final reclamation. Existing raw-arena size/byte-budget,
slicing and relevant text consumers remain covered by the same focused run.

```
python3 scripts/validate-native-gc.py --output NEW_DIRECTORY
python3 -m unittest discover -s scripts -p test_native_gc_layout.py
cargo test --locked -p neoclr-aot-poc --test console utf8_
```

Native platform qualification is pending. The C gate does not imply a fresh managed
Windows/JIT/HTTP matrix. No Raven Runtime Contract, bridge metadata or managed API
changes. This is lifetime/admission work, not a throughput or collection-pacing claim.
