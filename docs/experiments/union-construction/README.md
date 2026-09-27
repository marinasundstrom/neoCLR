# Case: preparing union constructors at startup

Development after Preview 10. This is the checked construction prerequisite for
runtime route-to-union mapping, not a general route mapper or an SDK routing API.

`ItemRouteFactory.Prepare` reads the `/items/{id}` attribute on `CatalogRoutes`,
checks its Int32 payload, and retains the case and union carrier `ConstructorInfo`
descriptors. `Create` invokes those exact constructors and returns an ordinary
Raven union for application `match` dispatch. Discovery occurs once; repeated calls
validate access and arguments and execute ordinary constructors without overload
selection. There is no claim that complete native execution plans are cached.

The consumer also checks reference-class construction, argument type/arity/null
rejection and independent union values. The constructor service returns boxed value
records; the case and carrier are separate values with exact types. The application
uses a type pattern to recover the carrier. This does not introduce implicit
case-to-union conversion for boxed values.

Run with matching development artifacts:

```sh
python3 docs/experiments/union-construction/verify.py \
  --toolchain-root target/experiments/nested-json/bundle \
  --runner target/release/examples/measure_async
```

General schema validation, overlap detection, capture binding and reusable route
parsing remain the next layer. The earlier generator experiment retains evidence
for those contracts; source generation remains a future alternative.


Validation on macOS arm64: the [saved evidence](validation.json) records the public
consumer, 24 focused native tests, 28 bridge signature checks, and regression runs
of attribute introspection and the earlier generated HTTP route mapper (including
nine independent HTTP requests and a two-request neoCLR client/server pair). The
matching library and API snapshots pass; 18 website checks and 1,273 generated
reference pages validate. No benchmark or performance improvement is claimed.
