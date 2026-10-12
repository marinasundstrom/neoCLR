# StackTrace

`System.Diagnostics.StackTrace` is a development class in `System.Runtime`.
Construct it to capture the current logical guest stack, then use `ToString()` to
print that owned snapshot. Formatting later does not capture a new stack.

```raven
import System.Diagnostics.*

let trace = StackTrace()
Console.WriteLine(trace.ToString())
```

| Member | Behavior |
| --- | --- |
| `init()` | Captures from the caller, innermost first. |
| `ToString() -> string` | Returns the captured method names and IL instruction indexes. |

Snapshots survive return and GC. Capture is bounded to 64 frames and 64 KiB UTF-8,
with explicit truncation. Allocation exhaustion remains a terminal Fault. No local
values or arguments are retained. Source lines, structured frame access, native
unwinding, cross-thread capture and suspended async callers are not implemented.
Generated async-body names may appear. Native capture requires a matching development
host/backend with published GC roots and stack-capture binding enabled. This feature
does not introduce recoverable exceptions or change Fault handling.

[Implementation, .NET comparison and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/stack-trace/README.md).
