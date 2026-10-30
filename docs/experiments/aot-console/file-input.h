#ifndef NEOCLR_AOT_FILE_INPUT_H
#define NEOCLR_AOT_FILE_INPUT_H
#include "text-arena.h"
/* Bounded blocking regular-file input (POSIX and Windows). Returns native fault 0/3/5, publishing only on 0.
 * Output: erased String or Byte status 1 invalid limit, 2 invalid path, 3 missing,
 * 4 denied, 5 nonregular, 6 read failed, 7 too large, 8 invalid UTF-8.
 * Text is copied to managed arena storage; host buffers/handles are released. */
int32_t neoclr_file_read_utf8_v1(const neoclr_aot_text *path, int32_t limit,
    neoclr_aot_text_arena *arena, void *output);
/* Collects only during final decoding, after closing the descriptor. */
int32_t neoclr_file_read_utf8_rooted_v1(const neoclr_aot_text *, int32_t,
    neoclr_aot_text_arena *, void *);
#endif
