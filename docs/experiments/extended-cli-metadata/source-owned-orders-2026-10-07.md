# Source-owned native orders gate — 2026-10-07

**Passed:** unchanged application-order-collections compiles against the separately
emitted 197-input class-library aggregate, imports native symbols and executes with
exact expected output and exit 0. Library sources are absent from the consumer command.
[Commands, compiler/runtime/dependency hashes and output](source-owned-orders-2026-10-07.json).

Raven's root-selection change is **3e5406d8f**. The tested compiler snapshot is labelled
801f131ba+imported-root; binary hashes identify its contents. The existing source library
and runtime seed come from [the retained-catalog audit](retained-catalog-2026-10-07.md).
No metadata writer or runtime modification was needed for this slice.

## Contract

MetadataImportOptions.WithObjectAssemblyName selects a native System.Object owner,
independently of the primitive bootstrap. The selected root is public, abstract,
fieldless, nongeneric and baseless; only that declaration gains semantic Object identity.
Source bases, generic imported bases, object signatures and bootstrap base facts now
agree. The driver exposes --object-library with an explicitly supplied native reference.
Runtime selection is independently explicit with --object-root. No arbitrary inheritance
capability was relaxed, and emission does not consult loader objects.

Compared with CLR, the compiler still runs on .NET but selects the guest's root from
native metadata rather than treating the host/bootstrap Object as authoritative. This
requires an explicit owner catalog and consistent symbol canonicalization; it does not
replace the .NET Reflection/Emit backend. Ordinary .NET defaults stay unchanged.

## Reproduce

Run verify_source_owned_orders.py with --compiler, --compiler-revision, --core, --runtime,
--audit (a successful full-owned-handle audit with System.runtime.neox), and a fresh
--output directory. Exact paths are in the evidence. The script compiles the checked-in
sample, verifies, executes, compares exact stdout and exit status, and tests rejected
missing/conflicting root selections without output publication.

The native load set verifies **2,431 IL functions**, maximum stack 7. Execution has
empty stderr and matches all expected lines, including collection mutation and shared
object identity. The C# imported-root probe checks object lookup, source/generic bases,
bootstrap base facts, missing/CLI/non-root/unselected/.NET owners and source/imported
conflicts. All **47** focused source-root, metadata-option and erased-value ownership
regressions pass. Compiler builds and whitespace checks pass.

## Remaining work

This is the broad native application gate under the permitted primitive core and
retained seed. Numbers is still the diagnostic aggregate's identity, not a proposed
production System assembly name. No complete class-library API execution, bootstrap-free
compiler, new .NET broad-sample run or release qualification is implied.

Carry root ownership into project/LSP catalogs and split the candidate core/Data/
Networking/Web groups with separate-library acceptance. Reuse this unchanged orders
consumer as the regression gate when replacing the aggregate. Existing focused .NET
checks preserve the target default; these optional native contracts are not an independent
.NET fix to backport.
