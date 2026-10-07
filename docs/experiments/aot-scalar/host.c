#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/* Scalar experimental ABI only; no managed handles or runtime dependency. */
#include "abi.h"

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: aot-scalar <int32>\n");
        return 2;
    }
    char *end;
    errno = 0;
    intmax_t value = strtoimax(argv[1], &end, 10);
    if (errno || end == argv[1] || *end || value < INT32_MIN || value > INT32_MAX) {
        fprintf(stderr, "invalid Int32 input\n");
        return 2;
    }
    int32_t result = 123456789;
    int32_t status = neoclr_entry_v2((int32_t)value, &result);
    if (status != NEOCLR_AOT_OK) {
        if (result != 123456789) {
            fprintf(stderr, "AOT failure modified the output\n");
            return 3;
        }
        fprintf(stderr, "%s\n", status == NEOCLR_AOT_DIVIDE_BY_ZERO ? "DivideByZero" :
                status == NEOCLR_AOT_ARITHMETIC_OVERFLOW ? "ArithmeticOverflow" :
                status == NEOCLR_AOT_RUNTIME_ERROR ? "RuntimeError" :
                status == NEOCLR_AOT_USER_FAULT ? "UserFault" : "UnknownFault");
        return 1;
    }
    printf("%" PRId32 "\n", result);
    return 0;
}
