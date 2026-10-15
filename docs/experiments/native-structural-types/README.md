# Native structural type qualification

Development branch, 2026-10-15. This is integration evidence, not a release claim.

## Tuple AOT cost comparison

Runtime implementation: `4fe78429b`. Tool hashes and machine details are in the
raw result files. On Apple M1, the two-integer structural tuple and nominal record
both emit the **same 800-byte object**; sixteen-integer versions both emit the
**same 1,352-byte object**. Full object hashes match in each pair, not merely sizes.
Neither has undefined symbols, so these closed workloads cannot call a heap
allocator. Tuple construction uses inline slots with the same ABI as nominal records.

| Elements | Structural median | Nominal median | Structural / nominal |
| --- | ---: | ---: | ---: |
| 2 | 3.9608 ns/call | 3.9624 ns/call | 0.9996 |
| 16 | 13.124 ns/call | 13.122 ns/call | 1.0002 |

These differences are noise, not evidence of a speedup. Identical object code is
stronger evidence of equivalent cost for these workloads. The increased cost with
size is shared aggregate copy/call work and an optimization candidate for both
representations. Do not introduce a tuple-only optimization without measuring the
shared codegen path first.

Each workload constructs a value, passes it through Echo, copies it into a local
and reads the first and last elements. Inputs vary and the native host verifies
the accumulated result. Nine interleaved samples follow warmup, with five million
calls per pair sample and one million per sixteen-element sample. Timing uses the
monotonic clock around the loop, excluding compilation, linking and process launch.
The backend uses its normal Cranelift settings; the C timing host uses `-O2`.
Emission time is separately retained and is not a compilation benchmark.

