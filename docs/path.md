# Lexical paths

Implemented 2026-09-08: `System.IO.Path.Combine(String,String) -> String` and
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
let output = System.IO.Path.Combine("reports", "summary.txt")
System.Console.WriteLine(System.IO.Path.GetFileName(output))
```

Run `cargo test --locked --test path` for source/artifact, empty/rooted/trailing-path,
Unicode, lexical preservation and runtime-service checks.
