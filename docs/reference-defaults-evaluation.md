# Reference and value defaults: a bounded architecture evaluation

Evaluated 2026-09-12 against `c15839f6be736d40aa148e81f90e3eb27d7fd266`.
This compares alternatives; it does not select a migration or change current semantics.
The criteria come from [conversation entries 23–26](development-timeline.md): familiar
application programming, useful object behavior for primitive/user-defined data,
Result/Option APIs, nullable support, and meaningful separation of language and runtime.

## Evaluation priority clarified after review

On 2026-09-12 the author emphasized that neoCLR is the primary product of the
experiment. Neo is a small language for demonstrating and testing patterns by emitting
IL and executing it. Improving both is welcome, but hiding differences in Neo is not
sufficient evidence of a better runtime. Preserve useful CLR behavior and the foundations
that enable its strengths; seek compatibility and avoid unnecessary divergence.

The required compatibility level is not yet selected: familiar APIs, source targeting,
metadata/IL interoperability and existing binary execution are different promises.
Do not interpret this clarification as a claim of existing binary compatibility, or
as approval to retain every current runtime decision behind a compiler convention.

Before the proposed compiler experiment, assess the relevant runtime contracts against
CLR: object references versus references to slots, construction/copying, initialization
and nullable storage, and generic/interface use. For each, identify what can be retained,
what compatibility gap should be repaired, and what deliberate difference has a concrete
benefit. Use Neo to exercise those contracts; adjust its projection as the runtime design
warrants. The recommendation below is a candidate technique, not a language-first priority.

## Finding and recommendation

The order workflow can express its sharing and snapshot behavior using existing
runtime capabilities. Most visible reference syntax can plausibly be removed by a
compiler projection. That observation does **not** establish that a language-only model
is sufficient for a multi-language platform, or that the entire C# object model can
already be projected faithfully.

Prefer evaluating **shared type-declared usage defaults, lowered to explicit storage
signatures**, before considering a new runtime type category. A producer could describe
its intended ordinary reference/value use for importing languages, while execution
continues to use explicit signatures. This is a recommended candidate, not a chosen
metadata schema. A convention is insufficient if its meaning is that direct value use
must be forbidden or identity/immutability must be guaranteed by the runtime.

After that runtime-contract assessment, a bounded language projection of the workflow
with explicit cross-module import/export checks remains a possible validation experiment. Use it to determine the
minimal shared information required. Do not change the meaning of existing Named types,
newobj, or artifacts just to obtain cleaner source spelling. In particular, resolve the
reference-to-reference-slot gap before claiming general C# targeting.

## Evidence and limits

Inputs inspected:

