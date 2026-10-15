using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredCompositionChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new("AuthoredCompositions", new Version(1, 0, 0, 0)), core);
        var left = builder.AddInterface("Example", "Left");
        var right = builder.AddInterface("Example", "Right");
        var union = builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, left, right);
        var intersection = builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Intersection, left, right);
        if (union.Composition?.Kind != ReferenceCompositionKind.Union || intersection.Composition?.Kind != ReferenceCompositionKind.Intersection)
            throw new Exception("structural kind lost");
        if (builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, right, union) != union)
            throw new Exception("union normalization lost");
        var echo = builder.AddFunction("Echo", new MethodSignature(union, [union]));
        echo.LoadArgument(0); echo.Return();
        var widen = builder.AddFunction("Widen", new MethodSignature(union, [left]));
        widen.LoadArgument(0); widen.CastReference(union); widen.Return();
        var narrow = builder.AddFunction("Narrow", new MethodSignature(left, [union]));
        narrow.LoadArgument(0); narrow.CastReference(left); narrow.Return();
        var box = builder.AddGenericClass("Example", "Box", ["T"]);
        var genericUnion = builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union,
            box.MakeGenericInstance(left), right);
        var wrapper = box.MakeGenericInstance(union);
        var nested = builder.AddFunction("Nested", new MethodSignature(wrapper, [wrapper]));
        nested.LoadArgument(0); nested.Return();
        if (genericUnion.Composition!.Constituents.Count != 2) throw new Exception("generic union lost");
        var main = builder.AddFunction("Main");
        main.GetILGenerator().LoadTypeToken(union); main.GetILGenerator().Emit(OpCode.Pop); main.LoadConstant(42); main.Return(); builder.EntryPoint = main;
        var image = builder.WriteNativeAssembly();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteBinary(image), allowExperimentalCompositions: true);
        if (!snapshot.MainModule.Functions.Single(f => f.Name == "Echo").TryGetSignature(out var signature) || signature!.ReturnType.Composition?.Kind != ReferenceCompositionKind.Union)
            throw new Exception("native structural signature lost");
        Reject(() => builder.Write());
        var values = builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, left, PrimitiveType.Int32);
        var valueEcho = builder.AddFunction("ValueEcho", new MethodSignature(values, [values]));
        valueEcho.LoadArgument(0); valueEcho.Return();
        var replace = builder.AddFunction("Replace", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(values), values]));
        replace.LoadArgument(0); replace.LoadArgument(1);
        replace.GetILGenerator().StoreObject(values); replace.Return();
        var callbackType = SignatureType.Function(new MethodSignature(values, [values]));
        var callbackEcho = builder.AddFunction("CallbackEcho", new MethodSignature(callbackType, [callbackType]));
        callbackEcho.LoadArgument(0); callbackEcho.Return();
        var valueImage = NativeModuleContainer.WriteLibraryBinary(builder);
        var valueSnapshot = AssemblyDefinition.ReadNativeAssembly(valueImage, true);
        if (!valueSnapshot.MainModule.Functions.Single(f => f.Name == "ValueEcho").TryGetSignature(out var valueSignature) ||
            valueSignature!.ReturnType.Composition!.Constituents.All(t => t.Primitive != PrimitiveType.Int32))
            throw new Exception("scalar union alternative lost");
        if (!valueSnapshot.MainModule.Functions.Single(f => f.Name == "Replace").TryGetSignature(out var replaceSignature) ||
            replaceSignature!.ParameterTypes[0].ByReferenceElement?.Composition?.Kind != ReferenceCompositionKind.Union)
            throw new Exception("managed union parameter lost");
        if (!valueSnapshot.MainModule.Functions.Single(f => f.Name == "CallbackEcho").TryGetSignature(out var callbackSignature) ||
            callbackSignature!.ReturnType.FunctionSignature?.ReturnType.Composition?.Kind != ReferenceCompositionKind.Union)
            throw new Exception("callback union result lost");
        Reject(() => AssemblyDefinition.ReadNativeAssembly(valueImage));
        Reject(() => builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Intersection, left, PrimitiveType.Int32));
        Reject(() => builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, left, SignatureType.TypeParameter(0)));
        var foreign = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core).AddInterface("", "Other");
        Reject(() => builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, left, foreign));
        var final = builder.AddClass("", "Final"); final.SetSealedClass();
        Reject(() => builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Intersection, final, final));
        var mutable = builder.AddClass("", "Mutable");
        _ = builder.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Intersection, mutable, left);
        mutable.SetSealedClass();
        Reject(() => builder.WriteNativeAssembly());
        Console.WriteLine("PASS authored native reference compositions");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or NotSupportedException or InvalidDataException) { return; }
        throw new Exception("unsupported composition accepted");
    }
}
