# Deferred captured-callback reproduction

`CapturedCancellation.rvn` is the intermediate split-function probe, **not a passing
sample**. With the frozen compiler used by this checkpoint, it compiled/imported but
faulted while reading the captured pending task in Observe. The final product fixture
uses explicit operation fields and method-group callbacks instead.

To reproduce with that same matching bundle:

```sh
python3 docs/experiments/http-context/verify-cancellation.py \
  --toolchain-root /path/to/bundle --runner target/release/examples/measure_async \
  --source docs/experiments/http-context/repros/CapturedCancellation.rvn
```

The expected failing assertion includes a NullReference fault at Task.get_IsCancelled
from the Observe closure. Compiler SHA-256 used:
`57b6c6e33727de470fe529ee4b5c2b8fb338aeb60d1d6405d9e515276dccd96a`.
Root cause has not been isolated between emitted capture initialization and bridge
import. Do not label this a GC bug or claim a compiler fix. Reduce/check it independently
on Raven's current general branch before deciding whether it is target-specific.
