using Mono.Cecil;
using Mono.Cecil.Cil;

static class ArrayImportChecks
{
    public static Dictionary<string, string> Run(string application, string core, string output)
    {
        var results = new Dictionary<string, string>();
        foreach (var name in new[] { "WrongElement", "WrongStore", "WrongIndex", "NonVector", "UninitializedLocal" })
        {
            using var image = AssemblyDefinition.ReadAssembly(application);
            var originalReferences = image.MainModule.AssemblyReferences.ToHashSet();
            var methods = image.MainModule.GetTypes().SelectMany(t => t.Methods).ToArray();
            var make = methods.Single(m => m.Name == "Make");
            var change = methods.Single(m => m.Name == "Change");
            var main = image.EntryPoint;
            switch (name)
            {
                case "WrongElement":
                    make.Body.Instructions.Single(i => i.OpCode.Code == Code.Newarr).Operand = new TypeReference("System", "String", image.MainModule, image.MainModule.AssemblyReferences.Single(r => r.Name == CoreDeclarations.Identity)); break;
                case "WrongStore":
                    change.Body.Instructions.Single(i => i.OpCode.Code == Code.Stelem_I4).OpCode = OpCodes.Stelem_Ref; break;
                case "WrongIndex":
                    var index = change.Body.Instructions.First(i => i.OpCode.Code == Code.Ldc_I4_0);
                    index.OpCode = OpCodes.Ldstr; index.Operand = "invalid index"; break;
                case "NonVector":
                    make.ReturnType = new ArrayType(((ArrayType)make.ReturnType).ElementType, 2); break;
                case "UninitializedLocal":
                    main.Body.InitLocals = false;
                    var il = main.Body.GetILProcessor();
                    var first = main.Body.Instructions[0];
                    il.InsertBefore(first, il.Create(OpCodes.Ldloc_0));
                    il.InsertBefore(first, il.Create(OpCodes.Pop)); break;
            }
            var path = Path.Combine(output, name + ".dll");
            var destination = Path.Combine(output, name + ".neoil");
            WriteTarget(image, originalReferences, path);
            try { UnionImport.Write(path, core, destination); }
            catch (InvalidDataException error) when (error.Message.Contains(name switch {
                "WrongElement" => "Only Int32 vector allocation",
                "WrongStore" => "Unsupported reachable instruction: stelem.ref",
                "WrongIndex" => "Input stack type mismatch",
                "NonVector" => "Unsupported Result profile type: System.Int32[,]",
                "UninitializedLocal" => "Read of uninitialized or unsupported default local",
                _ => throw new Exception("Unknown negative case")
            }))
            {
                if (File.Exists(destination)) throw new Exception("Rejected array input produced executable output.");
                results[name] = error.Message;
                continue;
            }
            throw new Exception("Array importer failed to reject " + name);
        }
        using var nullImage = AssemblyDefinition.ReadAssembly(application);
        var nullReferences = nullImage.MainModule.AssemblyReferences.ToHashSet();
        var body = nullImage.EntryPoint.Body;
        body.InitLocals = true;
        body.Instructions.Clear();
        var writer = body.GetILProcessor();
        writer.Append(writer.Create(OpCodes.Ldloc_0));
        writer.Append(writer.Create(OpCodes.Ldlen));
        writer.Append(writer.Create(OpCodes.Pop));
        writer.Append(writer.Create(OpCodes.Ret));
        var nullPath = Path.Combine(output, "NullDefault.dll");
        WriteTarget(nullImage, nullReferences, nullPath);
        UnionImport.Write(nullPath, core, Path.Combine(output, "NullDefault.neoil"));
        return results;
    }

    static void WriteTarget(AssemblyDefinition image, HashSet<AssemblyNameReference> original, string path)
    {
        // Match VoidProjection: remove only Cecil-created, unused core references.
        foreach (var reference in image.MainModule.AssemblyReferences.Where(r => !original.Contains(r)).ToArray())
        {
            if (image.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Mutation introduced an external scope.");
            image.MainModule.AssemblyReferences.Remove(reference);
        }
        image.Write(path);
    }
}
