using Mono.Cecil;
using Mono.Cecil.Cil;

static class PropagationImportChecks
{
    public static Dictionary<string, string> Run(string application, string core, string output)
    {
        var results = new Dictionary<string, string>();
        foreach (var name in new[] { "MissingContract", "WrongOutput", "ThrowValue" })
        {
            using var image = AssemblyDefinition.ReadAssembly(application);
            using var library = AssemblyDefinition.ReadAssembly(core);
            var original = image.MainModule.AssemblyReferences.ToHashSet();
            var references = library.MainModule.AssemblyReferences.ToHashSet();
            if (name == "MissingContract") library.MainModule.GetType("System.Result`2").Interfaces.Clear();
            if (name == "WrongOutput")
                library.MainModule.GetType("System.Propagatable`3").Methods.Single(m => m.Name == "TryGetOutput").Parameters[0].IsOut = false;
            if (name == "ThrowValue")
            {
                var method = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Normalize");
                var sentinel = method.Body.Instructions.Single(i => i.OpCode.Code == Code.Throw).Previous;
                sentinel.OpCode = OpCodes.Ldc_I4_0; sentinel.Operand = null;
            }
            var selectedCore = Path.Combine(output, name + ".Core.dll");
            WriteTarget(library, references, selectedCore);
            var path = Path.Combine(output, name + ".dll");
            WriteTarget(image, original, path);
            var destination = Path.Combine(output, name + ".neoil");
            try { UnionImport.Write(path, selectedCore, destination, collectionProfile: true); }
            catch (InvalidDataException error)
            {
                if (File.Exists(destination)) throw new Exception("Rejected propagation input produced output.");
                var expected = name == "ThrowValue" ? "Input stack type mismatch" : "propagation";
                if (!error.Message.Contains(expected, StringComparison.OrdinalIgnoreCase)) throw;
                results[name] = error.Message; continue;
            }
            throw new Exception("Expected propagation rejection: " + name);
        }
        return results;
    }
    public static void RejectVoidParameter(string application, string core, string output)
    {
        using var image = AssemblyDefinition.ReadAssembly(application);
        var original = image.MainModule.AssemblyReferences.ToHashSet();
        var call = image.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody)
            .SelectMany(m => m.Body.Instructions).Select(i => i.Operand).OfType<MethodReference>()
            .Single(m => m.Name == "TryGetResidual");
        call.Parameters[0].ParameterType = new ByReferenceType(image.MainModule.TypeSystem.Void);
        var invalid = Path.Combine(output, "InvalidVoidParameter.dll");
        WriteTarget(image, original, invalid);
        try { UnionImport.Write(invalid, core, Path.Combine(output, "InvalidVoidParameter.neoil"), collectionProfile: true); }
        catch (InvalidDataException error) when (error.Message.Contains("CLI VOID marker")) { return; }
        throw new Exception("CLI VOID marker accepted as parameter storage.");
    }
    static void WriteTarget(AssemblyDefinition image, HashSet<AssemblyNameReference> original, string path)
    {
        foreach (var method in image.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        // Cecil may synthesize unused host core references while reading method bodies.
        foreach (var reference in image.MainModule.AssemblyReferences.Where(r => !original.Contains(r)).ToArray())
        {
            if (image.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Mutation introduced an external scope.");
            image.MainModule.AssemblyReferences.Remove(reference);
        }
        image.Write(path);
    }
}
