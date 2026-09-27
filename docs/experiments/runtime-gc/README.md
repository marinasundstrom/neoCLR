# Public GC consumer

Compile and run the public class against matching development artifacts:

```sh
python3 docs/experiments/runtime-gc/verify.py \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime target/release/neoclr \
  --runner target/release/examples/measure_async
```

Checks collection and object counters, garbage reclamation, retained references,
null KeepAlive and the configured heap limit. See [the contract](../../runtime-gc.md).
