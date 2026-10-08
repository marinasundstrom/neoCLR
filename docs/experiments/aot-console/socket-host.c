#include "socket-listener.h"
#include "root-probe.h"
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
int main(void) {
    uint64_t storage[8193];
    storage[8192] = UINT64_C(0x1122334455667788);
    neoclr_aot_context context = {.text = {(unsigned char *)storage, 65536, 0}};
    neoclr_socket_scope scope;
    if (neoclr_socket_scope_enter_v1(&scope, &context)) return 2;
    int32_t result = -99;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    /* Host cleanup is unconditional, including guest faults and forgotten Close. */
    int descriptors[64], count = 0;
    for (unsigned i = 0; i < 64; i++) if (scope.slots[i].id) descriptors[count++] = scope.slots[i].descriptor;
    int32_t cleanup = neoclr_socket_scope_leave_v1(&scope);
    if (cleanup || neoclr_root_probe_head_v1() || storage[8192] != UINT64_C(0x1122334455667788)) return 2;
    for (int i = 0; i < count; i++) { errno = 0; if (fcntl(descriptors[i], F_GETFD) != -1 || errno != EBADF) return 2; }
    fprintf(stderr, "SOCKET_CLEANUP %d\n", count);
    if (status) {
        if (result != -99 || context.fault.code != (uint32_t)status) return 2;
        neoclr_aot_render_fault(stderr, &context.fault);
        return 1;
    }
    return result;
}
