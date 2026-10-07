#ifndef NEOCLR_AOT_SCALAR_ABI_H
#define NEOCLR_AOT_SCALAR_ABI_H
#include <stdint.h>

/* Experimental scalar ABI v2. These numbers are not Rust FaultCode ordinals. */
#define NEOCLR_AOT_OK 0
#define NEOCLR_AOT_DIVIDE_BY_ZERO 1
#define NEOCLR_AOT_ARITHMETIC_OVERFLOW 2
#define NEOCLR_AOT_RUNTIME_ERROR 3

/* result must point to writable, aligned Int32 storage. Only success writes it.
   Fault status terminates guest invocation; no exception crosses this boundary.
   Rebuild old scalar consumers: the v1 neoclr_entry symbol is not exported. */
extern int32_t neoclr_entry_v2(int32_t value, int32_t *result);
#endif
