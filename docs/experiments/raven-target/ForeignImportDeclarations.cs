static class ForeignImportDeclarations
{
    internal const string Source = """
        namespace Runtime.InteropServices {
            public enum NativeCallingConvention { Cdecl = 0, System = 1 }
            public enum NativeStringMarshalling { None = 0, Utf8 = 1, Utf16 = 2 }
            public enum NativeType { Utf8 = 0, Utf16 = 1, Bool8 = 2, Bool32 = 3 }
            [System.AttributeUsage(System.AttributeTargets.Method, Inherited = false)]
            public class NativeImportAttribute : System.Attribute {
                public NativeImportAttribute(string libraryName) { }
                public string LibraryName => default;
                public string EntryPoint { get; set; }
                public NativeCallingConvention CallingConvention { get; set; }
                public NativeStringMarshalling StringMarshalling { get; set; }
            }
            [System.AttributeUsage(System.AttributeTargets.Method, Inherited = false)]
            public class NativeExportAttribute : System.Attribute {
                public NativeExportAttribute() { }
                public NativeExportAttribute(string entryPoint) { }
                public NativeExportAttribute(string entryPoint, NativeCallingConvention callingConvention) { }
                public string EntryPoint => default;
                public NativeCallingConvention CallingConvention => default;
            }
            [System.AttributeUsage(System.AttributeTargets.Parameter | System.AttributeTargets.ReturnValue, Inherited = false)]
            public class MarshalAsAttribute : System.Attribute {
                public MarshalAsAttribute(NativeType nativeType) { }
                public NativeType Value => default;
            }
        }
        """;
}
