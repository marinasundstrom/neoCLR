using System.Reflection;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

// Compile the selected library source through Raven's ordinary frontend. The temporary
// seed is never shipped: both bootstrap and consumer cores receive the same projection.
static class SourceUnionReferences
{
    static readonly string[] Owners = ["System.Networking.IPAddressError", "System.Networking.Sockets.SocketError", "System.Networking.DnsError", "System.UriError",
        "System.Storage.StorageLookupError",
        "System.IO.TextReadError",
        "System.IO.StreamError",
        "System.Storage.FileReadError",
        "System.Storage.FileWriteError",
        "System.ConsoleReadError",
        "System.Text.Utf8SliceError",
        "System.Int32ParseError",
        "System.Linq.SingleError",
        "System.IntegerDivisionError", "System.Web.Http.HttpError"];

    public static void Project(string corePath)
    {
        var directory = Path.Combine(Path.GetTempPath(), "neoclr-source-union-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        try
        {
            var seed = Path.Combine(directory, CoreDeclarations.Identity + ".dll");
            File.Copy(corePath, seed);
            var trees = Owners.Select(owner => {
                var resource = "NeoCLR." + owner.Split('.').Last() + ".rvn";
                using var sourceStream = typeof(SourceUnionReferences).Assembly.GetManifestResourceStream(resource)
                    ?? throw new InvalidDataException("Missing union reference source: " + resource);
                using var reader = new StreamReader(sourceStream);
                return SyntaxTree.ParseText(reader.ReadToEnd());
            }).ToArray();
            var options = new CompilationOptions(OutputKind.DynamicallyLinkedLibrary,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity))
                .WithTargetCoreAssemblyName(CoreDeclarations.Identity)
                .WithGraphemeChar(true)
                .WithRuntimeUnitContract(new RuntimeUnitContract(CoreDeclarations.Identity, "System.Void"));
            var input = seed;
            for (var index = 0; index < Owners.Length; index++)
            {
                // Each subsequent compilation consumes the already projected support
                // types, so Raven reuses the supplied core's IUnion identity.
                var compilation = Compilation.Create("SourceUnionReferences", [trees[index]],
                    [MetadataReference.CreateFromFile(input)], options);
                var source = Path.Combine(directory, "SourceUnionReferences" + index + ".dll");
                using (var output = File.Create(source))
                {
                    var result = compilation.Emit(output, null, new EmitOptions(AssemblyName.GetAssemblyName(seed)));
                    if (!result.Success) throw new InvalidDataException(string.Join("\n", result.Diagnostics));
                }
                var projected = Path.Combine(directory, Owners[index] + ".dll");
                StandardUnionReference.WriteReference(source, input, Owners[index], projected);
                input = projected;
            }
            File.Copy(input, corePath, overwrite: true);
        }
        finally { Directory.Delete(directory, recursive: true); }
    }
}
