# Order workflow on neoCLR

The newest prepared local environment is [the .9 stabilization build](raven-stabilization-local-build.md), which opens this workflow by default.

This post-Preview-4 sample moves beyond calls to a predefined library catalog. Raven
compiles application classes and an interface into ordinary CLI metadata; neoCLR
imports and executes them against its own runtime library.

[application-orders.rvn](experiments/raven-target/samples/application-orders.rvn) defines:

- `Order`, a class whose status changes are visible through shared references.
- `OrderStore`, an application interface for adding, finding, saving and querying orders.
- `FileOrderStore`, with an `ArrayList<Order>` and a bounded text-file report.
- `Process`, which handles Option absence and Result outcomes using case patterns.
- A deferred `Pending()` query, projected to names with Select and materialized with ToList.

`Save` uses `?` to propagate `FileWriteError` before changing the order status. Its
success type is `Result<System.Void, FileWriteError>`: completion has no payload.
`Find` returns `Option<Order>`, retaining the reference to the existing order.

This adopts familiar CLR class identity, interface dispatch and generic collection
behavior. The deliberate library difference is explicit Result/Option error and
absence handling instead of exceptions/null. It is not a .NET binary-compatibility,
transactional storage or production repository claim. The report is one text file,
not a serialization format or database; globalization and async are not involved.

The updated source sample imports `System.Result.*` and `System.Option.*`, and uses
`Some(let order)`, `.Error(let failure)` and `Ok(let text)`. It requires the pattern
slice of the experimental compiler/bridge and refreshed declaration metadata;
archived .7 tools still use the earlier typed-case sample. See the
[pattern matrix](raven-match-matrix.md) for supported forms.

## Try the combined workflow locally

The expanded source sample runs on the installed `.8` tools. A separate project is
prepared at `/Users/robert/.neoclr/experiments/order-workflow-20260913`; the existing
query demo and its edits were preserved. Open it with the isolated extension:

```sh
code --new-window \
  --user-data-dir /Users/robert/.neoclr/vscode/queries-20260913 \
  --extensions-dir /Users/robert/.neoclr/vscode/queries-20260913/extensions \
  /Users/robert/.neoclr/experiments/order-workflow-20260913
```

Save `Main.rvn`, then run **neoCLR: Run saved project** from the task menu. The report
is written to this demo folder. Change the second `Process` limit from `2` to `64`
to let both writes succeed: the pending summary becomes empty. No SDK or compiler
change was needed for this sample; the .8 archive still contains its original version.

## Run with the source toolchain

Use the experimental Raven branch at `000ed511e` or a later revision containing it.
Published Preview 4 tools do not include these changes.
Build Raven.CodeAnalysis and the current neoCLR runtime; the source bridge builds
when invoked. Use the .8 demo project and its matching declaration assembly, as
explained in the [source integration guide](experiments/raven-target/README.md).

Copy `application-orders.rvn` to the project's `Main.rvn`. From a disposable working
directory, run (substitute your absolute paths):

```
python3 /path/to/neoCLR/docs/experiments/raven-target/run_project.py /path/to/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

The program creates or replaces `neoclr-orders.txt` in that working directory. It prints:

```
Saved
Order report exceeds limit
Order not found
Saved
Queued
Order: Coffee
Pending orders
Tea
```

The second order's failed write leaves it queued and preserves the first report.
The first order's status changes through the interface/store reference and is visible
to the caller. No explicit reference-address syntax is needed for these classes. The pending query
is created before processing, but observes the later status changes when materialized.
A failed write leaves the order pending; when both writes succeed, no names remain.
This follows the familiar deferred query model described in [the query contract](raven-query-api.md).
The new list holds copied names; it does not clone the Order objects. This sample uses
normal completed iteration and does not claim automatic early-exit or fault cleanup.

For the repeatable check, which creates its own temporary working directory:

```
python3 docs/experiments/raven-target/verify_orders.py /path/to/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

It checks output, report content, state after a rejected write, missing-parent
read/write errors, deferred pending membership and an empty all-saved summary. Tests
use temporary working directories and reject stale executable fallback. Raven source
debugging remains separate future work.
