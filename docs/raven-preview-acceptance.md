# Raven/neoCLR fundamental preview acceptance

The preview should demonstrate a strong, useful subset of the new platform, not a
complete runtime or class library. Familiar .NET type and IL behavior is the baseline;
the visible differences are the runtime library's Result/Option error model, generic
Void, and target library contracts consumed through Raven.

## Demonstration boundary

| Area | Required evidence before public preview | Current state |
| --- | --- | --- |
| Value/reference behavior | Value-carrier copies remain independent; class and array aliases share mutations | Verified in the combined Raven project checks; arbitrary application class import remains outside the bounded profile |
| Runtime library | Calls execute neoCLR library implementations, not metadata stubs or host .NET implementations | Verified for the admitted Math, Console, union and collection APIs |
| Errors and absence | Success, expected error and absent-value paths use Result/Option | Verified with explicit case handling |
| Propagation | Success continues; failure returns a compatible carrier; incompatible carriers are rejected | Runtime extraction interface and carrier factories implemented; Raven projection/import still pending |
| Match syntax | Match expression and statement forms compile and execute; rejected forms and diagnostics are documented | Raven-target release matrix still required; Neo support alone does not establish this |
| Text files | Read/write text, show expected I/O errors through Result, and demonstrate round-trip data | Existing runtime APIs need bounded Raven admission and end-to-end checks |
| Date and time | Obtain the system's current local date/time using the separate date/time library concepts | Existing runtime APIs need bounded Raven admission; full formatting/globalization is not required |
| VS Code | Completion resolves target APIs and the saved project runs through the neoCLR task | Verified using the installed experimental extension server and prepared workspace |
| Reflection | List useful type/member information from the target runtime | Desirable; admit and document a bounded surface if it fits the demo |

Normal execution prints only guest output. Return-value and GC inspection are opt-in
runtime diagnostics. Terminal runtime faults remain distinct from recoverable Result
values. Cleanup/unwinding guarantees must not be inferred from the absence of ordinary
exception flow; their separate design is still deferred.

## Distribution boundary

Fix regressions and merge the neoCLR experiment into neoCLR main, while Raven remains
on `codex/neoclr-target-resolution`. Evaluate Raven changes separately as the experiment
advances. Include matching experimental Raven SDK/VS Code extension builds with the
NeoCLR preview, pinned to exact revisions and accompanied by the target declarations,
runtime library, demo sources and build/run instructions.

Validate the bundle outside the developer checkout before publishing it. The current
local task runner depends on a built Raven checkout and is not yet that self-contained
public bundle. This plan does not announce a release version or promise unchanged
.NET binary compatibility. Keep published release notes frozen and record the final
supported/unsupported matrix with the new preview.

See [the runnable demonstration](experiments/raven-target/README.md#fundamental-raven-demonstration),
[VS Code setup](experiments/raven-target/VSCODE.md), and
[target contract comparison](raven-target-contracts.md).
