using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

// Minimal compiler declarations only. None of these placeholder bodies may execute.
static class CoreDeclarations
{
    public const string Identity = "NeoCLR.CoreProbe";
    public static void Write(string path, bool includeConsole = true, bool stringParameter = true, bool unionProbe = false, bool collectionProbe = false, bool libraryBootstrap = false)
    {
        var declarations = TargetSurface.Declarations(includeConsole, stringParameter);
        if (unionProbe)
            declarations = declarations.Replace("public static class Console {", "public static class Console { " + ProcessBindings.ConsoleDeclaration).Replace("public static class Math {",
                "public static class Math { " + DoubleBindings.MathDeclarations + " public static Result<int, OverflowError> Abs(int value) => default; public static Result<int, InvalidRangeError> Clamp(int value, int min, int max) => default;")
                + "public static class FaultFunctions { public static void Fault(string message) { } }" + UnionDeclarations.Source + DelegateBindings.Declarations + ProcessBindings.Declarations + CalendarBindings.Declarations + HashCodeBindings.Declarations + PathBindings.Declarations + FileBindings.Declarations + StorageItemBindings.Declarations + StreamBindings.Declarations + ReaderBindings.Declarations + StorageProviderBindings.Declarations + FileSystemBindings.Declarations + ResultBindings.Declarations;
        if (collectionProbe) declarations += SocketBindings.Declarations + WorkerBindings.Declarations + TaskBindings.ForReference(libraryBootstrap) + Utf8Bindings.Declarations + UnicodeScalarBindings.Declarations + QueryBindings.Declarations + OutcomeOperatorBindings.Declarations + CollectionDeclarations.Source + ReflectionBindings.Declarations + ReflectionBindings.ProviderDeclarations + NativeMemoryBindings.Declaration + InterfaceBindings.Declarations;
        if (libraryBootstrap) declarations += RuntimeFailureBindings.Declarations + NativeAllocationBindings.Declarations + ParameterSnapshotBindings.Declarations + CheckedStorageBindings.Declarations + RuntimeServiceBindings.Declarations + ValueStorageBindings.Declarations;
        var source = Source.Replace("public struct Double { }", unionProbe ? DoubleBindings.Declarations : "public struct Double { }").Replace("public struct Int32 { }", unionProbe ? Int32Bindings.Declarations : "public struct Int32 { }").Replace("public sealed class String { }", StringBindings.Declarations(unionProbe, collectionProbe))
            .Replace("public static class Console { public static void WriteLine(string value) { } }", declarations)
            .Replace("// Union probe attribute", unionProbe ? "public sealed class UnionAttribute : System.Attribute { }" : "");
        if (unionProbe) source = PrimitiveBindings.Project(source).Replace("public struct Boolean { }", BooleanBindings.Declaration);
        if (collectionProbe) source = InterfaceBindings.Project(source.Replace("public class Type { }", "")
            .Replace("public abstract class Object {", "public abstract class Object { public System.Introspection.TypeInfo GetType() => default;"));
        var compilation = CSharpCompilation.Create(Identity,
            [CSharpSyntaxTree.ParseText(source)], references: [],
            options: new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary, allowUnsafe: true));
        using var stream = new MemoryStream();
        var result = compilation.Emit(stream, options: new Microsoft.CodeAnalysis.Emit.EmitOptions(metadataOnly: true, includePrivateMembers: false));
        if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        stream.Position = 0;
        if (unionProbe)
        {
            using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(stream);
            var module = image.MainModule;
            CalendarBindings.ProjectLayout(module);
            HashCodeBindings.ProjectLayout(module);
            if (collectionProbe) {
                var stringFactory = module.GetType("System.String").Methods.Single(m => m.Name == "CreateFromCharacters");
                stringFactory.IsPublic = false;
                stringFactory.IsAssembly = true;
            }
            if (libraryBootstrap) { PrimitiveLibrary.Project(module); OpaqueLibrary.Project(module); EmptyLibrary.Project(module); ErrorCarrierLibrary.Project(module); GenericUnionLibrary.Project(module); }
            if (libraryBootstrap && collectionProbe)
            {
                ArrayLibrary.Project(module);
                var local = module.GetType("System.LocalDateTime");
                var localFactory = new Mono.Cecil.MethodDefinition("FromUnixTimeTicks",
                    Mono.Cecil.MethodAttributes.Assembly | Mono.Cecil.MethodAttributes.Static | Mono.Cecil.MethodAttributes.HideBySig, local);
                localFactory.Parameters.Add(new Mono.Cecil.ParameterDefinition("ticks", Mono.Cecil.ParameterAttributes.None,
                    module.GetType("System.Instant").Methods.Single(m => m.Name == "FromUnixTimeTicks").Parameters[0].ParameterType));
                local.Methods.Add(localFactory);
                var info = module.GetType("System.Introspection.RuntimeTypeInfo");
                if (!info.Methods.Any(m => m.Name == "FromHandle"))
                {
                    var factory = new Mono.Cecil.MethodDefinition("FromHandle",
                        Mono.Cecil.MethodAttributes.Assembly | Mono.Cecil.MethodAttributes.Static | Mono.Cecil.MethodAttributes.HideBySig, module.GetType("System.Introspection.TypeInfo"));
                    factory.Parameters.Add(new Mono.Cecil.ParameterDefinition("handle", Mono.Cecil.ParameterAttributes.None,
                        module.GetType("System.RuntimeTypeHandle")));
                    info.Methods.Add(factory);
                }
            }
            if (collectionProbe) { IntrospectionHierarchy.Project(module); TaskBindings.Project(module); WorkerBindings.Project(module); SocketBindings.Project(module); ReaderBindings.Project(module); }
            StorageHierarchy.Project(module);
            NamespaceFunctions.ProjectMath(module);
            NamespaceFunctions.ProjectFault(module);
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
        #nullable enable annotations
        [assembly: System.Runtime.CompilerServices.ReferenceAssembly]
        namespace System {
            public abstract class Object {
                public static bool ReferenceEquals(object? left, object? right) => false;
                public virtual bool Equals(object? other) => false;
                public virtual int GetHashCode() => 0;
                public virtual string ToString() => "";
            }
        #nullable restore annotations
            public abstract class ValueType { }
            public abstract class Enum : ValueType { }
            public struct Void { }
            public struct Value { }
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
            // Metadata-only dependency of Raven's extension marker stub. Throwing
            // or constructing it in executable code remains unsupported.
            public class NotImplementedException { }
            public sealed class FlagsAttribute : Attribute { }
            public enum AttributeTargets { All = 32767 }
            public sealed class AttributeUsageAttribute : Attribute {
                public AttributeUsageAttribute(AttributeTargets targets) { }
                public bool AllowMultiple { get; set; }
                public bool Inherited { get; set; }
            }
            public static class Console { public static void WriteLine(string value) { } }
        }
        namespace System.Runtime.CompilerServices {
            public static class IsExternalInit { }
            // Union probe attribute
            public sealed class ReferenceAssemblyAttribute : System.Attribute { }
            // Raven namespace-member metadata marker; not an executable runtime API.
            public sealed class TopLevelAttribute : System.Attribute { }
            public sealed class CompilerGeneratedAttribute : System.Attribute { }
            public sealed class ExtensionAttribute : System.Attribute { }
            public sealed class RefSafetyRulesAttribute : System.Attribute {
                public RefSafetyRulesAttribute(int version) { }
            }
        }
        """;
}
