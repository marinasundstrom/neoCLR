# Int32.Parse: typed error contract

`System.Int32.Parse(String) -> System.Result<Int32,System.Int32ParseError>` returns
an ordinary Result with a successful Int32 or a specific parsing error. The non-generic
System.Int32ParseError carrier contains one of its directly nested ordinary types:

- InvalidFormat: empty text, a sign without digits, or any non-ASCII-digit after the sign.
- Overflow: syntactically valid decimal text outside the Int32 range.

An optional leading + or - is accepted. Whitespace is not trimmed. Leading zeroes are
accepted, including long zero-prefixed values. The entire grammar is validated before
range conversion: `999999999999999999999x` is InvalidFormat, not Overflow. Culture-aware
parsing is outside this contract.

The carrier exposes IsInvalidFormat/IsOverflow properties, checked GetInvalidFormat and
GetOverflow accessors, and ToString for presentation. Incorrect case extraction Faults.
Cases have ordinary constructors and use the existing explicit System.Value carrier
storage; no inheritance, implicit conversion or union-specific IL is involved.

## Native service contract

`neoCLR.Runtime.ParseInt32(String) -> System.Value` uses a narrow internal protocol:

| Erased payload | Meaning |
| --- | --- |
| Int32 | Successful value |
| Byte 1 | InvalidFormat |
| Byte 2 | Overflow |

The host validates syntax and performs range conversion. Platform IL constructs the
public error cases and Result. It never compares diagnostic strings to determine the
case. Unexpected payload types or status codes Fault. The registry validates the helper
signature; reachability reports ParseInt32 and ValueStorage. No new intrinsic was added.

A future platform-written parser can replace this helper without changing the public
contract. Faults such as allocation or execution failures are not converted into parsing
errors.

## Caller migration

```text
ldstr "42"
call System.Int32::Parse(String)
call instance System.Result<Int32,System.Int32ParseError>::GetOkCase()
call instance System.Result.Ok<Int32>::get_Value()
```

That excerpt assumes success. General callers preserve the Result, branch on
get_IsErrorCase, then extract System.Result.Error<System.Int32ParseError>.Value and
inspect the parsing error's case. Message text is not a discriminant.

The errors and file-input samples deliberately translate parsing errors into their
application-level System.Error messages when combining parsing with domain or file
failures. This conversion is explicit in application IL; the library preserves the
specific error type. Their malformed-input output is now `InvalidFormat`.

Reassemble applications and System together: the former Result<Int32,Error> return
contract and native Error payload no longer apply. No parallel Typed API is provided.
JSON format 3 remains unchanged; the public library contract is intentionally breaking.
