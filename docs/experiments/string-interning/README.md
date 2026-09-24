# Explicit String interning — development, 2026-09-24

The original owned-pool experiment is now integrated as `String.Intern(value)`.
Each interpreter execution owns a fresh pool, including executions started by isolated
workers. The implementation does not introduce a mutable host runtime-session object,
a process-wide pool or automatic literal interning. This is development after Preview 9;
rebuild matching runtime, compiler reference and bridge together.

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

The original four pool checks pass. The Rust parser sample prints 4,000 occurrences, four retained text
owners and 28 UTF-8 payload bytes. This counts the prototype's pool contents, not
process memory or allocation reduction: input Strings are still created first.
The pool lives in `src/string_interning.rs`; its checks live in
`src/string_interning_tests.rs`. It stores the String owner once per unique value in a content-keyed HashSet;
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

## Execution ownership and public API

LoadedProgram holds immutable metadata; each execution creates fresh mutable state.
An execution-owned pool fits this existing boundary. A host session pool would require
a new ownership/configuration abstraction and shared-retention policy with no current
sample requirement. Runtime-owned suspension may eventually extend the lifetime of
execution state; the pool must travel with that state if suspension is introduced.
No contract spanning future sessions or suspension mechanisms is inferred here.

Host tests invoke the same loaded function twice with fresh equal inputs and retain
both results: their contents agree but their identities differ. Passing an already
shared host String into another invocation can preserve identity, as ordinary input
ownership does. The pool is not an isolation/security boundary. Both StartWorker and
QueueWorker receive independent quotas. Worker result transport still copies text;
it does not promise cross-worker identity.

`String.Intern(text: string) -> string` requires non-null text and returns the existing
canonical owner or retains and returns the input on first insertion. A null String
default raises NullReference. Existing aliases
are unchanged. Matching is exact, including empty text and embedded NUL; no normalization,
case folding or literal interning. No IsInterned/lookup helper is selected.

`Limits.intern_entries` defaults to 4096 and `Limits.intern_bytes` to 1 MiB. New entries
beyond either limit fault with InternPoolLimitExceeded, following the runtime's terminal
resource-budget model. Existing hits remain usable when the pool is full. The caller
may configure lower or higher budgets; these numbers are not a permanent tuning promise.
Zero entries prevents insertion. Host allocator reservation failure remains RuntimeError.

The pool releases its references on normal completion, Fault or cancellation. External
results remain valid while retained by their host/managed owner. Quotas count unique
pool UTF-8 payload, not all interpreter or allocator memory. Hosts using exhaustive
Limits struct literals must add the two fields; `..Limits::default()` accepts defaults.

## Raven sample and validation

Main.rvn models repeated incoming field names with fresh String copies. It interns
64 values, checks their four canonical references, and exercises empty and Unicode
text. It deliberately has no parser API dependency; the Rust sample parses simple
key=value records to establish the original case.

```sh
python3 docs/experiments/string-interning/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
```

The verifier checks exact output, repeated collection and zero remaining managed
objects. Runtime tests separately cover repeated host invocations, quota hits/misses,
worker independence, and release after completion, faults and cancellation. The new
Fault code has stable symbolic serialization. API reference and website describe the
execution boundary explicitly. Compiler metadata adds Intern and exact importer
binding; Raven language semantics, emission and Runtime Contract configuration do
not change. Publication and SDK release are separate operations.

## Integration validation

The nine focused runtime interning cases pass, including execution isolation,
worker pools, quotas, null input and cleanup on completion, fault and cancellation.
The four fault-code tests pass. The Raven sample passes against the matching
compiler reference and generated runtime (523 allocations, zero live objects at
completion, nine collections). Native owner-lifetime checks separately verify
pool cleanup. The String sequence sample also verifies the renamed parameters,
including reordered named arguments and rejection of the old generic names.
