# Explicit disposal and fallible close

System.Disposable and System.Closable<E> are ordinary library interfaces:

```text
.interface System.Disposable
    .method instance byref Dispose() -> Void
    .end
.end

.interface System.Closable<E>
    .method instance byref Close() -> System.Result<Void,E>
    .end
.end
```

Both borrow the original value through an explicit managed interface view. Changes
take effect in its slot immediately; there is no receiver copy or implicit writeback.
Copying an ordinary value remains independent of either method. A copied resource
descriptor may still share backing storage under its documented contract; interface
conformance alone does not establish safe ownership of that storage.

Dispose releases a resource or discards temporary state and leaves a valid disposed
value. Repeated Dispose calls must be harmless. Dispose returns real Void, not a
recoverable error; implementations must use another API when callers need to handle
expected release failures. Runtime Faults remain possible as for any guest method.

Close completes a resource-specific operation and exposes expected failure as E.
Examples include finalization and flushing, but this slice introduces no file handle
or stream service. Successful completion, release and destruction are distinct:
a resource may release as part of Close, or preserve readable closed state until
Dispose. Its documentation must specify which. A failed Close must leave a value
that can still be disposed; retryability and partial external effects are specific
to the implementation. Dispose must not claim that an earlier failed Close succeeded.

For the initial protocol, a previously successful Close remains successful when
called again, including after subsequent disposal. Otherwise, closing a disposed
value returns a documented E. Implementations must preserve enough state to make
those cases distinguishable. Destruction can later invoke Dispose explicitly through
declared lifecycle metadata; implementing an interface is not itself such metadata.

These are behavioral interface contracts. The VM checks declared conformance,
receiver modes and exact result signatures, but does not prove idempotence or a
resource-specific state machine. The methods are ordinary guest IL. No automatic
scope cleanup, using syntax, destructor dispatch or special Dispose/Close opcode is
introduced. Explicit calls are required today, including on ordinary Result errors.
Terminal Faults retain their existing teardown behavior.

## Sample and validation

The [draft sample](../examples/disposal.neoil) uses an ordinary value containing text,
closed/disposed flags and a release counter. It models the protocol without external
I/O or pointer ownership: Close rejects an empty draft, successful Close marks it
complete, and Dispose clears the text once. The counter measures this sample's
disposal action, not a native resource release or runtime destructor call.

```sh
cargo run --locked -- verify examples/disposal.neoil
cargo run --locked -- run examples/disposal.neoil
cargo test --locked --test disposal
```

Output is `Draft closed`, `1`, `Draft is empty`, `1`, then `=> Void`, each on its own
line. Tests cover source/artifact execution, forwarded interface calls, repeated
disposal, independence of copied drafts, repeated close, failure followed by disposal,
and mismatched interface implementations. The sample's public fields make it an
instructional fixture; a production type should restrict mutation of lifecycle state.

See [Clonable](cloning.md) for explicit cloning and the [lifecycle proposal](lifecycle.md)
for the remaining copy/release, destruction and Fault-cleanup work.
