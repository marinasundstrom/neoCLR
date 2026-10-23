---
uid: T:System.Environment
---
## Process environment

The static Environment class exposes command-line arguments, the current directory and
environment-variable lookup. Variable lookup distinguishes an absent value from
a host-access error through Option and Result.

These are host-process services. Check each function's result and documented host
behavior rather than assuming a variable or directory is available.

Development APIs also expose `GetOperatingSystem()`, `NewLine`,
`ProcessArchitecture` and `OSArchitecture`. Discovery returns a Result; platform
records retain domain-specific versions and provide readable ToString output.
Linux distribution details are currently unavailable.
