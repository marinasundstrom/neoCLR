---
uid: N:System.Runtime.InteropServices.NativeMemory
---
## Unmanaged allocation

Alloc reserves unmanaged memory by byte count or checked element-count multiplication;
Free releases an allocation. These unsafe module functions are separate from
managed objects and arrays.

Callers own allocation lifetime and pointer validity. Consult each overload for
faults and supported pointer operations; managed garbage collection does not
replace the required Free call.
