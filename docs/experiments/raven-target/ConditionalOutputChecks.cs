static class ConditionalOutputChecks
{
    public static void Write(string application, string core, string output, string methodName, string errorType)
    {
        foreach (var mode in new[] { "IgnoredExtraction", "InvertedExtraction", "UninitializedError" })
        {
            using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(application);
            var references = image.MainModule.AssemblyReferences.ToHashSet();
            var method = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == methodName);
            if (mode == "UninitializedError")
            {
                // With standard unions, zero-initialized locals are valid inactive
                // values. Disable CLI initialization to test a genuinely unread local.
                method.Body.InitLocals = false;
                var local = method.Body.Variables.First(v => v.VariableType.FullName == errorType);
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
                if (File.Exists(destination)) throw new Exception("Rejected Result extraction produced output.");
                File.WriteAllText(Path.Combine(output, mode + ".rejected.txt"), error.Message);
                continue;
            }
            throw new Exception("Invalid Result extraction admitted: " + mode);
        }
    }

}
