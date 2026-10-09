---
uid: N:System.Math
---
## Numeric functions and constants

Module functions provide integer and floating-point operations such as Abs, Min,
Max, Clamp, rounding, roots and trigonometry. Pi, E and Tau are Double constants.
These members belong directly to the module; no container type is required.

Integer Abs can return an overflow error, and Clamp can return an invalid-range
error. Check individual overloads for their result types and floating-point behavior.
See [expected outcomes](/features/outcomes/) for Result-based error handling.
