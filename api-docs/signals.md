# Neo.Signals (development)

Assembly: `Neo.Signals`. Namespace: `Neo.Signals`. This separate experimental
extension requires matching native Raven/neoCLR revisions. Inclusion in the runtime
has not been decided. It uses structural function types and ordinary properties;
there is no event metadata and no nominal delegate requirement.

<a id="signal"></a>
## Signal<T> where T: func

A subscription-only view. Applications obtain it from `SignalSource<T>.Signal`;
its constructor is internal, and it exposes no publishing operation.

<a id="subscribe"></a>
### Subscribe(handler: T) -> Disposable

Registers the supplied function object, retaining its receiver/captures. Returns
an idempotent handle that removes exactly this registration. Duplicate function
objects are independent registrations. A removal does not retract an entry from
an already active dispatch snapshot. The handle can be retained for explicit
lifetime management or disposed through the normal scope-disposal contract.

<a id="source"></a>
## SignalSource<T> where T: func

Owns the registration list and publishing authority.

<a id="constructor"></a>
### init()

Constructs an empty source and its stable subscription view. `ReturnType<T>` must
be unit; another result type calls `Panic` in this initial profile. This runtime
policy is specific to this library, not to the `func` constraint.

<a id="view"></a>
### val Signal: Signal<T>

Returns the same subscription-only view on each access. Expose it through a public
read-only property while keeping the source private.

<a id="emit"></a>
### func Emit: T with args

The closed method has T's parameter list and result. For `(int, string?) -> unit`,
call `source.Emit(value, label)`. For `() -> unit`, call `source.Emit()`.
`args` is the implementation's owned `object?[]`, not a public parameter.

Delivery is synchronous in registration order, using a snapshot. No handlers is a
successful no-op. Reentrant changes affect later dispatches, including a nested
one started after the change. Handler faults propagate and stop the active delivery.
Argument marshalling has the same scalar/reference limits as
[FunctionReflection.Invoke<T>](functions.html#typed-invocation-and-method-signatures-development).
Concurrent access requires caller synchronization. There is no completion/error
channel, scheduler, Task handling or `+=`/`-=` operator in this slice.

The implementation allocates argument arrays and boxes values as necessary; it
makes no performance claim relative to .NET events. See the
[extension README and executable samples](https://github.com/marinasundstrom/neoCLR/blob/main/extensions/Neo.Signals/README.md).

This manual reference preserves the dependent `Emit: T` spelling. The current
RavenDoc/CLI bridge's ordinary parameter-list model cannot represent that open
signature or its function constraint faithfully. Both public types and every public
member are documented here pending native renderer support.

Execution qualification: the signal consumers pass in the interpreter. The native
compiler builds and shares signature specialization, but the signal JIT probe stops
at unsupported boxing of inhabited unit (`Void`). JIT/AOT execution of this library
is not qualified; unit boxing/unboxing support remains follow-up work.
