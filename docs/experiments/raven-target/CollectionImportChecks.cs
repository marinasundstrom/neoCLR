using Mono.Cecil;
using Mono.Cecil.Cil;

static class CollectionImportChecks
{
    public static Dictionary<string, string> Run(string application, string core, string output)
    {
        var results = new Dictionary<string, string>();
        foreach (var name in new[] { "WrongElement", "WrongReceiver", "DirectInterfaceCall", "UninitializedLocal", "ProfileNotSelected", "WrongInheritance" })
        {
            using var image = AssemblyDefinition.ReadAssembly(application);
            using var library = AssemblyDefinition.ReadAssembly(core);
            var original = image.MainModule.AssemblyReferences.ToHashSet();
            var libraryReferences = library.MainModule.AssemblyReferences.ToHashSet();
            var print = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Print");
            var add = print.Body.Instructions.Single(i => i.Operand is MethodReference m && m.Name == "Add");
            var selectedCore = core;
            switch (name)
            {
                case "WrongElement":
                    add.Previous.OpCode = OpCodes.Ldstr; add.Previous.Operand = "wrong"; break;
                case "WrongReceiver":
                    add.Previous.Previous.OpCode = OpCodes.Ldstr; add.Previous.Previous.Operand = "wrong"; break;
                case "DirectInterfaceCall": add.OpCode = OpCodes.Call; break;
                case "UninitializedLocal":
                    image.EntryPoint.Body.InitLocals = false;
                    var il = image.EntryPoint.Body.GetILProcessor(); var first = image.EntryPoint.Body.Instructions[0];
                    il.InsertBefore(first, il.Create(OpCodes.Ldloc_0)); il.InsertBefore(first, il.Create(OpCodes.Pop)); break;
                case "WrongInheritance":
                    library.MainModule.GetType("System.Collections.List`1").Interfaces.Clear();
                    selectedCore = Path.Combine(output, "WrongInheritance.Core.dll"); WriteTarget(library, libraryReferences, selectedCore); break;
            }
            var path = Path.Combine(output, name + ".dll");
            WriteTarget(image, original, path);
            var destination = Path.Combine(output, name + ".neoil");
            try { UnionImport.Write(path, selectedCore, destination, collectionProfile: name != "ProfileNotSelected"); }
            catch (InvalidDataException error) when (error.Message.Contains(name switch {
                "WrongElement" or "WrongReceiver" => "Input stack type mismatch",
                "DirectInterfaceCall" => "Unsupported collection call shape",
                "UninitializedLocal" => "Read of uninitialized or unsupported default local",
                "ProfileNotSelected" => "Unsupported Result profile type",
                "WrongInheritance" => "Incompatible collection contract",
                _ => throw new Exception("Unknown rejection")
            }))
            {
                if (File.Exists(destination)) throw new Exception("Rejected input produced executable output.");
                results[name] = error.Message; continue;
            }
            throw new Exception("Collection importer failed to reject " + name);
        }
        // A CLI InitLocals reference is valid null storage; dereferencing it faults.
        using var nullImage = AssemblyDefinition.ReadAssembly(application);
        var nullReferences = nullImage.MainModule.AssemblyReferences.ToHashSet();
        var count = nullImage.MainModule.GetTypes().SelectMany(t => t.Methods).SelectMany(m => m.HasBody ? m.Body.Instructions : Enumerable.Empty<Instruction>())
            .Single(i => i.Operand is MethodReference m && m.Name == "get_Count").Operand;
        var body = nullImage.EntryPoint.Body;
        body.InitLocals = true; body.Instructions.Clear();
        var writer = body.GetILProcessor();
        writer.Append(writer.Create(OpCodes.Ldloc_0));
        writer.Append(writer.Create(OpCodes.Callvirt, (MethodReference)count));
        writer.Append(writer.Create(OpCodes.Pop)); writer.Append(writer.Create(OpCodes.Ret));
        var nullPath = Path.Combine(output, "NullCollection.dll"); WriteTarget(nullImage, nullReferences, nullPath);
        UnionImport.Write(nullPath, core, Path.Combine(output, "NullCollection.neoil"), collectionProfile: true);
        return results;
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
