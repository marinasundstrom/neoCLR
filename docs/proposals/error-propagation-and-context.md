# Raven Error Propagation and Context

## Summary

Raven should provide a unified model for recoverable error propagation built around `Result`, the propagation protocol, the `?` operator, and the `Error` interface.

The propagation mechanism itself should remain general. A propagatable value exposes either an output or a residual, and `?` propagates that residual into the enclosing computation. This mechanism may apply to errors, but also to other concepts such as `None` in `Option`.

`Error` provides additional behavior specifically for recoverable failures.

A concrete error does not need to be statically typed as `Error` to gain this behavior. Domain-specific errors and unions may retain their precise types while implementing the `Error` interface. This allows Raven to preserve the actual error value throughout propagation while using virtual/interface dispatch to provide common facilities such as diagnostics, contextualization, chaining, and error inspection.

The model should be shared between Raven targeting .NET and Raven targeting neoCLR, while allowing each target to integrate it according to the capabilities and conventions of the underlying platform.

On .NET, Raven must interoperate with an ecosystem built around exceptions. Exceptions may therefore be captured and reused as residual/error carriers inside `Result`, and selected .NET APIs may be retargeted to adapted implementations returning `Result`.

On neoCLR, recoverable failures can use the Raven model natively. neoCLR has no exception mechanism, allowing built-in failures to directly implement `Error` and allowing the runtime to provide deeper support for diagnostics and propagation.

The central design principles are:

> **Propagation is structural. `Error` is behavioral. Concrete errors should be preserved whenever possible.**

---

## Propagation

The `?` operator operates on values participating in Raven's propagation protocol.

The current .NET-facing abstraction is `IPropagatable`; neoCLR provides the corresponding `Propagatable` abstraction. The exact Raven-facing protocol may be revised independently of this proposal.

Conceptually:

```raven id="ak9c0m"
let value = operation()?
```

separates a propagatable value into either:

```text id="yifv1i"
operation()
    │
    ▼
┌──────────┬──────────┐
│  Output  │ Residual │
└──────────┴──────────┘
     │           │
     ▼           ▼
 continue     propagate
```

The propagation protocol describes control flow. It does not inherently describe errors.

This distinction allows other types to participate.

For example, an `Option<T>` may eventually propagate `None`:

```text id="18wrro"
Option<T>
   │
   ├─ Some(T) → output T
   │
   └─ None    → residual None
```

`None` does not need to be considered an `Error`.

---

## Residual propagation

Consider:

```raven id="e8mhw8"
func operation(): Result<T, E1>

func caller(): Result<U, E2> {
    let value = operation()?
}
```

The compiler conceptually performs three operations:

1. obtain the output or residual;
2. determine how the source residual participates in the enclosing computation;
3. construct the residual case of the enclosing propagatable type.

Conceptually:

```text id="wz51r5"
Result<T, E1>
     │
     │ ?
     ▼
    E1
     │
     │ residual compatibility/adaptation
     ▼
    E2
     │
     │ reconstruct residual case
     ▼
Result<U, E2>
```

These operations should remain conceptually separate even when `Result` makes them appear trivial.

---

## Error

`Error` is Raven's common behavioral abstraction for recoverable failures.

For example:

```raven id="aj49nu"
union FileError : Error {
    NotFound(Path)
    AccessDenied(Path)
}

union JsonError : Error {
    InvalidSyntax(String)
    UnexpectedToken(String)
}
```

An API can retain its precise domain error:

```raven id="n9ozfb"
func read(Path path): Result<String, FileError>
```

The caller therefore receives:

```raven id="3ug4pd"
Result<String, FileError>
```

rather than:

```raven id="f3u6y0"
Result<String, Error>
```

merely for the sake of accessing common error functionality.

The important relationship is:

```text id="f4y5fk"
Static residual type
      FileError
          │
          │ implements
          ▼
        Error
          │
          ├─ diagnostics
          ├─ contextualization
          ├─ chaining
          └─ propagation behavior
```

`Error` is therefore primarily a **behavioral abstraction**, not necessarily the static error type.

---

## Preserve the concrete error

