# Native parameter defaults (development, 2026-10-30)

Configuration drives this bounded slice: optional and reloadOnChange are bool
parameters defaulting to false. Native Raven previously rejected any explicit
parameter default. Its target capability now accepts scalar constants and null
reference defaults, exports them and imports their symbol values for caller binding.
There is no new syntax or Runtime Contract entry; ordinary .NET emission is unchanged.
This is target-specific integration on Raven codex/neoclr, not a merge from main.

## Native representation

A method's optional `parameter_defaults` array aligns with declared parameters,
excluding the instance receiver. An absent/empty array means no defaults. A null
slot means required; an object means a present default, including explicit null:

```json
[{"kind":"Boolean","value":"false"}, {"kind":"Null","value":""}]
```

Kinds are Boolean, SByte, Byte, Int16, UInt16, Int32, UInt32, Int64, UInt64, Single,
Double, String and Null. Integers use canonical invariant decimal; Boolean uses
true/false. Single/Double encode IEEE bits as 8/16 lowercase hex digits, preserving
negative zero and nonfinite constants without JSON number ambiguity. Strings retain
their contents; Null requires an empty payload and a reference or nominal value-type
parameter. On a nominal value type it denotes zero initialization, not a null value.
Raven currently admits ordinary non-union struct defaults, including CancellationToken. Loaders
validate alignment, kind, range, canonical spelling and type compatibility.
The optional field extends current native metadata; older readers need updating
before consuming assemblies that carry defaults.

The metadata model exposes ParameterDefaultValue, MethodDefinition.ParameterDefaults,
SetParameterDefault (also on MethodBuilder), and ParameterInfo.HasDefaultValue/
DefaultValue. Clearing a default passes null to the setter; an explicit null constant
uses `new ParameterDefaultValue(null)`. Loaded definitions remain immutable.

## Caller semantics and CLI bridge

Omitted arguments are materialized by Raven at the call site. Native runtime calls
still pass every argument; defaults neither change arity nor enter method identity.
Named arguments can skip optional parameters. Changing a library default requires
recompiling callers to observe it, as with .NET optional arguments.

The CLI bridge uses standard Param Optional/HasDefault flags and the Constant table,
not custom attributes. The metadata library owns native/CLI round trips; Raven owns
symbol import and call binding; neoCLR validates metadata and executes full-arity
calls. Native metadata already replaces any need for a CLI default-attribute encoding.
The comparison is with ECMA-335 II.22.9 (Constant) and II.22.33 (Param), not a claim
that ordinary .NET scalar defaults require attributes.

Enum/char/decimal constants, bare generic-parameter defaults and Optional.None
remain outside the native optional-parameter profile.
They remain diagnosed by the native callable capability and require separate semantic
and representation qualification. No language-service or grammar change is needed.

## Focused validation

- Metadata tests: `dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --parameter-defaults`.
- Runtime: `cargo test --test parameter_defaults` validates accepted/invalid metadata
  and proves defaults do not change execution arity.
- Raven: ParameterDefaultCapabilityTests plus imported native constructor/method
  defaults and named argument calls in the Configuration reload consumer.
- `scripts/validate-configuration-reload.py` qualifies real file edits, malformed
  JSON, atomic replacement, required/optional deletion, DI live sections and disposal
  in native and interpreter execution. See the dated validation receipt.

## Runtime API consolidation (2026-10-30)

`CancellationToken = default` now survives source binding, native library import and
CLI projection. The existing Null constant plus the declared value type describes
zero initialization; no custom attribute, constructor call or runtime arity shortcut
is needed. This follows the typed CLI Constant convention rather than introducing
an ambiguous boxed struct literal. The runtime validates nominal value storage;
Raven main fixes null-constant binding before the target capability admits it.

Default arguments are declared on InputStream/OutputStream interfaces as well as
concrete stream views, so calls use the static receiver's contract. Stream, socket,
DNS, TLS and HTTP operations remove forwarding overloads whose only difference was
CancellationToken.None. Selected text/gzip factories use `leaveOpen: bool = false`,
FileSystemWatcher uses `filter: string = "*"`, and TimeOfDay.Create uses
`fractionTicks: int = 0`. Encoding-selection overloads and values computed from
current culture, runtime state or a nonconstant factory remain distinct.

This changes method identities and interface slots in the development libraries:
rebuild all binary consumers and implementations together. Ordinary omitted-argument
source calls continue to bind. Method-group/delegate bindings that depended on a
removed arity need a lambda that supplies or omits the optional argument. Defaults
are inserted by the caller; they do not create additional callable signatures.
As with .NET, changing a default requires recompiling callers. Unlike maintaining
old .NET binary overloads, this author-directed development cleanup removes them.

[Focused runtime qualification](validation/runtime-defaults-20261030/README.md)
covers omission, explicit/named tokens, pre-cancellation, scalar defaults and stream
ownership through both native and interpreted consumers.
