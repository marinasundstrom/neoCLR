# Native memory through Raven

The Raven profile no longer exposes the old System.Array<T> native descriptor.
Array<T> now describes managed arrays. Native allocation uses the bounded
System.Runtime.InteropServices.NativeMemory API; there is no legacy alias.

See [generic arrays and the native migration](generic-managed-arrays.md) for the
contract, direct IL examples, Raven support limits and validation. The saved
`library-native-buffer.rvn` sample now checks allocation/release signatures only;
typed native access is demonstrated by `examples/native_memory.neoil` until Raven's
native integer and pointer conversions are supported. Do not use older published
native-descriptor samples against a newly generated target profile.
