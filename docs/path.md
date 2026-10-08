# Lexical paths

Implemented 2026-09-08: `System.Storage.Path.Combine(String,String) -> String` and
`GetFileName(String) -> String`. These use host-platform path syntax, with no file
access, existence checks, canonicalization, trimming, or removal of `.`/`..`.

Combine returns the other operand when one is empty. A rooted second operand
replaces the first; otherwise it adds the host directory separator if needed.
GetFileName returns the final lexical component, including its extension; a trailing
separator or an empty input returns an empty string. Backslash is an ordinary
filename character on Unix. Windows recognizes both slash forms, drive prefixes
(including drive-relative C:foo), and UNC prefixes.

These contracts follow the shipped [.NET 10 Path implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/IO/Path.cs).
String inputs are nonnullable. Invalid filesystem characters are preserved here and
rejected by file operations when appropriate. In particular Combine is not a way to
confine a user-supplied path beneath a directory. We do not claim full Windows device
path compatibility before Windows validation; ordinary drive/UNC cases have tests
conditional on that target. Current validation was on macOS.

Two InternalCalls declare PathOperations. Host-aware lexical work currently belongs
in these bootstrap helpers because general character indexing remains unsettled for
UTF-8 String; public wrappers are ordinary library IL. Moving more into library code
can follow the text primitives. This is an implementation choice, not an opcode or
metadata change. General path resolution, extensions and directory APIs remain later
work driven by applications.

```swift
let output = System.Storage.Path.Combine("reports", "summary.txt")
System.Console.WriteLine(System.Storage.Path.GetFileName(output))
```

Run `cargo test --locked --test path` for source/artifact, empty/rooted/trailing-path,
Unicode, lexical preservation and runtime-service checks.

Both methods are now projected into the experimental Raven target; see the
[Raven sample, checks and limitations](raven-path-api.md).


## Native compilation POC (2026-10-08)

The macOS ARM64 backend now binds these two exact InternalCall contracts with
`--compile-system --reference-arena --bind-paths`. Public Path wrappers remain
ordinary library CIL. The opt-in is separate from UTF-8 string services and does not
permit file reads/writes. Selection reports the original definitions and Unix lexical
semantics; managed methods, altered contracts and implicit bindings are rejected.

The linked private adapters use slash separators, retain `.`/`..`, preserve UTF-8
and embedded NUL bytes, and allocate an immutable result in the existing text arena
or native GC. They never normalize paths or touch the filesystem. Backslash and
Windows drive syntax are ordinary characters on this target. The interpreter retains
its existing host-specific behavior; native Windows semantics are not implemented.
Null arguments fault, allocation/size failures leave output untouched, and adapters
never collect or reenter managed code. Generated native calls keep input text rooted.

This reuses the established .NET lexical comparison above (source link corrected and
rechecked against .NET 10.0.0 on 2026-10-08), rather than defining a new path model.
The existing neoCLR API uses nonnullable UTF-8 text; .NET GetFileName also accepts null.
neoCLR keeps its existing null-fault behavior. Results always copy, including unchanged
text; .NET may reuse a string. That simplifies the current ownership contract but
costs an allocation/copy and is not a claimed performance improvement. A span/view
optimization would need separate lifetime and compatibility evidence.

Thirty-three native/interpreter cases cover empty/rooted/trailing paths, Unicode,
NULs, Unix treatment of backslashes/drive-looking text, nested owned results and null
faults. Sanitized C checks cover overflow/exhaustion, unchanged failure outputs,
canaries and GC retention/reclamation in both arena and GC builds. Exact-contract
rejection is tested separately.

[Recorded Raven validation](../benchmarks/native-web/path-validation.json) runs the
unchanged [library-paths sample](experiments/raven-target/samples/library-paths.rvn)
in interpreted, sanitized native and standalone modes with asserted output, cleanup
and libSystem-only linkage. HTTP compiler admission remains accepted. Reproduce with
`benchmarks/native-web/verify_callbacks.py --case Paths` and the usual explicit
compiler/runtime/AOT/bundle/output arguments. This is a WIP native capability, not
filesystem support, cross-platform qualification or a new benchmark result.
