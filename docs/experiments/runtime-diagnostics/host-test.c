#include "../aot-console/native-gc.h"
#include <stdio.h>
int main(void) {
    uint64_t memory[128] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    int command;
    puts("ready"); fflush(stdout);
    while ((command = getchar()) != EOF && command != 'q') {
        if (command != 'c') continue;
        void *value;
        if (neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_TEXT, &value) ||
            neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 1;
        puts("collected"); fflush(stdout);
    }
    return 0;
}
