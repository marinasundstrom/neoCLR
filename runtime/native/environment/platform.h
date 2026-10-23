#ifndef NEOCLR_PLATFORM_INFO_H
#define NEOCLR_PLATFORM_INFO_H
#include <stddef.h>
/* Private selectors: 0 OS, 1 kernel release, 2/3 optional distribution id/version,
 * 4 process architecture, 5 OS architecture, 6 newline, 10..13 version components,
 * 14 Android API level. Returns byte length or -1; always NUL-terminates success. */
int neoclr_platform_info(int selector, char *output, size_t capacity);
#endif
