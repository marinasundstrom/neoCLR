using System.Reflection;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;
using AssemblyDefinition = Mono.Cecil.AssemblyDefinition;

static class InterfaceProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, collectionProbe: true);
        var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-interfaces.rvn"));
        Compilation Create(string text) => Compilation.Create("CoreInterfaces", [SyntaxTree.ParseText(text)],
            [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
        var compilation = Create(source);
        var path = Path.Combine(output, "CoreInterfaces.dll");
        using (var stream = File.Create(path))
        {
            var result = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        var closure = ClosureAudit.Inspect(path, core);
        if (closure.Length != 0) throw new Exception(string.Join("\n", closure));
        using var library = AssemblyDefinition.ReadAssembly(core);
        using var image = AssemblyDefinition.ReadAssembly(path);
        var interfaces = library.MainModule.GetTypes().Where(t => t.IsInterface).ToArray();
        if (interfaces.Length != 4 || interfaces.SelectMany(t => t.Methods).Any(m => !m.IsAbstract || !m.IsVirtual || !m.IsNewSlot))
            throw new Exception("Expected ordinary abstract CLI interface contracts.");
        var list = library.MainModule.GetType("System.Collections.ArrayList`1");
        if (list.IsValueType || !list.Interfaces.Any(i => i.InterfaceType.FullName == "System.Collections.List`1<T>"))
            throw new Exception("Expected a class implementing the existing List contract.");
        var methods = image.MainModule.GetType("NamespaceMembers").Methods.ToArray();
        var instructions = methods.SelectMany(m => m.Body.Instructions).ToArray();
        if (instructions.Any(i => i.OpCode.Code is Code.Box or Code.Unbox or Code.Unbox_Any or Code.Constrained))
            throw new Exception("Reference-class interface scenario unexpectedly needs value-receiver conversion.");
        var calls = instructions.Where(i => i.OpCode.Code == Code.Callvirt).Select(i => ((MethodReference)i.Operand).FullName).ToArray();
        foreach (var member in new[] { "::Add(", "::get_Count(", "::GetIterator(", "::MoveNext(", "::get_Current(", "::Dispose(" })
            if (!calls.Any(c => c.Contains(member))) throw new Exception("Missing interface dispatch: " + member);
        var negatives = new Dictionary<string, string[]>();
        foreach (var (name, text) in new[] { ("WrongElement", source.Replace("values.Add(42)", "values.Add(\"wrong\")")),
            ("WrongReceiver", source.Replace("Print(values)", "Print(42)")) })
        {
            var errors = Create(text).GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
            if (!errors.Any(e => e.Contains("RAV1503"))) throw new Exception("Expected incompatible-type rejection: " + name);
            negatives[name] = errors;
        }
        var destination = Path.Combine(output, "CoreInterfaces.neoil");
        string rejection;
        try { UnionImport.Write(path, core, destination); throw new Exception("Interface execution unexpectedly admitted."); }
        catch (InvalidDataException error) when (error.Message.Contains("Unsupported Result profile type: System.Collections.ArrayList")) { rejection = error.Message; }
        if (File.Exists(destination)) throw new Exception("Unimplemented interface import produced executable output.");
        File.WriteAllText(Path.Combine(output, "interface-results.json"), JsonSerializer.Serialize(new {
            RecordedDate = "2026-09-12", Scope = "Candidate library interface metadata and Raven emission only; no runtime execution",
            ApplicationClosureErrors = closure,
            Interfaces = interfaces.Select(t => new { t.FullName, Parents = t.Interfaces.Select(i => i.InterfaceType.FullName).ToArray() }),
            InterfaceCalls = calls, NegativeDiagnostics = negatives, ImportRejection = rejection
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("PASS: candidate collection contracts emit through ordinary interface calls; runtime admission remains unsupported.");
    }
}
