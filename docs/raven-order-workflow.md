# Order workflow on neoCLR

The newest prepared local environment is [the .12 collection build](raven-collections-local-build.md),
which opens the collection scenario described below. The file/report workflow remains
available as application-orders.rvn in its sample directory.

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

## Collection integration scenario (2026-09-13 source slice)

The separate [collection workflow](experiments/raven-target/samples/application-order-collections.rvn)
combines the newer library APIs with an application-defined Order class. It does
not replace the file/report scenario above. It requires current core metadata and
System library; the installed .11 bundle does not contain the collection additions.

The example registers three orders through MutableMap<int, Order> and ArrayList<Order>.
TryAdd rejects a duplicate before the list is changed. Find returns Option<Order>;
PendingOrder propagates absence with `?` and also returns None for a shipped order.
No default Order or null placeholder is needed. HashMap's equality and hash callbacks
are explicit in this prototype; a default comparer is still future work.

FindAll eagerly captures the pending entries in new list storage. Updating an order
through the map is visible through both lists: the entries retain ordinary class
identity. The filtered list keeps its original membership; it does not automatically
remove an order whose Pending flag changes. A fresh query observes the current state.
This distinction is intentional and is visible in the sample's output.

OnlyPending accepts Iterable<Order>, calls Single with a predicate, and propagates the
Result. Initially two pending orders yield Multiple, then one yields order 303,
then none yield Empty. An Order[] also supports Where/Select/ToList through the array's
Iterable contract. FindIndex demonstrates Some(0), distinguishing the first position
from absence. Imported Some/None and Ok/Error patterns destructure application-class
payloads without managed-reference syntax or manual casts.

This scenario reuses the [.NET collection comparison](collection-contracts.md),
[Map contract](map-contracts.md), [filtering review](arraylist-filtering.md) and
[LINQ terminal review](raven-query-api.md). Shared class identity and shallow collection
copies preserve familiar CLR behavior; explicit absence/cardinality results are
neoCLR library policy. The sample adds no instruction, metadata convention, runtime
intrinsic or Raven compiler change. Registration is not a transaction: a terminal
allocation fault is not rolled back, and concurrent access is outside the prototype.

### Run and inspect

Prepare a fresh source project using the [source integration guide](experiments/raven-target/README.md)
and `prepare_editor.py --collections`, regenerating core metadata with the current
`--interfaces` probe. Do not reuse a .11 declaration DLL for this sample. In the
following commands, replace the project, Raven and runtime paths with that fresh
project and its matching built tools; run from the neoCLR checkout:

```sh
cp docs/experiments/raven-target/samples/application-order-collections.rvn /path/to/fresh/editor/Main.rvn
python3 docs/experiments/raven-target/run_project.py /path/to/fresh/editor/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

Open that editor folder in VS Code and use **neoCLR: Run saved project**. The normal
Raven toolbar is not the neoCLR execution path. Completion should expose Order's
members when destructuring the map lookup or accessing a filtered list element.
The [expected output](experiments/raven-target/samples/application-order-collections.expected.txt)
includes duplicate rejection, absence, Multiple/Empty results, order 303 and index 0.

`verify_application.py` runs the scenario with GC pressure and with deliberately
colliding hashes, checking unchanged output and requiring a collection to occur in
the GC variant. `verify_editor.py --maps` checks member discovery on the application
payload through both map lookup and FindAll. The package builder already includes
the sample directory and verification scripts, so a future fresh bundle will carry
this scenario; this source slice does not itself build, install or publish a release.

The later predicate-overload source slice simplifies OnlyPending to
`orders.Single(predicate)?`. The installed .12 sample retains its equivalent
Where(predicate).Single() spelling; do not copy this newer source into .12 without
refreshing its matching core metadata and System library.
