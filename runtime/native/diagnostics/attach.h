#ifndef NEOCLR_DIAGNOSTIC_ATTACH_H
#define NEOCLR_DIAGNOSTIC_ATTACH_H
/* Private POSIX proof: called only after collection by the runtime owner thread.
 * Copies scalar diagnostics; never exposes roots or accepts execution commands. */
void neoclr_diagnostics_publish_v1(void);
#endif
