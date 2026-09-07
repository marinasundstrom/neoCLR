# Preview 1: programs and their IL mappings

Preview 1 is defined by a small set of programs and observable behaviors. The
[release checklist](preview-1.md) records the platform capabilities and publication checks
needed to support them. A feature earns priority by enabling one of these programs,
removing a prototype shortcut in its implementation, or making it reproducible.

These are **Raven-like pseudocode**, not compiler inputs or a language specification.
No high-level compiler is required for Preview 1: runnable equivalents are authored in
neoIL, assembled, verified and tested. Later, a compiler can lower the same program
contracts to the same metadata/IL. Do not add parser, inference, pattern-checking or
compiler self-hosting work to this release merely because it appears in the notation.

The notation takes `func`, `let`/`var`, name-before-type parameters and expression
`match` from [Raven's language examples](https://github.com/marinasundstrom/raven),
and carrier/case terminology from its [union specification](https://github.com/marinasundstrom/raven/blob/main/docs/lang/spec/unions.md)
(reviewed 2026-09-07). The programs below are original neoCLR examples. Deliberate
adaptations include real `Void`, `Err` as the error wrapper, explicit buffer lifetime,
and one value model without struct/class storage semantics. Raven's full syntax,
generated union representation and runtime dependencies are not adopted here.

## The program set

| ID | Program | Required evidence | Existing executable groundwork |
| --- | --- | --- | --- |
| P1 | Hello and a free helper | UTF-8 text, free calls, real Void, artifact round trip | hello.neoil, strings.neoil |
| P2 | Console calculation | Prompt, byte input, computation, EOF versus recoverable Error | console_input.neoil, errors.neoil |
| P3 | Typed data and alternatives | Constructors, private storage, properties, generic carriers, independent copies | constructors.neoil, ordinary_unions.neoil, ordinary_unions tests |
| P4 | Explicit buffer | Array allocation, indexing/loop, aliasing, exactly one free | arrays.neoil |
| P5 | A deliberate Fault | Terminal failure with guest caller frames and IL locations | stack-trace tests, array_bounds.neoil |
| P6 | Inspect a type | Type/value acquisition, exact identity, name, closed generic arguments | Host identity APIs only; guest support still missing |

The existing files are groundwork, not a claim that every program contract below is
already implemented or has an exact fixture. Preview 1 must give each row a checked-in
neoIL equivalent, expected output/exit behavior and automated coverage. P2 still uses
bootstrap unions; P3 has ordinary System carriers and a selected member convention,
but still needs API/host/native migration; P6 lacks guest APIs. Those gaps prevent
declaring Preview 1 complete.

## P1 — Hello and a free helper

```text
func Message() -> String {
    return "Hello, world!"
}

func Main() -> Void {
    Console.WriteLine(Message())
}
```

Expected stdout: `Hello, world!` and the CLI's final `=> Void`. Run both the source
and its assembled JSON artifact. No class container or high-level frontend is needed.
The executable IL equivalent is:

```text
.module Hello
.entry Main
.function Message() -> String
    ldstr "Hello, world!"
    ret
.end
.function Main() -> Void
    call Message()
    call System.Console::WriteLine(String)
    ret
.end
```

Void occupies a real evaluation-stack slot. A discarded WriteLine result requires
`pop` if more work follows; the final call above already supplies Main's return value.
UTF-8 text checks include non-ASCII strings in the companion string fixtures.

## P2 — Console calculation and recoverable failure

```text
func ReadDigit() -> Result<Option<Int32>, Error> {
    return Console.ReadByte() match {
        Err(let error) => Err(error)
        Ok(None) => Ok(None)
        Ok(Some(let byte)) => {
            let digit = Int32(byte) - 48
            if digit < 0 || digit > 9 {
                return Err(Error.FromMessage("Expected an ASCII digit"))
            }
            return Ok(Some(digit))
        }
    }
}

func Main() -> Void {
    Console.WriteLine("Enter one digit:")
    ReadDigit() match {
        Ok(Some(let digit)) => Console.WriteLine(digit * 2)
        Ok(None) => Console.WriteLine("No input")
        Err(let error) => Console.WriteLine(error.Message)
    }
}
```

For input byte `7`, print the prompt then `14`. For immediate EOF, print `No input`.
For byte `x`, print `Expected an ASCII digit`. A host-injected read failure takes the
Err path and prints its message. Prompt output must reach the host before a read blocks.
This deliberately reads one byte; it does not imply ReadLine, a Stream hierarchy,
text decoding of arbitrary input, or a console parsing intrinsic. ReadDigit is
application code. The existing ReadNumber sample demonstrates a larger bounded parser.

Lowering uses `call System.Console::ReadByte()`, branches, integer arithmetic, calls
to ordinary carrier constructors and typed accessors. Member calls include signatures;
Error.Message becomes a call to its getter. The conversion to Int32 follows the
existing small-integer stack rules. A match stores its scrutinee once, tests it, then
extracts only inside the selected branch. Expected absence/failure returns through
ordinary functions; it is not a Fault and creates no exception regions.

The final lowering must not contain `some`, `none`, `ok`, `err`, `is.case` or `ldcase`.
Those still occur in the current console implementation and must be removed before
this program meets its Preview 1 acceptance contract.

## P3 — Constructed values and ordinary alternatives

The following record/union declarations describe source intent. Constructor and
property shorthand would be compiler synthesis, not extra VM type categories.

```text
record Box<T>(value: T) {
    private let stored: T = value
    public Value: T { get => stored }
}

union Option<T> {
    case None
    case Some(value: T)
}
union Result<T, E> {
    case Ok(value: T)
    case Err(error: E)
}

func Describe(result: Result<Int32, Int32>) -> String {
    return result match {
        Ok(let value) => "success"
        Err(let error) => "failure"
    }
}

func Main() -> Void {
    let original = Box<Int32>(42)
    let copy = original
    Console.WriteLine(copy.Value)
    Console.WriteLine(Describe(Ok(7)))
    Console.WriteLine(Describe(Err(7)))
}
```

Expected output: `42`, `success`, `failure`. Companion cases must distinguish None
from Some<Void>, support Ok<Void>, nested carriers and Result<T,T>, return false on a
failed query without exposing a payload, and preserve the original when an extracted
record copy is updated. Real Void values are written `Void()` in this pseudocode and
lower to `ldvoid`; that spelling does not imply a runtime constructor on System.Void.

Box lowers to an ordinary `.type Box<T>` with a private T field, a public instance
`.ctor(T) -> Void`, a getter method and property association. The constructor
assembles the complete record then uses `starg this`; caller code is:

```text
.local Box<Int32> original
.local Box<Int32> copy
ldc.i4 42
newobj instance Box<Int32>::.ctor(Int32)
stloc original
ldloc original
stloc copy
ldloc copy
call instance Box<Int32>::get_Value()
call System.Console::WriteLine(Int32)
```

The last fragment leaves the WriteLine Void result on the stack. Local names map to
indices 0 and 1; source identifiers are not runtime identities. The compiler may use
different authoring names, but emitted signatures and metadata rows must agree.

Union declarations synthesize ordinary carrier and wrapper types. The
[System library carriers](union-convention.md) use a private System.Value field with
explicit erasure. Their marker aids future tooling, not execution. Match lowering is:

```text
ldloc result
call instance System.Result<Int32,Int32>::get_IsOk()
brfalse Failed
ldloc result
call instance System.Result<Int32,Int32>::GetOkCase()
call instance System.Result.Ok<Int32>::get_Value()
; Consume the success payload and branch to the join.
```

These are the implemented ordinary System member names. Generic
payload identity is preserved by distinct wrappers, not guessed from payload type.
Ordinary constructor/accessor bodies do the packing/testing/extraction. A compiler
recognizes the convention and checks exhaustiveness; the VM does neither. P3 is
complete only after existing API callers and host/native adapters replace
bootstrap union handling. This representation does not fix a native layout or allocator.

## P4 — An explicit buffer

```text
func Main() -> Void {
    let values = Array<Int32>.Allocate(3, 10)
    let alias = values
    alias.Set(1, 42)
    var index = 0
    while index < values.Length {
        Console.WriteLine(values.Get(index))
        index = index + 1
    }
    values.Free()
}
```

Expected output: `10`, `42`, `10`. Copying an array descriptor copies its pointer;
the descriptors alias explicit native storage, so free that allocation exactly once.
This does not change ordinary record value copying or imply an automatic destructor.
The actual [array sample](../examples/arrays.neoil) provides the full lowering:
Allocate/Set/Get/Free are ordinary calls, Length calls its getter, and while becomes
indexed locals plus conditional/unconditional branches. No array or stream hierarchy,
implicit allocation expression, covariance or general managed array is required.

## P5 — Faults and guest stack traces

```text
func Fail() -> Void {
    fault "Demonstration fault"
}
func Work() -> Void { Fail() }
func Main() -> Void { Work() }
```

`fault` here is deliberate pseudocode for the existing `fault "message"` IL operation.
It is not an Error result or a catchable exception. The process must fail, report the
message, and show guest frames in order Fail, Work, Main with member identities and
logical IL instruction locations. No statement after the Fault executes. Source-line
mapping and a guest StackTrace class are not Preview 1 requirements. Also retain a
system-generated fault case, such as out-of-range array access, and frame-limit tests.

## P6 — Minimal type inspection

```text
func Main() -> Void {
    let value = Box<Int32>(42)
    let declared = typeOf(Box<Int32>)
    let actual = typeOfValue(value)
    Console.WriteLine(declared == actual)
    Console.WriteLine(declared.Name)
    Console.WriteLine(declared.GenericArguments[0].Name)
}
```

`typeOf`, `typeOfValue`, descriptor equality/Name and GenericArguments are conceptual
operations, **not existing functions or settled API names**. The Boolean printing is
pseudocode too: the IL equivalent can branch and print String literals with the existing
WriteLine(String), avoiding an unrelated overload prerequisite. Expected semantic
results are true, the Box definition's name, and System.Int32's name; exact display
format must be specified with the descriptor API. Comparing Box<Int32> with Box<String>
must be false. Definition identity must also account for module identity.

Lowering will acquire a descriptor from a closed type token or a value, then use
ordinary read-only accessors. Opcode spelling, collection access and descriptor
lifetime need a separate implementation contract; no fictitious runnable IL is shown.
Generic-argument inspection need not require native Array<Type> storage or a collection
library. A count plus indexed getter is sufficient. No member invocation, reflective
construction/mutation, visibility bypass or copying of the .NET reflection hierarchy
is implied by this program.

## Running and accepting the set

For every final fixture, follow the [README workflow](../README.md): build, assemble
to a fresh JSON artifact, verify it, then run it. Tests must compare values/output,
including negative cases; P5 expects failure. Console fixtures use controlled input
and injected failures instead of waiting for a person. P1–P4 should also remain
usable through the embedding API, with explicitly supported host boundaries.

These programs define behavioral acceptance; the release checklist still covers
cross-platform/toolchain evidence, documentation, source license and reproducibility.
Passing the examples alone is not publication readiness. Adding a new feature should
identify the program and missing behavior it enables, or explicitly amend this set.
