#include "platform.h"
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <sys/utsname.h>
#endif
#ifdef __APPLE__
#include <sys/sysctl.h>
#include <TargetConditionals.h>
#endif
#ifdef __ANDROID__
#include <sys/system_properties.h>
#endif

static int copy_text(char *out, size_t capacity, const char *text) {
    size_t length = strlen(text);
    if (length >= capacity) return -1;
    memcpy(out, text, length + 1);
    return (int)length;
}
static const char *process_architecture(void) {
#if defined(__aarch64__) || defined(_M_ARM64)
    return "arm64";
#elif defined(__x86_64__) || defined(_M_X64)
    return "x64";
#elif defined(__i386__) || defined(_M_IX86)
    return "x86";
#elif defined(__arm__) || defined(_M_ARM)
    return "arm";
#elif defined(__riscv) && __riscv_xlen == 64
    return "riscv64";
#else
    return "unknown";
#endif
}
#ifndef _WIN32
static const char *normalize_architecture(const char *name) {
    if (!strcmp(name, "x86_64") || !strcmp(name, "amd64")) return "x64";
    if (!strcmp(name, "aarch64") || !strcmp(name, "arm64")) return "arm64";
    if (!strcmp(name, "i386") || !strcmp(name, "i686")) return "x86";
    if (!strncmp(name, "armv", 4)) return "arm";
    return name;
}

#endif

int neoclr_platform_info(int selector, char *out, size_t capacity) {
    if (!out || capacity < 1) return -1;
    out[0] = 0;
    if (selector == 0) {
#if defined(__ANDROID__)
        return copy_text(out, capacity, "android");
#elif defined(_WIN32)
        return copy_text(out, capacity, "windows");
#elif defined(__APPLE__) && TARGET_OS_IOS
        return copy_text(out, capacity, "ios");
#elif defined(__APPLE__) && TARGET_OS_OSX
        return copy_text(out, capacity, "macos");
#elif defined(__linux__)
        return copy_text(out, capacity, "linux");
#else
        return copy_text(out, capacity, "other");
#endif
    }
    /* Distribution metadata is optional. No guessing from the kernel version. */
    if (selector == 2 || selector == 3) return 0;
    if (selector == 4) return copy_text(out, capacity, process_architecture());
    if (selector == 6) {
#ifdef _WIN32
        return copy_text(out, capacity, "\r\n");
#else
        return copy_text(out, capacity, "\n");
#endif
    }
    if (selector == 5) {
#ifdef _WIN32
        SYSTEM_INFO info;
        GetNativeSystemInfo(&info);
        const char *name = "unknown";
        switch (info.wProcessorArchitecture) {
            case PROCESSOR_ARCHITECTURE_AMD64: name = "x64"; break;
            case PROCESSOR_ARCHITECTURE_INTEL: name = "x86"; break;
            case PROCESSOR_ARCHITECTURE_ARM: name = "arm"; break;
            case PROCESSOR_ARCHITECTURE_ARM64: name = "arm64"; break;
        }
        return copy_text(out, capacity, name);
#else
#ifdef __APPLE__
        int arm64 = 0;
        size_t length = sizeof(arm64);
        if (sysctlbyname("hw.optional.arm64", &arm64, &length, NULL, 0) == 0 && arm64)
            return copy_text(out, capacity, "arm64");
#endif
        struct utsname info;
        if (uname(&info)) return -1;
        return copy_text(out, capacity, normalize_architecture(info.machine));
#endif
    }
    if (selector == 1) {
#ifndef _WIN32
        struct utsname info;
        if (uname(&info)) return -1;
        return copy_text(out, capacity, info.release);
#else
        return -1;
#endif
    }
#ifdef __ANDROID__
    if (selector == 14) {
        char value[PROP_VALUE_MAX];
        if (__system_property_get("ro.build.version.sdk", value) <= 0) return -1;
        return copy_text(out, capacity, value);
    }
#endif
    if (selector >= 10 && selector <= 13) {
        unsigned parts[4] = {0, 0, 0, 0};
#ifdef _WIN32
        typedef LONG (WINAPI *version_fn)(OSVERSIONINFOW *);
        HMODULE module = GetModuleHandleW(L"ntdll.dll");
        version_fn get_version = module ? (version_fn)(void *)GetProcAddress(module, "RtlGetVersion") : NULL;
        OSVERSIONINFOW version;
        memset(&version, 0, sizeof(version));
        version.dwOSVersionInfoSize = sizeof(version);
        if (!get_version || get_version(&version) != 0) return -1;
        parts[0] = version.dwMajorVersion;
        parts[1] = version.dwMinorVersion;
        parts[2] = version.dwBuildNumber;
#elif defined(__APPLE__)
        char version[256];
        size_t length = sizeof(version);
        if (sysctlbyname("kern.osproductversion", version, &length, NULL, 0) || !length || length > sizeof(version)) return -1;
        version[sizeof(version) - 1] = 0;
        if (sscanf(version, "%u.%u.%u", &parts[0], &parts[1], &parts[2]) < 2) return -1;
#else
        return -1;
#endif
        int written = snprintf(out, capacity, "%u", parts[selector - 10]);
        return written < 0 || (size_t)written >= capacity ? -1 : written;
    }
    return -1;
}
