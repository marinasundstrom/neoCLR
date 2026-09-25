# Idiomatic Raven in neoCLR

These conventions apply to all hand-authored Raven code in the project: runtime
library implementations, application code, tooling, tests, experiments, samples,
and code shown in documentation or on the website. Apply them when writing or
editing code, not only when preparing examples. The specific bootstrap and test
exceptions below remain applicable. They follow Raven's own source style and feature intent;
neoCLR's target API and bootstrap have narrower capabilities than ordinary Raven
on .NET. Compile examples against the target rather than assuming similar syntax
or host APIs work here.

## Sources and scope

Reviewed in the Raven `neoclr` checkout on 2026-09-19, revision `9b2a558ae`:

- [Raven style guide](https://github.com/marinasundstrom/raven/blob/9b2a558ae/docs/lang/style-guide.md):
  indentation, bindings, spacing, file organization and one statement per line.
- [Feature meaning](https://github.com/marinasundstrom/raven/blob/9b2a558ae/docs/lang/feature-meaning.md):
  model the domain before selecting syntax; use patterns to establish facts.
- [Absence and failure](https://github.com/marinasundstrom/raven/blob/9b2a558ae/docs/lang/features/option-and-result.md)
  and [patterns](https://github.com/marinasundstrom/raven/blob/9b2a558ae/docs/lang/features/patterns.md):
  Option, Result, propagation and exhaustive interpretation.
- [Properties specification](https://github.com/marinasundstrom/raven/blob/9b2a558ae/docs/lang/spec/properties-and-events.md):
  property-first declarations and expression-bodied computed getters.

Those links identify the reviewed source revision, which may remain local until
pushed. They document language guidance, not a claim that every Raven/.NET feature
is admitted by neoCLR's preview importer. Examples below use the neoCLR contracts;
for example its Result error case is named `Error`.

## Bindings, properties and layout

Use `let` for an immutable local binding and `var` when reassignment is needed.
Use `val` for a read-only property. `val` remains legal for locals, but matching
Raven's standard spelling makes the roles easier to distinguish. Neither `let`
nor `val` promises deep immutability of an object.

For ordinary private storage, prefer private `var` or `val` declarations. Raven
emits these storage members as fields rather than CLR properties; a private computed
property with an accessor remains a different construct. Use explicit `field` when
field declaration itself is intended, whether public or private, or for a concrete
compatibility requirement. Bootstrap validation should inspect the emitted metadata
contract, not require explicit `field` syntax in the source. This author clarification
was recorded on 2026-09-24; it does not require rewriting every existing storage
declaration at once.

Prefer an expression-bodied property for a single getter expression:

```raven
val DeclaringType: Option<TypeInfo> => Some(StoredDeclaringType)
```

Use properties for contextual state or identity, such as TaskQueue.Current and
TaskQueue.Default, including internal Raven-facing runtime-service accessors. Native
transport calls behind those getters remain implementation details.

Use a getter block when it actually needs statements, and retain explicit accessor
contracts when setter visibility or initialization matters. Interface declarations
still describe the contract rather than an implementation.

Use four spaces, one statement per line, a blank line between declarations and
logical groups, and braces on multiple lines for control flow with a body. A short
expression is useful; compressed control flow is not a goal. Infer obvious local
types, but annotate a binding when it supplies an important expected union or
interface type.

## Construct cases and extract payloads

Use case constructors rather than spelling out both the case and carrier storage:

```raven
import System.*
import System.Option.*

func FindPrice(product: int) -> Option<int> {
    if product == 7 {
        return Some(42)
    }
    return None()
}

let price: Option<int> = Some(42)
```

Imported cases also avoid qualified Result factories when the expected carrier is
known:

```raven
import System.*
import System.Result.*

let failure: Result<int, string> = Error("Unavailable")
```

Prefer this to repeating `Result<int, string>.Error("Unavailable")` on the right.
The same rule applies to Option and other imported union cases throughout runtime
code, samples, tests and website content. Keep qualification when required to
resolve a name or provide type information that cannot otherwise be inferred.

The development library removes the legacy `System.Error` message wrapper that
collided with this case name. Rebuild the reference library and callers together;
older SDK/reference bundles can still require qualification.

The return type or annotation supplies the carrier type. Keep that context: an
unannotated case construction can infer the case type instead of the intended
carrier. Qualify a constructor when required to disambiguate a name. The desired
`Option<T>.Some(value)` spelling is not currently exposed by this target's Option
reference contract; imported `Some(value)` works and is the tested spelling here.
Do not add explicit carrier wrappers merely to compensate for a missing expected
type. Payload-free construction currently uses `None()` in these target samples.

Prefer a case pattern or destructuring over `IsSome` followed by
`GetSomeCase().Value`:

```raven
match FindPrice(7) {
    Some(let price) => Console.WriteLine(price)
    None => Console.WriteLine("Product not found")
}
```

With `import System.Option.*` and `import System.Result.*`, prefer plain case
patterns (`Some`, `None`, `Ok`, `Error`) over member case patterns (`.Some`,
`.None`, `.Ok`, `.Error`). The imports bring the case names into scope; a leading
dot is unnecessary. Qualify a constructor only when name resolution requires it.

Use `if value is Some(let item)` when only the present branch needs work. Use
`match` when every alternative needs meaning. Nested patterns can unpack nested
carriers, such as `Some(Ok(let number))`. Use `_` for a payload that is genuinely
unused; do not hide distinct meaningful cases behind a catch-all.

Use `?` to propagate compatible absence or failure when the current function
cannot add useful handling. Return typed Result errors for expected failure and
Option for meaningful absence. A fault is not a substitute for those ordinary
outcomes; it is appropriate when an invariant required for execution has failed.

## Completion and the unit spelling

Keep Raven's `unit` keyword for now. In the neoCLR target it maps to System.Void,
with `()` as its value. Use `Result<unit, E>` for completion or an expected error.
The target does not introduce a separate System.Unit runtime type. Ordinary calls
with no result still leave no value on the execution stack; the compiler supplies
the unit value when an expression or generic payload requires it.

The author considered a lowercase `void` source keyword, then chose to keep `unit`
for now. No new keyword or compiler policy is required. System.Void remains a valid
explicit type spelling in existing samples and bootstrap declarations.

## Model contracts rather than implementation details

Use an enum for named constants, a union for a closed set of payload-bearing
alternatives, and a sealed interface hierarchy when separate type identities are
part of a closed model. Prefer exhaustive named cases so additions force a review.
For MemberInfo that means TypeInfo, FieldInfo, MethodInfo and PropertyInfo. Keep
the cases together in one source file, as Raven requires for a sealed family.

Use a class for identity, lifecycle or encapsulated state; do not create one just
to hold unrelated operations. Plain functions suit operations without an owner.
Choose collection interfaces by the required capabilities. Introspection queries
return Sequence<T>: Count, indexing and iteration are public, mutable array
storage is an implementation detail.

Keep ordinary .NET boundaries recognizable where they are part of an integration
contract. neoCLR's own API choices may deliberately differ; document their meaning
and migration instead of silently reproducing .NET shapes or treating different
spelling as an improvement.

An empty-case union such as SocketError is a basic union, not the main motivation
for the feature. Unions are particularly useful when variants carry their own
different data. Enums remain appropriate for a set of named constants; choosing
standard syntax for an existing union does not require choosing unions for all
finite sets. SocketError remains a union in the current API.

## Prefer standard union declarations

Use Raven's standard `union` syntax for class-library unions by default. It supports
payload-bearing cases, ordinary methods and computed properties; an authored
`override ToString()` can provide domain-specific diagnostics. Keep case construction
and pattern matching in source instead of hand-writing erased storage, constructors,
predicates and checked accessors for each domain error family.

Do not require an `Is*` property or `Get*` accessor for every case, including cases
in Option and Result. Prefer construction and patterns through the generated case
contract. Existing helpers are implementation history, not a template or condition
for recognizing every future union. Migrate applicable existing unions to standard
syntax; document any remaining manual exception and its blocker.

Within a union, prefer unqualified case names in `match` arms when they resolve
unambiguously. SocketError.ToString exercises this form for every declared case,
including boxed calls. Keep qualification where it clarifies an external union.

Hand-authored carriers are rare exceptions. Record the concrete bootstrap or runtime
constraint that requires one and the condition for removing it. Existing custom
carriers are migration candidates, not templates for new APIs. An importer gap should
first become a reduced compiler/bridge test; do not silently turn it into a permanent
manual implementation requirement.

Separate source authoring, the public member contract and physical storage. Using
standard syntax does not commit neoCLR to the .NET ABI or a particular Raven lowering.
Validate construction, extraction, inactive/default states, copying, boxing and GC
for the target representation. Broader .NET comparisons inform that decision without
requiring binary compatibility.

## Bootstrap exceptions and validation

The current Option/Result carrier implementation is hand-authored to satisfy the
compiler/runtime union metadata and storage contract. Its checked case accessors,
TryGet methods, constructors and residual/output adapters are current bootstrap
plumbing. Their continued need must be assessed as normal union emission becomes
supported; this is not a permanent exemption from the standard-syntax policy.
Replacing their implementation with patterns that call those same methods could
create recursion. Ordinary callers and teaching samples should use patterns;
removing the ABI is a separate compiler/runtime change, not this style cleanup.

Likewise, case-payload mutability tests intentionally manipulate case values to
verify copy semantics. Rejection fixtures may intentionally use invalid or low-level
syntax. Generated neoIL and historical proposals are not style-edit targets.

Direct compiler probes must configure the target core and unit type just as
`NeoCLR.Raven.props` does; otherwise `()` can denote the host unit type instead of
neoCLR System.Void. Do not reintroduce carrier wrappers to mask that mismatch.

Run edited samples through the saved-project runner, regenerate changed Raven
runtime slices, and check source admission when declarations change. The
[Option sample](experiments/raven-target/samples/library-option.rvn),
[nested type sample](experiments/raven-target/samples/library-nested-type-info.rvn)
and [runtime descriptors](../runtime/raven/src/System/Introspection/Descriptors.rvn)
provide executable examples of these conventions.

## Query operator terminology

For the neoCLR development API after Preview 8, use `Filter` and `Map` with initial
capitals. Follow [the naming principle](api-policy.md#query-operator-naming-direction-2026-09-19)
rather than copying every .NET name or mechanically copying another language.
Preview 8 uses `Where` and `Select`; its published samples remain unchanged.
C# tooling still targets .NET and keeps its actual .NET method names.

## Extension declarations

Use Raven's extension syntax for APIs intended to be called as extensions:

```raven
public extension OptionOperators<T> for Option<T> {
    func Map<U>(mapper: Func<T, U>) -> Option<U> {
        return self match {
            Some(let value) => Some(mapper(value))
            None => None
        }
    }
}
```

Import System.Option.* for these case patterns. The receiver is self; the compiler
emits the extension marker and ordinary static call contract. Do not rely on a
bootstrap declaration alone to turn an ordinary runtime static class into an
extension API. See [supported extension boundaries](raven-extension-methods.md).

## Lambda signatures

Prefer inferred callback types when the receiving method supplies enough context:

```raven
let fallback = absent.OrElse(() => Some(7))
```

Write idiomatic Raven throughout the project, including the runtime library.
Do not add type annotations merely to explain the language or spell out what is already
clear from the operation and surrounding code. Prefer inference for locals and
callbacks; retain annotations only when the compiler requires them or a contract
would otherwise be unclear. Examples must compile as shown. Record required
compiler workarounds in the relevant technical notes without turning the example
into a language tutorial. For example, the development outcome sample currently needs
`Then((value: int) -> Result<int, string> => Ok(value + 1))`; the shorter
union-returning callback fails inference in Raven SDK .15. Both OrElse callbacks
in that sample compile without annotations.


## Presenting Result and Option APIs

Author direction, 2026-09-23: ordinary samples should prefer `?` for propagating
errors through a compatible Result and pattern bindings for optional values, rather
than requiring a match at every call. Combined Result<Option<T>, E> is a useful
example: propagate E, then bind Some or handle None. Use match when explaining case
structure, applying different policies to cases, or handling the final error at an
application boundary. Do not silently discard errors merely to shorten a sample.

Author clarification, 2026-09-25: this preference applies to all examples showing
how to write user code, including propagation through Option where the enclosing
contract supports it. Explicit union handling remains appropriate when it expresses
a real decision. Keep diagnostic assertions in fixtures separate from the ordinary
application example; see the [buffered POST example](experiments/http-post/Sample.rvn).

For code after a successful binding, the current toolchain uses
`let Some(input) = expression else { return ... }`. The else branch must exit.
For a success block, use `if let Some(input) = expression { ... } else { ... }`.
Compile the sample before publishing syntax or interpolation examples. See the
[Console propagation examples](experiments/console-streams/README.md).

With the current Raven precedence fix, prefer `await Foo()?` and `try Foo()?`.
Propagation applies to the complete await or try expression; `(await Foo())?` and
`(try Foo())?` do not need the outer parentheses. Parenthesize the operand only
when inner propagation is intended. Use a matching compiler when verifying samples.

## Propagating errors through conversions

Raven's `?` can implicitly convert the error to the enclosing Result's error type,
including a conversion defined in an extension. Keep domain adaptation owned by the
application instead of forcing each API to return a shared catch-all error. For example:

```raven
union AppError {
    case Http(reason: HttpError)
}

extension HttpErrorConversion for HttpError {
    static func implicit(error: HttpError) -> AppError {
        return AppError.Http(error)
    }
}
```

A function returning `Result<int, AppError>` can propagate `Result<int, HttpError>`
with `?`; Raven reports RAV1506 for that conversion. The
[HTTP status fixture](experiments/http-status/Main.rvn) also checks propagation to
Object through boxing. Matching the nested cause retains structured information;
converting to Object loses that static structure. Use conversions deliberately at
application boundaries, not as a replacement for distinct error contracts.

Author direction, 2026-09-25: keep application-specific converters near the consuming
method or in its nearby declarations. Map each lower-level error into a meaningful
common application error so `?` can preserve readable method bodies. This is an
organizational preference, not a claim that conversions can be declared inside methods.

Future exploration: System.Error could own a common wrapping conversion that preserves
the original error as its cause and supports added context. The author points to
Raven.Core's Error interface as a reference. Interface conversion lookup, wrapper shape
and cause/context contracts still need investigation; neoCLR does not implement them
in this slice. Generic wrapping would not replace domain-specific application mapping.

## Pattern-friendly public contracts

Consider deconstruction when designing ordinary classes as well as records. A useful
Deconstruct contract can expose a stable semantic view for matching/extraction without
promising value equality, copying or immutability. Choose components and ordering from
real caller examples; do not mechanically expose all stored members. Named properties
remain useful alongside positional patterns. This author direction (2026-09-25) asks
for deliberate Raven-oriented API design, not wholesale adoption of .NET API shapes
or automatic Deconstruct members on every type.

The author subsequently clarifies that patterns are optional developer-experience
choices, not a preferred style. Property patterns inspect readable properties and
must be distinguished from positional patterns requiring Deconstruct. Compile concrete
examples; Raven rejects combining outer `if let` with an inline `let` binding inside
that same pattern. Prefer direct properties, patterns or ordinary branching according
to the sample's purpose, without imposing a universal style.

For captures, distinguish an outer binding context from an `is` pattern. The author
clarifies these forms: `let Ok(HttpResponse { Headers: headers }) = result` inherits
capture binding from the outer `let` (handle refutable bindings as required), whereas
`if result is Ok(HttpResponse { Headers: let headers }) { ... }` uses an explicit
inline capture. The status fixture covers both forms alongside `if let`.
