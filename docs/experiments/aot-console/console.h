#ifndef NEOCLR_AOT_CONSOLE_H
#define NEOCLR_AOT_CONSOLE_H
#include <stdint.h>
/* Experimental linked service ABI, independent of private System.Value layout.
 * 0..255: byte, -1: EOF, -2: unavailable, -3: read failed.
 * Any other result is an invalid service status, diagnosed by managed Console.
 * Hosts without input may supply a replacement implementation returning -2.
 */
int32_t neoclr_console_read_byte_v1(void);
#endif
