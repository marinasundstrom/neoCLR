# Raven delegates and lambdas on neoCLR

The post-Preview-4 source bridge supports the existing `Func` family with static
application functions, class instance method groups, virtual/interface method groups,
and Raven's non-capturing and capturing lambdas. A callback can leave its creating
function; copied callbacks retain the same capture object. Mutable captures shared by
multiple lambdas observe the same storage.

[application-delegates.rvn](experiments/raven-target/samples/application-delegates.rvn)
demonstrates these forms, including `ArrayList.FindIndex` with a predicate and
`Func<int, System.Void>` for a callback without a payload. Run it using the source
instructions in [application types](raven-application-types.md), or run
`verify_application.py` for the isolated checks. The sample output is:

```
8
42
99
12
15
42
42
123
123
1
-2147483648
```

The final line exercises ordinary CLI unchecked addition, not Result-based arithmetic.
The importer now admits the existing `add`, `sub` and `mul` instructions for its
Int32/Int64/floating stack categories. Checked arithmetic and additional numeric
instructions remain bounded separately; this does not change the library's error policy.

## CLR comparison and layers

This reuses the [delegate design research](delegates.md) and CLI method-pointer/delegate
patterns: `ldftn` selects a method, `ldvirtftn` resolves through the receiver, and the
delegate constructor consumes the receiver and method address. Raven's ordinary
compiler lowering creates closure classes and capture fields. Those emitted objects
use neoCLR's managed heap; no language-specific closure object or new opcode is needed.

The importer keeps function addresses as a validated, short-lived intermediate that
must be consumed by an admitted delegate constructor. They are not guest native
pointers. `ldvirtftn` preserves a binding-time null check. Runtime delegate binding now
accepts nominal object references, retains them for GC and resolves virtual/interface
implementations. Existing restrictions on retaining frame-backed managed references
remain enforced, including execution without the optional typed verifier.

The bridge emits small adapter methods for bound application callbacks. These preserve
`ldftn` direct/base selection versus `ldvirtftn` dispatch and adapt a no-stack-result
CLI void method to the runtime's unit-return `Func<Void>` contract. Class adapters
reuse the original receiver. Interface adapters currently allocate a small wrapper
holding that receiver. That wrapper and the extra adapter call are implementation
costs, not claimed performance improvements. Future loader/JIT work can remove them
without changing source semantics.

The source toolchain requires the experimental Raven branch including `3142f2f13`.

Raven previously emitted `ldftn` for interface/abstract method groups and used virtual
binding even for an explicit base method group. Its experimental branch now respects
both distinctions. An ordinary CLR execution test checks interface/abstract dispatch,
base selection and null failure at binding; target execution checks the neoCLR path.
This is a general compiler correctness finding, not a required divergence for neoCLR.

## Boundaries and validation

Supported callbacks use the runtime library's `Func` family (zero to four inputs).
Generic application definitions, new application delegate declarations, value-receiver
method groups, multicast delegates, expression trees and unrestricted method-pointer
operations remain outside this importer. Nested non-generic application types are
flattened to token-derived internal names so Raven closure classes can be admitted;
this does not yet supply full nested-type reflection metadata.

The application checks exercise shared mutable captures, escaping callbacks, a
collection predicate, arithmetic wrapping, and failure at null interface binding.
They also insert allocation pressure and require a real GC collection while an
escaping callback remains usable. `tests/nominal_delegates.rs` separately checks
receiver lifetime across returns, copies and GC, virtual/interface targets, and null
binding. Legacy delegate/closure tests remain in the validation set.

Published Preview 4 SDK/VSIX assets remain unchanged. Refreshing a distribution and
Raven source debugging are future release work. Task<Void>, suspension and async/await
remain a separate design; supporting synchronous Func<Void> does not settle them.
