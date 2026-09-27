# Interface defaults and static helpers

Development slice, 2026-09-27; not included in Preview 10. The Raven consumer
executes a public default through a nominal interface reference, calls a private
static helper and a public static helper, dispatches back to an abstract instance
member, and verifies that a class implementation wins over a default. Void defaults
and static-helper reflection flags are covered as well.

Run with a matching development bridge/runtime and the current System/reference:

```sh
python3 docs/experiments/interface-helpers/verify.py \
  --runtime /path/to/neoclr --bridge /path/to/Probe.dll \
  --system /path/to/System.neoil --reference /path/to/NeoCLR.CoreProbe.dll \
  --evidence docs/experiments/interface-helpers/validation.json
```

The verifier also rejects external private access and private instance helpers.
The latter remain outside this bounded admission: Raven currently emits the probed
private instance body as Private, Virtual, NewSlot. That general compiler candidate
needs independent CLI/.NET validation before changing emission; the bridge does not
silently erase those flags. Static virtual defaults, explicit derived replacements
and protected/internal interface members remain follow-up work. No Runtime Contract
configuration changes or public System API additions are needed.

Native checks: `cargo test --test interface_helpers --test default_interfaces
--test static_number_contracts`. They cover nominal default execution, helper
reachability, access/dispatch rejection, existing managed-receiver defaults and
static abstract conformance. See [contract and comparison](../../default-interface-implementations.md#raven-nominal-defaults-and-static-helpers-development).


Validation recorded 2026-09-27: all 16 focused native tests pass (12 existing default
cases, two helper cases and two static-contract cases). The final Raven consumer
passes typed-stack verification and prints `42`, `99`, then its success marker;
both negative source cases are rejected. [Artifact/source hashes](validation.json)
pin the run. The matching API snapshot check passes; its regenerated reference DLL
is byte-identical because this slice adds no System signature. No full suite,
website build or performance benchmark was run. Target integration documentation
is committed on Raven's `neoclr` branch as `f79af3566`; compiler code is unchanged.
