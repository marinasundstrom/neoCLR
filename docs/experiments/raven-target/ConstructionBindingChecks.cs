using Mono.Cecil;

static class ConstructionBindingChecks
{
    public static void Verify(string path)
    {
        using var image = AssemblyDefinition.ReadAssembly(path);
        var owner = image.MainModule.Types.Single(t => t.FullName == "System.Runtime.CompilerServices.NativeReflection");
        foreach (var method in owner.Methods)
        {
            var binding = RuntimeServiceBindings.Bind(method, method)
                ?? throw new Exception("Construction facade not bound");
            if (binding.Name != "neoCLR.Runtime." + method.Name)
                throw new Exception("Construction facade selected the wrong runtime service");
        }
        var construct = owner.Methods.Single(m => m.Name == "ReflectionConstruct");
        var result = construct.ReturnType;
        construct.ReturnType = image.MainModule.TypeSystem.Int32;
        Reject(() => RuntimeServiceBindings.Bind(construct, construct));
        construct.ReturnType = result;
        var name = image.Name.Name;
        image.Name.Name = "UntrustedFacade";
        Reject(() => RuntimeServiceBindings.Bind(construct, construct));
        image.Name.Name = name;
        construct.Name = "ReflectionInvoke";
        Reject(() => RuntimeServiceBindings.Bind(construct, construct));
        Console.WriteLine("PASS construction facade signatures, exact bootstrap owner and operation allowlist");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("Invalid construction facade accepted");
    }
}
