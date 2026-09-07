# Native lowering contract

neoIL is designed to be interpreted first, but its instructions should also map
cleanly to native machine code on every supported target. The portable instruction
set therefore uses typed operands, explicit control flow, and explicit memory
operations. A backend should be able to lower arithmetic, comparisons, branches,
locals, fields, calls, pointer arithmetic, and typed loads/stores without recovering
hidden language semantics.

Operations that need platform support are explicit runtime services: console I/O,
allocation, bounds/provenance checks, native interop, and interrupt polling. Their
metadata identifies the required service so an AOT compiler can link or reject it
deterministically. They are not implicit consequences of a type such as `Array<T>`
or `System.Value`.

Each instruction has one canonical stack effect and normalized operand types. The
interpreter, JIT, and NativeAOT backend must agree on those effects, fault conditions,
layout rules, and pointer lifetime checks. Target-specific instruction selection may
choose registers, calling conventions, and addressing modes, but it must preserve
the same observable contract.

The first native-lowering subset should cover integer and floating arithmetic,
conditional branches, typed local/field access, explicit `Ptr<T>` operations,
fixed-layout records, and direct calls. Variable-sized arrays, allocation policies,
native calls, and asynchronous interrupts remain explicit lowering boundaries rather
than reasons to make the core IL target-specific.
