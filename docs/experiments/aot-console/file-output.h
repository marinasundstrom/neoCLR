#ifndef NEOCLR_AOT_FILE_OUTPUT_H
#define NEOCLR_AOT_FILE_OUTPUT_H
#include "text-arena.h"
/* Experimental blocking POSIX file output. Borrowed text is never retained.
 * Returns native fault status 0/3/5; writes output only on status 0.
 * Output is the managed WriteAllText protocol: 0 success, 1 invalid limit,
 * 2 invalid path, 3 missing path, 4 access denied, 5 nonregular file,
 * 6 write failure, 7 too large. Preflight errors never open/truncate the file.
 * Handles and temporary path storage are released before returning.
 */
int32_t neoclr_file_write_utf8_v1(const neoclr_aot_text *path,
    const neoclr_aot_text *text, int32_t limit, int32_t *output);
#endif
