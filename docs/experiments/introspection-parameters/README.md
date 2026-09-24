# Parameter ownership and Object contracts

Development, 2026-09-24. Use a matching development runtime and library; parameter
snapshot layout changed. Build and run:

```sh
cargo build --example measure_async
python3 docs/experiments/introspection-parameters/verify.py \
  --toolchain-root /path/to/bundle --runner target/debug/examples/measure_async
cargo test --test raven_reflection parameter_identity_uses_owner_kind_closed_type_definition_and_position
```

The sample inspects two Converter methods, distinguishes parameters by owner and
position, and uses repeated queries as HashMap keys. It checks reference identity
versus equality, symmetric Object equality, wrong types/null, hash consistency and
Name display. It retains parameter owner data through repeated GC and requires full
reclamation. Recorded run: 894 allocated/reclaimed objects, peak 72, 25 collections,
zero retained objects under a 256-object limit. These counts describe this fixture,
not a comparison with .NET performance.

The raw metadata regression sets all parameter tokens to zero. It compares repeated
queries, different positions, different methods, closed generic owners and separate
type definitions. Two properties share one getter: their index parameters must differ
from each other and from that getter's parameters despite shared names/types/tokens.

The key is (closed declaring type, owner kind, definition index, position). Names,
tokens and type alone cannot identify a parameter. Owner data is internal; a public
Member property needs direct scoped resolution in a later slice. ToString returns
Name, potentially empty; defaults, return parameters and custom attributes remain
unimplemented. The on-site API guide describes this departure from .NET's broader
ParameterInfo surface and its base Object identity behavior.
