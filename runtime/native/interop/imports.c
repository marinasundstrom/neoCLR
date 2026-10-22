/* Development macOS C-ABI import resolver. Successful entries own the library
   until process exit. Never hold the cache mutex while running loader code. */
#include <dlfcn.h>
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define IMPORT_CAPACITY 256
struct import_entry { char *library; char *symbol; void *owner; void *address; };
static struct import_entry entries[IMPORT_CAPACITY];
static size_t count;
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;

static void *lookup(const char *library, const char *symbol) {
    for (size_t i = 0; i < count; ++i)
        if (!strcmp(entries[i].library, library) && !strcmp(entries[i].symbol, symbol))
            return entries[i].address;
    return NULL;
}

/* Private runtime status ABI: 0 success, 3 RuntimeError. Output is unchanged on
   failure. Library/symbol are trusted NUL-terminated UTF-8 metadata strings. */
int32_t neoclr_native_import_resolve_v1(const char *library, const char *symbol, void **output) {
    if (!library || !*library || !symbol || !*symbol || !output) return 3;
    pthread_mutex_lock(&lock);
    void *address = lookup(library, symbol);
    pthread_mutex_unlock(&lock);
    if (address) { *output = address; return 0; }

    void *owner = dlopen(library, RTLD_NOW | RTLD_LOCAL);
    if (!owner) return 3;
    dlerror();
    address = dlsym(owner, symbol);
    if (dlerror() || !address) { dlclose(owner); return 3; }
    char *library_copy = strdup(library);
    char *symbol_copy = strdup(symbol);
    if (!library_copy || !symbol_copy) {
        free(library_copy); free(symbol_copy); dlclose(owner); return 3;
    }
    pthread_mutex_lock(&lock);
    void *existing = lookup(library, symbol);
    if (existing || count == IMPORT_CAPACITY) {
        pthread_mutex_unlock(&lock);
        free(library_copy); free(symbol_copy); dlclose(owner);
        if (!existing) return 3;
        *output = existing;
        return 0;
    }
    entries[count++] = (struct import_entry){library_copy, symbol_copy, owner, address};
    pthread_mutex_unlock(&lock);
    *output = address;
    return 0;
}
