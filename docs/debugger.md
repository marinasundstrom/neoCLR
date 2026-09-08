# Interactive debugger

The terminal debugger inspects a live neoCLR execution without changing guest
values. It shows the call stack, arguments, locals, evaluation stacks, managed heap
objects, GC statistics and tracked native allocation bytes. Neo source locations
are retained through compilation and artifact loading.

## Start a session

From the repository root:

```sh
cargo run --locked -- debug examples/source/debugger.neo
```

Execution starts paused before the first IL instruction. The header identifies the
current frame, IL instruction and Neo source line. The sample has a frame-owned
Counter, a managed heap Counter, an owned array and calls through reference parameters.
It returns 20 when allowed to complete.

The command also accepts `.neoil` and JSON artifacts:

```sh
cargo run --locked -- assemble examples/source/debugger.neo /tmp/debugger.neo.json
cargo run --locked -- debug /tmp/debugger.neo.json
cargo run --locked -- debug examples/reference_receivers.neoil
```

Assembly refuses to overwrite an existing output file; choose a fresh path when
repeating the command. IL/artifact module sets support the same `--module` and
`--system` options as execution. Neo source currently uses one file and bundled System.

## Walkthrough

At `neo debug>`, enter these commands one at a time:

```text
source
stack
break Add 0
continue
```

`continue` resumes execution immediately. Once the breakpoint is reached, inspect it:

```text
status
bt
stack
stack 0
heap
```

The innermost frame is `Add`. `stack` shows its `counter` and `amount` arguments;
`stack 0` shows Main's locals. The first call's counter points to `heap#0`, and
`heap` shows its Value field. Reference diagnostics identify their owner and interior
field/element path, rather than exposing interpreter host addresses.

Now step through the mutation and return to Main:

```text
next
heap
out
source
clear
continue
```

`next` runs to the next mapped source line at the same call depth, stepping over
calls. `out` runs until the current frame returns. After completion, `status`, `gc`
and `output` remain available. The final result is 20 and the heap Counter has been
reclaimed. Finish with `quit`.

For a source-line breakpoint, restart and enter `break 13`, then `continue`. Source
breakpoints use the current mapped document, or the input document if there is no
current source mapping. A line with multiple sequence points can stop more than once.
`break Add 0` is an IL breakpoint: function names must match `bt` exactly, including
qualified names for methods. There is no automatic breakpoint name resolution yet.

## Commands

| Command | Meaning |
| --- | --- |
| `continue`, `c` | Resume execution |
| `pause`, `p` | Request a pause at the next instruction boundary |
| `step`, `s` | Execute one IL instruction, entering guest calls |
| `next`, `n` | Next source line at this depth, stepping over guest calls; without a mapping, next instruction at this depth |
| `out` | Run until the current frame returns |
| `bt` | Call stack, innermost first, with source and IL locations |
| `stack [frame]` | Arguments, locals and evaluation stack of a frame; defaults to innermost |
| `heap [id]` | Managed heap payloads; optionally select an allocation identity |
| `memory` | Tracked native allocations, ownership and initialized bytes; `??` means guest-uninitialized |
| `source` | Source location and a short excerpt when debugging the original `.neo` input |
| `il` | Current IL instruction |
| `gc` | Allocated/live/peak objects, collections and reclaimed objects |
| `output` | Last 32 captured guest output lines |
| `watch` | Toggle live call-stack, stack and heap display while execution continues |
| `break <line>` | Add a source-line breakpoint |
| `break <function> <index>` | Add an IL instruction breakpoint |
| `clear` | Remove all breakpoints |
| `input <text>` | Queue UTF-8 input followed by a newline for the guest console |
| `eof` | Signal end of guest console input after queued bytes are consumed |
| `status` | Current debugger state and latest snapshot overview |
| `help`, `h` | Command reference |
| `quit`, `q` | Stop the debug execution and leave the debugger |

Entering an empty command does nothing. Step, next and out require a paused
execution. Pause/step commands wait briefly for a new snapshot; a slow host/native
call may leave the request pending. Use `status` to inspect progress.

For a longer live demonstration, launch `cargo run --locked -- debug examples/source/debugger_live.neo`,
enable `watch`, then enter `continue`. Its heap counter increases to 1,000,000. Updates are printed while the
command prompt remains available; enter `pause` to inspect a stable snapshot or
`watch` again to stop automatic display. Console input belongs to the debugger, not
directly to the guest: use `input 42` and `eof` when debugging an input-driven program.
A waiting console read is reported as `waiting for input` and `quit` releases it.

