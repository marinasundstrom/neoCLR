---
uid: N:System.Runtime.CompilerServices
---
## Compiler and runtime integration

This module contains compiler-facing contracts such as union metadata,
AsyncStateMachine and AsyncTaskMethodBuilder. They connect generated code to
runtime services rather than forming a separate application programming model.

See [async builders](/docs/async-builders.html) for the transitional lowering
contract and [reference support](/docs/reference-support.html) for bridge-only
metadata. Internal bootstrap services are not public application APIs.