Propagation should preserve the actual residual whenever no adaptation is necessary.

Given:

```raven id="74ohuq"
func read(): Result<String, FileError>

func load(): Result<Config, FileError> {
    let text = read()?
    ...
}
```

the error path should effectively be:

```text id="7hj1gv"
FileError.NotFound(path)
          │
          │ ?
          ▼
FileError.NotFound(path)
```

The error should not unnecessarily become:

```text id="sk6esb"
FileError
    ↓
generic Error wrapper
    ↓
another wrapper
```

Implementing `Error` does not imply that a value must be boxed, converted, or replaced merely because it is propagated.

The concrete error should survive unless a real conversion or contextual wrapper is required.

---

## Virtual error behavior

Although Raven should not require a traditional inheritance hierarchy for errors, common error functionality benefits from virtual/interface dispatch.

Raven should not require structures such as:

```text id="4ubnhm"
Error
  └─ IOError
      └─ FileError
          └─ FileNotFoundError
```

Domain errors are often better represented as unions:

```raven id="2tr1cz"
union FileError : Error {
    NotFound(Path)
    AccessDenied(Path)
    InvalidPath(Path)
}
```

The interface instead establishes a behavioral hierarchy:

```text id="fjytwa"
                 Error
                   ▲
                   │
       ┌───────────┼────────────┐
       │           │            │
   FileError   NetworkError  ContextError
    (union)      (union)       (class)
```

Each concrete representation remains independent while participating in common error operations through interface dispatch.

This is what allows Raven to add richer facilities to errors without forcing every error into a class hierarchy.

---

## Error behavior throughout propagation

A value implementing `Error` should retain its error capabilities throughout the propagation chain even when its static type remains concrete.

For example:

```raven id="z9nkgw"
Result<T, FileError>
```

can expose `FileError` precisely while Raven's propagation infrastructure can still recognize:

```raven id="f5h29g"
FileError : Error
```

and invoke appropriate error behavior.

This enables common functionality without requiring APIs to erase their domain-specific error types.

The principle is:

> **Typing an error precisely should not opt it out of Raven's common error infrastructure.**

---

## Error context

Higher layers frequently need to add information to an existing error without replacing it.

Raven should therefore provide a versatile contextual error implementation.

Conceptually:

```raven id="r04yb7"
class ContextError : Error {
    Error Cause
    String? Message
    ErrorDiagnostics Diagnostics
}
```

The exact representation remains an implementation detail.

A concrete error:

```text id="z85br3"
FileError.NotFound("settings.json")
```

may be decorated:

```text id="6hqkl4"
ContextError
├─ "Unable to load configuration"
└─ FileError.NotFound("settings.json")
```

and subsequently decorated again:

```text id="1vthmw"
ContextError
├─ "Application initialization failed"
└─ ContextError
   ├─ "Unable to load configuration"
   └─ FileError.NotFound("settings.json")
```

Every layer remains an `Error`.

The chain may therefore contain different concrete error representations while sharing common behavior.

---

## Contextualization versus conversion

Contextualization should be distinguished from conversion.

A conversion:

```text id="x0f6rz"
FileError → ApplicationError
```

means that one representation becomes another.

Contextualization means:

```text id="l4kax8"
ApplicationError
└─ FileError
```

The original failure remains present.

This distinction matters because higher layers usually want to **add information**, not destroy lower-level information.

Conversion remains appropriate when two error representations genuinely describe the same information differently.

Wrapping is appropriate when a layer is adding context.

---

## Adding context

Context should normally be added explicitly before propagation.

For example:

```raven id="ljppob"
let config = readConfig()
    .context("Loading application configuration")?
```

Conceptually:

```text id="gx93ma"
Result<T, E>
    │
    │ context(...)
    ▼
Result<T, ContextError>
    │
    │ ?
    ▼
T or ContextError
```

This keeps the semantics of `?` simple.

Ordinary propagation preserves the existing residual.

Contextualization explicitly creates additional semantic information.

---

## Error chains

Error chains should represent structured data rather than merely formatted messages.

For example:

