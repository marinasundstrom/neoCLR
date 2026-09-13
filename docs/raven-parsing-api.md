# Raven integer parsing

The experimental target exposes `System.Int32.Parse(string)` returning
`Result<int, Int32ParseError>`. `IsInvalidFormat` and `IsOverflow` distinguish the
error outcomes. Typed matches and `?` propagation work with this carrier:

```swift
import System.*
func ParseInput(text: string) -> Result<int, Int32ParseError> {
    let value = Int32.Parse(text)?
    return Result<int, Int32ParseError>(Result.Ok<int>(value))
}
```

The [full sample](experiments/raven-target/samples/library-parsing.rvn) prints the
successful value or the specific error. It tests signed values, Int32 limits,
overflow, empty/malformed text, whitespace and non-ASCII digits. Failure returns
before the code following `?` runs. Propagating into a different error carrier is
rejected unless the caller supplies a conversion.

## Existing contract and .NET comparison

This projects the [existing parser](int32-parse.md); it does not change its grammar.
Parsing is ASCII decimal, accepts an optional sign, consumes the entire string and
does not trim whitespace. It is not culture-aware. Reuse the
[API policy comparison](api-policy.md) and [library research](library-preview.md):
.NET offers exception-producing Parse and Boolean/out TryParse; neoCLR returns an
ordinary typed Result. Callers retain the familiar entry point but must adapt their
error flow. This is intentionally not source/API equivalence to .NET parsing.

Raven's normal .NET framework projections must be disabled for this target
(`RavenFrameworkProjections=None`, already set in prepared projects). Otherwise its
standard parsing projection expects the .NET signature and rejects this declaration.
The bridge calls the neoCLR method directly; it does not lower a catch-to-Result
wrapper. No changes to Raven's default .NET behavior are required.

## Verification and scope

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/path/to/Raven -p:BuildProjectReferences=false \
  -- --parsing /tmp/FRESH-PARSE-PROBE
python3 docs/experiments/raven-target/verify_parsing.py /tmp/FRESH-PARSE-PROBE \
  --runtime /path/to/neoclr
```

The standalone probe also rejects ignored/inverted conditional extraction,
uninitialized errors and incompatible residual propagation. The saved-project suite
includes this sample; `verify_editor.py --parsing` checks Parse completion and that
host TryParse does not leak into the target surface. Use these with a fresh prepared
project as described in the [integration instructions](experiments/raven-target/README.md).

This slice exposes Parse and its error predicates, not every Int32 or error-case
member. [Error constructors, checked accessors and ToString](raven-error-api.md) are also projected.
The [Int32 instance methods](raven-integer-api.md) are projected separately. The metadata/error catalog now
supports this additional Int32 Result carrier while preserving existing Ok<int>
bindings. No arbitrary generic-payload support is implied. Installed SDK/VSIX assets
have not been refreshed.

The separate [integer division slice](raven-division-api.md) projects the existing
Result-returning Divide helper using the same carrier mechanics.
