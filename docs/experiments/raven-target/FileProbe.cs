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
        CoreDeclarations.Write(core, unionProbe: true);
        var sample = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples/library-files.rvn"));
        Compile("Files", sample);
        var extra = sample[..sample.IndexOf("func Main()")];
        Compile("Failures", extra + "func Main() { ShowRead(Load(\"missing.txt\", 64)); ShowRead(Load(\"invalid.txt\", 64)); ShowRead(Load(\"missing.txt\", -1)); ShowWrite(Save(\"empty.txt\", \"\", 0)); ShowRead(Load(\"empty.txt\", 0)) }");
        void Compile(string name, string source)
        {
            var compilation = Compilation.Create(name, [SyntaxTree.ParseText(source)],
                [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                    metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                    runtimePropagationContract: new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3")));
            var raw = Path.Combine(output, name + ".raw.dll");
            using (var stream = File.Create(raw))
            {
                var emitted = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
                if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
            }
            var projected = Path.Combine(output, name + ".dll");
            VoidProjection.Write(raw, core, projected);
            UnionImport.Write(projected, core, Path.Combine(output, name + ".neoil"));
        }
        CheckRejectedOutputs(Path.Combine(output, "Files.dll"), core, output);
        Console.WriteLine("Imported file propagation and failure fixtures: " + output);
    }
    static void CheckRejectedOutputs(string application, string core, string output)
    {
        foreach (var mode in new[] { "IgnoredExtraction", "InvertedExtraction", "UninitializedError" })
        {
            using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(application);
            var references = image.MainModule.AssemblyReferences.ToHashSet();
            var method = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "ShowWrite");
            if (mode == "UninitializedError")
            {
                var local = method.Body.Variables.First(v => v.VariableType.FullName == FileBindings.WriteError);
                var il = method.Body.GetILProcessor();
                var first = method.Body.Instructions[0];
                il.InsertBefore(first, il.Create(Mono.Cecil.Cil.OpCodes.Ldloc, local));
                il.InsertBefore(first, il.Create(Mono.Cecil.Cil.OpCodes.Pop));
            }
            else
            {
                var call = method.Body.Instructions.First(i => i.Operand is Mono.Cecil.MethodReference m
                    && m.Name == "TryGetValue" && m.Parameters[0].ParameterType.FullName.Contains("Error"));
                var branch = call.Next;
                branch.OpCode = mode == "IgnoredExtraction" ? Mono.Cecil.Cil.OpCodes.Pop : Mono.Cecil.Cil.OpCodes.Brtrue;
                if (mode == "IgnoredExtraction") branch.Operand = null;
            }
            foreach (var candidate in image.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
            { _ = candidate.Body.Instructions.Count; _ = candidate.Body.Variables.Count; }
            foreach (var reference in image.MainModule.AssemblyReferences.Where(r => !references.Contains(r)).ToArray())
            {
                if (image.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                    throw new InvalidDataException("Mutation introduced an external scope.");
                image.MainModule.AssemblyReferences.Remove(reference);
            }
            var invalid = Path.Combine(output, mode + ".dll");
            var destination = Path.Combine(output, mode + ".neoil");
            var raw = Path.Combine(output, mode + ".raw.dll");
            image.Write(raw);
            VoidProjection.Write(raw, core, invalid);
            try { UnionImport.Write(invalid, core, destination); }
            catch (InvalidDataException error) when (error.Message.Contains("uninitialized") || error.Message.Contains("must be tested"))
            {
                if (File.Exists(destination)) throw new Exception("Rejected file extraction produced output.");
                File.WriteAllText(Path.Combine(output, mode + ".rejected.txt"), error.Message);
                continue;
            }
            throw new Exception("Invalid file extraction admitted: " + mode);
        }
    }

}
