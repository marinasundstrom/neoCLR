using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

static class SourceString
{
    internal static int Build(AssemblyBuilder core, string output)
    {
        var root = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../../.."));
        var sources = new[] { "String", "Array", "EquatableTo", "Disposable", "Collections/Iterator", "Collections/Iterable", "Collections/Collection", "Collections/Sequence", "Collections/MutableSequence", "StringComparison", "Propagatable", "Option", "Result", "Attribute", "Runtime/CompilerServices/UnionAttribute", "Text/Utf8SliceError" };
        var catalog = NeoClrReferenceCatalog.ReadNative(Path.Combine(output, "NativeCore.dll"), []);
        var options = CompilationOptions.NeoCLR.WithOutputKind(OutputKind.DynamicallyLinkedLibrary)
            .WithTargetCoreAssemblyName(core.Identity.Name).WithRuntimeTypeOfContract(null)
            .WithRuntimeUnitContract(new(core.Identity.Name, "System.Void"))
            .WithRuntimeFailureContract(new(core.Identity.Name))
            .WithRuntimeIterationContract(new("NativeString", "System.Collections.Iterable`1", "System.Collections.Iterator`1", ArraysImplementIterable: true, ArrayShapeTypeName: "System.Array`1"))
            .WithMetadataImportOptions(new MetadataImportOptions(core.Identity.Name, null, [SpecialType.System_String])
                .WithObjectAssemblyName(core.Identity.Name).WithNativeMetadata());
        var trees = sources.Select(s => { var path = Path.Combine(root, "runtime/raven/src/System", s + ".rvn"); return SyntaxTree.ParseText(File.ReadAllText(path), path: path); }).ToArray();
        var compilation = Compilation.Create("NativeString", trees, catalog.References.ToArray(), options);
        using var image = new MemoryStream();
        var emitted = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
            new(new("NativeString", new(1, 0, 0, 0)), core.Identity, catalog.Dependencies, primitiveImplementations: [PrimitiveType.String]))));
        foreach (var diagnostic in emitted.Diagnostics) Console.Error.WriteLine(diagnostic);
        if (!emitted.Success) return 1;
        File.WriteAllBytes(Path.Combine(output, "NativeString.dll"), image.ToArray());
        var consumerCatalog = NeoClrReferenceCatalog.ReadNative(Path.Combine(output, "NativeCore.dll"), [Path.Combine(output, "NativeString.dll")]);
        var consumerOptions = options.WithOutputKind(OutputKind.ConsoleApplication)
            .WithMetadataImportOptions(new MetadataImportOptions(core.Identity.Name, new Dictionary<SpecialType, string> { [SpecialType.System_String] = "NativeString" })
                .WithObjectAssemblyName(core.Identity.Name).WithNativeMetadata());
        foreach (var (name, file) in new[] {
            ("StringConsumer", "source-string-consumer.rvn"),
            ("StringObjectConsumer", "source-string-object-consumer.rvn"),
            ("StringFaultConsumer", "source-string-fault-consumer.rvn") })
        {
            var path = Path.Combine(root, "docs/experiments/native-core-bootstrap", file);
            var consumer = Compilation.Create(name, [SyntaxTree.ParseText(File.ReadAllText(path), path: path)], consumerCatalog.References.ToArray(), consumerOptions);
            using var executable = new MemoryStream();
            var result = consumer.Emit(executable, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
                new(new(name, new(1, 0, 0, 0)), core.Identity, consumerCatalog.Dependencies))));
            foreach (var diagnostic in result.Diagnostics) Console.Error.WriteLine(diagnostic);
            if (!result.Success) return 1;
            File.WriteAllBytes(Path.Combine(output, name + ".dll"), executable.ToArray());
        }
        return 0;
    }
}
