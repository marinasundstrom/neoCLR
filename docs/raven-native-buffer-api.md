# Native memory through Raven

The Raven profile no longer exposes the old System.Array<T> native descriptor.
Array<T> now describes managed arrays. Native allocation uses the bounded
System.Runtime.InteropServices.NativeMemory API; there is no legacy alias.

See [generic arrays and the native migration](generic-managed-arrays.md) for the
contract, direct IL examples, Raven support limits and validation. The saved
`library-native-buffer.rvn` sample now checks allocation/release signatures only;
typed native access is demonstrated by `examples/preview/native_memory.neoil` until Raven's
native integer and pointer conversions are supported. Do not use older published
native-descriptor samples against a newly generated target profile.

The API-preserving source port authors both Alloc overloads and Free in Raven.
A bootstrap-only NativeAllocation contract supplies allocation, release and checked
unsigned native multiplication; its CIL stubs never execute and it is absent from
consumer metadata. Pointer identities, zero-size behavior, overflow and explicit
lifetimes retain the [existing .NET comparison](generic-managed-arrays.md).
No new Runtime Contract setting or pointer conversion capability is introduced.
Five source admission cases, 28 native/pointer tests and the saved Raven allocation
sample pass on 2026-09-19.
