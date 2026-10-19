# Native async match regression

Raven `f61cffbfd` visits match-arm guards and bodies during cleanup discovery and
suspension rewriting. RuntimeMatch.rvn covers statement and expression arms, a
pending task, explicit GC, return and cancellation with reverse-order disposal.
All three execution modes pass; the prior compiler rejects the await (baseline.log).
The routed HTTP sample now retains its match dispatch and compiles. This is a
compiler-correctness result, not a throughput measurement.
