using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class DependentFunctionSignatureChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("DependentFunctions", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var owner = graph.AddGenericClass("Example", "Forwarder", ["T"]);
        owner.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        var signature = MethodSignature.FromFunctionType(SignatureType.TypeParameter(0));
        owner.SetAbstractClass();
        var method = owner.AddAbstractMethod("Invoke", signature);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(graph));
        var context = new MetadataLoadContext([snapshot]);
        var definition = context.Resolve(graph.Identity).GetTypes().Single(t => t.Name == "Forwarder`1");
        var open = definition.GetMethods().Single();
        if (open.FunctionType is not GenericParameterTypeInfo { SpecialConstraints: TypeParameterConstraints.FunctionType })
            throw new Exception("open method lost its constrained function parameter");
        Expect<InvalidOperationException>(() => open.GetParameters());
        Expect<InvalidOperationException>(() => _ = open.ReturnType);
        foreach (var shape in new[] {
            SignatureType.Function(new(PrimitiveType.Int32, Array.Empty<SignatureType>())),
            SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32])),
            SignatureType.Function(new(PrimitiveType.Void, [PrimitiveType.Int32, PrimitiveType.Boolean])) })
        {
            var closed = definition.MakeGenericType(context.ResolveSignature(shape)).GetMethods().Single();
            if (closed.FunctionType is not FunctionTypeInfo function ||
                closed.GetParameters().Count != shape.FunctionSignature!.ParameterTypes.Count ||
                !ReferenceEquals(closed.ReturnType, function.ReturnType))
                throw new Exception("closed method did not materialize its full signature");
            var reference = method.MakeConstructedReference([shape]);
            if (reference.Signature.ParameterTypes.Count != shape.FunctionSignature.ParameterTypes.Count)
                throw new Exception("constructed reference retained open arity");
        }
        var projectedGraph = new AssemblyBuilder(new("ProjectedResults", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var projector = projectedGraph.AddGenericClass("Example", "Projector", ["T"]);
        projector.SetSpecialConstraints(0, TypeParameterConstraints.FunctionType);
        projector.SetAbstractClass();
        var projectedResult = SignatureType.FunctionReturn(SignatureType.TypeParameter(0));
        var projectedMethod = projector.AddAbstractMethod("Result", new MethodSignature(projectedResult, []));
        var projectedSnapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(projectedGraph));
        var projectedContext = new MetadataLoadContext([projectedSnapshot]);
        var projectedOwner = projectedContext.Resolve(projectedGraph.Identity).GetTypes().Single();
        if (projectedOwner.GetMethods().Single().ReturnType is not FunctionReturnTypeInfo)
            throw new Exception("open result projection was erased");
        var boolFunction = SignatureType.Function(new(PrimitiveType.Boolean, [PrimitiveType.Int32]));
        var projectedClosed = projectedOwner.MakeGenericType(projectedContext.ResolveSignature(boolFunction)).GetMethods().Single();
        if (projectedClosed.ReturnType is not PrimitiveTypeInfo { Kind: PrimitiveType.Boolean } ||
            projectedMethod.MakeConstructedReference([boolFunction]).Signature.ReturnType != (SignatureType)PrimitiveType.Boolean)
            throw new Exception("result projection did not substitute");
        // Imported open contracts must consume the materialized arity, not the
        // boxed-body transport's placeholder arity.
        var consumer = new AssemblyBuilder(new("DependentConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var importedOwner = consumer.CreateTypeReference(graph.Identity, graph.CoreLibrary, new string('0', 64), "Example", "Forwarder`1", 1);
        consumer.SetSpecialConstraints(importedOwner, 0, TypeParameterConstraints.FunctionType);
        var importedMethod = consumer.CreateMethodReference(importedOwner, "Invoke", signature, isStatic: false);
        var unary = SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var call = consumer.AddFunction("Call", new(PrimitiveType.Int32, [importedOwner.MakeGenericInstance(unary), PrimitiveType.Int32]));
        call.LoadArgument(0);
        call.LoadArgument(1);
        call.Call(importedMethod.MakeConstructedReference([unary]));
        call.Return();
        _ = NativeModuleContainer.WriteLibraryBinary(consumer);
        Expect<ArgumentException>(() => SignatureType.FunctionReturn(PrimitiveType.Int32));
        Expect<ArgumentException>(() => MethodSignature.FromFunctionType(PrimitiveType.Int32));
        Console.WriteLine("PASS dependent function signature authoring and generic reflection materialization");
    }

    private static void Expect<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
