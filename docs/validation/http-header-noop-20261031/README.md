# Response header retention — 2026-10-31

`HttpServerResponse.Respond` previously copied its header sequence even when the
new content had no media type and there was no Content-Type to clear. The private
header implementation now checks for that case and retains the entries. It unwraps
an attached mutable collection before retaining storage, avoiding a self-reference
when the response updates that same view. Content-Type removal/replacement and
transport validation semantics remain unchanged. The no-media-type path scans names
before deciding whether to copy; clearing an existing Content-Type therefore adds
a preliminary scan. This trades that work for avoiding copies on unchanged fields;
no broader latency or throughput claim is made.

This is an internal allocation optimization of the existing contract, not a new
.NET divergence. The existing [API comparison](../../../api-docs/http-headers.md#net-comparison)
continues to apply. No public signatures, compiler/runtime behavior or release
settings change. Website reference prose is updated; the unchanged API snapshot
and prior website build remain applicable.

## Reproduction and evidence

Base neoCLR revision: `27ab46c69`. Compiler: Raven `a1f2c8588` from the qualified
`target/http-message-surfaces/inputs9/bundle`. Rebuild System.Web and Neo.Web using:

```sh
python3 docs/validation/http-message-surfaces-20261031/compose-input-bundle.py \
  --base-bundle target/http-message-surfaces/inputs9/bundle \
  --output target/http-header-noop/inputs
```

The composed bundle's component evidence records source/compiler/artifact hashes.
Run `scripts/test-runtime-library.py` with the new bundle and:

- `--suite http-header-allocations --mode interpreter`: three checks, including
  a real loopback HTTP consumer and repeated response configuration.
- `--suite http-header-values --mode interpreter --mode native --mode jit` with
  `--aot target/release/neoclr-codegen`, `--runtime target/release/neoclr` and
  `--jit-host target/http-header-noop/inputs/bundle/bin/neoclr-jit-host.dylib`.

The negative control runs the new allocation test against the previous bundle.
Over 128 warmed calls with reused content and one unrelated header it reports
**512 managed allocations**, failing the zero-allocation assertion. The changed
library reports **zero**. This measures this configuration path in the interpreter;
it does not establish native allocation counts or an HTTP throughput improvement.
The test also checks view identity, preserved entries and subsequent mutation.

Final results: **16/16 header tests in each of interpreter, JIT and AOT**;
**3/3 interpreter allocation tests**, including the loopback HTTP consumer.
[Contracts](contracts.json), [allocation checks](allocations.json) and the
[expected failing baseline](baseline-negative-control.json) retain inputs and outputs.
Final reports are stored alongside this document. Earlier exploratory runs are
excluded: the initial test was not registered in its explicit Main, then the
runner's exact expected output needed its test count updated. One preliminary
contract run detected the runner changing during validation; the final rerun uses
stable inputs.
