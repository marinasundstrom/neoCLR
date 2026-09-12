using System.Reflection;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

static class ArrayProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true);
        var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-arrays.rvn"));
        var compilation = Compilation.Create("CoreArrays", [SyntaxTree.ParseText(source)],
            [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
        var path = Path.Combine(output, "CoreArrays.dll");
        using (var stream = File.Create(path))
        {
            var result = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        var closure = ClosureAudit.Inspect(path, core);
        if (closure.Length != 0) throw new Exception(string.Join("\n", closure));
        using var image = AssemblyDefinition.ReadAssembly(path);
        var operations = image.MainModule.GetType("NamespaceMembers").Methods.Where(m => m.HasBody)
            .SelectMany(m => m.Body.Instructions).Select(i => i.OpCode.Code).ToArray();
        foreach (var opcode in new[] { Code.Newarr, Code.Ldelem_I4, Code.Stelem_I4, Code.Ldlen })
            if (!operations.Contains(opcode)) throw new Exception("Missing standard array opcode: " + opcode);
        UnionImport.Write(path, core, Path.Combine(output, "CoreArrays.neoil"));
        var negatives = ArrayImportChecks.Run(path, core, output);
        File.WriteAllText(Path.Combine(output, "array-results.json"), JsonSerializer.Serialize(new {
            Scope = "Raven Int32 vector emission and import; execute CoreArrays.neoil separately on neoCLR",
            NegativeRejections = negatives, ApplicationClosureErrors = closure, Opcodes = operations.Select(c => c.ToString()).Distinct().ToArray(),
            ExpectedOutput = new[] { "42", "2" }
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("PASS: Raven Int32 arrays imported using ordinary array references.");
    }
}
