using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class FunctionKindConstraintChecks
{
    internal static void Run()
    {
        FunctionTypeTokens();
        MethodConstraints();
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var assembly = new AssemblyBuilder(new("FunctionConstraint", new(1, 0, 0, 0)), core);
        var signal = assembly.AddGenericClass("Example", "Signal", ["T"]);
        signal.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        var function = SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        signal.MakeGenericInstance(function);
        Reject(() => signal.MakeGenericInstance(PrimitiveType.Int32));
        var invokable = assembly.AddClass("Example", "InvokeObject");
        var invoke = invokable.AddInstanceMethod("Invoke", new(PrimitiveType.Int32, []));
        invoke.LoadConstant(42); invoke.Return();
        Reject(() => signal.MakeGenericInstance(invokable));
        var nominal = assembly.CreateTypeReference(new("NominalDelegates", new(1, 0, 0, 0)), core, new string('0', 64), "System", "Func`2", 2);
        Reject(() => signal.MakeGenericInstance(nominal.MakeGenericInstance(PrimitiveType.Int32, PrimitiveType.Int32)));
        Reject(() => signal.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType | TypeParameterConstraints.ReferenceType));
        var image = NativeModuleContainer.WriteLibraryBinary(assembly);
        var consumer = new AssemblyBuilder(new("FunctionConsumer", new(1, 0, 0, 0)), core);
        var imported = consumer.CreateTypeReference(assembly.Identity, core,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "Example", "Signal`1", 1);
        consumer.SetSpecialConstraints(imported, 0, TypeParameterConstraints.FunctionType);
        imported.MakeGenericInstance(function);
        Reject(() => imported.MakeGenericInstance(PrimitiveType.Int32));
        var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
        var context = new MetadataLoadContext([snapshot]);
        var parameter = (GenericParameterTypeInfo)context.Resolve(snapshot.Identity).GetTypes()
            .Single(t => t.Name == "Signal`1").GetGenericArguments().Single();
        if (parameter.SpecialConstraints != TypeParameterConstraints.FunctionType) throw new Exception("lost function constraint");
        try { assembly.Write(); throw new Exception("CLI export erased native constraint"); }
        catch (InvalidDataException) { }
        Console.WriteLine("PASS native function-kind constraint and introspection");
    }
    private static void FunctionTypeTokens()
    {
        var graph = new AssemblyBuilder(new("FunctionTokens", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var shape = SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var token = graph.AddFunction("Token", new(PrimitiveType.RuntimeTypeHandle, []));
        token.GetILGenerator().LoadTypeToken(shape); token.Return();
        var json = System.Text.Json.Nodes.JsonNode.Parse(graph.WriteNativeAssembly())!;
        var instruction = json["functions"]![0]!["body"]![0]!;
        if (instruction["op"]!.ToString() != "ldtoken" || instruction["arg"]!["Function"] is null)
            throw new Exception("native function token lost its structural signature");
        AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(graph));
        Expect<InvalidDataException>(() => graph.Write());
    }

    private static void MethodConstraints()
    {
        var cliSnapshot = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(FunctionKindConstraintChecks).Assembly.Location), expectedExtended: false);
        var cliMethod = cliSnapshot.MainModule.Types.Single(t => t.Name == nameof(FunctionKindConstraintChecks)).Methods.Single(m => m.Name == nameof(ReferenceOnly));
        Expect<InvalidDataException>(() => _ = cliMethod.SpecialConstraints);
        var graph = new AssemblyBuilder(new("FunctionMethods", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var method = graph.AddFunction("Keep", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["F"]));
        method.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        method.LoadArgument(0); method.Return();
        var function = SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        method.MakeGenericInstance(function);
        Reject(() => method.MakeGenericInstance(PrimitiveType.Int32));
        Reject(() => method.MakeGenericInstance(graph.AddClass("Example", "NotFunction")));
        Reject(() => method.SetSpecialConstraints(1, TypeParameterConstraints.FunctionType));
        Reject(() => method.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType));
        var contract = graph.AddInterface("Example", "Contract");
        Reject(() => method.AddInterfaceConstraint(0, contract));
        method.SetSpecialConstraints(0, TypeParameterConstraints.None);
        method.AddInterfaceConstraint(0, contract);
        Reject(() => method.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType));
        var keep = graph.AddFunction("Identity", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["F"]));
        keep.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        keep.LoadArgument(0); keep.Return();
        var forward = graph.AddFunction("Forward", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["F"]));
        forward.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        forward.LoadArgument(0); forward.GetILGenerator().Call(keep.MakeGenericInstance(SignatureType.MethodParameter(0))); forward.Return();
        var unbounded = graph.AddFunction("Unbounded", new(PrimitiveType.Int32, [], ["F"]));
        Reject(() => unbounded.GetILGenerator().Call(keep.MakeGenericInstance(SignatureType.MethodParameter(0))));
        unbounded.LoadConstant(0); unbounded.Return();
        var native = NativeModuleContainer.WriteLibraryBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        var definition = snapshot.MainModule.Functions.Single(m => m.Name == "Identity");
        var context = new MetadataLoadContext([snapshot]);
        var parameter = (MethodGenericParameterTypeInfo)context.Resolve(definition).GetGenericArguments().Single();
        if (parameter.SpecialConstraints != TypeParameterConstraints.FunctionType || parameter.GetTypeConstraints().Count != 0)
            throw new Exception("lost method category constraint");
        Expect<InvalidDataException>(() => parameter.GetInterfaceConstraints());
        Expect<InvalidOperationException>(() => definition.SetSpecialConstraints(0, TypeParameterConstraints.None));
        Expect<InvalidDataException>(() => graph.Write());
        Expect<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary));
        var consumer = new AssemblyBuilder(new("MethodConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var imported = consumer.ImportReference(definition, graph.CoreLibrary);
        imported.MakeGenericInstance(function);
        Reject(() => imported.MakeGenericInstance(PrimitiveType.Int32));
        var forwardedImport = imported.MakeGenericInstance(SignatureType.MethodParameter(0));
        var importedCaller = consumer.AddFunction("Forward", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["F"]));
        importedCaller.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        importedCaller.LoadArgument(0); importedCaller.Call(forwardedImport); importedCaller.Return();
        NativeModuleContainer.WriteLibraryBinary(consumer);
        importedCaller.SetSpecialConstraints(0, TypeParameterConstraints.None);
        Expect<InvalidDataException>(() => NativeModuleContainer.WriteLibraryBinary(consumer));
        var json = graph.WriteNativeAssembly();
        foreach (var invalid in new[] { "duplicate", "ordinal", "category", "mixed" })
        {
            var tree = System.Text.Json.Nodes.JsonNode.Parse(json)!;
            var constraints = tree["functions"]!.AsArray().First(m => m!["generic_constraints"]?.AsArray().Any(c => c!["kind"]?.ToString() == "FunctionType") == true)!["generic_constraints"]!.AsArray();
            if (invalid == "duplicate") constraints.Add(constraints[0]!.DeepClone());
            else if (invalid == "ordinal") constraints[0]!["parameter"] = 10;
            else if (invalid == "category") constraints[0]!["kind"] = "ReferenceType";
            else constraints.Add(new System.Text.Json.Nodes.JsonObject { ["parameter"] = 0, ["kind"] = new System.Text.Json.Nodes.JsonObject { ["TypeBound"] = new System.Text.Json.Nodes.JsonObject { ["Named"] = "Example.Contract" } } });
            Expect<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(tree.ToJsonString())));
        }
        Console.WriteLine("PASS native method function-kind authoring, forwarding, import, introspection and CLI rejection");
    }
    private static T ReferenceOnly<T>(T value) where T : class => value;

    private static void Expect<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (ArgumentException) { return; }
        throw new Exception("expected invalid function type argument");
    }
}
