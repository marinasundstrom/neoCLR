using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

// Minimal compiler declarations only. None of these placeholder bodies may execute.
static class CoreDeclarations
{
    public const string Identity = "NeoCLR.CoreProbe";
    public static void Write(string path, bool includeConsole = true, bool stringParameter = true, bool unionProbe = false, bool collectionProbe = false)
    {
        var declarations = TargetSurface.Declarations(includeConsole, stringParameter);
        if (unionProbe)
            declarations = declarations.Replace("public static class Console {", "public static class Console { " + ProcessBindings.ConsoleDeclaration).Replace("public static class Math {",
                "public static class Math { " + DoubleBindings.MathDeclarations + " public static Result<int, OverflowError> Abs(int value) => default; public static Result<int, InvalidRangeError> Clamp(int value, int min, int max) => default;")
                + UnionDeclarations.Source + DelegateBindings.Declarations + ProcessBindings.Declarations + CalendarBindings.Declarations + PathBindings.Declarations + FileBindings.Declarations + ResultBindings.Declarations;
        if (collectionProbe) declarations += CollectionDeclarations.Source + ReflectionBindings.Declarations;
        var source = Source.Replace("public struct Double { }", unionProbe ? DoubleBindings.Declarations : "public struct Double { }").Replace("public struct Int32 { }", unionProbe ? Int32Bindings.Declarations : "public struct Int32 { }").Replace("public sealed class String { }", StringBindings.Declarations(unionProbe))
            .Replace("public static class Console { public static void WriteLine(string value) { } }", declarations)
            .Replace("// Union probe attribute", unionProbe ? "public sealed class UnionAttribute : System.Attribute { }" : "");
        if (unionProbe) source = PrimitiveBindings.Project(source).Replace("public struct Boolean { }", BooleanBindings.Declaration)
            .Replace("public abstract class Array {", "public abstract class Array { public static void ForEach<T>(T[] values, Func<T, PropagationUnit> action) { }");
        if (collectionProbe) source = source.Replace("public class Type { }", "");
        var compilation = CSharpCompilation.Create(Identity,
            [CSharpSyntaxTree.ParseText(source)], references: [],
            options: new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));
        using var stream = new MemoryStream();
        var result = compilation.Emit(stream, options: new Microsoft.CodeAnalysis.Emit.EmitOptions(metadataOnly: true, includePrivateMembers: false));
        if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        stream.Position = 0;
        if (unionProbe)
        {
            using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(stream);
            var module = image.MainModule;
            var unit = module.GetType("System.PropagationUnit");
            var targetVoid = new Mono.Cecil.TypeReference("System", "Void", module, module, true);
            Mono.Cecil.TypeReference Project(Mono.Cecil.TypeReference type)
            {
                if (type.FullName == unit.FullName) return targetVoid;
                if (type is Mono.Cecil.ByReferenceType byref) return new Mono.Cecil.ByReferenceType(Project(byref.ElementType));
                if (type is Mono.Cecil.GenericInstanceType generic)
                    for (var i = 0; i < generic.GenericArguments.Count; i++) generic.GenericArguments[i] = Project(generic.GenericArguments[i]);
                return type;
            }
            foreach (var type in module.Types)
            {
                foreach (var contract in type.Interfaces) contract.InterfaceType = Project(contract.InterfaceType);
                foreach (var method in type.Methods)
                {
                    method.ReturnType = Project(method.ReturnType);
                    foreach (var parameter in method.Parameters) parameter.ParameterType = Project(parameter.ParameterType);
                }
            }
            module.Types.Remove(unit);
            foreach (var method in module.Types.SelectMany(t => t.Methods).Where(m => m.HasBody))
            { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
            foreach (var reference in module.AssemblyReferences.ToArray())
            {
                if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                    throw new InvalidDataException("Core projection introduced an external type scope.");
                module.AssemblyReferences.Remove(reference);
            }
            image.Write(path);
        }
        else File.WriteAllBytes(path, stream.ToArray());
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
            public abstract class Array { public int Length => 0; }
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
            // Union probe attribute
            public sealed class ReferenceAssemblyAttribute : System.Attribute { }
            public sealed class CompilerGeneratedAttribute : System.Attribute { }
            public sealed class RefSafetyRulesAttribute : System.Attribute {
                public RefSafetyRulesAttribute(int version) { }
            }
        }
        """;
}
