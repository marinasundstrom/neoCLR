# Native worker ownership diagnostic — 2026-10-23

This POSIX probe exercises the canonical native collector/root code. It demonstrates
why dispatching an existing rooted callback to an arbitrary worker is not yet a
supported shared-heap operation.

The parent allocates/roots a payload, then waits for a pthread worker. The worker's
attempt to read/release the parent handle is rejected: lookup is owner-local. It
separately allocates, roots, collects and reclaims its own payload. After joining,
the parent's root still preserves its payload, and final release allows reclamation.

There is no concurrent shared-arena access: the parent is quiescent until join;
the worker never dereferences parent payloads. This tests the present boundary,
not a permanent affinity contract, parallel HTTP, or shared-heap race freedom.

Run validate.py --output FRESH with the matching toolchain. On this macOS host,
set DEVELOPER_DIR to /Applications/Xcode.app/Contents/Developer and SDKROOT to
/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk.

[Validation](validation.json) passes optimized and fail-fast UBSan builds/executions.
The combined [ASan/UBSan execution](asan-incomplete.json) timed out after 60 seconds;
its cause is unresolved and no ASan success is claimed. The optional
--address-sanitizer flag permits reproducing that separate attempt. Initial
default-SDK attempts failed to link: the older linker could not read CommandLineTools
macOS 27 SDK architecture metadata. The matching Xcode 26.2 SDK resolves this;
no runtime source change was needed. Windows and TSan were not run.

See the [worker investigation](../../design/native-http-worker-threads.md) for
framework comparisons, prerequisites and the proposed first implementation slice.
