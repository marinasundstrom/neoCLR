# Int32.Parse: ordinary Result boundary

`System.Int32.Parse(String) -> System.Result<Int32,Error>` now constructs its result
through ordinary platform-library constructors. Successful parsing produces System.Ok<Int32>;
invalid input produces System.Err<Error> containing the existing `InvalidInt32` message.
The [ordinary carrier convention](union-convention.md) supplies predicates and checked
accessors. No exception handling or implicit error conversion is involved.

The decimal grammar is unchanged: an optional sign followed by ASCII digits within
Int32 range. Whitespace, empty text, malformed digits and overflow produce Error.
Culture-aware parsing and broader text APIs remain future work.

The long-term error parameter may become a closed ordinary union such as
`ParseError.InvalidFormat` and `ParseError.Overflow`, yielding
`Result<Int32,ParseError>`. Those cases remain regular nested types and constructors;
the VM needs no exception or error-specific instruction. The preview keeps the
existing `System.Error` payload while carrier migration continues.

`System.Math.AbsTyped` is the first parallel library API using the same ordinary
nested Result cases. It demonstrates incremental migration without changing the
legacy `Abs` contract yet.

## Native service contract

The existing InternalCall helper `neoCLR.Runtime.ParseInt32(String) -> System.Value`
returns exactly one erased Int32 or Error. Rust still performs numeric text parsing,
but does not construct a System.Result, recognize union metadata, or use the old
bootstrap Union value for this operation. No additional intrinsic was introduced.

The platform Parse method tests for Int32, extracts the primitive, constructs Ok<Int32>
and then Result<Int32,Error>. Its other path extracts Error and constructs Err<Error>
then the carrier. An unexpected helper payload fails checked extraction and Faults;
it cannot silently become a successful parse. The exact return signature is validated
by the binding registry. Service reports include ParseInt32 and ValueStorage, including
when the helper is reached without the public wrapper.

This is a narrow internal host-service protocol, not the final user-facing union
convention. A future platform implementation may replace the text parser when its
required string/character operations exist. The public Result behavior should survive
that replacement.

## Caller migration

```text
ldstr "42"
call System.Int32::Parse(String)
call instance System.Result<Int32,Error>::GetOk()
call instance System.Ok<Int32>::get_Value()
```

The excerpt assumes success. General code first calls get_IsOk/get_IsErr and branches
on the same saved Result before extraction. See [errors.neoil](../examples/errors.neoil)
for an application that handles both parse and domain errors and continues execution:

```sh
cargo run --locked -- run examples/errors.neoil
```

Its output remains `42`, `InvalidInt32`, `Expected a positive number: -1`,
`Execution continued`, then `=> Void`. It contains no bootstrap union instructions.
Returned carriers can be re-imported through the [bounded host boundary](erased-inputs.md).

This slice **changes the return contract** of both Parse and its InternalCall helper.
Old clients using ldcase/is.case after Parse must be reassembled with ordinary member
calls; the verifier rejects the old extraction and unverified execution Faults.
The old native-helper return signature is rejected during binding. There is no hidden
adapter. JSON format 3 remains in use during migration; this is not a promise that old
clients are compatible with a changed System library.

The errors, types, features and file-input callers have been updated. ReadNumber in
the file example now returns an ordinary Result, but ReadAllText still returns a
bootstrap Result: its failure branch extracts the Error and constructs an ordinary
carrier explicitly in application IL. Divide, Abs, slicing, file and console helpers
still need migration. Final bootstrap removal and its format break remain Preview 1 work.
