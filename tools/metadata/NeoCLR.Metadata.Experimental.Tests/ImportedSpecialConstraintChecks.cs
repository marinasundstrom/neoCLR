using System.Runtime.Loader;
using System.Security.Cryptography;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ImportedSpecialConstraintChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ConstraintLibrary", new(1, 0, 0, 0)), core);
        var t = SignatureType.TypeParameter(0);
        TypeBuilder AddBox(string name, TypeParameterConstraints flags)
        {
            var type = library.AddGenericClass("Example", name, ["T"]);
            type.SetSpecialConstraints(0, flags);
            var field = type.AddField("value", t, FieldVisibility.Private);
            var ctor = type.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
            ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(field); ctor.Return();
            var get = type.AddInstanceMethod("Get", new(t, []));
            get.LoadArgument(0); get.LoadField(field); get.Return();
            return type;
        }
        AddBox("ReferenceBox", TypeParameterConstraints.ReferenceType);
        AddBox("ValueBox", TypeParameterConstraints.ValueType | TypeParameterConstraints.DefaultConstructor);
        AddBox("ConstructorBox", TypeParameterConstraints.DefaultConstructor);
        var bound = library.AddClass("Example", "Bound");
        bound.AddConstructor([]).Return();
        AddBox("NominalBox", TypeParameterConstraints.None).AddBaseTypeConstraint(0, bound);
        var provider = RuntimeAssemblyContainer.WriteLibraryBinary(library);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(provider);
        var metadata = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]);
        var referenceParameter = (NeoCLR.Metadata.Experimental.Introspection.GenericParameterTypeInfo)metadata.Resolve(snapshot.Identity)
            .GetTypes().Single(type => type.Name == "ReferenceBox`1").GetGenericArguments().Single();
        if (referenceParameter.SpecialConstraints != TypeParameterConstraints.ReferenceType)
            throw new Exception("native introspection lost special constraints");
        var nominalParameter = (NeoCLR.Metadata.Experimental.Introspection.GenericParameterTypeInfo)metadata.Resolve(snapshot.Identity)
            .GetTypes().Single(type => type.Name == "NominalBox`1").GetGenericArguments().Single();
        if (nominalParameter.GetTypeConstraints().Single().DisplayName != "Example.Bound")
            throw new Exception("native introspection lost nominal bound");
        var hash = Convert.ToHexString(SHA256.HashData(provider));
        var app = new AssemblyBuilder(new("ConstraintApp", new(1, 0, 0, 0)), core);
        Reject<InvalidDataException>(() => app.ImportReference(snapshot.MainModule.Types.First(), core));
        var reference = app.CreateTypeReference(library.Identity, core, hash, "Example", "ReferenceBox`1", 1);
        var value = app.CreateTypeReference(library.Identity, core, hash, "Example", "ValueBox`1", 1);
        app.SetSpecialConstraints(reference, 0, TypeParameterConstraints.ReferenceType);
        app.SetSpecialConstraints(reference, 0, TypeParameterConstraints.ReferenceType);
        app.SetSpecialConstraints(value, 0, TypeParameterConstraints.ValueType | TypeParameterConstraints.DefaultConstructor);
        Reject<ArgumentException>(() => reference.MakeGenericInstance(PrimitiveType.Int32));
        Reject<ArgumentException>(() => value.MakeGenericInstance(PrimitiveType.String));
        Reject<InvalidDataException>(() => app.SetSpecialConstraints(reference, 0, TypeParameterConstraints.DefaultConstructor));
        Reject<ArgumentException>(() => app.SetSpecialConstraints(reference, 1, TypeParameterConstraints.ReferenceType));
        Reject<InvalidDataException>(() => app.SetSpecialConstraints(reference, 0, TypeParameterConstraints.ValueType));
        var constructor = app.CreateTypeReference(library.Identity, core, hash, "Example", "ConstructorBox`1", 1);
        app.SetSpecialConstraints(constructor, 0, TypeParameterConstraints.DefaultConstructor);
        var importedBound = app.CreateTypeReference(library.Identity, core, hash, "Example", "Bound", 0);
        app.DeclareDefaultConstructor(importedBound, true);
        app.DeclareDefaultConstructor(importedBound, true);
        Reject<InvalidDataException>(() => app.DeclareDefaultConstructor(importedBound, false));
        constructor.MakeGenericInstance(importedBound);
        constructor.MakeGenericInstance(PrimitiveType.Int32);
        Reject<ArgumentException>(() => constructor.MakeGenericInstance(PrimitiveType.String));
        var abstractType = app.AddClass("Example", "Abstract"); abstractType.SetAbstractClass();
        abstractType.AddConstructor([]).Return();
        Reject<ArgumentException>(() => constructor.MakeGenericInstance(abstractType));
        var nominal = app.CreateTypeReference(library.Identity, core, hash, "Example", "NominalBox`1", 1);
        app.AddBaseTypeConstraint(nominal, 0, importedBound);
        app.AddBaseTypeConstraint(nominal, 0, importedBound);
        nominal.MakeGenericInstance(importedBound);
        Reject<ArgumentException>(() => nominal.MakeGenericInstance(PrimitiveType.Int32));
        Reject<ArgumentException>(() => app.SetSpecialConstraints(nominal, 0, TypeParameterConstraints.ValueType));
        var referenceCtor = app.CreateMethodReference(reference, ".ctor", new(PrimitiveType.Void, [t]));
        var referenceGet = app.CreateMethodReference(reference, "Get", new(t, []));
        var valueCtor = app.CreateMethodReference(value, ".ctor", new(PrimitiveType.Void, [t]));
        var valueGet = app.CreateMethodReference(value, "Get", new(t, []));
        app.CompleteClassBaseReference(reference, [referenceGet], 1);
        var derived = app.AddGenericClass("Example", "Derived", ["Unused", "T"]);
        derived.SetSpecialConstraints(1, TypeParameterConstraints.ReferenceType);
        derived.SetBaseType(reference.MakeGenericInstance(SignatureType.TypeParameter(1)));
        var derivedCtor = derived.AddConstructor(new MethodSignature(PrimitiveType.Void, [SignatureType.TypeParameter(1)]));
        derivedCtor.LoadArgument(0); derivedCtor.LoadArgument(1); derivedCtor.Call(referenceCtor.MakeConstructedReference([SignatureType.TypeParameter(1)])); derivedCtor.Return();
        var payload = app.AddClass("Example", "Payload");
        var number = payload.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
        var payloadCtor = payload.AddConstructor([PrimitiveType.Int32]);
        payloadCtor.LoadArgument(0); payloadCtor.LoadArgument(1); payloadCtor.StoreField(number); payloadCtor.Return();
        if (reference.MakeGenericInstance(payload).SpecialConstraints[0] != TypeParameterConstraints.ReferenceType)
            throw new Exception("constructed reference lost constraint metadata");
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.LoadConstant(40); main.NewObject(payloadCtor); main.NewObject(derivedCtor.MakeConstructedReference([PrimitiveType.Boolean, payload]));
        main.Call(referenceGet.MakeConstructedReference([payload])); main.LoadField(number);
        main.LoadConstant(2); main.NewObject(valueCtor.MakeConstructedReference([PrimitiveType.Int32]));
        main.Call(valueGet.MakeConstructedReference([PrimitiveType.Int32])); main.Add(); main.Return();
        var image = RuntimeAssemblyContainer.WriteBinary(app);
        var context = new AssemblyLoadContext("constraint-imports", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("constrained dependency CLR execution failed");
        }
        finally { context.Unload(); }
        if (Environment.GetEnvironmentVariable("NEOCLR_CONSTRAINT_IMPORT_OUTPUT") is { } output)
        {
            Directory.CreateDirectory(output);
            File.WriteAllBytes(Path.Combine(output, "Library.dll"), provider);
            File.WriteAllBytes(Path.Combine(output, "App.dll"), image);
        }
        derived.SetSpecialConstraints(1, TypeParameterConstraints.None);
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        var late = new AssemblyBuilder(new("LateConstraints", new(1, 0, 0, 0)), core);
        var lateType = late.CreateTypeReference(library.Identity, core, hash, "Example", "ReferenceBox`1", 1);
        var stale = lateType.MakeGenericInstance(PrimitiveType.Int32);
        var identity = late.AddFunction("Identity", new(stale, [stale]));
        identity.LoadArgument(0); identity.Return();
        late.SetSpecialConstraints(lateType, 0, TypeParameterConstraints.ReferenceType);
        Reject<InvalidDataException>(() => late.WriteNativeAssembly());
        Console.WriteLine("PASS imported special/nominal constraints, constructor facts, symbolic base proofs, late checks and CLR execution");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
