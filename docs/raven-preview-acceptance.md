# Raven/neoCLR fundamental preview acceptance

The preview should demonstrate a strong, useful subset of the new platform, not a
complete CLR implementation. All existing public runtime-library APIs developed with
Neo are in scope for Raven access in this POC; this does not require new .NET APIs or
full language-feature parity. Familiar .NET type and IL behavior is the baseline;
the visible differences are the runtime library's Result/Option error model, generic
Void, and target library contracts consumed through Raven.

## What the experiment should make visible

API feedback is an intended outcome of the POC. Familiar platform principles and
useful code compatibility guide the design, while neoCLR can define its own APIs.
Result-based error handling deliberately changes calling contracts and requires
adaptation of exception-based code. Evaluators should help test whether these
contracts are useful and coherent; similarity to .NET does not promise identical APIs.

Visible rough edges are part of the POC's value. An evaluator should be able to see
which APIs follow familiar .NET behavior, which deliberately differ, and which are
provisional, absent or under discussion. That is concrete material for evaluating
neoCLR's direction and participating in its design.

Present the library as an evolving experiment. Keep the examples useful, describe
what they demonstrate, and link to the implemented/unsupported matrix and open API
choices. Record test evidence for supported behavior alongside those choices; visual
or naming familiarity alone does not imply .NET compatibility or a settled contract.

## Demonstration boundary

| Area | Required evidence before public preview | Current state |
| --- | --- | --- |
| Value/reference behavior | Value-carrier copies remain independent; class and array aliases share mutations | Verified in saved-project and application checks, including bounded application classes, value copies, inheritance and interfaces; [import limits](raven-application-types.md) remain explicit |
| Runtime library | Calls execute neoCLR library implementations, not metadata stubs or host .NET implementations | The [existing library audit](raven-runtime-api-coverage.md) is complete for the bounded projection; declarations without concrete implementations are identified there |
| Errors and absence | Success, expected error and absent-value paths use Result/Option | Verified with explicit case handling |
| Propagation | Success continues; failure returns a compatible carrier; incompatible carriers are rejected | Existing Result/Option payload projections, file errors and Result<Void,E> execute with early returns; [carrier and signature limits](raven-union-api.md) remain explicit |
| Match syntax | Match expression and statement forms compile and execute; rejected forms and diagnostics are documented | [Bounded Raven matrix verified](raven-match-matrix.md): typed/imported cases, target-typed destructuring, both expression spellings and statement actions work; the statement-arm return caveat remains documented |
| Text files | Read/write text, show expected I/O errors through Result, and demonstrate round-trip data | [Bounded read/write projection verified](raven-file-api.md), including propagation, typed matches and temporary-file checks; [error-type APIs are projected](raven-error-api.md) |
| Date and time | Obtain the system's current local date/time using the separate date/time library concepts | [Existing calendar/clock APIs are projected](raven-calendar-api.md); full formatting/globalization is not required |
| VS Code | Completion resolves target APIs and the saved project runs through the neoCLR task | Verified with the isolated [local .8 SDK/extension](raven-query-local-build.md), target completion/hover and packaged tasks |
| Queries and callbacks | Deferred filtering/projection, materialization and generic target interfaces work together | [Where/Select/ToList](raven-query-api.md), captured callbacks and custom Raven iterators pass; [the order workflow](raven-order-workflow.md) combines queries with Result/Option and file I/O |
| Reflection | List useful type/member information from the target runtime | [Public introspection APIs are projected](raven-reflection-api.md), including class descriptors and flags |

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
latest local SDK/extension is 0.1.12-neoclr.8. Its six packaged suites include 50
saved-project checks, 14 query checks, application/workflow checks, direct NeoIL and
editor coverage; see [current build provenance](experiments/raven-target/query-toolchain.json).
The earlier .6 audit and nine-suite package record remains historical evidence for
that build. The updated order-summary sample runs on .8 but was added after packaging;
its new source checks do not retroactively change the archive validation record.

## Distribution boundary

The neoCLR experiment is on main, while Raven remains
on `codex/neoclr-target-resolution`. Evaluate Raven changes separately as the experiment
advances. Include matching experimental Raven SDK/VS Code extension builds with the
NeoCLR preview, pinned to exact revisions and accompanied by the target declarations,
runtime library, demo sources and build/run instructions.

Validate the bundle outside the developer checkout before publishing it. The
[experimental release procedure](experiments/raven-target/RELEASING.md) uses separate
Raven packaging and focused neoCLR checks rather than Raven's full release cycle.
The packaged task runner now uses a published compiler bridge and supplied library
without either development checkout. The macOS arm64 bundle has been validated locally;
publication remains a separate step. This plan does not announce a release version or promise unchanged
.NET binary compatibility. Keep published release notes frozen and record the final
supported/unsupported matrix with the new preview.

See [the runnable demonstration](experiments/raven-target/README.md#fundamental-raven-demonstration),
[VS Code setup](experiments/raven-target/VSCODE.md), and
[target contract comparison](raven-target-contracts.md).
