# Raven generic class virtual dispatch

Run the local and separate-provider consumers with the current `codex/neoclr`
compiler (`b3b3608c3`) and the recorded System bundle:

```sh
python3 docs/validation/generic-class-virtual/raven/run.py \
  --compiler /Users/robert/Projects/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/raven-generic-virtual-local-20261016
```

Add `--imported` and use a separate output directory for the dependency case.
The provider declares `Derived<Unused,T> : Base<T>`. The consumer also declares
its own override, using a direct `base.Echo(Get())` call that must not redispatch.
Calls through `Base<int>` return stored payloads, rather than supplied arguments;
reference payload dispatch preserves identity. Expected result: 42.

`results.json` and `imported-results.json` retain compiler/SDK/runtime/bundle
fingerprints and interpreter/AOT/JIT commands, including native cleanup and AOT
sanitizers. This is macOS ARM64 development evidence. Abstract generic classes,
generic virtual methods, generic Object overrides and constrained dependency
imports are not qualified by this fixture. Structural types remain parked.
