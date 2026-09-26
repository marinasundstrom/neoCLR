# Deferred generic JSON client prototype — 2026-09-26

This is an investigation record, not a public API or a working sample. The author
prioritized HttpResponse.Request and then client default headers while the generic
GetFromJson/PostAsJson prototype was being explored. The prototype was removed from
active library compilation so those independent changes can be validated and shipped.

With Raven neoCLR commit 9da3c7cb7, a bounded converter was declared as:

```raven
static func Convert<T>(result: Result<Object, HttpJsonError>) -> Result<T, HttpJsonError> {
    let value = result?
    return Ok((T)value)
}
```

Inside a generic GetFromJson<T> extension, assigning the method group explicitly:

```raven
let transform: Func<Result<Object, HttpJsonError>, Result<T, HttpJsonError>> = JsonClientOperations.Convert<T>
```

produced RAV2203: no overload matches the delegate type. HttpJsonError in the
prototype was a standard union with Http(HttpError) and Json(JsonError) cases.
The enclosing source slice shadowed matching reference declarations, like other
runtime library slices. This observation has not yet been reduced to a general
standalone Raven/.NET case; do not assume the source-shadowing aspect is irrelevant.

More seriously, putting the method group directly inside the task Map expression
compiled without an error but generated only a default Task return. Supplying Map's
explicit result type did not fix that. Execution then faulted on the null Task.
Explicitly typing the delegate made the diagnostic visible. This is not an acceptable
fallback behavior for the library and was not integrated. It needs a focused Raven
regression on a main-based feature branch before deciding on an implementation fix.

The prototype source and bridge patch were preserved locally at
`/tmp/neoclr-http-json-verbs-20260926`; this path is temporary evidence, not a build
input. It includes the fixture and selected generated artifacts for diagnosis.
Relevant logs are `/tmp/json-verbs-slice.log` and `/tmp/json-verbs-public.log`.

Next investigation: reduce method-group binding and missing-diagnostic behavior;
compare ordinary CLI generic methods, source/reference shadowing and extension
contexts. Validate generic Post value-to-Object conversion and GC as well: emitted
`box T` must preserve reference identity when T is a reference type. No runtime fix
for that separate observation has been made in this slice.

Public GetFromJson<T>/PostAsJson<T> remain the agreed next API layer. The shared
JsonContent helpers are already implemented and remain usable independently.


## Resolution — 2026-09-26

The method-group failure reproduced independently on Raven main: a generic caller's
supplied type parameter was treated as an unresolved callee parameter. Typed
assignment reported RAV2203, while inline higher-order calls returned defaults.
The binder now preserves constructed methods and validates their substituted
signature. Four positive regressions failed before and pass after; the uninferred
negative case remains rejected. All 399 overload-resolution checks pass before and
after, with five focused runtime/diagnostic checks also passing on the neoCLR branch.
A C# .NET 11 comparison agrees for reference and value instantiations.

General fix: Raven main `13b9105d8`, cherry-picked as neoCLR `56083626e`.
The [active client fixture](../http-json-client/README.md) replaces the deferred
prototype. The neoCLR bridge admits only its bounded generic converter callback,
substituting the callee parameter with the current caller parameter. Generic `box T`
now preserves reference objects and typed nulls; existing value-copy semantics remain.
These changes do not resolve every possible missing-diagnostic issue in Raven.
