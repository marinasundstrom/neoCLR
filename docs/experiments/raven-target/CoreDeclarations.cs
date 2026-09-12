using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

// Minimal compiler declarations only. None of these placeholder bodies may execute.
static class CoreDeclarations
{
    public const string Identity = "NeoCLR.CoreProbe";
    public static void Write(string path, bool includeConsole = true, bool stringParameter = true)
    {
        var compilation = CSharpCompilation.Create(Identity,
            [CSharpSyntaxTree.ParseText(includeConsole
                ? (stringParameter ? Source : Source.Replace("WriteLine(string value)", "WriteLine(int value)"))
                : Source.Replace("public static class Console { public static void WriteLine(string value) { } }", ""))], references: [],
            options: new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));
        using var stream = File.Create(path);
        var result = compilation.Emit(stream, options: new Microsoft.CodeAnalysis.Emit.EmitOptions(metadataOnly: true, includePrivateMembers: false));
        if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
    }

    public static string[] ReadDeclaredTypes(string path)
    {
        using var stream = File.OpenRead(path);
        using var pe = new PEReader(stream);
        var reader = pe.GetMetadataReader();
        if (reader.AssemblyReferences.Count != 0)
            throw new Exception("Core declarations depend on external assemblies.");
        return reader.TypeDefinitions.Select(handle => reader.GetTypeDefinition(handle))
            .Select(type => reader.GetString(type.Namespace) + "." + reader.GetString(type.Name)).Order().ToArray();
    }

    const string Source = """
        [assembly: System.Runtime.CompilerServices.ReferenceAssembly]
        namespace System {
            public class Object {
                public virtual bool Equals(object other) => false;
                public virtual int GetHashCode() => 0;
                public virtual string ToString() => "";
            }
            public abstract class ValueType { }
            public abstract class Enum : ValueType { }
            public struct Void { }
            public struct Boolean { }
            public struct Char { }
            public struct SByte { }
            public struct Byte { }
            public struct Int16 { }
            public struct UInt16 { }
            public struct Int32 { }
            public struct UInt32 { }
            public struct Int64 { }
            public struct UInt64 { }
            public struct Single { }
            public struct Double { }
            public struct IntPtr { }
            public struct UIntPtr { }
            public sealed class String { }
            public abstract class Array { }
            public class Type { }
            public class Attribute { }
            public enum AttributeTargets { All = 32767 }
            public sealed class AttributeUsageAttribute : Attribute {
                public AttributeUsageAttribute(AttributeTargets targets) { }
                public bool AllowMultiple { get; set; }
                public bool Inherited { get; set; }
            }
            public static class Console { public static void WriteLine(string value) { } }
        }
        namespace System.Runtime.CompilerServices {
            public sealed class ReferenceAssemblyAttribute : System.Attribute { }
            public sealed class CompilerGeneratedAttribute : System.Attribute { }
            public sealed class RefSafetyRulesAttribute : System.Attribute {
                public RefSafetyRulesAttribute(int version) { }
            }
        }
        """;
}
