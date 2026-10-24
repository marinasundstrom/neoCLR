# Reuse owner candidates for interface validation — 2026-10-24

A fresh startup profile after `ceefedb30` identifies remaining interface validation
work. In a one-second capture beginning 200 ms after collections JIT launch, 607
of 619 main-thread samples pass through closed-world preparation and LoadedProgram
construction. The captured stack includes 229 samples under interface validation
and 171 under `ensure_members`; 144 of those are in type-definition lookup.
These inclusive counts overlap and describe a selected startup window, not exact
whole-process CPU percentages or warm request costs. The profile is separate from
performance trials and retained with its command and process results.

## Change and contract

`ensure_members` now uses the existing immutable-scope method-owner index to select
candidate rows for each inherited interface. It retains original method order,
owner checks, helper and explicit-mapping exclusions, generic substitution,
implementation selection and virtual dispatch checks. `has_self_contract` uses the
same owner candidates across the interface closure; it still recognizes Self in
inherited type arguments and member signatures. It caches no signature result.

The small iterator adapter owns an Rc of row indexes and borrows the module. It
adds no boxed iterator or cloned metadata. Outside a matching lookup scope it
returns the full function sequence, so callers retain their owner/contract checks.
The Self boolean query traverses interfaces before methods; it has no observable
member ordering or diagnostic side effects after closure validation. This can
repeat the linear signature scan for each interface outside a lookup scope; the
optimization targets the existing scoped loading/execution paths.

The owner map's lazy construction, immutable lifetime, thread isolation and
unwind/mutation behavior are unchanged. Some scopes can initialize it earlier
because interface validation now uses it. No managed object headers, collector
policy, root publications, generated execution metadata or public APIs change.

## .NET comparison and tradeoff

Reuse the [shared metadata research](../../../docs/design/allocation-metadata-runtime-comparison.md)
and [owner-index contract](../owner-index/README.md). This extends bounded shared
resolution without changing .NET-like interface semantics. A full conformance
cache would require instantiated generic, Self and lifetime keys; candidate
narrowing leaves those decisions in the existing checked implementation. The
cost is retaining the existing per-scope index wherever these queries trigger it.
No new global registry, metadata format or Raven compiler bridge is introduced.

## Validation and measurements

Baseline is `ceefedb30`, including the preceding inheritance-owner optimization.
Three alternating pairs per ordinary sample/mode give these medians:

| Sample / measurement | Baseline seconds | Candidate seconds | Reduction |
| --- | ---: | ---: | ---: |
| Tasks / interpreter process | 3.357 | 2.858 | 14.9% |
| Collections / interpreter process | 3.392 | 2.878 | 15.2% |
| Tasks / JIT selection + verification | 2.423 | 2.052 | 15.3% |
| Collections / JIT selection + verification | 2.520 | 2.140 | 15.1% |
| HTTP / JIT process-to-READY (two pairs) | 6.188 | 5.926 | 4.2% |

Interpreter median process CPU falls from 3.351 to 2.853 seconds for tasks and
3.387 to 2.873 seconds for collections. These process timings include loading and
verification, not isolated interpreter execution. The first task pair is slower
in both variants. HTTP candidate readiness ranges from 5.819 to 6.032 seconds;
baselines are 6.187 and 6.189 seconds. Small desktop cohorts establish a local
improvement, not a universal performance estimate or peer-server result.

Single AOT compilation checks take 2.644 → 2.270 seconds for tasks and
2.790 → 2.463 seconds for collections; these support the shared-loading result
but are not repeated AOT timing cohorts. Task, collection and HTTP AOT objects
are byte-identical. All 24 ordinary invocations and four HTTP invocations pass
output/response, guest-status and shutdown checks. Warm native throughput is not
measured or claimed to improve.

Two unit tests pass, with the existing manual timing probe intentionally ignored.
The new test compares indexed and unscoped inherited-interface validation, including
wrong returns, readonly changes, private implementations and missing methods.
The existing Self test covers inherited signatures and Self in generic arguments.
All 104 integration tests pass across interface inheritance, explicit/default
implementations, helper methods, variance, Self, boxed and class receivers, common
interfaces and GC retention. The prior owner-index lifetime tests remain applicable;
index construction and scope ownership are unchanged.

`ordinary.json.gz`, `http.json.gz`, `summary.json`, `validation.json`,
`startup.sample.txt.gz` and `profile.json.gz` retain measurements, exact commands,
source/binary identities, unchanged bundle hashes, validation and the separate
startup profile. No public API/website snapshot or Raven bridge refresh is required:
semantics and emitted artifacts are unchanged. Native worker/collector checks are
not rerun for this Rust metadata-query change.

## Reproduction

Reuse the [local artifact preparation](../gc-mode-comparison/README.md). Retain
baseline binaries before building the candidate, then run sequentially:

```sh
python3 benchmarks/execution/owner-index/measure.py --output target/interface-owner-trials --baseline-runtime target/owner-index-followup/baseline-neoclr --baseline-codegen target/owner-index-followup/baseline-codegen
python3 benchmarks/execution/owner-index/http.py --output target/interface-owner-http --baseline-codegen target/owner-index-followup/baseline-codegen
python3 benchmarks/execution/owner-index/summarize.py target/interface-owner-trials/results.json target/interface-owner-http/results.json
```

The reusable harness now accepts explicit baseline binary paths; its prior defaults
remain unchanged. Ordinary timing uses three alternating pairs per sample/mode;
HTTP uses two pairs of process-to-READY measurements with short checked traffic
and shutdown. AOT object equality separates loading changes from generated-code
changes. These are not warm HTTP capacity trials or a Kestrel comparison.
