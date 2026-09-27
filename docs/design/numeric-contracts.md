# Number and concrete parsing

**Implemented in development, 2026-09-27.** The author requested
Number and Parsable interfaces, then confirmed the proposed scope: the eight
fixed-width integer types, Single and Double implement the numeric contract;
Boolean receives a concrete parser only. The author then put Parsable on hold,
selecting Number plus concrete Parse methods. A narrower ParsableNumber<T> is a
future design candidate, not part of this implementation; it should remain separate
from arithmetic conformance. The author subsequently selected NumberParseError for all numeric parsers.

The Number<T> scope is basic arithmetic, ordering, Zero and One. It is not
a complete .NET generic-math hierarchy: native-sized integers, Decimal, transcendental
functions, numeric conversion modes and culture-sensitive parsing are not selected.
Char represents a grapheme and is not a numeric implementer.

## Prerequisites and comparison

[.NET generic math](https://learn.microsoft.com/en-us/dotnet/standard/generics/math)
uses static abstract interface members to make numeric algorithms type-generic.
[IParsable](https://learn.microsoft.com/en-us/dotnet/api/system.iparsable-1?view=net-10.0)
provides parsing with provider and failure conventions. Sources retrieved 2026-09-27.
neoCLR retains familiar constrained generic calls while using Result<T,E> rather
than exceptions/out parameters and retaining explicit parsing grammars.

Initial probes exposed authored-static, inherited-constraint and target-metadata
emission gaps in Raven, plus neoCLR's generic-call admission gap. General Raven
fixes were validated independently on a main-based branch before integration.
The runtime validates exact nominal static conformance; Number is not a marker
interface accepted by name alone. The [focused consumer and metadata checks](../experiments/numeric-contracts/README.md)
cover the implemented development surface and its explicit admission limits.

## Selected contracts and tradeoffs

Number<T> inherits ComparableTo<T> and adds static Zero/One plus binary +, -, *, /.
It deliberately omits parsing, conversion modes, transcendental functions and the
large .NET generic-math interface hierarchy. The benefit is a small first arithmetic
consumer; the cost is that algorithms cannot yet express finer capabilities such
as addition-only or floating-only constraints. Ordinary integer arithmetic wraps;
integer division retains terminal Fault behavior. This does not replace the existing
Result-returning Int32.Divide. Floating arithmetic and existing CompareTo semantics
are retained, including NaN ordering distinct from IEEE equality operators.

Concrete parsing uses whole-text, culture-independent ASCII grammar. All numeric
types admit an optional sign; unsigned negative nonzero values overflow, while -0
is zero. Single/Double admit decimal mantissas and exponents plus exact NaN and
Infinity spellings. Numeric floating overflow is an error; underflow can round to
signed zero. Boolean accepts ASCII case-insensitive true/false. No parser silently
trims, decodes bytes or accepts grouping. Whole grammar validation precedes range.

New numeric parsers return NumberParseError (InvalidFormat/Overflow); Boolean has
BooleanParseError (InvalidFormat only). The author requested one numeric error contract, so Int32/Int64 parsing also uses
NumberParseError; their former development error types are removed from the Raven
reference surface. Archived Neo parsing retains its legacy carrier boundary.
A common error type is a design choice, not evidence that general parsing cannot be
abstracted: Rust's [FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html)
uses an associated error type. Raven's available contract mechanisms and actual
consumers should determine whether such generality is useful here.

### Proposed numeric parsing capability

The author clarified the purpose of ParsableNumber<T>: let a generic method
require that its numeric type parameter supplies a static Parse method. The
proposed contract associates T with Number<T> and exposes Parse(text) returning
Result<T, NumberParseError>. A constrained method could then call T.Parse(text)
and handle known numeric errors without repeating an error-type parameter.
This is a proposal; the interface is still on hold and is not implemented.

This capability is useful even when the generic method only parses: it establishes
the available operation and its result contract. Number<T> alone intentionally
does not promise parsing. Keeping the capability separate also lets arithmetic
algorithms accept numeric types without requiring a text representation.

Compared with .NET IParsable<T>, the proposed interface fixes the domain and typed
error contract rather than adopting exception/out-parameter failure conventions.
Compared with Parsable<T, TError>, it avoids carrying a redundant error parameter
through numeric consumers. The cost is a domain-specific interface that cannot
describe arbitrary parsers or numeric parsers with different errors. General typed
parsing remains possible in principle; the question is how to associate its error
type ergonomically, not whether exceptions are required.

Before implementation, validate constrained T.Parse calls and the relationship
to Number<T> through Raven and the importer. The current specialization path admits
only Number<T> constraints, so documenting this proposal does not establish target
support for ParsableNumber<T>. A focused generic parsing consumer should prove
successful parsing, shared error handling and rejection of missing implementations.

Unlike [default .NET Double.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.double.parse?view=net-10.0),
this grammar has no ambient culture or grouping, uses typed errors rather than
exceptions, and rejects numeric overflow instead of producing infinity. The benefit
is predictable configuration/protocol parsing; the cost is less convenient human
input and no .NET parsing compatibility promise. UTF-8 storage does not dictate
numeric grammar or require a byte-oriented Parse signature. These primary sources
were reviewed 2026-09-27.

## Interface implementation boundaries

General Raven fixes are isolated and independently validated on the main-based
`codex/static-interface-contracts` branch before integration into Raven main and
its neoCLR branch. They cover authored static contracts, inherited constrained
members, constrained methods/property reads/operators and metadata-only constraint
classification. No Runtime Contract option is added.

The current target integration specializes closed static application functions
whose type parameters have exactly Number<T> as their constraint, for the ten selected
primitive types. It retains the ordinary importer checks after closing the body;
the runtime validates exact nominal static member conformance. This is bounded
compile-time specialization (maximum 128 copies), not general runtime generic
static dispatch. It trades duplicated bodies for a small verifiable first consumer.
Custom numeric types, additional constraints, generic classes and generic delegates
remain unsupported by that path. No performance improvement is claimed.

The author explicitly wants static/default/access-controlled interface capabilities
in general. The [runtime tracker](../tracking/runtime-language.md#interfaces-as-a-platform-capability)
owns that direction; these restrictions are interim admission limits, not the
intended public interface model. Existing native default bodies and explicit mappings
supply useful foundations but do not prove all Raven default/private helper behavior.


## Validation

- Raven: 15 focused compiler tests for authored/imported static contracts,
  inherited members, metadata-only constraint emission and invalid implementations.
- Native: three parsing test groups, two static-contract tests, 12 existing
  default-interface regressions and four archived parsing checks.
- Target: one nested generic arithmetic/ordering consumer over ten numeric types;
  concrete success/error parsing including Int32/Int64's shared error, JSON's
  dependent Int32 conversion, and rejection of Boolean/additional constraints.
- Exact core member checks: 210 assertions; source/bootstrap hashes and matching
  RavenDoc reference snapshots checked. The earlier casing/Int64 consumer is updated
  for NumberParseError and revalidated. No full suite or website build.

The historical runtime-api inventory audit still references the already-missing
`runtime/raven/generated/File.methods.neoil`; its old audit cannot be refreshed
without separate inventory maintenance. This does not substitute for the current
RavenDoc type/member selection and snapshot checks used here.
