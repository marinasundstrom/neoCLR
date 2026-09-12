# Raven/neoCLR fundamental preview acceptance

The preview should demonstrate a strong, useful subset of the new platform, not a
complete CLR implementation. All existing public runtime-library APIs developed with
Neo are in scope for Raven access in this POC; this does not require new .NET APIs or
full language-feature parity. Familiar .NET type and IL behavior is the baseline;
the visible differences are the runtime library's Result/Option error model, generic
Void, and target library contracts consumed through Raven.

## Demonstration boundary

| Area | Required evidence before public preview | Current state |
| --- | --- | --- |
| Value/reference behavior | Value-carrier copies remain independent; class and array aliases share mutations | Verified in the combined Raven project checks; arbitrary application class import remains outside the bounded profile |
| Runtime library | Calls execute neoCLR library implementations, not metadata stubs or host .NET implementations | Partial: the admitted Math, Console, union and collection APIs work; audit and close all existing public API gaps before release |
| Errors and absence | Success, expected error and absent-value paths use Result/Option | Verified with explicit case handling |
| Propagation | Success continues; failure returns a compatible carrier; incompatible carriers are rejected | Bounded Result<Int32,OverflowError>, Option<Int32> and Result<Void,OverflowError> execute with early returns; file carriers also work, and other existing payload shapes remain to be projected |
| Match syntax | Match expression and statement forms compile and execute; rejected forms and diagnostics are documented | [Bounded Raven matrix verified](raven-match-matrix.md): typed cases, both expression spellings and statement actions work; deconstruction/shorthand and arm-return caveats are documented |
| Text files | Read/write text, show expected I/O errors through Result, and demonstrate round-trip data | [Bounded read/write projection verified](raven-file-api.md), including propagation, typed matches and temporary-file checks; full error-type API projection remains pending |
| Date and time | Obtain the system's current local date/time using the separate date/time library concepts | Existing runtime APIs need bounded Raven admission; full formatting/globalization is not required |
| VS Code | Completion resolves target APIs and the saved project runs through the neoCLR task | Verified using the installed experimental extension server and prepared workspace |
| Reflection | List useful type/member information from the target runtime | Required as part of existing API coverage; Raven admission pending |

Normal execution prints only guest output. Return-value and GC inspection are opt-in
runtime diagnostics. Terminal runtime faults remain distinct from recoverable Result
values. Cleanup/unwinding guarantees must not be inferred from the absence of ordinary
exception flow; their separate design is still deferred.

## Existing API coverage gate

Use the [runtime API inventory and coverage plan](raven-runtime-api-coverage.md).
Inventory the public declarations in `runtime/System.neoil` and its included modules,
then track metadata visibility, callable import, runtime behavior and an executable
Raven example/test for each API. Include primitive/string helpers, Console input and
output, Math, arrays/collections, Result/Option, delegates and foundational interfaces,
Environment, Path/File, date/time, Type/reflection and their error contracts. Internal
implementation fields/helpers are not application APIs. Count overloads and generic
forms explicitly rather than declaring a type covered because one method works.

Language and runtime feature limits may remain documented, but must not silently hide
an existing public API. Record necessary type-category/contract adaptations and their
.NET comparison. Refresh and test the packaged tools after the coverage work; the
current installed SDK is an intermediate development build.

## Distribution boundary

The neoCLR experiment is on main, while Raven remains
on `codex/neoclr-target-resolution`. Evaluate Raven changes separately as the experiment
advances. Include matching experimental Raven SDK/VS Code extension builds with the
NeoCLR preview, pinned to exact revisions and accompanied by the target declarations,
runtime library, demo sources and build/run instructions.

Validate the bundle outside the developer checkout before publishing it. The
[experimental release procedure](experiments/raven-target/RELEASING.md) uses separate
Raven packaging and focused neoCLR checks rather than Raven's full release cycle.
The local task runner depends on a built Raven checkout and is not yet that
self-contained public bundle. This plan does not announce a release version or promise unchanged
.NET binary compatibility. Keep published release notes frozen and record the final
supported/unsupported matrix with the new preview.

See [the runnable demonstration](experiments/raven-target/README.md#fundamental-raven-demonstration),
[VS Code setup](experiments/raven-target/VSCODE.md), and
[target contract comparison](raven-target-contracts.md).
