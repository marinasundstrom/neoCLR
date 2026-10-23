using System.Reflection;
using System.Security.Cryptography;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class InterfaceTypeConstraintChecks
{
    internal static void Run()
    {
        TypeConstraintChecks.Run();
        var core = typeof(object).Assembly.GetName();
        var coreIdentity = new AssemblyIdentity(core.Name!, core.Version!, core.CultureName ?? "", Convert.ToHexString(core.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("InterfaceTypeBounds", new(1, 0, 0, 0)), coreIdentity);
        var contract = graph.AddInterface("Example", "Failure");
        var reference = graph.AddClass("Example", "ReferenceError");
        reference.AddInterfaceImplementation(contract);
        reference.AddConstructor([]).Return();
        var value = graph.AddValueType("Example", "ValueError");
        value.AddInterfaceImplementation(contract);
        var owner = graph.AddGenericClass("Example", "Context", ["E"]);
        owner.AddBaseTypeConstraint(0, contract);
        owner.MakeGenericInstance(reference);
        owner.MakeGenericInstance(value);
        owner.MakeGenericInstance(contract);
        Reject(() => owner.MakeGenericInstance(PrimitiveType.Int32));
        var valueOwner = graph.AddGenericClass("Example", "ValueContext", ["E"]);
        valueOwner.SetSpecialConstraints(0, TypeParameterConstraints.ValueType);
        valueOwner.AddBaseTypeConstraint(0, contract);
        valueOwner.MakeGenericInstance(value);
        Reject(() => valueOwner.MakeGenericInstance(reference));
        var constrainedMethod = graph.AddFunction("BoundAnswer", new(PrimitiveType.Int32, [], ["T"]));
        constrainedMethod.AddInterfaceConstraint(0, contract);
        constrainedMethod.GetILGenerator().LoadConstant(42);
        constrainedMethod.GetILGenerator().Return();
        var forward = owner.AddMethod("Forward", new(PrimitiveType.Int32, []));
        forward.GetILGenerator().Call(constrainedMethod.MakeGenericInstance(SignatureType.TypeParameter(0)));
        forward.GetILGenerator().Return();
        var unboundedOwner = graph.AddGenericClass("Example", "Unbounded", ["T"]);
        var invalidForward = unboundedOwner.AddMethod("Forward", new(PrimitiveType.Int32, []));
        Reject(() => invalidForward.GetILGenerator().Call(constrainedMethod.MakeGenericInstance(SignatureType.TypeParameter(0))));
        invalidForward.GetILGenerator().LoadConstant(0);
        invalidForward.GetILGenerator().Return();
        var loaded = Assembly.Load(graph.Write());
        var context = loaded.GetType("Example.Context`1")!;
        if (!context.GetGenericArguments()[0].GetGenericParameterConstraints().Single().IsInterface)
            throw new Exception("CLI interface type bound lost");
        var closedContext = context.MakeGenericType(loaded.GetType("Example.ValueError")!);
        if ((int)closedContext.GetMethod("Forward")!.Invoke(null, null)! != 42)
            throw new Exception("Owner interface bound was not forwarded to generic method");
        Reject(() => context.MakeGenericType(typeof(int)));
        var native = graph.WriteNativeAssembly();
        var snapshot = NativeAssemblyDefinition.ReadAssembly(native);
        _ = snapshot.CreateReferenceAssembly(coreIdentity);
        var consumer = new AssemblyBuilder(new("InterfaceBoundConsumer", new(1, 0, 0, 0)), coreIdentity);
        var hash = Convert.ToHexString(SHA256.HashData(native));
        var importedContract = consumer.CreateInterfaceReference(graph.Identity, coreIdentity, hash, "Example", "Failure", 0);
        var importedOwner = consumer.CreateTypeReference(graph.Identity, coreIdentity, hash, "Example", "Context`1", 1);
        consumer.AddBaseTypeConstraint(importedOwner, 0, importedContract);
        var localValue = consumer.AddValueType("Example", "LocalError");
        localValue.AddInterfaceImplementation(importedContract);
        importedOwner.MakeGenericInstance(localValue);
        Reject(() => importedOwner.MakeGenericInstance(PrimitiveType.Int32));
        var loadedNative = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(native, coreIdentity));
        var importing = new AssemblyBuilder(new("LoadedInterfaceBounds", new(1, 0, 0, 0)), coreIdentity);
        var loadedOwner = importing.ImportReference(loadedNative.MainModule.Types.Single(t => t.Name == "Context`1"), coreIdentity);
        var loadedValue = importing.ImportReference(loadedNative.MainModule.Types.Single(t => t.Name == "ValueError"), coreIdentity);
        loadedOwner.MakeGenericInstance(loadedValue);
        Reject(() => loadedOwner.MakeGenericInstance(PrimitiveType.Int32));
        // Interface conformance never proves a reference-type constraint.
        var referenceOnly = graph.AddGenericClass("Example", "ReferenceOnly", ["T"]);
        referenceOnly.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
        var invalidBase = graph.AddGenericClass("Example", "InvalidBase", ["T"]);
        invalidBase.AddBaseTypeConstraint(0, contract);
        invalidBase.SetBaseType(referenceOnly.MakeGenericInstance(SignatureType.TypeParameter(0)));
        Reject(() => graph.Write());
        Console.WriteLine("PASS local and imported interface type constraints preserve value conformance and reject unrelated arguments");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (ArgumentException) { return; }
        catch (InvalidDataException) { return; }
        throw new Exception("Invalid constraint accepted");
    }
}
