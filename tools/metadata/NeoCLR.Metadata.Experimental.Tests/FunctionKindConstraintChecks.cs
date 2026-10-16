using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class FunctionKindConstraintChecks
{
    internal static void Run()
    {
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
    private static void Reject(Action action)
    {
        try { action(); } catch (ArgumentException) { return; }
        throw new Exception("expected invalid function type argument");
    }
}
