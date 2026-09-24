# Explicit String interning experiment — 2026-09-24

This is a test-only exploration using the real neoCLR StringValue owner, not a
production pool, guest String.Intern API or automatic literal interning. It follows
the author's request to investigate interning after establishing String identity,
without defining the public contract prematurely.

## Concrete case

A parser reads 1,000 structured log records with four repeated field names:
`timestamp`, `level`, `service` and `message`. Each parsed name starts as separately
allocated text. Interning returns the first retained owner for equal contents.
The caller must retain that returned value; interning cannot rewrite other references.

Run the experiment:

```sh
cargo test --lib interning_tests -- --nocapture
dotnet run --project docs/experiments/string-interning/dotnet -c Release
```

The four Rust checks pass. The sample prints 4,000 occurrences, four retained text
owners and 28 UTF-8 payload bytes. This counts the prototype's pool contents, not
process memory or allocation reduction: input Strings are still created first.
The prototype lives in `src/string_interning_tests.rs` and is excluded from production
builds. It stores the String owner once per unique value in a content-keyed HashSet;
there is no duplicate owned text key. Lookup uses exact UTF-8 contents, with no
case folding or normalization. Empty text and embedded NUL are accepted in the probe.

## Memory and lifetime evidence

The pool strongly retains canonical owners until it is dropped. Separate entry and
UTF-8 payload quotas reject insertion without changing pool contents. Existing hits
remain usable at capacity. The entry limit matters for empty strings and collection
overhead; neither limit accounts exactly for HashSet capacity, Arc headers or allocator
metadata. HashSet reservation is fallible, but this is not complete host OOM handling.

Checks collect a managed array containing pooled text, release the pool, and retain
one result in the host. Pool-only text is released when the pool is dropped; the host
result remains valid with the same identity hash until its last owner is dropped.
Separate pools do not automatically share independently created equal inputs. They
may naturally share a String owner that a caller supplies to both. Pool separation
is not a security or ownership-isolation boundary.

## .NET comparison and alternatives

The .NET 10 baseline passes: explicitly interned equal dynamic strings return the
same reference, original distinct references remain distinct, and interned text stays
alive after local references are released and GC runs. It uses unique dynamic text
to avoid relying on compiler literal handling. Primary sources reviewed 2026-09-24:
[String.Intern](https://learn.microsoft.com/en-us/dotnet/api/system.string.intern?view=net-10.0)
and [String.IsInterned](https://learn.microsoft.com/en-us/dotnet/api/system.string.isinterned?view=net-10.0).
The CLR pool can retain strings until runtime termination. IsInterned is a lookup
returning a String or null, not a Boolean; no corresponding neoCLR helper is selected.

| Alternative | Benefit | Cost or open question |
| --- | --- | --- |
| Process-wide strong pool | Familiar long-lived canonical identity | Retains unrelated applications' text and needs shared quotas/synchronization |
| Explicitly owned strong pool (prototype) | Predictable release and stable identity during pool lifetime | Requires a clear owner and insertion-limit behavior |
| Weak pool | Can release unreferenced text sooner | Canonical owner can change after collection; stale entries and cleanup need policy |
| Automatic literal interning | Can share loaded constants | Compiler/loader and artifact-lifetime obligations, not needed for this case |

## Next integration checkpoint

LoadedProgram currently holds immutable metadata; each execution creates fresh
mutable state. Do not silently attach a process-wide cache to StringValue or mutable
state to LoadedProgram. Compare one-execution ownership with a future host runtime
session using repeated host invocations and independent applications. Decide whether
cross-invocation canonical identity is actually needed by that case.

A candidate String.Intern spelling remains provisional. Public exposure also needs
an insertion-limit policy, execution/worker/suspension lifetime coverage and on-site
member documentation. These are integration questions, not approved contracts.
No public API, compiler metadata, Runtime Contract setting or published capability
changes in this experiment.
