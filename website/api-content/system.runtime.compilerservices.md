---
uid: N:System.Runtime.CompilerServices
---
## Compiler and runtime integration

This module contains compiler-facing contracts such as union metadata,
AsyncStateMachine and AsyncTaskMethodBuilder. They connect generated code to
runtime services rather than forming a separate application programming model.

See [async builders](/docs/async-builders.html) for supported lowering contracts
and [reference support](/docs/reference-support.html) for documentation-only
projection details. Production libraries use native metadata and explicit runtime
contracts. Internal runtime services are not public application APIs.
