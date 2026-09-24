using System.Reflection;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

// Compile the selected library source through Raven's ordinary frontend. The temporary
// seed is never shipped: both bootstrap and consumer cores receive the same projection.
static class SourceUnionReferences
{
    public static void Project(string corePath)
    {
        var directory = Path.Combine(Path.GetTempPath(), "neoclr-source-union-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        try
        {
            var seed = Path.Combine(directory, CoreDeclarations.Identity + ".dll");
            File.Copy(corePath, seed);
            using var sourceStream = typeof(SourceUnionReferences).Assembly.GetManifestResourceStream("NeoCLR.SocketError.rvn")
                ?? throw new InvalidDataException("Missing SocketError reference source.");
            using var reader = new StreamReader(sourceStream);
            var options = new CompilationOptions(OutputKind.DynamicallyLinkedLibrary,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity))
                .WithTargetCoreAssemblyName(CoreDeclarations.Identity)
                .WithGraphemeChar(true)
                .WithRuntimeUnitContract(new RuntimeUnitContract(CoreDeclarations.Identity, "System.Void"));
            var compilation = Compilation.Create("SourceUnionReferences", [SyntaxTree.ParseText(reader.ReadToEnd())],
                [MetadataReference.CreateFromFile(seed)], options);
            var source = Path.Combine(directory, "SourceUnionReferences.dll");
            using (var output = File.Create(source))
            {
                var result = compilation.Emit(output, null, new EmitOptions(AssemblyName.GetAssemblyName(seed)));
                if (!result.Success) throw new InvalidDataException(string.Join("\n", result.Diagnostics));
            }
            StandardUnionReference.WriteReference(source, seed, "System.Networking.Sockets.SocketError", corePath);
        }
        finally { Directory.Delete(directory, recursive: true); }
    }
}
