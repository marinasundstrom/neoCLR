# Scope-local method-owner index — 2026-10-24

The [cross-mode startup profile](../gc-mode-comparison/README.md) found repeated
method-owner resolution in inheritance validation. `declared_method` previously
scanned every function for each queried owner and contract. It now requests a
lazy owner index from the existing immutable `runtime_lookup::with_module` scope.

The first request resolves function owners once and groups function rows by exact
resolved type-definition identity. Subsequent requests inspect only that owner's
rows. Original module order, generic substitution, interface-implementation exclusion,
instance checks, owner checks, signature checks and first-match behavior remain.
Calls outside an active matching scope retain the linear path. Owner resolution
uses the existing type lookup, including generic arity and array backing rules.
Unresolved owners stay absent from candidate groups; source verification still
rejects malformed metadata through its existing checks.

There is no persistent pointer cache: identity keys are never dereferenced and the
index belongs to an immutable module scope. Nested scopes restore the prior index,
unwinding releases it, threads have separate scopes, and mutation between scopes
builds a fresh index. Cached rows contain integer function indexes, not cloned
methods, type objects or rich metadata. The cost is a lazy map plus one row per
resolved owned function, with transient Vec-to-Rc construction. A future compact
row layout could reduce that cost, but is not part of this change.

## .NET comparison and alternatives

This reuses the [CoreCLR metadata research](../../../docs/design/allocation-metadata-runtime-comparison.md):
resolve shared metadata for repeated operations without changing consumer-facing
semantics. It does not adopt MethodTable layout, bypass source verification or
introduce a new metadata identity contract. The deliberately narrower owner index
retains ordering and all existing overload checks. Indexing full instantiated
signatures would require more complex generic/lifetime invalidation; a global
pointer cache would not respect mutable-module scope boundaries. Keeping the
linear fallback also supplies a direct behavioral oracle for focused tests.

This can improve interpreter/JIT loading, AOT build selection and interpreter
virtual dispatch within an existing scope. Native generated request execution
does not perform this Rust lookup. No warm AOT server throughput improvement is
claimed, and root-publication/collection/scheduling behavior is unchanged.

## Validation and measurements

The retained baseline is `97d62c3fe`. Candidate source/binary hashes and matching
bundle identities are retained in `ordinary.json.gz` and `validation.json`.
Three alternating pairs per ordinary mode/sample produce these medians:

| Sample / measurement | Baseline seconds | Candidate seconds | Reduction |
| --- | ---: | ---: | ---: |
| Tasks / interpreter process | 4.825 | 3.300 | 31.6% |
| Collections / interpreter process | 4.770 | 3.339 | 30.0% |
| Tasks / JIT selection + verification | 3.129 | 2.442 | 22.0% |
| Collections / JIT selection + verification | 3.204 | 2.517 | 21.4% |
| HTTP / JIT process-to-READY (two pairs) | 6.935 | 6.212 | 10.4% |

Interpreter median process CPU falls from 4.764 to 3.296 seconds for tasks and
4.764 to 3.335 seconds for collections. Its process duration still includes
loading/verification, so this is not an interpreter instruction-rate measurement.
The first task pair is slower in both variants; raw trials retain that desktop
variation. JIT phase timing separates selection from compilation and invocation.
HTTP selection medians fall from 4.370 to 3.699 seconds. Readiness includes loading,
selection, JIT compilation, host initialization and the guest READY print.

Single AOT compilation checks fall from 3.338 to 2.683 seconds for tasks and 3.527
to 2.826 seconds for collections; these are supporting observations, not repeated
AOT timing cohorts. Task, collection and HTTP objects are byte-identical to their
baselines. All 24 ordinary invocations and four HTTP invocations check output or
responses and exit successfully; HTTP shutdown uses the control socket.

Focused validation: 44 integration tests cover class/interface inheritance,
generic bases, virtual dispatch, layout, object overrides, readonly/visibility
rejection and GC retention. Three existing lookup-scope tests and the owner-filtered
unit run pass (20 passed, one unrelated certificate-dependent TLS test ignored).
The two new tests compare owner groups against linear lookup, test lazy construction,
ordering, unknown/wrong-arity owners, reuse, threads, mutation and unwind restoration,
and compare indexed/linear dispatch and invalid-override faults.

Evidence: `ordinary.json.gz`, `http.json.gz`, `summary.json` and `validation.json`.
`summary.json` is reproduced with `summarize.py ordinary.json.gz http.json.gz`.
Small macOS ARM64 desktop cohorts do not qualify all platforms or a warm HTTP
capacity change. No public API, metadata encoding, bridge contract, GC policy or
object layout changed; website/API snapshots need no refresh for this slice.
Existing native GC/worker evidence remains applicable to byte-identical HTTP code.
Further owner scans and interpreter dispatch profiles remain follow-ups.

## Reproduction

This local experiment uses the retained build/bundle artifacts documented by the
[mode comparison](../gc-mode-comparison/README.md). Before rebuilding the candidate,
retain the baseline binaries as `target/gc-mode-comparison-final/baseline-neoclr`
and `baseline-codegen`. Build current release `neoclr` and `neoclr-codegen`, then run
without concurrent builds or load tests:

```sh
python3 benchmarks/execution/owner-index/measure.py --output target/owner-index
python3 benchmarks/execution/owner-index/http.py --output target/owner-index-http
```

`measure.py` alternates baseline/candidate ordering for three repetitions of each
ordinary sample in interpreter and paced JIT, checking guest output/status. It
also compares emitted AOT object bytes. `http.py` checks the HTTP AOT object and
runs two baseline/candidate JIT readiness trials each, followed by one second of
checked plaintext traffic and clean control-socket shutdown. Those short client
runs are functional admission, not HTTP capacity measurements.
