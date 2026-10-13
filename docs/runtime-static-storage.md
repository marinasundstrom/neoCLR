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

## Remaining slices before using the catalog

1. Type initialization: zero storage before execution; once-only state for each closed
   type; owner/reentrancy, competing contexts, cycles and fault propagation. Faults
   stay non-recoverable. Readonly stores require an admitted initializer context.
2. C# metadata writer/reader/reference projection and Raven lowering for static fields,
   auto-storage and static constructors. Preserve CLI-shaped Static/InitOnly flags;
   ordinary Raven/.NET behavior remains separate. Generic class support must not be
   reduced to a nongeneric special case. Textual neoIL syntax is also pending.
3. JIT/AOT storage, precise native GC roots, context reset and matching initialization.
   Closed-type data must remain distinct even where method code is shared. The current
   native backend does not implement the new operations; it must not silently
   substitute process globals or an interpreter fallback.
4. Static lazy HTTP catalog, repeated-identity and concurrency/GC checks, then matched
   allocation/throughput measurements and Windows validation. The earlier per-server
   catalog draft was removed; it is not the implemented API or final architecture.

The [static constructor guidance](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/static-constructors)
is the comparison for initialization. Thread-safe type initialization alone does not
make an arbitrary nullable-field lazy property thread-safe. Prefer per-name initialization
holders or an explicit publication primitive when integrating the catalog; do not
silently allow competing allocations or partially published values.
