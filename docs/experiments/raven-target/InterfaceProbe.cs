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
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-interfaces.rvn"));
        Compilation Create(string text) => Compilation.Create("CoreInterfaces", [SyntaxTree.ParseText(text)],
            [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                runtimeIterationContract: new RuntimeIterationContract(CoreDeclarations.Identity,
                    "System.Collections.Iterable`1", "System.Collections.Iterator`1"),
                runtimePropagationContract: new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3")));
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
        if (interfaces.Length != 5 || interfaces.SelectMany(t => t.Methods).Any(m => !m.IsAbstract || !m.IsVirtual || !m.IsNewSlot))
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
        UnionImport.Write(path, core, destination, collectionProfile: true);
        var aliasSource = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-collection-aliases.rvn"));
        var aliasPath = Path.Combine(output, "CollectionAliases.dll");
        using (var stream = File.Create(aliasPath))
        {
            var result = Create(aliasSource).Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        UnionImport.Write(aliasPath, core, Path.Combine(output, "CollectionAliases.neoil"), collectionProfile: true);
        var loopSource = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-foreach.rvn"));
        var loopPath = Path.Combine(output, "CollectionForEach.dll");
        using (var stream = File.Create(loopPath))
        {
            var result = Create(loopSource).Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        UnionImport.Write(loopPath, core, Path.Combine(output, "CollectionForEach.neoil"), collectionProfile: true);
        var workflowSource = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-workflow.rvn"));
        var workflowRaw = Path.Combine(output, "CollectionWorkflow.raw.dll");
        using (var stream = File.Create(workflowRaw))
        {
            var result = Create(workflowSource).Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        var workflowPath = Path.Combine(output, "CollectionWorkflow.dll");
        VoidProjection.Write(workflowRaw, core, workflowPath);
        var workflowClosure = ClosureAudit.Inspect(workflowPath, core);
        if (workflowClosure.Length != 0) throw new Exception(string.Join("\n", workflowClosure));
        UnionImport.Write(workflowPath, core, Path.Combine(output, "CollectionWorkflow.neoil"), collectionProfile: true);
        var propagationSource = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-propagation.rvn"));
        var propagationPath = Path.Combine(output, "Propagation.dll");
        using (var stream = File.Create(propagationPath))
        {
            var result = Create(propagationSource).Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
            if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        }
        UnionImport.Write(propagationPath, core, Path.Combine(output, "Propagation.neoil"), collectionProfile: true);
        var incompatible = Create(propagationSource.Replace("func Normalize(value: int) -> Result<int, OverflowError>",
            "func Normalize(value: int) -> Option<int>"));
        if (!incompatible.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
            throw new Exception("Incompatible propagation carrier accepted.");
        foreach (var sample in new[] { "library-option-propagation", "library-result-void-propagation" })
        {
            var raw = Path.Combine(output, sample + ".raw.dll");
            using (var stream = File.Create(raw))
            {
                var result = Create(File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", sample + ".rvn")))
                    .Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
                if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
            }
            var projected = Path.Combine(output, sample + ".dll");
            VoidProjection.Write(raw, core, projected);
            UnionImport.Write(projected, core, Path.Combine(output, sample + ".neoil"), collectionProfile: true);
            if (sample == "library-option-propagation")
            {
                PropagationImportChecks.RejectVoidParameter(projected, core, output);
                continue;
            }
            try { UnionImport.Write(raw, core, Path.Combine(output, sample + ".invalid.neoil"), collectionProfile: true); }
            catch (InvalidDataException) { continue; }
            throw new Exception("Raw Void marker storage was admitted: " + sample);
        }
        var propagationRejections = PropagationImportChecks.Run(propagationPath, core, output);
        var rejections = CollectionImportChecks.Run(path, core, output);
        File.WriteAllText(Path.Combine(output, "interface-results.json"), JsonSerializer.Serialize(new {
            RecordedDate = "2026-09-12", Scope = "Raven collection metadata, emission and import; runtime verification is separate",
            ApplicationClosureErrors = closure,
            Interfaces = interfaces.Select(t => new { t.FullName, Parents = t.Interfaces.Select(i => i.InterfaceType.FullName).ToArray() }),
            InterfaceCalls = calls, NegativeDiagnostics = negatives, ImportRejections = rejections, PropagationRejections = propagationRejections, ImportedProgram = Path.GetFileName(destination)
        }, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine("PASS: collection contracts emit and import through ordinary interface calls.");
    }
}
