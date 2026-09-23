using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Mono.Cecil;

static class StorageHierarchyProbe
{
    public static void Write(string output)
    {
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        var source = """
            public class ThirdKind : System.Storage.StorageItem {
                public string Name => null;
                public System.Storage.Path Path => null;
                public static void Main() { }
            }
            """;
        // C# does not enforce Raven's ClosedHierarchyAttribute. The importer must.
        var compilation = CSharpCompilation.Create("ThirdKind", [CSharpSyntaxTree.ParseText(source)],
            [MetadataReference.CreateFromFile(core)], new CSharpCompilationOptions(OutputKind.ConsoleApplication));
        var app = Path.Combine(output, "ThirdKind.dll");
        using (var stream = File.Create(app))
        {
            var result = compilation.Emit(stream);
            if (!result.Success) throw new InvalidDataException(string.Join("\n", result.Diagnostics));
        }
        var destination = Path.Combine(output, "Rejected.neoil");
        try { UnionImport.Write(app, core, destination, collectionProfile: true); }
        catch (InvalidDataException error) when (error.Message.Contains("External direct StorageItem branches"))
        {
            if (File.Exists(destination)) throw new InvalidDataException("Rejected hierarchy produced output.");
            using var image = AssemblyDefinition.ReadAssembly(core);
            var root = image.MainModule.GetType(StorageItemBindings.Root);
            root.CustomAttributes.Clear();
            try { StorageHierarchy.Validate(root); }
            catch (InvalidDataException malformed) when (malformed.Message.Contains("closed to File and Directory"))
            {
                Console.WriteLine("PASS: raw CIL third branch and missing closure metadata rejected.");
                return;
            }
            throw new InvalidDataException("Missing closure metadata accepted.");
        }
        throw new InvalidDataException("Third StorageItem branch accepted.");
    }
}
