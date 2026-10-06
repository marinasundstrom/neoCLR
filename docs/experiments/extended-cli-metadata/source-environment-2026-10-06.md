# Source-built Environment gate — 2026-10-06

The unchanged Environment functions and EnvironmentError now compile as Environment.dll
with three internal RuntimeServices adapters. An independently compiled consumer imports
that artifact with library sources absent, verifies and exits 0 with exact stdout
`Native source environment passed`.

The consumer checks command-line ordering and Unicode, mutates one argument snapshot and
checks the next remains unchanged, compares the current directory with the controlled
working directory, and distinguishes present/empty/absent variables and an invalid name.
The harness reads only explicitly selected fixture variables; inherited environment
contents are not written into its evidence. The current compiler still uses an explicit
namespace alias because the primitive bootstrap contains the legacy Environment type.

## Runtime boundary and supported array model

The internal EnvironmentArguments service now admits the native managed string-array
return emitted by Raven. The VM allocates a fresh managed array for each call and enforces
array-element and heap-object limits. The existing legacy runtime library's temporary
value-array transport still loads; it is not the supported Raven array model or a new
inline-value-array API. Managed arrays remain backed by the existing nominal Array<T>
contract. Inline arrays have no current platform story; possible interop use is future work.

This follows the existing [.NET comparison and Environment contract](../../environment.md):
callers receive an independent string-array snapshot, while missing variables differ from
empty ones. There are no new environment mutation APIs, runtime service capabilities or
performance claims. The four Environment runtime tests pass, including managed array/heap
limits, wrong return-category rejection and existing host/legacy execution controls.

## Bootstrap progress

After callable nullability, the full-owned-handle audit had 51 diagnostics. These adapters
remove three missing declarations and the new audit reports 48 diagnostics across 184
source inputs; it still publishes no System assembly. This is binding progress, not full
bootstrap or execution qualification of the full library. Console remains blocked partly
by native-width integer service/ownership support; retain its overloads rather than
substitute narrower signatures. Calendar service inputs and residual binding issues also
remain. See [audit](native-bootstrap-environment-2026-10-06.json).

## Reproduce

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_environment.py \
  --compiler /path/to/rvnc.dll --compiler-revision d19c6e4a3 \
  --library /path/to/Numbers.dll --ownership runtime/raven/native/poc-ownership.json \
  --seed /path/to/System.neox --core /path/to/Core.dll \
  --runtime target/debug/neoclr --output /tmp/environment-gate
cargo test --test environment
```

[Commands and hashes](source-environment-2026-10-06.json) identify the fixed compiler,
bootstrap inputs, source library/adapters, runtime binary and changed runtime source
files. The runtime revision records the pre-slice HEAD; the source hashes identify the
new boundary implementation. No Raven compiler changes were necessary for this slice.
Published Preview 12 artifacts remain unchanged.
