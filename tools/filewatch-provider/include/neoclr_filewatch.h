#ifndef NEOCLR_FILEWATCH_H
#define NEOCLR_FILEWATCH_H
#include <stdint.h>
/* No retained input/output pointers. Native code never invokes guest callbacks.
 * Positive process-unique handles; negative values encode recoverable errors. */
int64_t neoclr_filewatch_open_v1(const char *path, const char *filter, int32_t recursive);
int32_t neoclr_filewatch_read_v1(int64_t handle, unsigned char *output, int32_t capacity);
void neoclr_filewatch_close_v1(int64_t handle);
#endif
