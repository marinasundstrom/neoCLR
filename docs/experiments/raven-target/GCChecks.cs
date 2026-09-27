using Mono.Cecil;

static class GCChecks
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var path = Path.Combine(output, "NeoCLR.CoreProbe.dll");
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path);
        GCBindings.Validate(core.MainModule);
        var count = 0;
        foreach (var method in core.MainModule.GetType(GCBindings.Owner).Methods) {
            if (GCBindings.Bind(method, method) is null)
                throw new Exception("GC member not bound");
            count++;
            var wrong = new MethodReference(method.Name, core.MainModule.TypeSystem.Int32, method.DeclaringType);
            foreach (var parameter in method.Parameters)
                wrong.Parameters.Add(new ParameterDefinition(parameter.ParameterType));
            try {
                GCBindings.Bind(wrong, method);
                throw new Exception("GC malformed signature admitted");
            } catch (InvalidDataException) { count++; }
        }
        Console.WriteLine($"{count} GC signature checks passed");
    }
}
