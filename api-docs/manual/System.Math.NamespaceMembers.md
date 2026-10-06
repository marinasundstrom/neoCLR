# System.Math.NamespaceMembers

**Development compiler-reference container.** Raven emits the `System.Math`
namespace functions inside this public metadata type. Applications call the namespace
functions; they do not construct a NamespaceMembers instance.

The [System.Math reference](xref:System.Math) documents every overload of Abs, Clamp,
Min, Max, Sign, Sqrt, Floor, Ceiling, Truncate, Round, Exp, Log, Log10, Sin, Cos, Tan
and Pow. Int32 Abs and Clamp return typed Result values for overflow or invalid
bounds. Floating-point functions return Double values.

This page preserves the public container's reference entry while the function
signatures and descriptions remain on their generated member pages.


**Native development (2026-10-06):** the same public functions compile as native
module functions, without this CLI container. A separate consumer executes all 20
functions. The retained CLI bootstrap's Math type requires an explicit namespace alias
in that mixed configuration; see the [Math guide](math.html). Published Preview 12
artifacts are unchanged. No public signature or result contract changes in this slice.