```text id="z58u4u"
Application initialization failed
└─ Loading application configuration
   └─ Reading configuration file
      └─ FileError.NotFound("settings.json")
```

Each layer may contain:

- its own concrete type;
- contextual information;
- diagnostics;
- a cause;
- target-specific information.

Applications should be able to inspect, match, format, and log this structure programmatically.

---

## Diagnostics

`Error` should participate in Raven's standard diagnostic infrastructure.

Diagnostic information may include:

- where an error originated;
- where it was propagated;
- contextual layers added to it;
- stack information;
- target-specific diagnostic information.

A simple declaration:

```raven id="xfw12i"
union NetworkError : Error {
    Unreachable
    ConnectionReset
    Timeout
}
```

should require little or no additional boilerplate to participate.

The exact mechanism by which diagnostic information is associated with an error may differ between targets.

---

## Origin and propagation

Raven should distinguish between an error's **origin**, its **logical propagation path**, and its **context chain**.

For example:

```text id="2wzfjk"
Application initialization failed        ← context
│
├─ Loading configuration                 ← context
│
├─ propagated through Application.Start  ← propagation
├─ propagated through Config.Load        ← propagation
│
└─ FileError.NotFound                    ← concrete error
   └─ FileSystem.Open                    ← origin
```

These are related but different concepts.

A complete runtime stack trace need not be captured at every `?`.

A more efficient implementation may capture origin information once and add lightweight propagation information as the residual moves through the program.

This is especially useful for asynchronous code, where a physical runtime stack may not accurately represent the logical propagation path.

---

## Residual compatibility and conversion

Propagation should preserve a residual directly whenever possible.

Given:

```raven id="v0djsb"
func read(): Result<String, FileError>

func load(): Result<Config, Error> {
    let text = read()?
}
```

and:

```raven id="qduh7b"
FileError : Error
```

the propagation path requires only ordinary interface compatibility:

```text id="mh65sh"
FileError
    │
    │ interface compatibility
    ▼
  Error
```

No contextual wrapper is required.

More generally, residual resolution may consider:

1. identity;
2. ordinary type compatibility;
3. an applicable Raven conversion;
4. a propagation-specific adaptation;
5. otherwise, a compile-time error.

The precise ordering should align with Raven's broader conversion and overload-resolution rules.

---

## Error wrapping as propagation adaptation

Some enclosing computations may require a different concrete error type.

For example:

```raven id="16rx6z"
func load(): Result<Config, ApplicationError>
```

where:

```raven id="t5d43y"
class ApplicationError : Error {
    Error Cause
}
```

Rather than requiring:

```text id="08v7ra"
FileError    → ApplicationError
JsonError    → ApplicationError
NetworkError → ApplicationError
```

individually, `ApplicationError` may define that it can wrap an arbitrary `Error`:

```text id="8ng09v"
E where E : Error
        │
        ▼
ApplicationError(E)
```

This preserves dependency direction.

The application knows how to accept library errors. Libraries do not need to know about application-level error types.

The exact language mechanism may build upon Raven's conversion abstractions or introduce a propagation-specific mechanism.

---

# Raven targeting .NET

Raven's value-based error model remains available when targeting .NET.

Raven-authored APIs may freely use:

```raven id="0qshcc"
Result<T, FileError>
```

and Raven-defined errors may implement `Error`.

However, .NET itself is exception-oriented. Raven must therefore interoperate with exceptions rather than pretending they do not exist.

---

## The `try` expression

Raven's `try` expression bridges exception-based APIs into value-based propagation.

Conceptually:

```raven id="iy7ts4"
let result = try SomeDotNetOperation()
```

turns:

```text id="t5m95k"
.NET call
├─ returns T
└─ throws Exception
```

into:

```text id="sk0h0n"
Result<T, Exception>
├─ Ok(T)
└─ Err(Exception)
```

The existing exception becomes the residual.

Raven should normally preserve that exception rather than manufacture another error object containing equivalent information.

---

## Retargeted .NET APIs

Raven may retarget selected .NET APIs to adapted implementations returning `Result`.

Instead of exposing:

```text id="yj1uhz"
T Operation(...)
    throws E
```

