using System.Reflection;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

static class SliceProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true);
        var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples/library-string-slices.rvn"));
        var compilation = Compilation.Create("Slices", [SyntaxTree.ParseText(source)],
            [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                runtimePropagationContract: new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3")));
        var incompatible = Compilation.Create("WrongCarrier", [SyntaxTree.ParseText("""
            import System.*
            func Wrong(text: string) -> Result<string, IO.FileReadError> {
                let part = text.SliceUtf8(0, 0)?
                return Result<string, IO.FileReadError>(Result.Ok<string>(part))
            }
            """)], [MetadataReference.CreateFromFile(core)],
            compilation.Options.WithOutputKind(OutputKind.DynamicallyLinkedLibrary));
        var errors = incompatible.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
        if (errors.Length == 0) throw new Exception("Incompatible slice residual was accepted.");
        File.WriteAllText(Path.Combine(output, "WrongCarrier.rejected.txt"), string.Join("\n", errors.Select(d => d.ToString())));
        var raw = Path.Combine(output, "Slices.raw.dll");
        using (var stream = File.Create(raw))
        {
            var emitted = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
        }
        var projected = Path.Combine(output, "Slices.dll");
        VoidProjection.Write(raw, core, projected);
        UnionImport.Write(projected, core, Path.Combine(output, "Slices.neoil"));
        ConditionalOutputChecks.Write(projected, core, output, "ShowSlice", ResultBindings.SliceError);
        Console.WriteLine("Imported slicing and rejected unsafe extraction: " + output);
    }
}
