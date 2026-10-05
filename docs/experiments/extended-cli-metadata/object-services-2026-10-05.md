# Object services and assembly-scoped internal calls — 2026-10-05

The three missing source Object service adapters now compile and execute: reference
equality, nonvirtual base equality and identity hashing. A separate consumer imports a
small contract-test library containing the real internal adapters and a test facade.
It checks alias mutation, stable identity hashing, distinct objects, nullable reference
equality, and independence from Equals/GetHashCode overrides, including hash-call side effects. The facade is test-only;
it does not replace or emulate source System.Object.

## Linking fix

The first execution failed because the seed and source library both declared two of
these runtime services. Signature validation previously treated declarations from
different assemblies as a duplicate. Internal-call declarations can now coexist with
matching signatures and distinct assembly identities. They are not deduplicated.
Symbolic local calls bind to the caller's own declaration before linked validation;
explicit definition references stay exact. Without a local declaration, ambiguous
symbolic calls still reject. Normal function duplicate rules remain unchanged.

Native-registry signature/body validation and access checks still apply. Duplicate
internal-call declarations inside a single module and incompatible cross-module service
signatures reject. Unlike arbitrary host CLR InternalCall methods, these services only
execute when the existing NeoCLR registry accepts their full contract. The metadata
format and CLI-style MethodImpl flag are unchanged. No compiler change is needed.

## Evidence and reproduction

[Four driver/runtime commands and hashes](object-services-2026-10-05.json) record the
artifact-only consumer run. The matching compiler implementation is 1ac78fdfa; Raven
485b364fc documents the integration. Runtime revision fields name the pre-commit base;
the binary hash identifies the build containing this fix.

```sh
python3 docs/experiments/extended-cli-metadata/verify_object_services.py \
  --compiler /path/to/rvnc.dll --library /path/to/source-handle/Numbers.dll \
  --ownership /path/to/source-handle/ownership.json \
  --seed /path/to/source-handle/System.neox --core /path/to/IntrospectionCoreParams.dll \
  --runtime target/release/neoclr --output /tmp/object-service-consumers
```

Use the [source-handle gate](source-handle-ownership-2026-10-05.md) to build dependencies.
The script builds the adapters/facade separately and excludes those sources from the
consumer compilation. Exact stdout is `Object service checks passed` plus a newline,
and exit status is zero. Internal adapters add no public Raven API or RavenDoc selection;
existing public Object behavior is unchanged. Website content requires no feature claim
or build for this development integration prerequisite.

Focused release-mode checks: 4 internal-call scope regressions, 5 native-registry tests,
7 accessibility tests and 6 scoped-type tests pass (22 total). They include explicit
external internal-access rejection, duplicate/incompatible declarations, ambiguity
without a local declaration, execution and unchanged input artifacts. This machine
used the explicit MacOSX26.2 SDKROOT recorded in the HTTP gate to build the runtime.
No performance claim or full-suite claim is made.

## Source Object still needs canonical root ownership

The cumulative 140-production-source library also compiles with the two Object adapters
and test facade. Adding unchanged source Object to the production inputs (without the test facade) still fails binding and publishes no
assembly. The [post-adapter diagnostic probe](source-object-after-services-2026-10-05.json)
contains override and object-to-generic conversion failures, but no missing Object service
members. This is a diagnostic probe with retained root ownership, not a complete source
ownership configuration.

The next root contract must align `object` and named System.Object in binding, give the
metadata writer an explicitly owned canonical root/boxing/virtual-slot contract, and
make runtime dispatch follow that declared root identity. Current native boxing and
Object-slot validation require the seed binding, and runtime intrinsic dispatch checks
its System identity. Do not mark Object as a scalar primitive, merge unrelated symbols,
remove the seed root prematurely, or rewrite library sources to hide the conflict.
Full-System bootstrapping and native async remain open.