Raven may expose:

```text id="txq6mm"
Operation(...): Result<T, E>
```

The adapter catches the existing CLR exception and uses it directly:

```text id="p95aoh"
.NET operation
      │
      ├─ success ───────→ Ok(value)
      │
      └─ throws E
             │
             ▼
           Err(E)
```

This produces a Raven-friendly API without creating unnecessary error objects.

The resulting error carrier may be less native to Raven's ideal error model, but it remains natural to the .NET platform and retains all CLR diagnostic information.

---

## Exceptions as foreign error carriers

On .NET, exceptions should be considered legitimate foreign residual/error carriers.

For example:

```text id="g3dm00"
IOException
    │
    ▼
Result<T, IOException>
```

may participate in Raven propagation without first becoming a Raven-specific error object.

If additional context is needed:

```text id="t53ajy"
ContextError
├─ "Reading configuration"
└─ IOException
   └─ CLR diagnostics
```

The original exception remains available.

Raven may provide target-specific bridging between CLR exceptions and the richer `Error` behavior where appropriate.

This interoperability should not dictate neoCLR's native error design.

---

# Raven targeting neoCLR

neoCLR has no exception mechanism.

Recoverable failures can therefore use Raven's error model from their point of origin.

A platform API may naturally expose:

```raven id="ftj55x"
func open(Path path): Result<File, FileError>
```

where:

```raven id="91a0ig"
union FileError : Error {
    NotFound(Path)
    AccessDenied(Path)
    InvalidPath(Path)
}
```

The complete path remains value-based:

```text id="45rqvf"
neoCLR operation
       │
       ▼
Result<T, FileError>
       │
       │ ?
       ▼
   FileError
       │
       ▼
 propagation
```

No exception capture or adaptation boundary exists.

---

## Deeper neoCLR integration

Because neoCLR is designed around this model, it may take `Error` further than the .NET target.

Potential runtime integration includes:

- automatic error-origin information;
- lightweight logical propagation tracking;
- efficient context chains;
- standard error introspection;
- runtime-assisted diagnostics;
- optimized dispatch for `Error`;
- efficient handling of value-type and union errors.

These capabilities should enrich the common Raven model rather than redefine its semantics.

---

# Common behavior, target-native representation

Raven should aim for common behavior rather than identical representation.

On .NET, a failure may originate as:

```text id="n0q1v8"
IOException
```

while on neoCLR it may originate as:

```text id="75nxgf"
FileError.NotFound
```

Both can participate in a logical chain such as:

```text id="9qynan"
Loading application configuration
└─ Reading configuration
   └─ underlying failure
```

The leaf representation is target-specific.

The propagation and contextualization concepts are Raven concepts.

This distinction allows Raven to fit naturally into .NET while allowing neoCLR to implement the model natively.

---

# Relationship to Fault

`Error` and `Fault` serve different purposes.

`Error` represents recoverable failure participating in normal program control flow:

```raven id="xfepxo"
Result<T, Error>
```

or, more commonly, a precise error type implementing `Error`:

```raven id="fn99jk"
Result<T, FileError>
```

A `Fault` represents an unrecoverable execution failure.

Error diagnostics answer questions such as:

- Where did the failure originate?
- Through which logical boundaries was it propagated?
- What context was added?
- What concrete failure remains at the bottom of the chain?

Fault diagnostics primarily describe catastrophic execution failure.

neoCLR therefore does not need exceptions merely to provide rich diagnostic information for ordinary recoverable errors.

---

# Example

A low-level API exposes a precise error:

```raven id="dkblgp"
union FileError : Error {
    NotFound(Path)
    AccessDenied(Path)
}

func readFile(Path path): Result<String, FileError> {
    ...
}
```

A configuration layer retains Raven's error behavior while adding context:

```raven id="v7bjnk"
func loadConfig(Path path): Result<Config, Error> {
    let text = readFile(path)
        .context("Reading configuration file")?

    return parseConfig(text)
        .context("Parsing configuration")?
}
```

An application layer may add another level of context:

