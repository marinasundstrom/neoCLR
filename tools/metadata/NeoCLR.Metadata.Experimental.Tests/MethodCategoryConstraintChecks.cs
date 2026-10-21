using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class MethodCategoryConstraintChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("MethodCategories", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var value = Identity(graph, "Value", TypeParameterConstraints.ValueType);
        value.MakeGenericInstance(PrimitiveType.Int32);
        value.MakeGenericInstance(graph.AddEnum("Example", "State"));
        Reject(() => value.MakeGenericInstance(PrimitiveType.String));
        var reference = Identity(graph, "Reference", TypeParameterConstraints.ReferenceType);
        reference.MakeGenericInstance(PrimitiveType.String);
        Reject(() => reference.MakeGenericInstance(PrimitiveType.Int32));
        var constructible = Identity(graph, "Constructible", TypeParameterConstraints.DefaultConstructor);
        constructible.MakeGenericInstance(PrimitiveType.Int32);
        Reject(() => constructible.MakeGenericInstance(PrimitiveType.String));
        var concrete = graph.AddClass("Example", "Concrete");
        concrete.AddConstructor([]).Return();
        constructible.MakeGenericInstance(concrete);
        var inaccessible = graph.AddClass("Example", "Inaccessible");
        inaccessible.AddConstructor([], MethodVisibility.Private).Return();
        Reject(() => constructible.MakeGenericInstance(inaccessible));
        var referenceNew = Identity(graph, "ReferenceNew", TypeParameterConstraints.ReferenceType | TypeParameterConstraints.DefaultConstructor);
        referenceNew.MakeGenericInstance(concrete);
        Reject(() => referenceNew.MakeGenericInstance(PrimitiveType.Int32));
        Reject(() => referenceNew.MakeGenericInstance(PrimitiveType.String));
        Identity(graph, "ValueNew", TypeParameterConstraints.ValueType | TypeParameterConstraints.DefaultConstructor);
        Reject(() => value.SetSpecialConstraints(0, TypeParameterConstraints.ValueType | TypeParameterConstraints.ReferenceType));
        Reject(() => value.SetSpecialConstraints(0, TypeParameterConstraints.ValueType | TypeParameterConstraints.FunctionType));
        var bound = graph.AddInterface("Example", "Contract");
        var implementation = graph.AddValueType("Example", "Implementation");
        implementation.AddInterfaceImplementation(bound);
        var bounded = Identity(graph, "Bounded", TypeParameterConstraints.ValueType);
        bounded.AddInterfaceConstraint(0, bound);
        bounded.MakeGenericInstance(implementation);
        Reject(() => bounded.MakeGenericInstance(PrimitiveType.Int32));
        var owner = graph.AddGenericClass("Example", "Owner", ["T"]);
        owner.SetSpecialConstraints(0, TypeParameterConstraints.ValueType);
        var fromOwner = owner.AddMethod("Forward", new(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0)]));
        fromOwner.LoadArgument(0);
        fromOwner.Call(constructible.MakeGenericInstance(SignatureType.TypeParameter(0)));
        fromOwner.Return();
        var forward = Identity(graph, "Forward", TypeParameterConstraints.ValueType);
        forward.ClearBody();
        forward.LoadArgument(0);
        forward.GetILGenerator().Call(constructible.MakeGenericInstance(SignatureType.MethodParameter(0)));
        forward.Return();
        var unbounded = Identity(graph, "Unbounded", TypeParameterConstraints.None);
        Reject(() => unbounded.GetILGenerator().Call(value.MakeGenericInstance(SignatureType.MethodParameter(0))));
        foreach (var snapshot in new[] {
            AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)),
            AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false)
        })
        {
            var read = snapshot.MainModule.Functions.Single(m => m.Name == "Value");
            if (read.SpecialConstraints[0] != TypeParameterConstraints.ValueType)
                throw new Exception("method value constraint was not preserved");
            foreach (var expected in new[] { reference, constructible, referenceNew })
            {
                var actual = snapshot.MainModule.Functions.Single(m => m.Name == expected.Name);
                if (actual.SpecialConstraints[0] != expected.Definition.SpecialConstraints[0])
                    throw new Exception("method category combination was not preserved");
            }
            var boundedRead = snapshot.MainModule.Functions.Single(m => m.Name == "Bounded");
            if (boundedRead.SpecialConstraints[0] != TypeParameterConstraints.ValueType || boundedRead.TypeConstraints.Count != 1)
                throw new Exception("category plus interface constraint lost");
            var consumer = new AssemblyBuilder(new("Consumer", new(1, 0, 0, 0)), graph.CoreLibrary);
            var imported = consumer.ImportReference(read, graph.CoreLibrary);
            imported.MakeGenericInstance(PrimitiveType.Int32);
            Reject(() => imported.MakeGenericInstance(PrimitiveType.String));
        }
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        main.LoadConstant(42);
        main.Call(forward.MakeGenericInstance(PrimitiveType.Int32));
        main.Return();
        graph.EntryPoint = main;
        if (Environment.GetEnvironmentVariable("NEOCLR_METHOD_CATEGORY_ARTIFACT") is { } path)
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph));
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42))
            throw new Exception("CLI category forwarding failed");
        var cliValue = loaded.ManifestModule.GetMethod("Value")!;
        if (!Equals(cliValue.MakeGenericMethod(typeof(int)).Invoke(null, [42]), 42))
            throw new Exception("CLI constrained generic invocation failed");
        Reject(() => cliValue.MakeGenericMethod(typeof(string)));
        foreach (var kind in new[] { "ReferenceType", "ValueType", "FunctionType", "Unknown" })
        {
            var document = JsonNode.Parse(graph.WriteNativeAssembly())!;
            ((JsonArray)document["functions"]![0]!["generic_constraints"]!).Add(new JsonObject { ["parameter"] = 0, ["kind"] = kind });
            Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(document.ToJsonString())));
        }
        forward.SetSpecialConstraints(0, TypeParameterConstraints.None);
        Reject(() => graph.Write());
        forward.SetSpecialConstraints(0, TypeParameterConstraints.ValueType);
        owner.SetSpecialConstraints(0, TypeParameterConstraints.None);
        Reject(() => RuntimeAssemblyContainer.WriteBinary(graph));
        Console.WriteLine("PASS method category constraints, forwarding, imports and CLI/native/reference round trips");
    }

    private static MethodBuilder Identity(AssemblyBuilder graph, string name, TypeParameterConstraints flags)
    {
        var method = graph.AddFunction(name, new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]));
        method.SetSpecialConstraints(0, flags);
        method.LoadArgument(0);
        method.Return();
        return method;
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is ArgumentException or InvalidDataException or InvalidOperationException) { return; }
        throw new Exception("invalid method constraint accepted");
    }
}
