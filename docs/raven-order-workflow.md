# Order workflow on neoCLR

This post-Preview-4 sample moves beyond calls to a predefined library catalog. Raven
compiles application classes and an interface into ordinary CLI metadata; neoCLR
imports and executes them against its own runtime library.

[application-orders.rvn](experiments/raven-target/samples/application-orders.rvn) defines:

- `Order`, a class whose status changes are visible through shared references.
- `OrderStore`, an application interface for adding, finding and saving orders.
- `FileOrderStore`, with an `ArrayList<Order>` and a bounded text-file report.
- `Process`, which handles Option absence and Result outcomes using case patterns.

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

## Run with the source toolchain

Use the experimental Raven branch with commits `cbd87efa8` and `62105de24`, or a
later revision containing them. Published Preview 4 tools do not include these changes.
Build Raven.CodeAnalysis and the current neoCLR runtime; the source bridge builds
when invoked. Start from the Preview 4 demo project and its declaration assembly, as
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
```

The second order's failed write leaves it queued and preserves the first report.
The first order's status changes through the interface/store reference and is visible
to the caller. No explicit reference-address syntax is needed for these classes.

For the repeatable check, which creates its own temporary working directory:

```
python3 docs/experiments/raven-target/verify_orders.py /path/to/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

It checks output, report content, state after a rejected write, and missing-parent
read/write errors without stale executable fallback. The existing editor completion
surface is reused; refreshing the distributed SDK/VSIX and Raven source debugging are
separate release work.
