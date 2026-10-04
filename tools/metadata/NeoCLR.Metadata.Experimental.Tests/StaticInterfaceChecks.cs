using System.Reflection;
using System.Reflection.Emit;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;

internal static class StaticInterfaceChecks
{
    internal static void Run()
    {
        foreach (var detached in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("StaticContracts" + detached, new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var contract = graph.AddInterface("Example", "Identity");
            var signature = new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]);
            if (detached)
                contract.Definition.Methods.Add(new MethodDefinition("Echo", 0x556, signature));
            else
                contract.AddInterfaceMethod("Echo", signature, isStatic: true);
            var implementation = graph.AddClass("Example", "IdentityImpl");
            implementation.AddInterfaceImplementation(contract);
            var echo = implementation.AddMethod("Echo", signature);
            echo.GetILGenerator().LoadArgument(0); echo.GetILGenerator().Return();
            var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
            main.GetILGenerator().LoadConstant(42); main.GetILGenerator().Call(echo); main.GetILGenerator().Return();
            graph.EntryPoint = main;
            var loaded = Assembly.Load(graph.Write());
            var owner = loaded.GetType("Example.IdentityImpl", true)!;
            var declaration = loaded.GetType("Example.Identity", true)!.GetMethod("Echo")!;
            var call = new DynamicMethod("InvokeContract", typeof(int), [], owner.Module);
            var il = call.GetILGenerator();
            il.Emit(OpCodes.Ldc_I4, 42); il.Emit(OpCodes.Constrained, owner); il.Emit(OpCodes.Call, declaration); il.Emit(OpCodes.Ret);
            if (call.CreateDelegate<Func<int>>()() != 42) throw new Exception("CLI static interface dispatch");
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
            var view = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Identity").GetMethods().Single();
            if (!view.IsStatic || !view.IsAbstract) throw new Exception("static contract flags lost");
            if (Environment.GetEnvironmentVariable("NEOCLR_STATIC_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
        }
        foreach (var external in new[] { false, true })
        foreach (var correct in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("StaticSelf", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var owner = graph.AddClass("Example", "Value");
            if (external)
            {
                var contract = graph.CreateInterfaceReference(new("Contracts", new Version(1, 0, 0, 0)), graph.CoreLibrary, new string('A', 64), "Example", "Copy");
                graph.CreateMethodReference(contract, "Copy", new(SignatureType.Self, [SignatureType.Self]), isStatic: true);
                graph.CompleteInterfaceReference(contract);
                owner.AddInterfaceImplementation(contract);
            }
            else
            {
                var contract = graph.AddInterface("Example", "Copy");
                contract.AddInterfaceMethod("Copy", new(SignatureType.Self, [SignatureType.Self]), isStatic: true);
                var parent = graph.AddGenericInterface("Example", "Parent", ["T"]);
                contract.AddBaseInterface(parent.MakeGenericInstance(SignatureType.Self));
                owner.AddInterfaceImplementation(contract);
            }
            var method = correct ? owner.AddMethod("Copy", new(owner, [owner])) : owner.AddInstanceMethod("Copy", new(owner, [owner]));
            method.GetILGenerator().LoadArgument(correct ? 0 : 1); method.GetILGenerator().Return();
            try
            {
                var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
                if (!correct) throw new Exception("instance method satisfied static contract");
                if (!external)
                {
                    var contract = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Copy");
                    if (((ConstructedTypeInfo)contract.GetDeclaredInterfaces().Single()).TypeArguments.Single() is not SelfTypeInfo)
                        throw new Exception("inherited Self scope lost");
                }
            }
            catch (InvalidDataException) when (!correct) { }
        }
    }
}
