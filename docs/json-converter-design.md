# JSON converter and API follow-up assessment

Status: proposal with an [unfinished prototype](experiments/json-converters/README.md),
2026-10-11. No converter or options API is enabled in the runtime library.
The later author-directed .NET 11 union target supersedes the earlier union deferral
and .NET 10-only research scope below; the original assessment is retained as history.
The author asked to improve JSON and added: “Also consider any feature needed like
converters”. This reopens review after the basic JSON qualification, without
selecting every feature below for implementation. The author then clarified that the
developer experience should match .NET's JSON API without being exactly the same.
Treat familiar workflows, naming and predictable behavior as the goal, rather than
literal signature or implementation parity. Explain the benefit and cost of each
intentional difference. The current bounded code change
is [JsonArray collection integration](json-dom-design.md#read-only-array-sequence-view-development-2026-10-11).

## What callers still need

ObjectMapper currently supports string, Int32, Boolean, explicit DOM nodes, selected
collections and bounded reference models. A domain type represented by one JSON
string cannot customize that representation. Adding individual special cases to
ObjectMapper would couple application wire formats to the library. Converters are
therefore a useful extension point, not a substitute for ordinary primitive support.

| Priority within JSON | Proposed work | Concrete acceptance criterion |
| --- | --- | --- |
| First | Serializer options and explicitly registered typed converters | A domain identifier round-trips at the root, in a model property, vector, list and map value using one converter. |
| Alongside converter design | JSON error location and callback/validation semantics | Failures identify the property/index, and counters prove converters are not run twice by the mapper's validation/construction passes. |
| Next bounded coverage | Missing numeric types and nullable values | Int64 boundaries, finite floating-point values, explicit null versus absent fields, and overflow match documented .NET behavior; unsupported values fail explicitly. |
| Separate options work | Property naming, explicit rename/ignore, unknown-member policy | Defaults preserve existing matching; naming collisions and override precedence have tests. |
| Later wire contracts | Enums, inheritance, Optional/Result and Raven unions | A cross-target golden corpus pins each wire format and rejects discriminator collisions and unknown cases as specified. |
| Later resource work | Configurable limits and incremental UTF-8 reader/writer | Oversized or partial input stays bounded, reports location, and preserves borrowed-stream ownership. |

These are implementation gaps and proposals, not claims of improved semantics over
.NET. Do not expose a bag of unused options or root-only converter overloads: callers
reasonably expect the same converter to work recursively.

## Primary-source comparison

Sources inspected 2026-10-11; .NET 10 is the compatibility baseline. References to
.NET 11 additions on evolving documentation pages are outside this baseline.

- [System.Text.Json converter guidance](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/converters-how-to)
  describes typed converters, factories for open generic families, options-based
  registration, and reader/writer callbacks. Property attributes take precedence
  over the options list, then type attributes; the first matching options converter
  wins. Its null handling and default-converter delegation are explicit contracts.
  Adopt familiar typed registration and precedence, while treating factories and
  attributes as later coverage, not silently different defaults.
- [Newtonsoft.Json typed converter example](https://www.newtonsoft.com/json/help/html/CustomJsonConverterGeneric.htm)
  demonstrates a Version value represented as a string through JsonConverter<T>.
  This is a concrete library-level solution to domain wire shapes; no runtime JSON
  service is needed. Its serializer callback can support nested work, but its API
  and precedence should not be mixed accidentally with System.Text.Json's contract.
- [Serde custom serialization](https://serde.rs/impl-serialize.html) separates a
  type's serialization from the serializer format using a trait. That supports
  static composition and explicit errors. Requiring model types to implement a new
  protocol would limit customization of third-party types here; externally
  registered converters fit the .NET-oriented public API better.
- [dotnet/runtime #50205](https://github.com/dotnet/runtime/issues/50205) is an open
  issue, labeled Future when inspected. Its author reports difficulty wrapping a
  built-in collection converter while retaining custom element converters because
  converter state depends on the options used to resolve it. This is historical
  user evidence, not a reproduced defect in current .NET or a settled API decision.
  It motivates an explicit nested-delegation test, not a claim that neoCLR is faster.

## Recommended first contract and alternatives

Keep policy in System.Data.Json. Use familiar JsonSerializerOptions and
JsonConverter<T> concepts; exact signatures remain provisional. Explicit closed-type
registration is the first slice. Snapshot or freeze configuration per operation so
callback mutation cannot change resolution halfway through a document. Custom
converters precede built-ins for their registered types; no process-global registry.
Resolve root serialization from the actual type and nested values from their
retained declared type, with the distinction documented and tested. A later generic
Serialize<T> overload needs its own declared-root policy rather than silently
changing the existing Object overload.

The existing implementation already parses/writes a bounded DOM. A provisional
node-based adapter (`JsonValue` in/out and Result errors) could reuse that machinery
and expose explicit JsonNull. It costs allocation and departs from .NET's streaming
reader/writer signature; do not call it streaming or freeze the long-term API around
that shortcut. Alternatives are to implement public UTF-8 reader/writer contracts
first, or to prototype converters privately before deciding which surface to ship.
Prefer a private node-based experiment to answer nested dispatch and callback effects;
evaluate it as a possible public surface on its own merits. A DOM callback may be
appropriate for this bounded API if it retains familiar registration and recursive
use. A public reader/writer layer is not mandatory merely to reproduce .NET signatures;
its justification is streaming, allocation control and larger-document use cases.

A conversion context must carry options, location and a shared depth/occurrence
budget into nested calls. Re-entering the same conversion at the same location must
return an explicit error instead of recursively selecting itself forever. Delegating
to built-in mapping must bypass only the current converter selection, retaining
custom converters for child values. Arbitrary separate calls to JsonSerializer made
by user code cannot be made effect-free or bounded by the outer call automatically.

Null policy needs a deliberate bridge between explicit JsonNull and nullable target
values: preserve DOM null nodes, apply target nullability checks, and invoke a
converter for null only under an explicit opt-in policy. Do not infer permission to
construct arbitrary runtime types from converter registration. Factory support,
polymorphic discriminators and attribute-driven activation remain separate slices.

## Validation before construction and user effects

ObjectMapper.Read currently runs ReadValue with construct=0, then construct=1.
A naive converter call in both paths doubles callback effects. Calling only in the
construction pass could discover an invalid converted value after earlier model
constructors/setters have already run. Neither behavior may be introduced silently.

Proposed experiment: build an internal read plan. Validate built-in structure and
all ordinary member contracts first, invoke each needed custom conversion once and
retain its result, then materialize ordinary model objects from the plan. Cache by
value occurrence and target contract, not solely by shared DOM node identity.
Converter callbacks themselves may allocate, mutate state or fail; only ordinary
mapper construction is delayed. Document that distinction explicitly. A returned
error aborts materialization; no rollback of arbitrary converter effects is promised.
This costs retained values and a plan allocation, bounded by the document limits.
Compare this with an explicitly weaker single-pass opt-in mode before selecting the
public contract; preserve the existing no-converter guarantee.

Required evidence: root/property/vector/list/map conversion; converter ordering;
null opt-in; malformed siblings; exactly-once callback counts; returned conversion
errors and locations; nested built-in delegation; recursive re-entry; oversized or
cyclic converter-produced nodes; wrong runtime return types; and validation before
any stream output. Run a focused .NET 10 comparison for behavior intended to match,
then neoCLR interpreter and native AOT consumers. Windows qualification is required
before advertising platform parity. No such converter execution evidence exists yet.
