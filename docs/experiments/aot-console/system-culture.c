/* Private host-preference adapter matching interpreter sys-locale 0.3.2.
 * Capture one preference per call. Empty means unavailable; Raven resolves fallback.
 * The helper allocates from the text arena, only rooted entry points collect; none call guest code. */
#include "text-arena.h"
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#elif defined(__APPLE__)
#include <CoreFoundation/CoreFoundation.h>
#else
#error "Host culture adapter requires an explicit platform implementation"
#endif

extern int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *, uint64_t, neoclr_aot_text **);
static int32_t empty(neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    neoclr_aot_text *text;
    int32_t status = (rooted ? neoclr_text_allocate_rooted_values_v1(arena, 0, NULL, 0, &text) : neoclr_text_allocate_v1(arena, 0, &text));
    if (!status) *output = text;
    return status;
}

static int32_t system_culture_name(neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!arena || !output) return 3;
#ifdef _WIN32
    ULONG count = 0, length = 0;
    if (!GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &count, NULL, &length) || !count || !length)
        return empty(arena, output, rooted);
    if ((uint64_t)length > SIZE_MAX / sizeof(WCHAR)) return 5;
    WCHAR *buffer = (WCHAR *)malloc((size_t)length * sizeof(WCHAR));
    if (!buffer) return 5;
    ULONG capacity = length;
    if (!GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &count, buffer, &length) || length > capacity) {
        free(buffer);
        return empty(arena, output, rooted);
    }
    ULONG end = 0;
    while (end < length && buffer[end]) ++end;
    if (!end || end == length || end > INT_MAX) {
        free(buffer);
        return empty(arena, output, rooted);
    }
    int bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, buffer, (int)end, NULL, 0, NULL, NULL);
    if (!bytes) {
        free(buffer);
        return empty(arena, output, rooted);
    }
    neoclr_aot_text *text;
    int32_t status = (rooted ? neoclr_text_allocate_rooted_values_v1(arena, (uint64_t)bytes, NULL, 0, &text) : neoclr_text_allocate_v1(arena, (uint64_t)bytes, &text));
    if (!status && WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, buffer, (int)end,
        (char *)text->bytes, bytes, NULL, NULL) != bytes) status = 3;
    free(buffer);
    if (!status) *output = text;
    return status;
#else
    CFArrayRef languages = CFLocaleCopyPreferredLanguages();
    if (!languages) return empty(arena, output, rooted);
    if (!CFArrayGetCount(languages)) {
        CFRelease(languages);
        return empty(arena, output, rooted);
    }
    CFStringRef name = (CFStringRef)CFArrayGetValueAtIndex(languages, 0);
    CFIndex units = CFStringGetLength(name), bytes = 0;
    CFRange range = CFRangeMake(0, units);
    CFIndex converted = CFStringGetBytes(name, range, kCFStringEncodingUTF8, 0, false, NULL, 0, &bytes);
    if (converted != units || bytes <= 0) {
        CFRelease(languages);
        return empty(arena, output, rooted);
    }
    neoclr_aot_text *text;
    int32_t status = (rooted ? neoclr_text_allocate_rooted_values_v1(arena, (uint64_t)bytes, NULL, 0, &text) : neoclr_text_allocate_v1(arena, (uint64_t)bytes, &text));
    if (!status) {
        CFIndex written = 0;
        converted = CFStringGetBytes(name, range, kCFStringEncodingUTF8, 0, false, text->bytes, bytes, &written);
        if (converted != units || written != bytes) status = 3;
    }
    CFRelease(languages);
    if (!status) *output = text;
    return status;
#endif
}

int32_t neoclr_system_culture_name_v1(neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return system_culture_name(arena, output, 0);
}

int32_t neoclr_system_culture_name_rooted_v1(neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return system_culture_name(arena, output, 1);
}
