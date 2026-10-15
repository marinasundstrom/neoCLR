# Structural value union integration

The runtime and companion Raven compiler keep structural unions as values, whether
an alternative contains a value or an object reference.

- `project/`: boxing/extraction and ref/out writeback, exit 42.
- `callback-project/`: an exact callback returning a structural union, exit 42.
- `fresh-bundle-qualification.json`: the HTTP-header consumer compiled and executed
  against the fresh v3 Runtime/Data/Networking/Web bundle, exit 0, plus invalid
  configuration rejection without replacing the successful output. Includes commands
  and artifact hashes. Runtime evidence covers 53 focused structural tests.

The fresh local bundle is `target/structural-integrated-libraries-v3`; its props
file is `NeoCLR.ClassLibrary.props`. Bootstrap and source-build provenance are in
that directory's `build-evidence.json` and manifest. The qualified project is
`target/structural-integrated-libraries-v3-qualified/Headers.rvnproj`.

Reproduce qualification with `scripts/verify-native-library-project.py`, using the
paths and compiler revision recorded in the evidence and a new output directory.
These are development feature-branch artifacts. This result does not qualify every
HTTP endpoint, all open structural operands, or JIT/AOT execution.
