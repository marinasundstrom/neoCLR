#include "string-unicode.h"
#include "native-gc.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#define TEXT(name, value) static const struct { uint64_t length; unsigned char bytes[sizeof(value)]; } name = { sizeof(value)-1, value }
#define T(name) ((const neoclr_aot_text *)&name)
TEXT(unicode, "e\xcc\x81\xf0\x9f\x99\x82\0");
TEXT(invalid, "\xff");
TEXT(word, "Stra\xc3\x9f" "e");
TEXT(upper, "STRASSE");
TEXT(empty, "");
TEXT(one, "one");
static uint64_t storage[262144];
static uint64_t other_storage[1024];
int main(void) {
    neoclr_aot_context context = { .text = { (unsigned char *)storage, sizeof(storage), 0 } };
    neoclr_aot_context other = { .text = { (unsigned char *)other_storage, sizeof(other_storage), 0 } };
    const neoclr_aot_text *result = T(empty);
    int32_t count = -1;
    assert(neoclr_string_grapheme_count_v1(T(unicode), &count) == 0 && count == 3);
    assert(neoclr_string_grapheme_count_v1(T(invalid), &count) == 3 && count == 3);
    assert(neoclr_string_grapheme_at_v1(T(unicode), -1, &context.text, &result) == 8 && result == T(empty));
    assert(neoclr_string_grapheme_at_v1(T(unicode), 3, &context.text, &result) == 8 && result == T(empty));
    assert(neoclr_string_upper_v1(T(word), &context.text, &result) == 0);
    assert(result->length == upper.length && !memcmp(result->bytes, upper.bytes, upper.length));
    const void *vector = NULL;
    assert(neoclr_string_graphemes_v1(T(unicode), &context.text, &vector) == 0);
    assert(neoclr_string_from_chars_v1(vector, &context.text, &result) == 0);
    assert(result->length == unicode.length && !memcmp(result->bytes, unicode.bytes, unicode.length));
    /* Keep the first dynamic owner, despite collecting without guest roots. */
    const neoclr_aot_text *first = result;
    assert(neoclr_string_intern_v1(first, &context.text, &result) == 0 && result == first);
    assert(neoclr_gc_collect_v1(&context, NULL) == 0);
    assert(neoclr_string_intern_v1(T(unicode), &context.text, &result) == 0 && result == first);
    assert(!memcmp(result->bytes, unicode.bytes, unicode.length));
    assert(neoclr_string_intern_v1(T(unicode), &other.text, &result) == 0 && result == T(unicode));
    /* Entry reset drops pool ownership. */
    context.text.used = 0;
    assert(neoclr_string_intern_v1(T(unicode), &context.text, &result) == 0 && result == T(unicode));
    context.text.used = 0;
    static struct { uint64_t length; unsigned char bytes[16]; } keys[4097];
    for (unsigned i = 0; i < 4097; ++i) {
        keys[i].length = (uint64_t)snprintf((char *)keys[i].bytes, 16, "key-%u", i);
        int32_t status = neoclr_string_intern_v1(T(keys[i]), &context.text, &result);
        assert(status == (i == 4096 ? 10 : 0));
        assert(result == T(keys[i == 4096 ? 4095 : i]));
    }
    assert(neoclr_gc_collect_v1(&context, NULL) == 0);
    assert(neoclr_string_intern_v1(T(keys[0]), &context.text, &result) == 0 && result == T(keys[0]));
    context.text.used = 0;
    static struct { uint64_t length; unsigned char bytes[1048576]; } large = {1048576, {0}};
    assert(neoclr_string_intern_v1(T(large), &context.text, &result) == 0);
    assert(neoclr_string_intern_v1(T(one), &context.text, &result) == 10 && result == T(large));
    assert(neoclr_string_intern_v1(T(large), &context.text, &result) == 0);
    neoclr_aot_text_arena tiny = { (unsigned char *)other_storage, 8, 0 };
    assert(neoclr_string_upper_v1(T(word), &tiny, &result) == 5 && result == T(large));
    puts("native Unicode and intern lifetime/quota checks passed");
}
