#include "http-workers.h"
#include <stdint.h>
extern int32_t neoclr_entry_v4(int32_t, int32_t *, void *);
extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);
typedef struct { int argc; const char **argv; } arguments;
static int invoke(void *state) {
    arguments *args=state;
    return neoclr_jit_run_v1((const void *)neoclr_entry_v4,0,args->argc,args->argv);
}
int main(int argc,const char **argv) {
    arguments args={argc,argv};
    return neoclr_http_workers_run(OWNERS,invoke,&args);
}
