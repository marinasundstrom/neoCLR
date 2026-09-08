# Closed call-graph analysis

LoadedProgram::analyze_reachability accepts explicit FunctionRef roots and a maximum
number of distinct closed functions. It returns a conservative call graph without
executing IL or loading native libraries. This is a backend-planning foundation for
the interpreter/JIT/NativeAOT architecture, not a native compiler or compilability test.

```rust
let graph = program.analyze_reachability(
    &[neoclr::assembler::parse_function_ref("Main()")?],
    1024,
)?;
```

No entry point is inferred. Library methods, instance methods, pointer-bearing
signatures, runtime InternalCall declarations, and P/Invoke declarations can be roots.
Unlike host invocation, analysis does not need an importable receiver or argument value.
Roots must have closed signatures and obey the root module's direct-reference list,
scoped type checks, and exact definition/revision selectors.

## Graph contract

The report has one root index per request, retaining repeated roots. Functions are
ordered by discovery: roots first, then their calls in IL instruction order. A fixed
root order and equivalent resolved load set produce the same ordering. Indices identify
nodes only within that report; they are not metadata rows or persistent cache keys.

Each node records its canonical closed FunctionRef, including definition identity,
return type, implementation kind, outgoing calls with source instruction indices, and
[direct runtime-service uses](runtime-services.md).
Implementation kinds distinguish IL, runtime InternalCall, and P/Invoke; native imports
retain library, symbol, and calling-convention metadata. They are terminal graph nodes,
not claims that foreign code or runtime helpers have no further dependencies.

Deduplication uses the selected module/revision/function row plus closed declaring type.
Different instantiations of the same generic type remain distinct. Bound overloaded
methods remain distinct even if their signatures coincide after substitution. This
matches today's type generics; method generics would require extending the key.
Canonical type names still rely on the current globally unique type namespace.

Analysis specializes each IL body and follows every syntactic call, including calls
after ret and inside branches that would never execute. Repeated calls retain separate
edges; recursive calls to an existing instantiation reuse its node. Calls within
libraries use the loader-validated declaring module references, so a root may reach a
transitive dependency without being permitted to name that dependency directly.

## Bounds and remaining work

The function limit includes roots, closed instantiations, and import declarations.
An empty root list returns an empty graph even with a zero limit. Exceeding the limit
returns a Fault, with the caller/instruction location when expansion caused it, and no
partial report. Expanding recursion such as Grow<T> calling Grow<Grow<T>> therefore
fails within the supplied bound or the existing type-substitution nesting limit.
The bound counts functions, not source bytes, edges, elapsed time, or native code size.

Loading validates metadata first. Typed verification remains separate and opt-in.
The report does not scan fields for layout closure, evaluate control flow, discover
attribute constructors or reflection roots, infer initialization, enumerate native
transitive dependencies, or assign code-sharing policy. Implicit runtime services
used by instructions (such as allocation) are reported separately as service uses,
not call edges. It also does not emit
specialized IL artifacts, machine code, or a binary metadata format.

Future AOT work must combine this graph with target layout, explicit backend capability
checks, exported roots, runtime-service dependencies, and a compilation policy. New
indirect or virtual dispatch features will need an explicit reachability contract.

`cargo run --example reachability` reports Main, System.Console.WriteLine(String),
and neoCLR.Runtime.WriteLine(String), including the two call edges, without printing
the guest HelloWorld message.

## Borrowed interface dispatch

Each callvirt expands conservatively to all matching implementations in the loaded
module set, including implementations not otherwise constructed by reachable code.
Multiple call edges can share an instruction index. Abstract declarations are not
executable graph nodes or valid roots. Generic implementation arguments are inferred
from the closed interface; if some cannot be inferred, analysis returns a Fault
instead of silently omitting potential targets. See [interfaces](interfaces.md).

Reachable function nodes also carry receiver_byref, out_parameters and out_when_true
indices so backend planning preserves reference receiver and output contracts.


Delegate binding dependencies are reported in `ReachableFunction.bindings`, separate
from direct `calls`. `delegate_invocations` records (instruction, closed delegate
type) for indirect Invoke sites. Backends must retain binding edges as executable
dependencies; an Invoke site does not identify a unique target. Virtual/interface
binding conservatively includes dispatch candidates. An Invoke declaration alone
is not a concrete analysis root.
