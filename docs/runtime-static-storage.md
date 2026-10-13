# Runtime-owned static storage

Author-directed work, 2026-10-14. This is an incremental implementation record,
not a claim that Raven static storage is ready in every execution mode.

## Contract and comparison

The author requested a **static** header-name catalog and explicitly directed
fixing static storage rather than keeping an instance-cache workaround. The author
also clarified that each closed generic type needs its own static members.

Follow [.NET/C# static-field semantics](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes#155-fields):
one storage set per non-generic or closed constructed type. For example,
`Cache<int>` and `Cache<string>` do not share fields, even if their implementations
share machine code. Repeated `Cache<int>` accesses do share fields. Static storage
does not contribute to instance layout. Readonly storage permits initialization,
not later reassignment; a readonly reference does not make its target immutable.

neoCLR ownership is explicitly the runtime invocation/context and loaded program.
This supports isolated hosted runs, cleanup and non-process-global roots. It is not
an assembly-name dictionary or a property of the executing OS thread. Use the
existing resolved `TypeIdentity`, including definition module/revision and recursively
resolved type arguments. Names, generic definitions and code-sharing keys are
insufficient. Future longer-lived host contexts must preserve this ownership explicitly.

## Implemented first slice

- Native runtime format-5 `TypeDef.static_fields`, separate from instance fields;
  each row records name, type, visibility and readonly status. Omission means none.
- Structured `ldsfld` / `stsfld` operations name the declaring closed owner and a
  static-field ordinal. Type substitution, module-scope normalization and dependency
  checks include these signatures.
- Interpreter invocation-owned, lazily allocated storage, shared by that
  invocation's contexts. Scalars and nullable references use their defaults; owned
  arrays and other values without an admitted default remain checked-uninitialized
  until assigned. Managed graph access precedes the storage lock. Reads do
  not republish the whole root set; writes update service roots and inline array
  accounting. Entries are bounded by the invocation heap-object limit, with at most
  256 static fields per declaration.
- Runtime and verifier enforce visibility, ordinal/type checks and readonly-store
  rejection. Readonly writes remain rejected until type-initializer admission is
  implemented. No constructor-name bypass has been added.
- Storage roots survive allocation-pressure collection and disappear with invocation
  teardown. Inline arrays remain charged after their frame operand is consumed.

Validation: `cargo test --test static_storage`; adjacent readonly-field, generic-class,
accessibility and runtime-GC tests; `cargo check --workspace`. The tests cover closed
argument separation, `T` field substitution, reset, JSON metadata round-trip, instance
layout independence, non-verifying rejection, array budgets, and full reclamation of
21 objects under a three-live-object limit. These are local macOS interpreter checks,
not native JIT/AOT or Windows qualification. No HTTP performance claim follows.

## Native execution and Raven integration

Raven native branch `fac4ac736` opts into static-field lowering. The metadata
writer/reader preserves Static/InitOnly flags and separate static ordinals;
loads and stores retain the closed owner. Default private static `var` storage
and implemented static properties work. This does not change Raven main or
ordinary .NET defaults, and requires no new Runtime Contract setting.

JIT and AOT use one image-local table per native context, indexed by the selected
closed type among declarations with static fields (unrelated types consume no slot). Generic specialization includes static signatures and owner operands.
A table entry owns an independently allocated field block; instance layout stays
unchanged. Warm accesses load/store directly. Only a missing table/block publishes
pending operands and calls the allocating helper. Native storage currently admits
zero-initializable scalar/reference/record layouts; checked-uninitialized inline
arrays, runtime type handles and erased values remain a native admission gap.

The table and children use the native collector's existing conservative object-word
tracing. This is not a new precise or generational collector. A context root keeps
them alive across collection; ordinary entry completion releases static and intern
pool ownership. A rooted interned String remains alive through its other roots.
Bootstrap/callback hosting retains the context until its owner completes it.
The private native context grows from 1,072 to 1,080 bytes: **rebuild native hosts**
with the matching header/backend. Entry names remain experimental, not a stable ABI.
GC and non-GC native tests cover isolation, child retention, failure output stability,
generic separation and three repeated entries with full reclamation. The adjacent
collector test now explicitly enables collector hooks instead of diagnostic hooks.

## HTTP catalog

`System.Web.Http.HttpHeaderNames` is internal and static, with 13 read-only lazy
properties and private mutable backing fields. The getter interns its fixed literal
only on a cache miss, then caches the canonical result. The existing runtime intern
operation makes competing first readers return the same immutable value; first-use
races may still create discarded temporary literals. This is publication through
interning, **not** an implementation of `Lazy<T>.ExecutionAndPublication` or `.cctor`.
No request-provided names or values enter the intern pool.

Costs include the context table, field block, canonical strings and pool records;
up to 13 fixed names consume the existing invocation intern quota. Warm reads do
not look up the pool. Untouched names have no String/pool allocation, although the
field block contains all nullable slots. Type-initializer/atomic publication support
could later remove the pool coupling; measure that separately.

The 44 public HTTP wire cases pass in interpreter, JIT and AOT (132 checks), including
identity reuse across explicit collections, mixed casing, fragmented input, fallback
lookalikes and framing rejection. These are sequential macOS checks with runtime-await,
not concurrent first-read stress or Windows execution qualification. Windows x64
emission of the separate-library generic/static consumer also succeeds; execution
on Windows remains to be qualified. See the [header evidence](../benchmarks/http-server/header-static-names.md).

## Remaining work

- Type initialization: once-only state per closed type, recursion, competing contexts,
  cycles and non-recoverable fault propagation. Source static initializers and readonly
  static writes remain rejected; there is no constructor-name privilege bypass.
- Static field addresses, attributes/origin tokens, textual neoIL authoring, and the
  compiler's constructed-field owner representation for generic **static classes**.
  Ordinary generic classes with static members are tested and have separate storage.
- Native layouts that need checked-uninitialized defaults, concurrent first-read
  stress, Windows execution, and a public hosting completion contract.

The [static constructor guidance](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/static-constructors)
remains the initialization comparison. Thread-safe type initialization alone does
not make an arbitrary nullable-field getter thread-safe.
