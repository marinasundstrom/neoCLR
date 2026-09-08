# Constructing Result in Neo

Neo can call ordinary factories on the closed System.Result<T,E> carrier:

```swift
func Validate(quantity: int) -> Result<int, string> {
    if quantity <= 0 { return Result<int,string>.Error("Invalid quantity") }
    return Result<int,string>.Ok(quantity)
}

let result = Result<int,string>.Ok(42)
```

Both owner type arguments are explicit. The factory checks the payload against T
or E and returns Result<T,E>; an unannotated binding can infer that return type.
`Result.Ok(42)` and imported `Ok(42)` are not implemented by this slice.

The public static methods System.Result<T,E>.Ok(T) and Error(E) construct the existing
case wrapper and then the existing carrier. Neo emits an ordinary static call.
Direct IL and other frontends can call these methods without special Result opcodes.
Constructors, matching and metadata remain unchanged. Neo now uses a unique bundled
static signature to provide argument context, as it already does for instance calls;
this preserves reference payloads instead of automatically loading their targets.
Overloaded signatures retain existing exact-type resolution. This applies to library
static methods generally, with no Result-specific lowering or constructor inference.

A T& payload copies the managed reference, preserving identity and access mode.
The factories do not promote frame values or relax aggregate storage checks.
Frame-backed payloads still fault when stored. A Void success can be written as
`Result<void,string>.Ok(default(void))`. No inactive payload is fabricated.

The [order workflow](../examples/source/order-workflow.neo) now returns Result instead
of the Accepted/Receipt/Message workaround. Its output and managed storage choices
are unchanged. Run:

```sh
cargo run --locked -- run examples/source/order-workflow.neo
cargo test --locked --test neo_result_factories --test neo_match --test reference_experience
```

## Planned case projection: Raven's model

Raven imports simple case types using `import System.Result.*`. `Ok(42)` constructs
Ok<int>, inferring its one parameter from the payload. An expected Result<int,string>
can then accept that case through its constructor. The error type is supplied only
when forming the carrier, not inferred while forming the case. This supports an
intermediate binding such as `let success = Ok(42)` without selecting E yet.

The selected language direction treats variants as separate nominal types:

```swift
union PaymentMethod(Card | Cash)

union Shape {
    case Circle(radius: double)
    case Square(side: double)
}
```

These non-generic Neo declarations are now implemented; see [union declarations](neo-unions.md). The first form lists
existing types; the second declares case types with the carrier. A case value can
exist independently of any carrier. Case membership does not create inheritance or
make every case contain a union tag. Qualified case names/imports should follow the
existing companion-type pattern (as System.Result.Ok<T> does); inline Neo cases are public nested records and existing case types can be reused
across carriers. [Source case imports](neo-case-imports.md) are implemented; generic cases and external imports remain future work.

The union carrier's constructor signatures are authoritative: each variant has a
constructor receiving one value of that variant type. A declaration generates these
constructors; consumers discover accepted variants from constructor metadata under
the union convention. There is no separate case-membership table. A matching carrier
constructor authorizes the case conversion. Single-argument constructors on ordinary
non-union types are not implicitly eligible. This follows the platform's [ordinary carrier/variant direction](unions-and-enums.md)
and its recorded .NET comparison. Managed reference modes remain independent; such
conversion must not silently copy a referenced target or extend a frame's lifetime.

Remaining compiler work builds on these declarations:

1. Source case imports now participate in type/constructor lookup with ambiguity and
   shadowing rules. Extend discovery to external cases such as System.Result.Ok<T>;
   do not confuse type imports with static-member imports.
2. Infer generic constructor arguments from payloads, including diagnostics when
   arguments do not determine them.
3. Discover each accepted variant from the marked union carrier's one-parameter
   constructors and convert through the matching constructor. Diagnose missing or
   ambiguous matches; do not make ordinary non-union constructors implicit conversions.
   Preserve reference access modes and lifetime checks.
4. Test standalone case bindings, nested carriers, overload resolution and rejection
   of invalid or ambiguous conversions using more than Result alone.

A Result-specific contextual shortcut would construct a carrier directly from an
expected Result<T,E>, rather than model the independently typed case. It is not part
of this contract. The planned work is not a new runtime union kind or a promise that
all constructor calls will become implicit conversions.

## .NET comparison and tradeoffs (2026-09-08)

The related .NET API is [F# Result](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/results)
(consulted 2026-09-08): FSharp.Core exposes Ok/Error alternatives through F# case
construction. This is language/library behavior, not a special CLI Result primitive.
Raven's independently typed cases and constructor-based acceptance are a separate
projection; they are not claimed to reproduce F# inference or FSharp.Core binaries.

Closed-owner factories fit the present Neo static-call support and leave the runtime
unchanged. They cost repeated type arguments in return expressions but immediately
remove the placeholder outcome-record workaround. General case inference and carrier
conversion can later remove that verbosity without coupling lowering to the Result
name. Existing union representation costs and behavior are unchanged; no speedup is
claimed. This additive API requires rebuilding System artifacts to use the factories.

Tests cover explicit owners, nested cases, Void, generic factory calls, reference
identity, JSON loading/verification, bad payloads/arity, evaluation exactly once and
runtime rejection of frame-backed payloads. This does not certify general union
construction, constructor inference or implicit case-to-carrier conversion.