## Reading memory

Frame indices count from the entry frame (`0`), although `bt` lists innermost first.
Arguments and local slots are distinct from the evaluation stack. Evaluation-stack
indices run from bottom to top. Uninitialized locals are marked instead of being read.
Source local labels include their IL slot index, such as `shared_1`, to distinguish
names reused in different source scopes; compiler temporaries can remain unnamed.

Managed heap identities are stable for an execution. Frame slot coordinates are valid
within the displayed snapshot and can be reused after a frame returns. A reference
such as `&heap#0 path=[0]` points into that allocation; path components are record
field indices or array element indices. An interface view retains its concrete owner
and is marked accordingly. Snapshots never follow references recursively and never
register new GC roots. Cycles therefore do not recurse through the inspector.

“Stack memory” here means typed guest frame storage, not a dump of the interpreter's
native machine stack. Native allocations are a separate view. Only bytes belonging
to the interpreter's tracked allocations are inspected; untracked pointers are shown
as descriptors and never dereferenced. Managed heap values have no promised native ABI.

## Source mapping

The Neo compiler emits optional `sequence_points` on each Function. Each point records
an IL instruction index and a one-based document/line/column location. The `.sequence`
assembler directive carries that metadata without becoming an instruction. Labels,
branches, calls and instruction budgets retain their original indices and behavior.
Locations survive JSON round trips, linking and generic specialization, and also appear
in ordinary Neo fault stack traces.

`frontend::compile_named(source, document)` and `lower_to_il_named` allow embedding
hosts to supply a document name. Existing unnamed APIs use `<source>`. CLI compilation
uses the input path. The grammar is unchanged: mappings are compiler output, not Neo
syntax. Legacy artifacts without mappings retain IL-only debugging.

These are expression/statement locations, not PDBs or complete source spans. Generated
instructions can share a location. Optimized-code mapping, expression evaluation,
lexical lifetime ranges and exact source-level single stepping are future work.
The debugger reads excerpts only from its original `.neo` input; a JSON artifact retains
locations but does not embed source text or automatically open referenced files. A
modified source file may no longer match an older artifact.

## Embedding

Create a `debugger::Debugger`, clone its handle for the controller, and supply it as
`ExecutionOptions.debugger`. Run the program on a worker thread, keeping the Execution
on that worker: guest storage is intentionally not Send. The controller uses
`command`, `snapshot` and `wait` to inspect owned diagnostic data. Optionally install a
clone as `ExecutionOptions.console` to use the debugger's input queue and captured output.
Use one debugger per execution. A host that fails before launching the interpreter
can report that failure with `launch_failure`.

## Preview limits

- Launch under `neoclr debug` or supply an execution debugger handle. Attaching to an
  arbitrary existing OS process, remote debugging and editor/DAP integration are not implemented.
- Inspection is read-only. There are no memory writes, arbitrary expression evaluation,
  watchpoints, conditional breakpoints or reverse execution.
- Running snapshots refresh at most ten times per second at instruction boundaries.
  Paused and terminal states publish immediately, as do boundaries before host/native
  calls so a blocked call has an accurate guest stack. Snapshot revision identifies a captured
  state; running memory may already have advanced. Debugging adds execution overhead.
- Previews are bounded: 64 frames, 128 slots/stack entries per frame, 256 managed heap
  objects, 64 children per aggregate, nesting depth 8, a shared 8,192-value budget,
  256-character text previews, 128 native allocations and 128 bytes per allocation.
  Truncation is marked; an omitted object is not necessarily dead. There is no paging yet.
- At most 256 breakpoints and 64 KiB of queued console input are accepted.
- CLI debug removes the instruction-count timeout so interactive sessions can keep
  running; other memory/frame limits remain. Embedded executions retain their supplied limits.
- Pausing cannot interrupt an in-progress native or host call. `quit` exits the CLI
  without waiting indefinitely for such a call. Native machine frames are not unwound.
- Completed snapshots show the heap after final GC. Fault snapshots preserve the typed
  state before execution teardown; a failing instruction may already have consumed operands.

See [debugger tests](../tests/debugger.rs), [source grammar](neo-grammar.md),
[managed-reference semantics](managed-reference-semantics.md) and [GC](garbage-collection.md).
