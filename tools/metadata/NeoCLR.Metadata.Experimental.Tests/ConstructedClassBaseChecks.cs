using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ConstructedClassBaseChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ConstructedClassBase", new(1, 0, 0, 0)), core);
        var parameter = SignatureType.TypeParameter(0);
        var view = graph.AddGenericInterface("Example", "View", ["T"]);
        var echoContract = view.AddInterfaceMethod("Echo", new(parameter, [parameter]));
        var parent = graph.AddGenericClass("Example", "Base", ["T"]);
        parent.AddInterfaceImplementation(view.MakeGenericInstance(parameter));
        var value = parent.AddField("Value", parameter, FieldVisibility.Public);
        var parentCtor = parent.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        parentCtor.LoadArgument(0); parentCtor.LoadArgument(1); parentCtor.StoreField(value); parentCtor.Return();
        var echo = parent.AddInstanceMethod("Echo", new(parameter, [parameter]));
        echo.LoadArgument(1); echo.Return();
        var second = SignatureType.TypeParameter(1);
        var derived = graph.AddGenericClass("Example", "Derived", ["Unused", "T"], parent.MakeGenericInstance(second));
        var count = derived.AddField("Count", PrimitiveType.Int32, FieldVisibility.Public);
        var derivedCtor = derived.AddConstructor(new MethodSignature(PrimitiveType.Void, [second, PrimitiveType.Int32]));
        derivedCtor.LoadArgument(0); derivedCtor.LoadArgument(1); derivedCtor.Call(parentCtor.MakeConstructedReference([second]));
        derivedCtor.LoadArgument(0); derivedCtor.LoadArgument(2); derivedCtor.StoreField(count); derivedCtor.Return();
        var leaf = graph.AddClass("Example", "Leaf", derived.MakeGenericInstance(PrimitiveType.Boolean, PrimitiveType.Int32));
        var leafCtor = leaf.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32]));
        leafCtor.LoadArgument(0); leafCtor.LoadArgument(1); leafCtor.LoadConstant(2);
        leafCtor.Call(derivedCtor.MakeConstructedReference([PrimitiveType.Boolean, PrimitiveType.Int32])); leafCtor.Return();
        var upcast = graph.AddFunction("Upcast", new(parent.MakeGenericInstance(PrimitiveType.Int32), [leaf]));
        upcast.LoadArgument(0); upcast.Return();
        var asView = graph.AddFunction("AsView", new(view.MakeGenericInstance(PrimitiveType.Int32), [leaf]));
        asView.LoadArgument(0); asView.Return();
        var invoke = graph.AddFunction("Invoke", new(PrimitiveType.Int32, [leaf]));
        invoke.LoadArgument(0); invoke.LoadArgument(0); invoke.LoadField(count.MakeConstructedReference(PrimitiveType.Boolean, PrimitiveType.Int32)); invoke.CallVirtual(echoContract.MakeConstructedReference([PrimitiveType.Int32])); invoke.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var local = main.DeclareLocal(leaf);
        main.LoadConstant(40); main.NewObject(leafCtor); main.StoreLocal(local);
        var referenceLocal = main.DeclareLocal(derived.MakeGenericInstance(PrimitiveType.Boolean, leaf));
        main.LoadLocal(local); main.LoadConstant(2); main.NewObject(derivedCtor.MakeConstructedReference([PrimitiveType.Boolean, leaf])); main.StoreLocal(referenceLocal);
        main.LoadLocal(referenceLocal); main.LoadField(value.MakeConstructedReference(leaf));
        main.LoadField(value.MakeConstructedReference(PrimitiveType.Int32));
        main.LoadLocal(local); main.Call(invoke); main.Add(); main.Return();
        Reject<ArgumentException>(() => graph.AddClass("Example", "Unbound", parent.MakeGenericInstance(parameter)));
        Reject<ArgumentException>(() => graph.AddGenericClass("Example", "OutOfScope", ["T"], parent.MakeGenericInstance(second)));
        Reject<ArgumentException>(() => graph.AddGenericClass("Example", "MethodScope", ["T"], parent.MakeGenericInstance(SignatureType.MethodParameter(0))));
        Reject<ArgumentException>(() => graph.AddClass("Example", "InterfaceBase", view.MakeGenericInstance(PrimitiveType.Int32)));
        Reject<ArgumentException>(() => derivedCtor.Call(parentCtor.MakeConstructedReference([parameter])));
        Reject<ArgumentException>(() => main.Call(parentCtor.MakeConstructedReference([PrimitiveType.Int32])));
        var foreign = new AssemblyBuilder(new("Foreign", new(1, 0, 0, 0)), core).AddGenericClass("Other", "Base", ["T"]);
        Reject<ArgumentException>(() => graph.AddClass("Example", "ForeignBase", foreign.MakeGenericInstance(PrimitiveType.Int32)));
        var sealedDefinition = new TypeDefinition("Example", "Closed", 0x101,
            graph.Definition.MainModule.ImportReference(core, "System", "Object"), ["T"]);
        graph.Definition.MainModule.Types.Add(sealedDefinition);
        var closed = graph.Types.Single(t => ReferenceEquals(t.Definition, sealedDefinition));
        Reject<ArgumentException>(() => graph.AddClass("Example", "SealedBase", closed.MakeGenericInstance(PrimitiveType.Int32)));
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        var baseSignature = snapshot.MainModule.Types.Single(t => t.Name == "Derived`2").BaseTypeSignature!;
        if (baseSignature.ReferencedGenericInstance?.TypeArguments.Single().TypeParameterIndex != 1)
            throw new Exception("reordered constructed base signature lost: " + baseSignature + " / " + baseSignature.GenericInstance);
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("constructed base storage failed");
        var instance = Activator.CreateInstance(loaded.GetType("Example.Leaf")!, 40);
        if (!ReferenceEquals(instance, loaded.ManifestModule.GetMethods().Single(m => m.Name == "Upcast").Invoke(null, [instance])))
            throw new Exception("constructed base upcast lost identity");
        if (!Equals(loaded.ManifestModule.GetMethods().Single(m => m.Name == "Invoke").Invoke(null, [instance]), 2))
            throw new Exception("inherited constructed interface dispatch failed");
        if (Environment.GetEnvironmentVariable("NEOCLR_CONSTRUCTED_BASE_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
        var invalid = graph.AddFunction("Invalid", new(parent.MakeGenericInstance(PrimitiveType.Boolean), [leaf]));
        invalid.LoadArgument(0); invalid.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        invalid.ClearBody(); invalid.GetILGenerator().LoadDefault(parent.MakeGenericInstance(PrimitiveType.Boolean)); invalid.Return();
        derivedCtor.ClearBody(); derivedCtor.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        var constrained = new AssemblyBuilder(new("ConstrainedBase", new(1, 0, 0, 0)), core);
        var constrainedBase = constrained.AddGenericClass("Example", "Base", ["T"]);
        constrainedBase.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
        var constrainedChild = constrained.AddGenericClass("Example", "Child", ["T"], constrainedBase.MakeGenericInstance(parameter));
        Reject<InvalidDataException>(() => constrained.WriteNativeAssembly());
        constrainedChild.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
        _ = constrained.WriteNativeAssembly();
        _ = System.Reflection.Assembly.Load(constrained.Write()).GetTypes();
        Console.WriteLine("PASS constructed class base authoring, reordered arguments, transitive storage and CLR dispatch");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
