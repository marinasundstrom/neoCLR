# Console and standard stream POC

Development sample, 2026-09-23. System.Console stays a static class, with TextReader
In and TextWriter Out/Error, plus byte-stream factories. Run with a matching
platform-only development toolchain:

```sh
python3 docs/experiments/console-streams/verify.py --toolchain-root /path/to/bundle
```

Main.rvn prompts on stdout, reads a bounded UTF-8 line, prints a greeting on stdout
and a diagnostic on stderr. The verifier separately captures both output channels
and supplies UTF-8, CRLF, empty-line, EOF, unterminated-line, malformed and oversized
inputs. Contracts.rvn demonstrates a custom short-writing OutputStream, text writer
ownership, sequential line reading and raw standard output.

See the [on-site Console guide](../../../api-docs/console.md) for the .NET/Node
comparison, Result/Option contracts, LF/CRLF limitation, byte limits, embedding
hooks and wrapper ownership. This is synchronous console I/O, not runtime suspension.


Propagation.rvn and IfLet.rvn show Result/Option composition without nested matches:
`?` propagates a read error, and a Some binding distinguishes actual input from EOF.
The guard form returns from its else branch; the if-let form scopes input inside
its success block. Both preserve empty input lines as valid strings. Main is the
single application boundary that reports a propagated read error. The verifier
checks data, EOF, empty-line and invalid-UTF-8 paths in both programs.