- [Executable Neo workflow](../examples/source/order-workflow.neo),
  [existing experience report](experiments/reference-experience/README.md), and
  [C# baseline](experiments/reference-experience/dotnet/Program.cs).
- [Workflow regressions](../tests/reference_experience.rs),
  [reference assignment](../tests/neo_reference_assignment.rs), and
  [outputs](../tests/neo_outputs.rs), including source/artifact round trips.
- [Type signatures and definitions](../src/metadata.rs): Type::ByRef and ReadOnlyByRef
  describe use; TypeDef has no ordinary value/reference default. Representation selects
  Record/Runtime/Interface/Delegate, not copying or ownership policy.
- [Compiler](../src/frontend.rs): source reference types map to managed-reference
  signatures; aggregate calls emit newobj, and heap construction adds heap.new.
  Current `class` parsing handles construction/member form; it does not establish
  the reference-default model proposed here. Renaming `record` to `class` is insufficient.
- [Slot access](../src/slots.rs) rejects addressing a reference-valued field as a nested
  managed reference. [VM return checks](../src/vm.rs) reject references into the returning
  frame. [Storage conversion](../src/value.rs) preserves declared readonly restrictions.
- [Existing reference-assignment analysis](reference-assignment.md) identifies that
  out Foo& writes Foo storage, not a caller's Foo& binding.

This is one scripted application and a bounded source review, not a whole-runtime audit,
human usability study or allocation benchmark. Old documents contain milestone-specific
statements; the inspected source and current tests take precedence for this assessment.
No experimental syntax below has been compiled. Current emitted IL was inspected with
emit-il; it contains Product& signatures, ArrayList<Product&> and heap.new after Product
construction. No new compiler implementation is needed to establish that existing mapping.

## .NET baseline, separated by layer

Primary sources consulted 2026-09-12; runtime-specific examples use the pinned .NET 10
SDK 10.0.100 in the existing comparison project. No JIT implementation claim is made.

| Layer | Baseline and relevance |
| --- | --- |
| C# language | [Reference types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/reference-types) hold references; ordinary class assignment shares the object. [Parameter rules](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/method-parameters) distinguish copying that reference from passing the variable's storage by ref/out. |
| Value behavior | [Value types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types) copy contents; reference-valued fields still alias. This is not deep copying or a guarantee of stack allocation. |
| Metadata and execution | [.NET 10 newobj reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.newobj?view=net-10.0) distinguishes object construction from producing a value-type instance. Reusing the mnemonic does not make neoCLR's newobj plus heap.new contract identical. |
| Nullability | [Nullable reference annotations](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-reference-types) do not change the runtime type. NeoCLR's proposed unified, enforced nullable model is separate work, not supplied by a reference-default annotation. |
| Library policy | The C# baseline uses List<Product>, a value Receipt and a simple outcome record. Neo uses ArrayList, Option and Result. Their inventory behavior is comparable; their error API shape is intentionally different. |

Adopting CLR's full nominal value/reference split would provide a familiar model but
requires more than an annotation: construction, signatures, generic instantiation,
boxing/conversions and validation would need a coherent migration. A System.ValueType
base is not proposed here. Omitting that hierarchy does not eliminate the need to
specify the operational distinction. Broader CLI conformance is outside this evaluation.

## Three models for the same workflow

All models must preserve: shared Product stock/price; receipt total 24 after a later
price change; successful restocking through a returned callback; failed purchases that
neither mutate stock nor notify; and missing lookup represented by Option.

### A. Current Neo: explicit use modes

These excerpts are executable source from the current workflow:

```swift
record Product(Name: string, Price: int, Stock: int)
record Receipt(Name: string, Quantity: int, Total: int)

func FindProduct(products: readonly System.Collections.ArrayList<Product&>&,
                 name: string) -> Option<Product&> { /* existing body */ }
func Purchase(product: Product&, quantity: int,
              notifications: Notifications&) -> Result<Receipt, PurchaseError> {
    /* existing body */
}
```

The braces with comments abbreviate bodies; copy the linked full sample to run it.
Main uses these actual statements:

```swift
var products = System.Collections.ArrayList<Product&>.Allocate(0)
products.Add(new Product("Coffee", 12, 5))
var notifications = ConsoleNotifications()
let Some(product) = FindProduct(&products, "Coffee") else { return 5 }
let Ok(receipt) = Purchase(product, 2, &notifications) else { return 1 }
```

Product locals inferred from new or lookup already need no & at every use. The cost
concentrates in public signatures, nested generic arguments and borrowing a locally
stored receiver. The catalog descriptor is a value with shared managed state; its current
assignment behavior already matches whole-list sharing, without a nominal class default.

### B. Shared type-declared defaults

Illustrative source, **not accepted syntax or a settled immutability contract**:

```text
class Product(Name: string, Price: int, Stock: int)
value class Receipt(Name: string, Quantity: int, Total: int)

func FindProduct(products: readonly ArrayList<Product>,
                 name: string) -> Option<Product>
func Purchase(product: Product, quantity: int,
              notifications: Notifications) -> Result<Receipt, PurchaseError>

let products = new ArrayList<Product>(0)
products.Add(new Product("Coffee", 12, 5))
let notifications = new ConsoleNotifications()
let Some(product) = FindProduct(products, "Coffee") else { return 5 }
let Ok(receipt) = Purchase(product, 2, notifications) else { return 1 }
```

For this comparison, Product and the collection facade default to managed references;
Receipt and the existing union carriers retain direct value use; Notifications denotes
an interface reference view. Those classifications must be declared, not inferred from
names. `readonly` here means a readonly view, not an immutable binding or deep freeze.
The existing Allocate API would need a constructor or factory projection; its spelling
is not a benefit supplied by a runtime flag.

Two materially different implementations are possible:

1. **Shared projection contract:** importers learn the default from metadata and lower
   uses to explicit T/T& signatures. Existing IL still defines execution. Raw IL can
   express non-default uses unless independently prohibited.
2. **Enforced nominal category:** the runtime rejects uses inconsistent with the type's
   category or interprets nominal slots differently. This changes execution and artifact
   meaning; it is not merely shared naming or a convention.

This evaluation favors testing the first before adopting the second. A hint cannot
promise universal immutable/identity-free instances. If those are requirements, the
second path or separate enforceable capabilities need their own evidence and design.

### C. Language-only defaults over current runtime

The source could look exactly like B. Its distinction is artifact behavior: this
compiler knows Product means Product&, while another language sees explicit Product&
in method signatures but has no standard declaration-level default for constructing
or declaring new uses of Product. Existing signatures still interoperate precisely;
it is the unexpressed intent for new uses that is missing, not execution type safety.

A provisional lowering table makes the experiment concrete:

| Source concept in B/C | Existing explicit runtime form |
| --- | --- |
| Product parameter/field/element | Product& |
| Receipt parameter/field/element | Receipt |
| Option<Product> | System.Option<Product&> |
| ArrayList<Product> instance | System.Collections.ArrayList<Product&>& if the facade is also reference-default |
| readonly catalog view | readonly System.Collections.ArrayList<Product&>& |
| Notifications parameter | Notifications& |
| Product construction | Construct Product, then heap.new |
| Receipt construction | Construct Receipt directly |
| Product assignment | Store/copy the existing managed reference |
| Explicit borrowing of a value | Address its existing slot; preserve frame-escape checks |

The collection row differs from A: A already shares the backing state through a directly
held facade. Allocating that facade on the managed heap may add an allocation; hiding its
& is not proof of better memory use. B and C can instead preserve the facade representation
with a suitable API/compiler convention, but must not silently change copy/alias behavior.
No allocation elimination is assumed. A heap-backed notification likewise differs from
A's frame-local borrowed implementation. These costs need measurement if adopted.

## Behavioral comparison and boundaries

| Operation | A: current | B: shared defaults | C: language-only defaults |
| --- | --- | --- | --- |
| Construct ordinary object | Explicit new produces T&; direct constructor call produces T | Declaration selects ordinary mode | Compiler selects ordinary mode |
| Assign/pass Product | Existing reference copies naturally; parameter binding itself is immutable in Neo | Same intended object sharing | Same lowering is available |
| Copy Receipt | Direct snapshot, reference fields would remain shared | Explicit value declaration communicates intent | Compiler convention communicates intent locally |
| Collections | Element T versus T& is visible; descriptor can itself be direct or referenced | Element mode follows declaration; collection API must preserve semantics | Same, but imported default choices need a mapping |
| Interface calls | Explicit view; no copy/box required for this path | Interface spelling hides view formation | Same runtime dispatch available |
| Return/capture | Heap roots survive; retaining a frame reference faults | Heap-default construction avoids this common trap; explicit borrows still restricted | Same; no automatic lifetime promotion needed for ordinary new objects |
| Result/Option | Existing case/carrier conversions and extraction | Substitute effective payload use; retain value carriers | Same generic arguments after lowering |
| out Product | out Product& writes Product, not Product& storage | Needs distinct reference-slot output capability for C#-like semantics | Cannot be solved by token omission; helper-cell lowering would require a specified alias contract |
| Null/default | No new absence state from references alone | Must specify nullable slots and initialization independently | Cannot safely emulate general nullability with a spelling convention alone |

Explicit value access should remain possible even under B/C, but its syntax is not selected.
Do not automatically make `Foo&` mean borrowing a reference slot once Foo defaults to
references: that would change existing APIs. Copying a referent, rebinding a reference,
and filling an output are separate operations. C# permits parameter reassignment of its
local copy; Neo's immutable parameter bindings are a separate language difference.

## What belongs where

| Concern | Recommended boundary to investigate | Reason / cost |
| --- | --- | --- |
| Default source use | Shared declaration intent for interoperable APIs; compiler expands uses | Avoid repeating external-type configuration in Raven/C# frontends; needs versioned import rules |
| Stored type and copy behavior | Explicit signature and runtime validation | A producer cannot bypass safety merely by ignoring a frontend convention |
| Construction syntax and inference | Language | Cannot infer physical allocation solely from spelling or class keyword |
| Native primitive payload | Runtime-known representation plus ordinary members | Already separate from value/reference use; no wrapper field or universal base required |
| Equality, hashing, record helpers | Ordinary API contracts/compiler generation | Object-like behavior does not require shared reference identity or a new storage category |
| readonly access | Existing runtime capability, projected by language | Must survive IL/artifact inputs and interface dispatch |
| Immutability / identity-free optimization | Separate contract if required | No default-use hint may justify eliding observable identity or ignoring mutations |
| Result/Option shape | Ordinary library types and language conveniences | Reference defaults must not accidentally turn all union carriers into heap objects |
| Nullable representation and Fault containment | Separate runtime/API evaluation | Required for general targeting; neither solved by class defaults |

## Compatibility and implementation costs

**Source/API:** Changing the meaning of a bare type is breaking even if all code still
parses. Generic arguments, fields, overload resolution and inference change. Existing
class syntax already has value use, so a new mode cannot quietly reinterpret old sources.
Reference RHS versus value RHS assignment and explicit copies require migration diagnostics.

**Artifacts/generics:** Preserve explicit lowered signatures initially. A default change
can still alter newly compiled APIs from T to T& and must be treated as an API change.
Decide canonical identity, constraints and how imported generic arguments expand; avoid
applying a default twice to an already explicit reference. Any metadata addition needs
version handling, loader checks and unknown/default-field policy, not an informal marker.

**GC/lifetimes/native interop:** More automatic heap construction changes root/allocation
patterns. Existing frame checks must remain for explicit local values; never copy-promote
an aliased value as if it preserved identity. Native pointers still require explicit
layout/lifetime contracts. A class default cannot infer native ABI layout.

**Reflection/debugger:** Expose both the nominal definition and effective storage type.
If source says Product but the signature is Product&, tools need a documented projection;
reflection must not hide the distinction needed for safe invocation. Source maps should
continue to point to construction and call sites after generated operations.

**JIT/AOT:** Shared defaults alone supply no optimization proof. Compiled backends must
preserve observable aliasing, copy and readonly semantics. Inline layout and allocation
elision remain implementation opportunities subject to those proofs, not measured wins.

**.NET targeting:** Keeping instruction families is valuable, but current newobj/heap.new,
reference outputs, nullability and Result/Fault behavior prevent claiming drop-in CLR
semantics. C# exception handlers cannot automatically become Result without decisions
about return types, call chains, cleanup and catch boundaries. Treat this as source/API
migration, not binary compatibility. This evaluation does not propose exception opcodes.

## Recommended next slice and acceptance gate

Subject to the runtime-contract assessment above, build a deliberately limited, opt-in
compiler experiment for Product (reference default),
Receipt (value default), Notifications and the two collection uses. Keep the current
workflow as the control. Do not ship a new default or introduce general inline flags.

The experiment should emit existing explicit signatures, then import one produced API
through an independent declaration reader or minimal second projection. Compare a shared
default marker with an explicit importer mapping. This tests the reason for metadata:
consistent interpretation of newly declared uses across a module boundary, rather than
only reducing & in one source file. Avoid assuming a second full compiler is necessary.

Acceptance requires:

- Matching inventory, notifications, receipt snapshots and error cases in all projections.
- Inspectable emitted signatures for fields, returns, generics and interface calls;
  serialized-artifact execution must agree with source execution.
- Assignment aliases Product and copies Receipt; explicit copies remain distinct from
  reference retargeting. Reference-valued fields retain shallow-copy behavior.
- Returning/capturing heap objects succeeds; explicit frame-reference escape is rejected.
  Wrong payload types and readonly-to-writable conversions fail after artifact loading.
- Imported defaults neither double-wrap references nor silently rewrite explicit signatures.
  Missing/conflicting metadata produces a clear diagnostic or a documented explicit mapping.
- out-class-reference and nullable behavior are explicitly unsupported until their
  contracts are implemented; no target-write approximation presented as equivalent.

If that experiment shows a declaration convention is enough, standardize the minimal
shared projection information next. If consumers require guarantees such as forbidding
all value copies of a class, evaluate an enforced runtime category separately. Only then
recommend changing defaults. A second scenario (e.g. document editing with undo) and
human trials remain necessary before drawing general usability conclusions.

## Reproduction and validation

Run from the repository root unless noted:

```sh
cargo test --locked --test reference_experience --test neo_reference_assignment --test neo_outputs
cargo run --locked -- emit-il examples/source/order-workflow.neo
```

From `docs/experiments/reference-experience/dotnet`, run `dotnet run --project Comparison.csproj`
so its pinned SDK applies. C# prints the six workflow lines, followed by list results
20, 2, 2, 50, 50. The outcome representation differs, as noted above.

Validation on macOS ARM64, Rust 1.95.0, 2026-09-12:

- All 22 selected tests passed: 8 workflow, 10 reference-assignment and 4 output tests.
  These include rejection paths and source/artifact loading, not only successful output.
- emit-il completed successfully and verified the workflow without executing guest code.
  Inspection confirmed explicit reference signatures and Product construction followed
  by heap.new; the workflow test separately executed and checked its six output lines.
- The C# baseline ran successfully with SDK 10.0.100 and printed the expected six workflow
  lines and five list-probe values. Its assertions passed.

This is current-model evidence only; B/C remain illustrative. No alternative compiler,
full regression run, new release certification or performance measurement is claimed.
