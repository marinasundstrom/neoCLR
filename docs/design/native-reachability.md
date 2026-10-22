# Native reachability and dispatch — development notes, 2026-10-22

The author asked how other runtimes handle the HTTP application's selected-method
budget, then clarified that native AOT should avoid call dispatch wherever targets
are statically known. This is a release investigation, not a new language contract.

## Comparison and direction

.NET Native AOT trims unused code and requires bounded reflection. Its documentation
identifies value-type generic instantiation as a source of native code growth:
[Native AOT deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/).
Rust's [monomorphization collector](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_monomorphize/collector/index.html)
builds a graph of concrete items, including dependencies introduced by function
references and trait-object vtables. Go's [linker reachability pass](https://go.googlesource.com/go/+/32bc0976f9c4333e4a9247d6acc908972c6c5d79/src/cmd/link/internal/ld/deadcode.go)
retains methods needed through interfaces and broadens retention when reflection
can discover methods. These are compiler/linker mechanisms, not garbage collection.

The useful distinctions for neoCLR are:

- A retained type descriptor does not inherently require every method body.
- Executable roots include direct calls, explicit reflection policies, runtime
  adapters, initializers and reachable dispatch targets.
- Generic specializations require a concrete reason for retention. Code sharing
  is a separate possible optimization, with ABI/layout and specialization tradeoffs.
- A known call target can be linked directly even if its final numeric address is
  assigned at link/load time. Virtual/interface calls still require selection when
  multiple implementations are possible. A single proven receiver/target can
  support devirtualization; full AOT compilation alone is not that proof.
- NativeImport's current dynamic scalar path caches an address and uses an indirect
  call. It is not ordinary direct symbol linkage. Static import linkage remains a
  separate deployment feature, and callbacks remain runtime function values.

These principles align with existing runtime semantics. They do not authorize
removing required interface members from the private verification projection or
turning a multi-target call into a direct call without proof. The 2,048-function
bound is an experimental implementation budget, not a managed-language rule. Its
native root-probe counterpart must remain synchronized if it is changed.

## Current investigation and bounded optimization

A rebuilt HTTP JSON server reaches the existing budget. A diagnostic trace finds
real paths into JSON collection adapters, asynchronous state machines, stream
adapters and TLS cleanup; it does not establish a duplicate-selection bug. A
4,096-function diagnostic build with the original selection algorithm timed out
at 180 seconds in an unoptimized inspection run. No native execution occurred.
The temporary limit and trace instrumentation were removed from source.

Sampling that diagnostic run identified repeated interface-compatibility queries
inside closed-world selection. Selection and value-profile admission now share
answers within each immutable input scope. The key includes the complete target
type (including generic arguments) and the concrete source type row. Only type
relationships are cached: growing reachability, method targets and verification
results are not cached. A new scope is created for a changed or projected module.

The cost is temporary hash-map storage for queried type pairs, bounded by admitted
input inventories. Warm queries avoid repeating interface closure and compatibility
work. No cache is retained in the compiled application, and code selection and
calling conventions remain unchanged. This improves compiler work rather than
claiming better request throughput or completed HTTP qualification.

With the cache, diagnostic inspection completes and compilation admission passes:
2,055 functions and 662 types, including 932 generic clones. Only seven functions
exceed the shipping budget. This gives a concrete input for a coordinated budget
review; it does not qualify a mismatched native host or change the shipping bound.
The diagnostic elapsed time is not a controlled before/after startup measurement.
Both current managed JIT and AOT use this selection pipeline. Direct-call and
future devirtualization opportunities apply to both; runtime profiling/guarded
specialization are possible JIT techniques, not implemented by this change.

Validation: 135 backend unit tests pass (three manual probes excluded). All six
selection checks including the manual probe pass. The five-round unoptimized
relationship probe changes median time for 4,096 queries from about 12.5 ms to
1.5 ms, including cold entries in the first cached round.
[Evidence and limitations](../validation/release-stabilization-20261022/dispatch-relations.json).
