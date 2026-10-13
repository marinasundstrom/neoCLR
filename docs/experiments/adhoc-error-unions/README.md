# Ad-hoc error unions — development checkpoint, 2026-10-13

`LoadConfig` in [Config.rvn](Config.rvn) reads a file containing an integer port.
It returns `Result<Config, FileReadError | NumberParseError>`. The two `?` operations
preserve the original typed errors without a common error base/interface, an error
wrapper or application-defined implicit converters. The final `Ok(config)` wraps
the successful value; `?` itself yields the success payload.

The alternatives remain visible in the signature. [Main.rvn](Main.rvn) imports the
separately compiled API, extracts the Result error and uses typed patterns to
inspect the original NotFound or InvalidFormat case. For many public APIs a nominal
union or dedicated error type is still preferable: it names a domain and insulates
callers from implementation dependencies. Review existing aggregation-only wrappers
as candidates too; weigh signature compatibility and case meaning before migrating.
[HttpJsonError is a review candidate](../../raven-error-api.md#review-existing-api-aggregation),
not a migration performed by this change.

## Representation and .NET comparison

Raven's ordinary .NET backend already supports this propagation through its
Raven.Core System.Union carriers. The neoCLR bridge now follows that library-backed
approach with ordinary native nominal metadata. It does not change .NET APIs, CLR
metadata or ordinary Raven/.NET defaults. Compiler residual conversions are
normalized before backend admission, and native parenthesized unions no longer
invent empty case companions shared by different generic arities.

System.Runtime owns temporary source-authored `System.Union` carriers of arities
2–5. They are an implementation bridge for `A | B`, not native structural values.
Ordered generic identity, an inactive default carrier and the arity bound are
bridge limits; structural identity/normalization, arbitrary arity, flattening and
structural introspection remain open. The benefit here is direct, discoverable error
composition; costs include exposing lower-level API dependencies and a temporary
nominal representation. No performance improvement is claimed. See
[bridge ownership and replacement](../../raven-cli-bridge.md#ad-hoc-error-union-bridge-2026-10-13)
and [the structural proposal](../../proposals/runtime-unions-and-intersection-types.md).

## Run the focused gate

Use a rebuilt native library bundle and its matching Raven compiler:

```sh
python3 docs/experiments/adhoc-error-unions/verify.py \
  --bundle /path/to/bundle \
  --compiler /path/to/rvnc.dll \
  --runtime target/release/neoclr \
  --output target/adhoc-error-unions
```

The bundle contains `lib/bundle.json`, the matching runtime seed/core, ownership
manifest and source-built class libraries. Add `--codegen /path/to/neoclr-codegen`
and `--jit-host /path/to/neoclr-jit-host.dylib` for the macOS ARM64 compiled gate.
The compiled gate uses [CarrierChecks.rvn](CarrierChecks.rvn) to exercise the same
error conversion with supplied file-error values, numeric parsing and arities 2–5;
it does not qualify compiled file I/O. The real file loader is interpreter-qualified.
The output directory must be new, and contains commands, exit codes and input hashes.

## Observed results

- Raven main: the pre-change focused semantic baseline passed 13 tests; the new
  success and both-error runtime cases already passed on .NET 11. After the shared
  normalization, 30 focused propagation/conversion tests pass, including four
  dedicated ad-hoc union cases (success, both errors, unrelated error rejection).
  Early return and skipped later work are asserted. [Test result](raven-main-tests-20261013.log).
- Raven native branch: the pre-change union baseline passed 14 tests. After the
  change, 31 focused propagation/conversion tests pass on .NET 11.
  [Test result](raven-native-tests-20261013.log).
- Native metadata: same-name constructor-union arities import with the correct
  generic alternatives and no invented case companion.
  [Probe result](raven-native-metadata-20261013.log).
- All four native class-library projects rebuild successfully with the new
  System.Runtime carriers. [Library build identities](library-build-20261013.json).
- The imported config API verifies and runs: valid file, missing file/NotFound,
  invalid integer/InvalidFormat. A BooleanParseError outside the error union is
  rejected with RAV1503 before output publication.
- Native-GC AOT and JIT both pass the carrier consumer, including both error paths
  and arities 2–5. [Commands, output and binary hashes](validation-20261013.json).
- API reference regeneration/snapshot validation passes; all four carrier types
  are included. Eleven focused website link/API-coverage tests pass, along with
  extraction of the tested LoadConfig source and Raven public-page selection.
  Full website builds and publication were not performed.

The tested development compiler was built from Raven's separate
`codex/source-object-metadata-resolution` branch at base `c1cf9b815` plus this change;
ordinary .NET checks used the independent `main` checkout at base `0ccb9a265`.
The reports identify exact compiler/library/runtime binaries. These are macOS
ARM64 and .NET 11 results, not Windows, .NET Framework or NanoFramework qualification.


Implementation commits: Raven main `3abc0c954` and native branch `b6557ab85`.
The native branch carries the general change deliberately and adds its own
metadata companion fix; the branches were not merged.