```raven id="w59k2f"
func start(): Result<Application, Error> {
    let config = loadConfig("settings.json")
        .context("Loading application configuration")?

    ...
}
```

The resulting error may retain the complete logical structure:

```text id="53o1zr"
Loading application configuration
└─ Reading configuration file
   └─ FileError.NotFound("settings.json")
```

The concrete `FileError.NotFound` remains the original failure. Each contextual layer enriches it rather than replacing it.

If the same operation ultimately originated from a .NET API, the leaf might instead be:

```text id="7ssysr"
Loading application configuration
└─ Reading configuration file
   └─ FileNotFoundException
      └─ CLR exception diagnostics
```

The surrounding Raven error behavior can remain conceptually similar even though the leaf is a platform-native exception rather than a Raven-native `Error`.

On neoCLR, the entire chain can consist of native Raven error values and runtime-supported diagnostics.

---

# Design principles

## Propagation is structural

`?` operates through the propagation protocol.

It extracts the output of a successful propagatable value or propagates its residual into the enclosing computation.

The propagation protocol should not be intrinsically tied to `Error`, `Result`, or failure handling.

This allows constructs such as `Option` to use the same underlying mechanism:

```text id="b6whp1"
Result<T, E>
├─ Ok(T)  → output
└─ Err(E) → residual

Option<T>
├─ Some(T) → output
└─ None    → residual
```

The meaning of the residual depends on the type participating in the protocol.

---

## Error is behavioral

`Error` defines common behavior for recoverable failures.

It should not require APIs to erase precise error types:

```raven id="mlgnm8"
Result<T, FileError>
```

remains preferable when `FileError` accurately describes the possible failures.

Because:

```raven id="mgsc60"
FileError : Error
```

the value can still participate in diagnostics, contextualization, chaining, and other common error facilities.

Therefore:

> **`Error` is a behavioral abstraction, not necessarily the static error type.**

---

## Preserve the actual residual

Propagation should preserve the concrete residual whenever possible.

An error should not be wrapped merely because `?` was used.

```text id="l6axlh"
FileError.NotFound
        │
        │ ?
        ▼
FileError.NotFound
```

is preferable to:

```text id="npr5ys"
FileError.NotFound
        ↓
GenericError
        ↓
propagate
```

Wrapping should occur only when additional semantics require it.

---

## Context enriches rather than replaces

Higher layers should add information around existing errors rather than destructively translate them.

```text id="upkxjr"
ApplicationError
└─ ConfigurationError
   └─ FileError.NotFound
```

preserves considerably more information than repeatedly converting one error representation into another.

This also allows each architectural layer to contribute information relevant to its own abstraction level.

---

## Conversion and contextualization are distinct

Conversion answers:

> How can a value represented as `A` be represented as `B`?

Contextualization answers:

> How can `B` describe additional information about an underlying `A`?

These operations should not be conflated merely because both may occur during propagation.

---

## Virtual behavior without inheritance-heavy modeling

Raven should use interface dispatch to provide common error behavior without requiring a deep inheritance hierarchy.

This allows:

```text id="p7jvje"
                  Error
                    ▲
       ┌────────────┼────────────┐
       │            │            │
   FileError    ParseError   ContextError
    union         union         class
```

to share behavior while retaining representations appropriate to each error domain.

This is particularly important for Raven's union-oriented error modeling.

---

## Precise typing should retain common behavior

Using:

```raven id="y7wtzg"
Result<T, FileError>
```

instead of:

```raven id="y1hw4q"
Result<T, Error>
```

must not mean giving up Raven's error infrastructure.

The compiler and runtime should recognize that `FileError` participates in `Error` behavior and dispatch accordingly.

This allows Raven APIs to simultaneously provide precise static information and rich common error functionality.

---

## Avoid unnecessary allocations

Neither propagation nor participation in `Error` should inherently require allocating another error object.

This matters for both targets and is especially important for lightweight union errors on neoCLR.

Contextual decorators may introduce additional storage when context is actually requested, but ordinary propagation should remain as inexpensive as practical.

On .NET, existing exception objects should similarly be reused rather than copied into equivalent Raven error objects.

---

## Reuse target-native information

