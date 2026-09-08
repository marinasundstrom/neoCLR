# Reference experience: an order workflow

This first experiment asks whether explicit managed references earn their cost in
ordinary application code. It is an executable, scripted order workflow, not a
production application or a human usability study. No runtime or language contracts
are changed. The initial findings below describe Preview 3 behavior on 2026-09-08.

Follow-up: ArrayList now shares complete managed state, and Copy() explicitly creates
an independent sequence. The current list probe therefore prints the same five lines
as C#: `20, 2, 2, 50, 50`. Historical observations below are retained as the reason
for this change; see [the updated contract](../../array-list.md).

The second follow-up adds ordinary `Result<T,E>.Ok(value)` / `.Error(error)` library
factories usable from Neo. The current workflow uses Result; the placeholder
PurchaseOutcome and failed construction attempts below describe the initial experiment.
The C# baseline retains its original outcome record, so it is now a behavior comparison
rather than identical error-handling structure. See [Result construction](../../result-construction.md).

## Run it

From the repository root:

```sh
cargo run --locked -- run examples/source/order-workflow.neo
cargo run --locked -- debug examples/source/order-workflow.neo
cargo test --locked --test reference_experience
cargo run --locked -- run docs/experiments/reference-experience/list-copy.neo
cargo run --locked -- run docs/experiments/reference-experience/frame-callback.neo
```

The last command intentionally faults. The successful workflow prints:

```text
Purchased: Coffee
24
7
24
Out of stock
Orders complete
=> Int32(0)
```

It maintains a shared product catalog, finds a product using a predicate, purchases
stock through a service function, calls a notification interface, stores a receipt
snapshot, changes the current price, invokes a returned restocking closure, rejects
an oversized order, and checks a missing product. Monetary amounts are integer
units; there is no claim to implement currency, persistence or concurrent ordering.

The equivalent C# workflow and list-assignment probe are in [dotnet](dotnet/Program.cs).
They intentionally use the same simple outcome record, so the comparison does not
attribute error-handling verbosity to reference semantics. Run with SDK 10.0.100:

```sh
cd docs/experiments/reference-experience/dotnet
dotnet run
```

The pinned SDK is deliberate: execute from that directory so global.json is used.
The C# process throws on violated workflow expectations. Its first six lines match
Neo's guest output; its final five lines are the separate list probe.

## Observations from executing the programs

| Scenario | Observed behavior | Interpretation |
| --- | --- | --- |
| Shared inventory | `ArrayList<Product&>`, `new Product`, and `Product&` parameters preserve stock updates across lookup and purchase. Ordinary `product.Stock` access needs no dereference. | Explicit sharing is readable once the object is in reference form. Most local binding types are inferred. |
| Receipt snapshots | `Receipt` stored by value retains total 24 after the product price changes. | Direct values fit snapshot data. This receipt contains only string and integer fields; it does not establish deep-copy behavior for arbitrary records. |
| Notification interface | A local notifier is passed with `&notifications`; dispatch works without manual allocation. | Useful local interface view, at the cost of an address expression at the call site. |
| Returned closure | Capturing a heap-backed `Product&` works and restocks to 7. | No explicit lifetime management in the successful path, although managed capture allocations exist. |
| Frame-backed callback | The same helper called with `&product` from local value storage faults while capturing, even though the callback is invoked before Main returns. | The parameter type does not communicate that this implementation requires a heap-backed target. Current stored-reference rules are conservative; local use of the returned callback does not relax them. |
| List assignment | After copying a list descriptor, element edits are shared until growth; counts are independent immediately. | This is a library copy-contract issue, more surprising than spelling `&`. It is documented behavior, not a newly discovered implementation defect. |
| Value element editing | Reading a value element, modifying the local copy, and omitting writeback leaves the list unchanged. A reference element updates the shared product. | Users must know the element mode. The test exercises explicit local copies, not direct `list[0].Field` mutation. |

The list probe prints `20, 1, 2, 20, 50` in Neo and `20, 2, 2, 50, 50` in
C#. In Neo, growing the copied descriptor detaches its buffer. Capacity thereby
changes which later edits are shared. It is neither an independent snapshot nor a
shared whole-list alias. An explicit `ArrayList<T>&` aliases the entire descriptor,
but requiring callers to remember that choice is part of the experience under test.

### Friction unrelated to reference semantics

