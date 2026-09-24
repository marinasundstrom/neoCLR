# Terminal faults and codes

**Development after Preview 9.** Every host-visible runtime `Fault` now has a
machine-readable code as well as its diagnostic message and execution location.
This is a host API; guest programs cannot catch a Fault or select its code.

## Explicit faults from guest code

`System.Fault(message: string) -> unit` terminates the invocation with **UserFault**.
The explicit neoIL `fault "message"` instruction has the same code. There is no
code argument or overload. A message such as `StackOverflow` remains a UserFault;
the runtime never infers the classification from guest-supplied text. Explicit
fault calls in guest library code also use UserFault.

Use Result for expected errors that callers can handle. A stream's Closed result
or a Task's cancelled state is not a terminal Fault. These codes do not introduce
exceptions, catch/finally behavior or a new guest error hierarchy.

## Reading faults in an embedding host

Rust hosts receive `Result<Execution, neoclr::Fault>`. The `Fault` fields are:

| Field | Type and purpose |
| --- | --- |
| `code` | `neoclr::FaultCode`; runtime-assigned classification |
| `message` | `String`; human diagnostic text, not a stable discriminator |
| `function` | `Option<String>`; faulting function when available |
| `instruction` | `Option<usize>`; instruction index when available |
| `stack_trace` | `Option<StackTrace>`; owned execution frames; may be absent before execution |

`FaultCode` is a copyable, comparable, hashable, non-exhaustive Rust enum. Match
known variants with a fallback for future additions. `FaultCode::as_str()` returns
a stable, case-sensitive identifier. Display and Serde serialization use that same
string. Enum ordinals are not an API, and these are not .NET HRESULT values.

```rust
match program.run(neoclr::Limits::default()) {
    Ok(execution) => { /* consume the result */ }
    Err(fault) => match fault.code {
        neoclr::FaultCode::StackOverflow => { /* report excessive call depth */ }
        neoclr::FaultCode::UserFault => { /* report an explicit guest failure */ }
        _ => { /* handle another terminal failure */ }
    },
}
```

The code is selected at the failure site, independently of message formatting.
Stack-trace capture and worker error propagation retain it. A host that constructs
Fault values itself is trusted host code; this is not a guest capability.

## Codes

| Stable identifier / Rust variant | Meaning |
| --- | --- |
| `UserFault` | Explicit guest fault instruction or System.Fault call, including guest library calls |
| `StackOverflow` | Configured interpreter call-frame limit exhausted, including a zero-frame budget |
| `EvaluationStackOverflow` | Configured evaluation/operand-stack limit exhausted |
| `InstructionLimitExceeded` | Invocation instruction budget exhausted |
| `ExecutionCancelled` | Host cancellation or debugger stop, not guest Task cancellation |
| `DivideByZero` | Integer divide or remainder with a zero divisor |
| `ArithmeticOverflow` | Checked integer arithmetic or conversion overflow; also signed minimum divided by minus one |
| `NullReference` | Null managed object, array or interface dereference, including value unboxing |
| `InvalidCast` | Unboxing a value whose concrete boxed type differs from the requested type |
| `NullPointer` | Null native pointer dereference |
| `IndexOutOfRange` | Managed array index outside its range |
| `HeapLimitExceeded` | Managed heap object or identity budget exhausted |
| `ArrayLimitExceeded` | Managed array payload budget or supported length exceeded |
| `NativeMemoryLimitExceeded` | Native pointer heap byte or allocation budget exhausted |
| `InternPoolLimitExceeded` | String intern entry or unique UTF-8 payload budget exhausted |
| `InvalidProgram` | Classified verifier failures or execution falling through without a return |
| `RuntimeError` | Other runtime, loader or host failures not yet assigned a narrower category |

Classification is being introduced incrementally: not every old diagnostic has a
specialized code yet. Consumers must handle RuntimeError and unknown future values.
StackOverflow here describes the interpreter's checked frame limit; it does not
claim to catch a Rust/OS stack overflow or an arbitrary host process failure.

## CLI and debugger

The CLI retains the diagnostic and adds the code:

```text
Fault: frame limit exceeded [code=StackOverflow] at Main:0
```

Debugger snapshots retain the existing `fault` diagnostic and add
`fault_code: "StackOverflow"`. `fault_code` is null when there is no terminal fault.
It is populated for execution faults, pre-execution failures, cancellation and
stopping. The terminal debugger displays the code separately. Tools should use
this structured field or Rust's `Fault.code`, not parse the formatted message.

Adding the public Rust field requires source changes for hosts that construct
Fault with struct literals. The CLI diagnostic format also gains the code; raw
message comparisons can continue to use `Fault.message`.

## .NET comparison

.NET exposes exception types and [Exception.HResult](https://learn.microsoft.com/en-us/dotnet/api/system.exception.hresult?view=net-10.0)
for classification. [StackOverflowException](https://learn.microsoft.com/en-us/dotnet/api/system.stackoverflowexception?view=net-10.0)
has a defined HRESULT and concerns CLR execution-stack exhaustion. neoCLR keeps a
small terminal host outcome with a symbolic code. It supplies stable classification
without a guest exception hierarchy, but has coarser categories and no HRESULT
compatibility or catchable-fault semantics. The embedding process is not deliberately
aborted by an ordinary neoCLR Fault.


### String intern retention limits (development)

Rust hosts configure `Limits.intern_entries` (default 4096) and `Limits.intern_bytes`
(default 1 MiB). Each interpreter execution has its own strong intern pool, including
an isolated worker execution. A new entry beyond either quota raises
`InternPoolLimitExceeded`; looking up an existing value still succeeds at capacity.
Empty text consumes an entry even though it consumes no payload bytes. These budgets
count unique retained UTF-8 payload, not table capacity, Arc headers, allocator
metadata, temporary input Strings or total process memory. Setting the entry quota
to zero prevents all insertions. Limits are terminal runtime budgets, not recoverable
Storage errors or guest-selected Fault codes. Hosts constructing Limits exhaustively
must supply the two new fields; using `..Limits::default()` picks up the defaults.