Raven should preserve useful diagnostic information already supplied by the target.

A CLR exception already contains exception-specific diagnostic information and should not need to be recreated merely to fit Raven's error model.

Likewise, neoCLR should be free to provide error diagnostics using runtime mechanisms designed specifically for Raven's value-based model.

---

## Common semantics do not require implementation symmetry

Raven-on-.NET and Raven-on-neoCLR should expose recognizably similar propagation and contextualization behavior.

They do not need identical internal representations.

.NET needs to coexist with exception-oriented libraries and runtime behavior.

neoCLR can implement Raven's error model natively and integrate it more deeply.

The goal is therefore semantic consistency where useful, not artificial runtime symmetry.

---

# Open design questions

## Residual reconstruction

Raven needs a precise mechanism for reconstructing the residual case of the enclosing propagatable type.

Given:

```raven id="0t92db"
func operation(): Result<T, E1>

func caller(): Result<U, E2> {
    let value = operation()?
}
```

the compiler can obtain `E1` from the source propagation protocol and determine an appropriate destination residual.

It must then construct:

```raven id="14ed73"
Result<U, E2>
```

in its residual state.

This should ideally be described by the propagation abstraction rather than permanently hard-coded specifically for `Result`.

A future revision of `IPropagatable` / `Propagatable` may therefore need to represent both decomposition and reconstruction.

---

## Residual conversion protocol

It remains to be determined whether Raven's normal conversion machinery is sufficient for residual adaptation.

There is a meaningful semantic distinction between:

```text id="odn4kw"
A can implicitly convert to B
```

and:

```text id="hzwycx"
A can be propagated as B
```

A conversion that is desirable specifically at a `?` boundary may not be desirable as a general implicit conversion throughout the language.

Raven should therefore explore whether residual adaptation deserves its own protocol or whether the general conversion model can express this distinction.

---

## Error-aware propagation

When both source and destination residuals implement `Error`, Raven may be able to provide richer default behavior than for arbitrary residuals.

For example:

```text id="63nxdm"
E1 : Error
E2 : Error
```

may allow the propagation system to preserve `E1`, wrap it in `E2`, or invoke error-specific behavior according to the destination type's capabilities.

The rules need to remain predictable and should not cause implicit contextual wrappers to appear unexpectedly.

---

## Error wrapping convention

Raven needs a standard way for an error type to declare that it can wrap another error.

Conceptually:

```raven id="gajg3a"
ApplicationError : Error {
    ...
}
```

might support:

```text id="k5azkp"
E where E : Error
        ↓
ApplicationError(E)
```

This could be expressed through a constructor convention, a static interface member, Raven's conversion interfaces, or a dedicated propagation abstraction.

The destination should own this behavior so lower-level error types do not acquire dependencies on higher-level application errors.

---

## Error interface capabilities

The exact capabilities belonging directly to `Error` remain to be determined.

Potential capabilities include:

- access to diagnostic information;
- access to an underlying cause;
- contextualization;
- diagnostic formatting;
- propagation hooks;
- error-chain traversal.

The interface should provide enough common behavior to justify virtual dispatch while avoiding becoming an oversized abstraction that every simple domain error must explicitly implement.

Default behavior should cover the ordinary case.

---

## Default interface behavior

Raven should investigate how much behavior can be supplied automatically when a type declares:

```raven id="7eghzm"
union FileError : Error {
    NotFound(Path)
    AccessDenied(Path)
}
```

Ideally, such a declaration should immediately participate in Raven's standard error infrastructure.

The implementation strategy may differ between .NET and neoCLR.

The language should therefore specify observable behavior rather than unnecessarily mandate one runtime representation.

---

## Diagnostic attachment

If a lightweight union implements `Error`, Raven needs an efficient mechanism for associating diagnostics with that value.

Possible implementations include:

- data stored directly with the error;
- contextual wrappers;
- compiler-generated hidden state;
- runtime-managed diagnostic associations;
- lazy diagnostic creation.

neoCLR may be able to support mechanisms that are not practical on .NET.

The representation should not force all simple error values to become expensive heap objects.

---

## Origin capture

