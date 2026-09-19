using System.Reflection;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

static class FileProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        var sample = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples/library-files.rvn"));
        Compile("Files", sample);
        var extra = sample[..sample.IndexOf("func Main()")];
        Compile("Failures", extra + "func Main() { ShowRead(Load(\"missing.txt\", 64)); ShowRead(Load(\"invalid.txt\", 64)); ShowRead(Load(\"missing.txt\", -1)); ShowWrite(Save(\"empty.txt\", \"\", 0)); ShowRead(Load(\"empty.txt\", 0)) }");
        void Compile(string name, string source)
        {
            var compilation = Compilation.Create(name, [SyntaxTree.ParseText(source)],
                [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                    metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                    runtimePropagationContract: new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3"))
                    .WithTargetCoreAssemblyName(CoreDeclarations.Identity)
                    .WithRuntimeUnitContract(new RuntimeUnitContract(CoreDeclarations.Identity, "System.Void")));
            var raw = Path.Combine(output, name + ".raw.dll");
            using (var stream = File.Create(raw))
            {
                var emitted = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
                if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
            }
            var projected = Path.Combine(output, name + ".dll");
            VoidProjection.Write(raw, core, projected);
            UnionImport.Write(projected, core, Path.Combine(output, name + ".neoil"), collectionProfile: true);
        }
        ConditionalOutputChecks.Write(Path.Combine(output, "Files.dll"), core, output, "ShowWrite", FileBindings.WriteError);
        Console.WriteLine("Imported file propagation and failure fixtures: " + output);
    }
}
