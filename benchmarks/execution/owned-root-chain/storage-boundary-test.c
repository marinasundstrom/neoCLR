/* Exercise both sides of the small/large validator threshold. */
#include "../../../runtime/native/gc/root-probe.c"
#include <assert.h>
int main(int argc, char **argv) {
    assert(argc == 4);
    unsigned count = (unsigned)strtoul(argv[1], NULL, 10);
    unsigned index = (unsigned)strtoul(argv[2], NULL, 10);
    unsigned fault = (unsigned)strtoul(argv[3], NULL, 10);
    assert(count <= 64 && (!fault || index < count));
    uint64_t values[64] = {0};
    neoclr_probe_storage slots[64];
    for (unsigned i=0; i<64; i++)
        slots[i]=(neoclr_probe_storage){&values[i], i%2 ? 4 : 8, i%3};
    if (fault == 1) slots[index].address = NULL;
    if (fault == 2) slots[index].read_bytes = 3;
    if (fault == 3 || fault == 4) slots[index].flags = 3;
    validate_storage(slots, count, fault != 4);
    return 0;
}