Raven needs to define when an error acquires origin information.

Possible points include:

```text id="p9l0b6"
construct error
      ↓
place into Err
      ↓
first propagation
```

Capturing at construction provides the most accurate origin but may impose overhead on errors that are never propagated or inspected.

Capturing later may be cheaper but risks losing information.

This may ultimately involve different implementation strategies across targets while retaining similar observable behavior.

---

## Propagation diagnostics

It remains to be determined what information `?` should automatically record.

A useful distinction may be:

```text id="gtr0cq"
?                   → lightweight propagation record

.context("...")     → semantic contextual Error
```

This avoids constructing a visible error decorator for every propagation point while still allowing diagnostics to reconstruct the logical path an error followed.

---

## Context API

The precise API for adding context remains open.

For example:

```raven id="pr0a99"
operation()
    .context("Loading configuration")?
```

is conceptually straightforward, but contextualization may also need structured data rather than only text:

```raven id="lazd8p"
operation()
    .context(ConfigurationLoadError(path))?
```

Raven should allow context to remain strongly typed where useful rather than reducing all contextual information to strings.

---

## Error chain inspection

Applications need a consistent way to inspect heterogeneous error chains.

For example:

```text id="l9vp4a"
ContextError
└─ ApplicationError
   └─ FileError.NotFound
```

may contain classes, unions, foreign exceptions, and target-specific diagnostic carriers.

The inspection API should preserve concrete type information and allow callers to locate particular errors without requiring every layer to share the same representation.

---

## .NET exception integration

Raven needs to determine how closely CLR `Exception` should integrate with the `Error` protocol.

It may be undesirable or impossible to make arbitrary existing exceptions literally implement Raven's `Error` interface.

Possible approaches include compiler-known compatibility, adapters, overloads, or a broader common abstraction available only on the .NET target.

The solution should prioritize reuse of the original exception and avoid wrapper allocation unless additional Raven-specific behavior is actually required.

---

## Propagation specificity

If several propagation adaptations are applicable, Raven needs deterministic resolution.

For example:

```text id="crx65u"
FileError → ApplicationError

FileError → Error → ApplicationError

FileError → GeneralError → ApplicationError
```

could potentially provide several paths.

The compiler should either choose a clearly defined most-specific path or report ambiguity.

These rules should align with Raven's broader overload and conversion resolution model wherever possible.

---

## User-defined propagatable types

Raven should determine whether user-defined types may participate fully in `?`.

A complete protocol may need to express:

- the output type;
- the residual type;
- how output and residual are distinguished;
- how the enclosing type is reconstructed from a propagated residual.

The abstraction should support `Result` and `Option` naturally without being designed solely around either.

---

## `Option` and other residuals

`Option` provides an important test of whether the propagation abstraction is sufficiently general.

For example:

```raven id="pb6f8m"
func lookup(): Option<Item> {
    let parent = findParent()?
    ...
}
```

could propagate `None` through the same structural machinery used by `Result`.

However:

```text id="1nxun6"
None ≠ Error
```

No error diagnostics, contextual chain, or error dispatch is required.

This reinforces the architectural separation:

> **The propagation protocol defines early-return composition. `Error` defines the richer behavior of failure values.**

---

## Target-specific extensions

Raven should define a common semantic foundation while allowing target-specific capabilities.

The common model should include:

- output/residual propagation;
- residual compatibility and adaptation;
- precise concrete error types;
- the `Error` behavioral abstraction;
- contextual error composition;
- error-chain inspection.

The .NET target may additionally provide:

- exception capture through `try`;
- retargeted framework APIs returning `Result`;
- exception-aware propagation;
- integration with CLR exception diagnostics.

neoCLR may additionally provide:

- native error-origin tracking;
- runtime-assisted propagation diagnostics;
- optimized `Error` dispatch;
- efficient error metadata;
- native support for lightweight error values;
- richer logical propagation traces.

These differences are expected.

The goal is not to make neoCLR imitate .NET or to constrain neoCLR to what the CLR can support.

The goal is to give Raven a coherent propagation and error model that works across both targets while allowing neoCLR to realize that model more completely.