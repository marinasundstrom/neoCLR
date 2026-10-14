# Public reflection member consumer

Run against the matching development bridge, runtime and generated library:

```sh
python3 docs/experiments/reflection-members/verify.py \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime target/release/neoclr \
  --runner target/release/examples/measure_async
```

The consumer checks constructor metadata, typed construction with params arguments,
instance/static invocation, nullable void results, field read/write, source access
and read-only rejection, exact scalar validation and incompatible typed activation.
It uses the public library extensions, not application copies. See the
[contract and limitations](../../reflection-members.md).

## Native typed-construction relationship check

`TypeRelationships.rvnproj` exercises the updated public result guard through the
native source libraries: exact result type, Object widening and incompatible string
rejection. Build with the matching `NeoClrBundleRoot` and execute the emitted native
metadata using the interpreter with that bundle's modules. This focused fixture
avoids unrelated invocation and field-access paths in the older bridge consumer.
The broader fixture retains explicit TypeInfo assignment/instance assertions too;
its old CLI harness is not qualification of the current native compiler.
