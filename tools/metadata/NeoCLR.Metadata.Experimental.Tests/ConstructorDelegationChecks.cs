using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ConstructorDelegationChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ConstructorDelegation", new(1, 0, 0, 0)), core);
        foreach (var valueType in new[] { false, true })
        {
            var owner = valueType ? graph.AddValueType("Example", "Value") : graph.AddClass("Example", "Reference");
            var field = owner.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
            var terminal = owner.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32]));
            terminal.LoadArgument(0); terminal.LoadArgument(1); terminal.StoreField(field); terminal.Return();
            var forwarding = owner.AddConstructor(new MethodSignature(PrimitiveType.Void, []));
            forwarding.LoadArgument(0); forwarding.LoadConstant(42); forwarding.Call(terminal); forwarding.Return();
            var ordinary = owner.AddMethod("Invalid", new(PrimitiveType.Void, []));
            Reject<ArgumentException>(() => ordinary.Call(terminal));
            ordinary.Return();
        }
        var generic = graph.AddGenericClass("Example", "Box", ["T"]);
        var parameter = SignatureType.TypeParameter(0);
        var genericField = generic.AddField("Value", parameter, FieldVisibility.Public);
        var genericTerminal = generic.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter, PrimitiveType.Boolean]));
        genericTerminal.LoadArgument(0); genericTerminal.LoadArgument(1); genericTerminal.StoreField(genericField); genericTerminal.Return();
        var genericForwarding = generic.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        genericForwarding.LoadArgument(0); genericForwarding.LoadArgument(1); genericForwarding.GetILGenerator().Emit(OpCode.Ldc_Bool, true);
        genericForwarding.Call(genericTerminal.MakeConstructedReference([parameter])); genericForwarding.Return();
        Reject<ArgumentException>(() => genericForwarding.Call(genericTerminal.MakeConstructedReference([PrimitiveType.Int32])));
        var unrelated = graph.AddClass("Example", "Unrelated").AddConstructor(new MethodSignature(PrimitiveType.Void, []));
        Reject<ArgumentException>(() => unrelated.Call(genericForwarding.MakeConstructedReference([parameter])));
        unrelated.Return();
        _ = graph.WriteNativeAssembly();
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        foreach (var name in new[] { "Reference", "Value" })
        {
            var type = loaded.GetType("Example." + name)!;
            if (!Equals(type.GetField("Number")!.GetValue(Activator.CreateInstance(type)), 42))
                throw new Exception("Delegation lost initialized storage: " + name);
        }
        var box = loaded.GetType("Example.Box`1")!.MakeGenericType(typeof(int));
        if (!Equals(box.GetField("Value")!.GetValue(Activator.CreateInstance(box, 42)), 42))
            throw new Exception("Generic delegation lost storage.");
        // Calling a second constructor on the receiver must remain invalid.
        genericForwarding.ClearBody();
        for (int i = 0; i < 2; i++)
        {
            genericForwarding.LoadArgument(0); genericForwarding.LoadArgument(1); genericForwarding.GetILGenerator().Emit(OpCode.Ldc_Bool, true);
            genericForwarding.Call(genericTerminal.MakeConstructedReference([parameter]));
        }
        genericForwarding.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        Console.WriteLine("PASS class, struct and generic constructor delegation; invalid owner and repeated initialization rejection");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
