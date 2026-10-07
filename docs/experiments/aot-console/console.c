#include "console.h"
#include <stdio.h>

int32_t neoclr_console_read_byte_v1(void) {
    int value = fgetc(stdin);
    if (value != EOF) return (unsigned char)value;
    return ferror(stdin) ? -3 : -1;
}
