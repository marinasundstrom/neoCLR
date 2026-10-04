using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class SelfImplementationChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("NeoCLR.CoreProbe", new Version(1, 0, 0, 0));
        foreach (var external in new[] { false, true })
        foreach (var valid in new[] { false, true })
        foreach (var genericOwner in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("SelfImplementation", new Version(1, 0, 0, 0)), core);
            var owner = genericOwner ? graph.AddGenericClass("Example", "Copy", ["T"]) : graph.AddClass("Example", "Copy");
            SignatureType ownerType = genericOwner ? owner.MakeGenericInstance(SignatureType.TypeParameter(0)) : owner;
            if (external)
            {
                var contract = graph.CreateInterfaceReference(new("Contracts", new Version(1, 0, 0, 0)), core, new string('A', 64), "Example", "Copyable", 0);
                graph.CreateMethodReference(contract, "Copy", new(SignatureType.Self, [SignatureType.Self]));
                graph.CompleteInterfaceReference(contract);
                owner.AddInterfaceImplementation(contract);
                var ordinary = graph.CreateTypeReference(new("Classes", new Version(1, 0, 0, 0)), core, new string('B', 64), "Example", "Ordinary", 0);
                try { graph.CreateMethodReference(ordinary, "Copy", new(SignatureType.Self, [])); throw new Exception("class Self signature admitted"); }
                catch (InvalidDataException) { }
            }
            else
            {
                var contract = graph.AddInterface("Example", "Copyable");
                contract.AddInterfaceMethod("Copy", new(SignatureType.Self, [SignatureType.Self]));
                owner.AddInterfaceImplementation(contract);
            }
            var method = owner.AddInstanceMethod("Copy", new(valid ? ownerType : PrimitiveType.Int32, [ownerType]));
            var body = method.GetILGenerator();
            if (valid) body.LoadArgument(1); else body.LoadConstant(42);
            body.Return();
            try
            {
                var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
                if (!valid) throw new Exception("incompatible Self implementation admitted");
                if (new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == (genericOwner ? "Copy`1" : "Copy")).GetMethods().Single().ReturnType is SelfTypeInfo)
                    throw new Exception("implementation retained symbolic Self");
            }
            catch (InvalidDataException) when (!valid) { }
        }
    }
}
