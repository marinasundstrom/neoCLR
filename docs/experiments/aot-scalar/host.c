#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/* Scalar experimental ABI only; no managed handles or runtime dependency. */
extern int32_t neoclr_entry(int32_t value);

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
    printf("%" PRId32 "\n", neoclr_entry((int32_t)value));
    return 0;
}