Raw evidence: [pair](tuple-pair-results.json), [sixteen](tuple-16-results.json).
Exact generated inputs are retained next to these files. Reproduce with a matching
macOS ARM64 SDK (this host required Xcode's MacOSX26.2 SDK):

```sh
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --output target/tuple-cost-pair
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --elements 16 --iterations 1000000 \
  --output target/tuple-cost-16
```

The script preserves previous output directories. A flat nominal record currently
admits at most sixteen fields; larger comparisons need equally nested layouts.
There is no packed-layout, cache-pressure, managed-reference/GC, boxing, union,
interpreter or JIT qualification in this result. These remain explicit follow-ups.

## Direct JIT tuple execution

The shared JIT backend also executes both workloads successfully, with one million
calls per sample, nine interleaved measured samples after warmup and checked
results. The optimized Rust harness measured approximately 3.94 ns/call for the pair
and 13.9 ns/call for sixteen integers in both representations. These are comparable
within noise. Timing includes status checks/checksum but excludes compilation and
finalization. These measurements do not directly compare JIT and AOT speed because
the timing hosts differ. [Raw JIT samples and hashes](tuple-jit-results.json).

```sh
cargo test --release -p neoclr-codegen --lib structural_tuple_jit_costs \
  -- --ignored --nocapture
```

This explicit ignored performance test owns JIT code until synchronous invocation
finishes, then frees it. It validates direct backend construction, copying and
calls; the full managed host, reference-bearing tuples and collector stress remain
unqualified. Native intersections progress in the following slice; native unions remain a backend obligation.

## Native intersection reference views

The next backend slice preserves intersections as one checked reference slot.
Selection/specialization retain the complete bounds. Static assignment requires
all expected bounds; dynamic casts accept only concrete classes satisfying every
bound through explicit nominal conformance. Interface dispatch projects a guaranteed
nominal view. No member-name matching or carrier allocation is introduced. This
extends CLR-style explicit interface reference views with a conjunction signature;
it does not introduce a trait implementation model.

`intersection-views.neoil` and the focused console test compare AOT and interpreter
results/diagnostics across eight modes, including nulls, identity, interface calls,
managed output slots, missing interfaces and unrelated casts. The test repeats with
the second implementation removed, and with an extendable class/interface bound.
A closed-to-extension class bound is rejected before emitting an object file.
Tracing recipes retain one strong reference; collector-stress and dedicated JIT
intersection execution still need qualification. The failed-cast path uses native
InvalidCast status 11 and its standard message; legacy nominal cast behavior stays
unchanged. The subsequent union slices below extend native storage.

## Native reference-alternative union storage

The next slice represents unions of nominal reference alternatives with one payload
reference slot. This is a structural value representation: no synthetic nominal
union and no reference identity for the union container are introduced. The pointed
to object's tag provides the concrete identity for any-alternative membership.
Selection and specialization traverse native union signatures; assignment from a
union requires every possible alternative to satisfy the destination contract.
Extraction remains checked and explicit when a guarantee is absent.

The console test derives union signatures from `intersection-views.neoil`, inserting
an explicit View extraction before dispatch and an Echo function returning the union.
A further run exercises implicit admission of a conforming reference into a union
parameter. It compares eight modes for each of
four implementation sets: both bounds, just the first, just the second and neither.
The cases cover identity-preserving copies, calls, output slots, nulls, accepted and
rejected casts and missing membership. A mixed Int32/interface signature is rejected
before object emission when using an unsupported compound instance-test target. A separate projection test
ensures whole-union boxing cannot be rewritten into an Object payload cast.

This extends CLR-style reference casts with a structural union value contract. The
benefit for reference-only unions is a single slot with no carrier allocation; the
cost is runtime membership checks when static guarantees are insufficient. The
mixed scalar/reference path described below now preserves adjacent scalar
box/cast pairs through early projection and gives them tagged inline storage.
Nominal value alternatives still require additional layouts and conversions. Native reflection, array elements, JIT/collector stress and full Task
consumer qualification remain open. No general performance claim follows from the
single-slot layout test.

## Native scalar unions

Unions containing only Int32, Int64 and Boolean now use a private discriminator
plus a 64-bit numeric payload (two eight-byte backend slots). Neither slot is a
GC root. An exact adjacent `box T; castclass union<...>` pair retains the scalar
without allocating an Object box; the original instructions remain available to
verification and diagnostic positioning. Ordinary Object boxing is unchanged.
Compared with the current CLR-style boxed source conversion, this avoids an
allocation and indirection, at the cost of a two-slot copy and exact tag checks.
This is backend storage, not a stable public union ABI.

The focused `structural_scalar_union` AOT tests compare interpreter execution for
negative Int32, wide Int64 and Boolean values through local copies and calls/results,
output replacement, default null tests and failed extraction. The emitted objects
must have no allocator/runtime imports (C memory intrinsics are permitted).
A GC-layout test checks that numeric
payloads cannot become roots. These checks establish allocation elision in these
closed workloads, not a general throughput or managed-GC performance result.

Extraction currently accepts an immediate union local/argument load or union cast
before `unbox.any`. More general stack producers, compound-union `isinst` targets
and nominal value alternatives remain gated; bounded mixed storage is described below. Whole-union
boxing, native descriptors, arrays, dedicated JIT/GC stress and full Raven Task
consumers still block integration. The codegen qualification uses closed-world
selection, `--compile-system` and `--reference-arena` to admit the source
System.Object declaration;
no arena allocation is needed for the scalar payload.


### Scalar type-pattern tests

The native backend admits `isinst Int32`, `Int64` and `Boolean` on an immediate
scalar-union local/argument load or cast when followed immediately by `ref.isnull`.
It compares the inline tag and passes a one-lane non-null test token to the null
test. That token is neither an Object nor a GC root; it cannot escape this pair.
No box is allocated. Original operations remain in the verification model.
Ordinary nominal boxed queries retain their existing lowering.

`structural_scalar_union_pattern_tests_preserve_exact_types_and_null` compares
AOT and interpreter results for each active alternative and default null, tests
all three exact types, and extracts matched values (including false Boolean and
wide Int64). It rejects a duplicated test result before emitting an object. The
existing helper checks no allocator imports and zero arena usage. This extends
the CLR-style test/null-check instruction contract to inline union tags; its cost
is a tag comparison instead of payload allocation. General escaping test results,
compound targets and full Raven-produced pattern applications remain unqualified.


### Raven-produced scalar patterns

The [source consumer](scalar-pattern-project/README.md) now qualifies typed scalar
bindings, true/false guards, fallback arms and a match-result join from actual
Raven output. Six checks return 42 in the interpreter and macOS ARM64 AOT. The
native object has no undefined imports and the host observes zero arena use.
The [runner](verify-raven-scalar-patterns.py) retains fresh disassembly and the
selection report; [compact evidence](raven-scalar-pattern-results.json) identifies
compiler/runtime artifacts and the retained v3 bundle. This is one consumer,
not full mixed-union, JIT/collector or application qualification.

The [artifact-only imported consumer](scalar-import-project/README.md) also passes
all six inputs through a separately emitted library return and echo parameter,
including reordered source alternatives. Interpreter and AOT return 42 with no
native imports or arena use. [Imported evidence](raven-scalar-import-results.json)
records both fresh artifacts against the retained v3 bundle.

## Mixed scalar/reference union storage (2026-10-15)

The native backend now admits unions combining Int32, Int64 or Boolean with
selected nominal class/interface references. The union remains a structural
value. Its private layout uses three eight-byte storage slots: discriminator,
numeric bits and reference. Scalar injection clears the reference slot; reference
injection clears the numeric slot. Copies and calls preserve all three slots,
and only the reference slot participates in tracing recipes, including nested
tuple offsets. This deliberately spends one extra slot compared with scalar-only
unions to avoid interpreting arbitrary numeric bits as managed addresses.

The adjacent scalar `box; castclass union` sequence still omits the box. Reference
admission checks nominal conformance. Exact scalar tests/extraction use the tag;
nominal reference tests/extraction inspect only the separate reference slot.
Default/null, failed admission and extraction retain interpreter behavior.
Injection from Object or scalar-interface views needs boxed-payload classification.
Object views or interfaces implemented by scalar alternatives require boxing and
are rejected before native emission; a scalar must not silently become a null
Object view. Compound test targets, nominal value alternatives, intrinsic String
alternatives and general union-to-union conversions remain gated.

The `mixed-union.neoil` fixture exercises eight interpreter/AOT paths, repeated
with an explicit nominal interface alternative and a null-unboxing variant, including
wide Int64, false Boolean, copies through out parameters, returns, reference-to-
scalar replacement, null and failures. The host checks exact arena use: only the
explicit Cell/Other allocation, with no union carrier or scalar box allocation.
Tracing recipes are tested independently; they are not managed collector stress.
Reproduce with `cargo test -p neoclr-codegen --lib --test console mixed_union`.
This is macOS ARM64 qualification, without a throughput or managed-JIT claim.
Raven's Runtime Contract and emitted metadata/instructions are unchanged.
The [fresh Raven mixed consumer](mixed-pattern-project/README.md) now also passes
seven typed-pattern inputs in interpreter and AOT. Its only native import allocates
the explicit Response object; exact arena use is 16 bytes, without scalar boxes or
union carriers. [Artifact evidence](raven-mixed-pattern-results.json) identifies
the compiler, native tools, consumer and retained v3 bundle. Imported mixed calls
and broader native application qualification remain next.

The [imported mixed consumer](mixed-import-project/README.md) also passes all seven
inputs through separately emitted library returns and reordered union parameters.
It imports Response for its typed pattern/property call and allocates only that
explicit object (16 bytes). [Imported evidence](raven-mixed-import-results.json)
records both artifacts against the retained bundle. General payload layouts,
managed JIT/GC, boxing and full Task qualification remain open.

The [imported closed-generic consumer](generic-mixed-import-project/README.md)
also passes `int | long | bool | Response<int>` through the library boundary.
Response<T> stores a T field; the consumer reads the instantiated Int32 value and
uses a distinct reference-arm result. Exact arena use remains 16 bytes for the
explicit object. [Generic evidence](raven-generic-mixed-import-results.json)
records both artifacts. This does not qualify bare generic alternatives or all
open generic methods/constraints.

## Nominal value-record union alternatives (2026-10-15)

The native backend now admits unions of closed nominal value records, optionally
with Int32, Int64 and Boolean alternatives. These unions remain structural values;
the record alternatives retain their nominal identity. Equal field layouts do
not make two different record declarations the same alternative.

The private representation is one discriminator slot followed by one inline
payload region per alternative. Injection copies the active record and zeroes
every inactive region. Extraction checks the exact nominal discriminator before
returning a field snapshot. Calls, out copies and local replacement use the full
layout. References inside a record keep their usual identity; copying a record
does not clone the objects it references. Recursive tracing recipes preserve each
region's field offsets and exclude numeric fields. A pending record injection
also retains its reference tracing. These recipes are not collector stress tests.

This extends the CLR-style `box; castclass union`/`unbox.any` instruction sequence:
the exact adjacent injection pair avoids allocating a box or union carrier.
Ordinary Object boxing is unchanged. The tradeoff is that storage uses the sum of
alternative sizes plus a tag, rather than overlapping their fields. Inactive
reference regions are unambiguously null, at the cost of extra copy/zero work.
The existing 256-lane inline limit includes the union tag and every payload;
254 payload lanes plus one scalar and tag fit, while 255 exceed the limit.
Overlapping storage requires separate GC and performance qualification before it
can replace this layout. No throughput improvement or stable external ABI is claimed.

`record-union.neoil` checks numeric/floating fields, snapshot independence, out
copies, returns, exact nominal tests, replacement and null/wrong extraction.
`record-reference-union.neoil` checks embedded object identity, mutation visibility,
null fields and extraction failures against the interpreter. Its eight native
paths allocate only the explicit Cell object where present (16 bytes); the union
and record payload add no allocation. A separate record-only test checks unions
without scalar alternatives. Reproduce with:

```sh
cargo test -p neoclr-codegen --lib value_union
cargo test -p neoclr-codegen --test values --test console structural_record_union
```

Qualification is macOS ARM64 with the existing native fixture hosts. Native
verification also retains/restores Object for unbox-only entry points, preserving
the VM verifier contract without emitting an Object allocation. Enums,
whole-union boxing, managed JIT/GC and full Task/throughput qualification remain
open. Compiler emission and Runtime Contract configuration are unchanged.

The [fresh artifact-only Raven record consumer](record-import-project/README.md)
now also qualifies imported `int | long | bool | Response`, where Response is a
struct with Int32 and Int64 fields. Seven inputs preserve reordered union
parameters/results and the wide payload; interpreter and AOT return 42 with no
native imports and zero arena use. [Record consumer evidence](raven-record-import-results.json)
records both fresh artifacts and the retained v3 bundle. No throughput claim follows
from eliminating boxing in this bounded consumer.

### Direct reference alternatives beside records (2026-10-15)

The bounded value-union layout also admits nominal reference alternatives. The
first reference alternative's region holds any admitted reference payload; other
reference regions remain zero. This private storage choice does not select an
exclusive nominal case: casts and tests inspect the actual object's conformance,
including an object implementing multiple interface alternatives. GC recipes mark
these regions as union-reference payloads rather than asserting the first bound.
Record/scalar injection still clears every inactive region and allocates no box.

The console tests `structural_record_union_accepts_direct_reference_alternatives`
and `structural_record_union_reference_alternatives_use_nominal_conformance`
compare copies, calls, reference identity/mutation, type tests, replacement and
null/wrong extraction against the interpreter. Arena checks permit only the
explicit Cell allocation (16 bytes). Object and value-interface projections that
would require boxing remain gated. Testing against an entire value-union target
(`isinst union<...>`) also remains gated; testing individual alternatives is
qualified here. The sum-of-regions size and 256-lane cap remain;
this is not a compact-layout or managed collector qualification.


A [fresh artifact-only Raven record/reference consumer](record-reference-import-project/README.md)
also qualifies `int | long | bool | Response | HttpError`, where Response is a
struct and HttpError is a class. Eight imported Produce/Echo calls exercise
reordered signatures, scalar guards and distinct record/reference pattern arms.
Interpreter and AOT return 42; the only native import is object allocation and
arena use is exactly 16 bytes for HttpError. Sources, artifacts, tools and retained
v3 bundle are fingerprinted in the [evidence](raven-record-reference-import-results.json).
The bundle itself was not rebuilt. Compiler emission and Runtime Contract
configuration are unchanged; this adds no managed JIT/GC or throughput claim.


### Closed generic record identity (2026-10-15)

The [closed generic record/reference consumer](generic-record-reference-import-project/README.md)
qualifies `int | long | bool | Response<int> | Response<long> | HttpError` across
artifact-only imports. Nine inputs preserve scalar guards, both closed record
identities and their substituted fields, plus the direct class payload. Both
records have two native field lanes; layout similarity does not make them the
same type. Interpreter and AOT return 42 with only the explicit HttpError
allocation (16 bytes). [Evidence](raven-generic-record-reference-import-results.json)
fingerprints the rebuilt compiler, fresh library/consumer and retained v3 bundle.

The reproducer first exposed a general Raven reachability bug: Response<long>
was incorrectly marked unreachable after Response<int> because pattern comparison
erased generic arguments. Main-based Raven commit f7a4bc5a4 replaces that helper
with the shared symbol comparer; native-branch carry df02f07ad has its own 195-test
validation. The main-based regression run first failed four distinct-type cases
(class, struct, nested and imported), while the identical-type rejection passed;
after repair all 195 match/identity tests passed. The main branch itself was not
merged or advanced. Runtime Contract configuration and native emission are unchanged.

This is closed Int32/Int64 substitution coverage, not qualification of every open
generic or reference-valued T. Managed JIT/GC, boxing, compact layout performance
and full Task application execution remain open.