The first attempt used `Result<Receipt,string>.Error("Out of stock")`, which failed
with `unknown function overload System.Result.Error([String])`. Explicit wrapping
with `Result<Receipt,string>(System.Result.Error<string>(...))` failed with
`generic calls require a free function or static method`. These are the two attempted
forms, not proof that every possible construction is unavailable. The library has
case constructors, but the frontend path was not straightforward. The working
sample uses `PurchaseOutcome` with an Accepted flag and a placeholder receipt on
failure. That workaround allows invalid state combinations and is not a recommended
replacement for Result. Investigate ergonomic union construction independently.

Fully qualified collection/delegate names, nested matches, and a small library also
contribute verbosity. None should be counted as evidence against managed references.
The tests establish semantics; their author already knows the runtime, so they do
not measure learnability or productivity.

## .NET comparison and design alternatives

Primary sources consulted 2026-09-08:

- [C# reference types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/reference-types)
  describes class-reference assignment as copying a reference to shared data. The
  comparison uses a Product class and List<int>, with no C# ref parameters needed
  to mutate their objects.
- [C# value types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types)
  describes value copies and the shallow copying of reference-valued fields. The
  comparison uses a record struct for Receipt. Neo's list descriptor behaves like
  a struct containing a reference buffer and a separate count, not like List<T>.
- Existing platform analysis: [collection contracts](../../../docs/api-policy.md),
  [managed closures and CLR lowering](../../../docs/delegate-contract.md#closure-lowering),
  and [Neo managed access](../../../docs/neo.md).

These are language and library comparisons plus an executed .NET 10 probe. They do
not establish a CLR requirement for Neo's collection representation, make claims
about current JIT internals, or benchmark allocation/copying speed.

| Candidate | Benefit to investigate | Cost / unanswered question | Placement |
| --- | --- | --- | --- |
| Keep explicit modes | Sharing remains visible in contracts; local values can be borrowed. | Heap-only capture requirements are not apparent in T& alone. | Current runtime and Neo |
| Infer more borrowing | Fewer call-site annotations. | Does not repair partial list sharing or make a frame reference safe to retain. Avoid overload ambiguity and hidden changes to aliasing. | Language experiment |
| Make collections shared by default in a language | Matches the familiar C# whole-list alias behavior. | Weakens uniform value defaults; needs an explicit independent-copy operation and clear projection rules. | Language/library experiment |
| Independent collection copies | Assignment could produce an independent sequence. | Allocation and copying cost; element references still share targets. Copy-on-write would need enforcement through every mutation path. | Library plus possible runtime support |
| Broader automatic lifetime extension | More callbacks could work without a storage decision. | Must preserve identity across existing aliases; copying a target is not equivalent. Escape analysis or owner promotion is substantial work. | Separate compiler/runtime investigation |

Provisional conclusion: retain the runtime model while testing alternatives. Prioritize
collection copy expectations and callback lifetime diagnostics before merely removing
ampersands. No choice above is adopted here; performance and human experience remain
unmeasured. Language-level class defaults are a possible projection, not a requirement
to reinstate a nominal value/reference split in the runtime.

## Try changes before reading the implementation

For a human trial, predict the outcome, make one change, then run it. Record elapsed
time, diagnostics, surprising behavior, and whether documentation was needed. Do not
interpret these as benchmark scores or compare timings from different participants.

1. Purchase Tea as well and produce a combined total without changing old receipts.
2. Add a second notification implementation and swap it into the workflow.
3. Make a catalog snapshot, then edit stock and add a product. Specify which changes
   you expect the snapshot to observe before deciding how to represent it.
4. Move product creation into a helper and return a restock callback. Try direct
   value storage and heap storage; explain the observed difference.
5. Replace the outcome record with a useful Result-returning API when construction
   is supported, recording which difficulty came from the frontend rather than memory.

Before selecting the next contract, build a second scenario with different needs
(e.g. document editing with undo). One mutable catalog does not justify universal
reference defaults, and one receipt does not justify universal copying.

## Recorded validation

Local macOS ARM64, 2026-09-08: the Neo workflow ran with the output above; the focused
Rust integration suite passed four tests, including source-to-JSON loading of the
successful scenarios and the intentional frame-capture fault. The .NET comparison
ran with SDK 10.0.100 and printed the matching workflow plus its documented list
results. These are local experiment results, not a new release certification or a
full-suite run. The four tests characterize current behavior; they may need deliberate
revision if the experiment leads to a new contract.
