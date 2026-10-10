using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class FunctionViewChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("Callbacks", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var box = graph.AddGenericClass("Example", "Box", ["T"]);
        var callback = SignatureType.Function(new(PrimitiveType.Boolean, [SignatureType.TypeParameter(0)]));
        var echo = box.AddMethod("Echo", new(callback, [callback]));
        echo.GetILGenerator().LoadArgument(0); echo.GetILGenerator().Return();
        var methodCallback = SignatureType.Function(new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)]));
        var generic = graph.AddFunction("Transform", new(methodCallback, [methodCallback], ["U"]));
        generic.GetILGenerator().LoadArgument(0); generic.GetILGenerator().Return();
        var externalGraph = new AssemblyBuilder(new("Payloads", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        externalGraph.AddClass("Example", "Payload");
        var externalSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(externalGraph));
        var externalType = graph.ImportReference(externalSnapshot.MainModule.Types.Single(t => t.Name == "Payload"), graph.CoreLibrary);
        var externalCallback = SignatureType.Function(new(externalType, [externalType]));
        var externalEcho = graph.AddFunction("External", new(externalCallback, [externalCallback]));
        externalEcho.GetILGenerator().LoadArgument(0); externalEcho.GetILGenerator().Return();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var context = new MetadataLoadContext([snapshot, externalSnapshot]);
        var open = context.Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Box`1");
        var method = open.GetMethods().Single();
        var function = (FunctionTypeInfo)method.ReturnType;
        Check(!function.NoResult && !function.IsNominalType, "function category");
        Check(ReferenceEquals(function, method.GetParameters().Single().ParameterType), "canonical repeated signature");
        Check(ReferenceEquals(function.ParameterTypes.Single(), open.GetGenericArguments().Single()), "open owner parameter");
        var integer = context.ResolveSignature(PrimitiveType.Int32);
        var closed = open.MakeGenericType(integer).GetMethods().Single();
        var constructed = (FunctionTypeInfo)closed.ReturnType;
        Check(ReferenceEquals(integer, constructed.ParameterTypes.Single()), "constructed parameter substitution");
        Check(ReferenceEquals(constructed, closed.GetParameters().Single().ParameterType), "constructed canonical signature");
        var genericView = context.Resolve(snapshot.Identity).GetModules().SelectMany(module => module.GetFunctions()).Single(m => m.Name == "Transform");
        var openMethodFunction = (FunctionTypeInfo)genericView.ReturnType;
        Check(ReferenceEquals(openMethodFunction.ReturnType, genericView.GetGenericArguments().Single()), "method parameter scope");
        var closedMethodFunction = (FunctionTypeInfo)genericView.MakeGenericMethod(integer).ReturnType;
        Check(ReferenceEquals(closedMethodFunction.ReturnType, integer) && ReferenceEquals(closedMethodFunction.ParameterTypes.Single(), integer), "method substitution");
        var externalView = (FunctionTypeInfo)context.Resolve(snapshot.Identity).GetModules().SelectMany(module => module.GetFunctions()).Single(m => m.Name == "External").ReturnType;
        Check(ReferenceEquals(externalView.ReturnType, context.Resolve(externalSnapshot.Identity).GetTypes().Single(t => t.Name == "Payload")), "external identity inside callback");
        var missing = new MetadataLoadContext([snapshot]);
        try { _ = missing.Resolve(snapshot.Identity).GetModules().SelectMany(module => module.GetFunctions()).Single(m => m.Name == "External").ReturnType; throw new Exception("missing callback dependency admitted"); } catch (InvalidDataException) { }
        var noResult = (FunctionTypeInfo)context.ResolveSignature(SignatureType.Function(new(PrimitiveType.Void, [])));
        Check(noResult.NoResult && noResult.ParameterTypes.Count == 0, "explicit no-result");
        Check(ReferenceEquals(noResult, context.ResolveSignature(SignatureType.Function(new(PrimitiveType.Void, [])))), "canonical no-result shape");
        Check(!ReferenceEquals(noResult, context.ResolveSignature(SignatureType.Function(new(PrimitiveType.Int32, [])))), "result is part of function identity");
        Check(ReferenceEquals(constructed, context.ResolveSignature(SignatureType.Function(new(PrimitiveType.Boolean, [PrimitiveType.Int32])))), "independently authored equal shape");
        var other = new MetadataLoadContext([snapshot, externalSnapshot]);
        Check(!ReferenceEquals(constructed, other.ResolveSignature(SignatureType.Function(new(PrimitiveType.Boolean, [PrimitiveType.Int32])))), "context isolation");
        try { context.ResolveSignature(callback); throw new Exception("unscoped parameter admitted"); } catch (InvalidDataException) { }
    }
    private static void Check(bool value, string detail) { if (!value) throw new Exception(detail); }
}
