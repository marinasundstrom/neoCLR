using System.Diagnostics;
using System.Text.Json;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using TypeDefinition = NeoCLR.Metadata.Experimental.Model.TypeDefinition;
using FieldDefinition = NeoCLR.Metadata.Experimental.Model.FieldDefinition;

internal static class AuthoredDefinitionChecks
{
    internal static AssemblyDefinition Create()
    {
        var coreName = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(coreName.Name!, coreName.Version!, "", Convert.ToHexString(coreName.GetPublicKeyToken()!));
        var assembly = AssemblyDefinition.CreateAssembly(new("ManualStruct", new Version(1, 0, 0, 0)), core);
        var module = assembly.MainModule;
        var valueBase = module.ImportReference(core, "System", "ValueType");
        var type = new TypeDefinition("Example", "MyStruct", (uint)(TypeAttributes.Public | TypeAttributes.SequentialLayout | TypeAttributes.Sealed), valueBase);
        var field = new FieldDefinition("Original", (ushort)FieldAttributes.Public, PrimitiveType.Int32);
        type.Fields.Add(field);
        module.Types.Add(type);
        field.Name = "MyField";
        return assembly;
    }
    internal static void Run()
    {
        var executable = Executable();
        var executionContext = new AssemblyLoadContext("manual-execution", isCollectible: true);
        try
        {
            var loadedExecutable = executionContext.LoadFromStream(new MemoryStream(executable.Write()));
            var parameter = loadedExecutable.GetType("Example.Identity`1")!.GetGenericArguments().Single();
            if (parameter.Name != "T" || (parameter.GenericParameterAttributes & GenericParameterAttributes.NotNullableValueTypeConstraint) == 0)
                throw new Exception("manual generic parameter/constraint metadata");
            var boxType = loadedExecutable.GetType("Example.Box")!;
            var box = Activator.CreateInstance(boxType, new object[] { 42 });
            if (!Equals(boxType.GetProperty("Value")!.GetValue(box), 42)) throw new Exception("manual property reflection");
            if ((int)loadedExecutable.EntryPoint!.Invoke(null, null)! != 42)
                throw new Exception("manual definition CLR execution");
        }
        finally { executionContext.Unload(); }
        var assembly = Create();
        var type = assembly.MainModule.Types.Single(); var field = type.Fields.Single();
        var builder = AssemblyBuilder.ForDefinition(assembly);
        if (!ReferenceEquals(assembly.MainModule.Fields.Single(), field)) throw new Exception("module field view");
        var entry = executable.EntryPoint!;
        if (!ReferenceEquals(entry, executable.MainModule.Functions.Single(m => m.Name == "Main")) ||
            !ReferenceEquals(entry, executable.MainModule.Methods.Single(m => m.Name == "Main")) || entry.DeclaringType is not null ||
            !ReferenceEquals(entry.AuthoredSignature, AssemblyBuilder.ForDefinition(executable).EntryPoint!.Signature))
            throw new Exception("authored function definition identity");
        Reject<InvalidOperationException>(() => entry.GetSignature());
        Reject<ArgumentException>(() => assembly.EntryPoint = entry);
        Reject<NotSupportedException>(() => executable.MainModule.Functions.Clear());
        var writtenExecutable = AssemblyDefinition.ReadAssembly(executable.Write(), false);
        foreach (var declaration in executable.MainModule.Types.SelectMany(t => t.Methods))
        {
            var physical = writtenExecutable.MainModule.Methods.Single(m => m.Name == declaration.Name && m.DeclaringType?.Name == declaration.DeclaringType?.Name);
            if (physical.Attributes != declaration.Attributes || physical.IsStatic != declaration.IsStatic)
                throw new Exception("instance/constructor CLI flags differ");
        }
        var abstractBody = MethodBuilder.ForDefinition(executable.MainModule.Types.Single(t => t.Name == "IRead").Methods.Single());
        abstractBody.Return();
        Reject<InvalidDataException>(() => executable.Write());
        Reject<InvalidDataException>(() => executable.WriteNativeAssembly());
        abstractBody.ClearBody();
        var writtenEntry = writtenExecutable.EntryPoint!;
        Reject<NotSupportedException>(() => _ = writtenEntry.Body);
        var definitionBody = executable.EntryPoint!.Body;
        var emitter = MethodBuilder.ForDefinition(executable.EntryPoint);
        if (!ReferenceEquals(definitionBody.Method, executable.EntryPoint) || !ReferenceEquals(definitionBody.Locals, emitter.Locals))
            throw new Exception("canonical body/local identity");
        var savedLocals = definitionBody.Locals.ToArray();
        var destination = emitter.DefineLabel();
        if (!ReferenceEquals(destination, definitionBody.Labels.Single())) throw new Exception("canonical label identity");
        definitionBody.ClearInstructions();
        emitter.LoadConstant(42); emitter.Emit(OpCode.Br, destination); emitter.MarkLabel(destination); emitter.Return();
        if (!savedLocals.SequenceEqual(definitionBody.Locals)) throw new Exception("body clear lost local identity");
        var rewrittenContext = new AssemblyLoadContext("definition-body-rewrite", isCollectible: true);
        try
        {
            if ((int)rewrittenContext.LoadFromStream(new MemoryStream(executable.Write())).EntryPoint!.Invoke(null, null)! != 42)
                throw new Exception("definition body rewrite execution");
        }
        finally { rewrittenContext.Unload(); }
        if (entry.Attributes != writtenEntry.Attributes || entry.Name != writtenEntry.Name) throw new Exception("method declaration round trip");
        if (!ReferenceEquals(builder.Definition, assembly) || !ReferenceEquals(builder.Types[0].Definition, type) ||
            !ReferenceEquals(builder.Types[0].Fields[0].Definition, field) || builder.Types[0].Fields[0].Name != "MyField" ||
            !ReferenceEquals(type.ToReference().Resolve(), type) || !ReferenceEquals(field.DeclaringType, type))
            throw new Exception("definition/facade identity");
        var second = builder.Types[0].AddField("Second", PrimitiveType.Int64, FieldVisibility.Public);
        if (!ReferenceEquals(type.Fields[1], second.Definition)) throw new Exception("builder adds canonical definition");
        second.Definition.Name = "Renamed";
        Reject<ArgumentException>(() => second.Definition.Name = "MyField");
        Reject<ArgumentException>(() => assembly.MainModule.Types.Add(type));
        Reject<ArgumentException>(() => type.Fields.Add(field));
        Reject<NotSupportedException>(() => type.Fields.Remove(field));
        var foreign = AssemblyDefinition.CreateAssembly(new("Other", new Version(1, 0, 0, 0)), builder.CoreLibrary);
        Reject<ArgumentException>(() => foreign.MainModule.Types.Add(type));
        var scoped = new MethodDefinition("Scoped", new MethodSignature(builder.Types[0], Array.Empty<SignatureType>()));
        Reject<ArgumentException>(() => foreign.MainModule.Functions.Add(scoped));
        Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(scoped));
        assembly.MainModule.Functions.Add(scoped);
        var scopedBody = MethodBuilder.ForDefinition(scoped);
        scopedBody.LoadDefault(builder.Types[0]); scopedBody.Return();
        Reject<ArgumentException>(() => assembly.MainModule.Functions.Add(new MethodDefinition("Scoped", scoped.AuthoredSignature!)));
        var method = builder.Types[0].AddMethod("get_Answer");
        method.LoadConstant(42); method.Return();
        builder.Types[0].AddProperty("Answer", PrimitiveType.Int32, getter: method);
        var contract = builder.AddInterface("Example", "IAnswer");
        var required = contract.AddInterfaceMethod("get_Answer", PrimitiveMethodSignature.Int32(0, true));
        contract.AddProperty("Answer", PrimitiveType.Int32, getter: required);
        if (!ReferenceEquals(type.Methods.Single(), method.Definition) ||
            !ReferenceEquals(method.Definition.DeclaringType, type) ||
            !ReferenceEquals(method.Definition.Module, assembly.MainModule)) throw new Exception("type method definition identity");
        var direct = new MethodDefinition("PrivateAnswer", (ushort)(MethodAttributes.Private | MethodAttributes.Static), PrimitiveMethodSignature.Int32(0, true));
        var foreignType = AssemblyBuilder.ForDefinition(foreign).AddType("Other", "Owner");
        var detachedType = new TypeDefinition("Example", "Detached", 0x181, assembly.MainModule.ImportReference(builder.CoreLibrary, "System", "Object"));
        Reject<InvalidOperationException>(() => detachedType.Methods.Add(direct));
        type.Methods.Add(direct);
        Reject<ArgumentException>(() => foreignType.Definition.Methods.Add(direct));
        Reject<NotSupportedException>(() => type.Methods.Clear());
        MethodBuilder.ForDefinition(direct).LoadConstant(42); MethodBuilder.ForDefinition(direct).Return();
        Reject<InvalidOperationException>(() => type.Methods.Add(new MethodDefinition("Instance", (ushort)MethodAttributes.Public, PrimitiveMethodSignature.Int32(0, true))));
        Reject<ArgumentException>(() => new MethodDefinition(".cctor", (ushort)(MethodAttributes.Public | MethodAttributes.Static), PrimitiveMethodSignature.Int32(0, true)));
        var image = assembly.Write();
        if (!image.SequenceEqual(builder.Write())) throw new Exception("facade writer differs");
        var context = new AssemblyLoadContext("manual-definitions", isCollectible: true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(image)).GetType("Example.MyStruct")!;
            if (!loaded.IsValueType || loaded.GetField("MyField")!.FieldType != typeof(int) || loaded.GetField("Renamed")!.FieldType != typeof(long))
                throw new Exception("manual CLI shape");
        }
        finally { context.Unload(); }
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        foreach (var declared in assembly.MainModule.Methods)
        {
            var physical = snapshot.MainModule.Methods.Single(m => m.Name == declared.Name && m.DeclaringType?.Name == declared.DeclaringType?.Name);
            if (declared.Attributes != physical.Attributes || declared.GenericArity != physical.GenericArity || declared.IsStatic != physical.IsStatic)
                throw new Exception("authored method flags differ from encoding");
        }
        Reject<InvalidOperationException>(() => snapshot.EntryPoint = null);
        Reject<InvalidOperationException>(() => snapshot.MainModule.Functions.Add(new MethodDefinition("New", PrimitiveMethodSignature.Int32(0, true))));
        Reject<InvalidOperationException>(() => snapshot.MainModule.Types.Add(type));
        Reject<InvalidOperationException>(() => snapshot.MainModule.Types.Single(t => t.Name == "MyStruct").Fields[0].Name = "Changed");
        Reject<InvalidOperationException>(() => AssemblyBuilder.ForDefinition(snapshot));
        Reject<NotSupportedException>(() => _ = snapshot.MainModule.Types.Single(t => t.Name == "MyStruct").SpecialConstraints);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), builder.CoreLibrary));
        if (!projection.MainModule.Types.Single(t => t.Name == "MyStruct").IsValueType) throw new Exception("native category");
    }
    private static AssemblyDefinition Executable()
    {
        var assembly = Create(); var builder = AssemblyBuilder.ForDefinition(assembly);
        var type = builder.Types.Single(); var field = type.Fields.Single();
        var helperDefinition = new MethodDefinition("Answer", PrimitiveMethodSignature.Int32(0, true), @namespace: "Example");
        Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(helperDefinition));
        assembly.MainModule.Functions.Add(helperDefinition);
        Reject<ArgumentException>(() => assembly.MainModule.Functions.Add(helperDefinition));
        var helper = MethodBuilder.ForDefinition(helperDefinition);
        var answer = new MethodDefinition("Answer", (ushort)(MethodAttributes.Public | MethodAttributes.Static | MethodAttributes.HideBySig), PrimitiveMethodSignature.Int32(0, true));
        Reject<ArgumentException>(() => assembly.MainModule.Functions.Add(answer));
        type.Definition.Methods.Add(answer);
        Reject<ArgumentException>(() => type.Definition.Methods.Add(answer));
        Reject<ArgumentException>(() => type.Definition.Methods.Add(helperDefinition));
        var answerBody = MethodBuilder.ForDefinition(answer);
        answerBody.LoadConstant(-1); answerBody.Return();
        answer.Body.ClearInstructions();
        if (!ReferenceEquals(answer.DeclaringType, type.Definition) || answer.Namespace != type.Namespace || !ReferenceEquals(answerBody, type.Methods.Single()))
            throw new Exception("manual type-method identity");
        var objectType = new TypeDefinition("Example", "Box", 1, assembly.MainModule.ImportReference(builder.CoreLibrary, "System", "Object"));
        var value = new FieldDefinition("value", (ushort)(FieldAttributes.Private | FieldAttributes.InitOnly), PrimitiveType.Int32);
        objectType.Fields.Add(value); assembly.MainModule.Types.Add(objectType);
        var constructor = new MethodDefinition(".ctor", (ushort)(MethodAttributes.Public | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName),
            new MethodSignature(PrimitiveType.Void, new[] { PrimitiveType.Int32 }));
        objectType.Methods.Add(constructor);
        var constructorBody = MethodBuilder.ForDefinition(constructor);
        var valueBuilder = builder.Types.Single(t => t.Name == "Box").Fields.Single();
        constructorBody.LoadArgument(0); constructorBody.LoadArgument(1); constructorBody.StoreField(valueBuilder); constructorBody.Return();
        var read = new MethodDefinition("Read", (ushort)MethodAttributes.Public, PrimitiveMethodSignature.Int32(0, true));
        objectType.Methods.Add(read);
        var readBody = MethodBuilder.ForDefinition(read);
        readBody.LoadArgument(0); readBody.LoadField(valueBuilder); readBody.Return();
        var property = new PropertyDefinition("Value", PrimitiveType.Int32, read);
        objectType.Properties.Add(property);
        if (!ReferenceEquals(property.DeclaringType, objectType) || !ReferenceEquals(property.GetMethod, read) ||
            !ReferenceEquals(builder.Types.Single(t => t.Name == "Box").Properties.Single().Definition, property) ||
            !ReferenceEquals(assembly.MainModule.Properties.Single(), property)) throw new Exception("property definition identity");
        Reject<ArgumentException>(() => objectType.Properties.Add(property));
        Reject<ArgumentException>(() => objectType.Properties.Add(new PropertyDefinition("Again", PrimitiveType.Int32, read)));
        Reject<InvalidOperationException>(() => property.GetSignature());
        Reject<NotSupportedException>(() => objectType.Properties.Clear());
        var contract = new TypeDefinition("Example", "IRead", (uint)(TypeAttributes.Public | TypeAttributes.Interface | TypeAttributes.Abstract), null);
        assembly.MainModule.Types.Add(contract);
        var contractRead = new MethodDefinition("Read", (ushort)(MethodAttributes.Public | MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot), PrimitiveMethodSignature.Int32(0, true));
        Reject<InvalidOperationException>(() => objectType.Methods.Add(contractRead));
        contract.Methods.Add(contractRead);
        var derivedContract = new TypeDefinition("Example", "IDerivedRead", 0xa1, null);
        assembly.MainModule.Types.Add(derivedContract);
        var baseEdge = new InterfaceImplementation(contract.ToReference());
        derivedContract.Interfaces.Add(baseEdge);
        var implementation = new InterfaceImplementation(derivedContract.ToReference());
        objectType.Interfaces.Add(implementation);
        if (!ReferenceEquals(implementation.DeclaringType, objectType) || !ReferenceEquals(baseEdge.DeclaringType, derivedContract) ||
            !ReferenceEquals(implementation.InterfaceType.Resolve(), derivedContract)) throw new Exception("interface relationship identity");
        Reject<ArgumentException>(() => objectType.Interfaces.Add(implementation));
        Reject<ArgumentException>(() => objectType.Interfaces.Add(new InterfaceImplementation(derivedContract.ToReference())));
        Reject<ArgumentException>(() => contract.Interfaces.Add(new InterfaceImplementation(derivedContract.ToReference())));
        Reject<ArgumentException>(() => contract.Interfaces.Add(new InterfaceImplementation(objectType.ToReference())));
        Reject<NotSupportedException>(() => objectType.Interfaces.Clear());
        answerBody.LoadConstant(42); answerBody.NewObject(constructorBody); answerBody.CallVirtual(MethodBuilder.ForDefinition(contractRead)); answerBody.Return();
        Reject<InvalidOperationException>(() => contract.Methods.Add(new MethodDefinition("Concrete", (ushort)MethodAttributes.Public, PrimitiveMethodSignature.Int32(0, true))));
        Reject<ArgumentException>(() => new MethodDefinition("Incomplete", (ushort)(MethodAttributes.Public | MethodAttributes.Abstract), PrimitiveMethodSignature.Int32(0, true)));
        Reject<ArgumentException>(() => new MethodDefinition("Static", (ushort)(MethodAttributes.Public | MethodAttributes.Static | MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot), PrimitiveMethodSignature.Int32(0, true)));
        var invalidInterface = new TypeDefinition("Example", "InvalidInterface", 0xa1, null);
        invalidInterface.Fields.Add(new FieldDefinition("Storage", (ushort)FieldAttributes.Public, PrimitiveType.Int32));
        Reject<ArgumentException>(() => assembly.MainModule.Types.Add(invalidInterface));
        Reject<ArgumentException>(() => new MethodDefinition(".ctor", (ushort)MethodAttributes.Public, PrimitiveMethodSignature.Int32(0, true)));
        Reject<ArgumentException>(() => new MethodDefinition("Read", (ushort)(MethodAttributes.Public | MethodAttributes.SpecialName), PrimitiveMethodSignature.Int32(0, true)));
        Reject<ArgumentException>(() => new MethodDefinition(".ctor", (ushort)(MethodAttributes.Public | MethodAttributes.Static), new MethodSignature(PrimitiveType.Void, Array.Empty<PrimitiveType>())));
        Reject<ArgumentException>(() => new MethodDefinition(".ctor", (ushort)MethodAttributes.Public, new MethodSignature(PrimitiveType.Void, Array.Empty<SignatureType>(), new[] { "T" })));
        var staticOwner = builder.AddType("Example", "StaticOwner");
        var unattachedInstance = new MethodDefinition("Read", (ushort)MethodAttributes.Public, PrimitiveMethodSignature.Int32(0, true));
        Reject<InvalidOperationException>(() => staticOwner.Definition.Methods.Add(unattachedInstance));
        Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(unattachedInstance));
        var parameters = new[] { "T" };
        var generic = new TypeDefinition("Example", "Identity", 1, assembly.MainModule.ImportReference(builder.CoreLibrary, "System", "Object"), parameters);
        parameters[0] = "Changed";
        assembly.MainModule.Types.Add(generic);
        var identity = new MethodDefinition("Pass", (ushort)(MethodAttributes.Public | MethodAttributes.Static),
            new MethodSignature(SignatureType.TypeParameter(0), new[] { SignatureType.TypeParameter(0) }));
        generic.Methods.Add(identity);
        var identityBody = MethodBuilder.ForDefinition(identity); identityBody.LoadArgument(0); identityBody.Return();
        if (generic.Name != "Identity`1" || generic.GenericParameterNames!.Single() != "T" ||
            !ReferenceEquals(generic.GenericParameterNames, builder.Types.Single(t => t.Name == "Identity`1").GenericParameterNames))
            throw new Exception("generic parameter definition identity/copy");
        var genericBuilder = builder.Types.Single(t => t.Name == "Identity`1");
        var constraintView = generic.SpecialConstraints;
        genericBuilder.SetSpecialConstraints(0, TypeParameterConstraints.ValueType);
        if (!ReferenceEquals(constraintView, genericBuilder.SpecialConstraints) || constraintView[0] != TypeParameterConstraints.ValueType ||
            !ReferenceEquals(generic.GenericConstraints, genericBuilder.GenericConstraints)) throw new Exception("constraint storage identity");
        Reject<ArgumentException>(() => identityBody.MakeConstructedReference(new SignatureType[] { PrimitiveType.String }));
        Reject<ArgumentException>(() => new TypeDefinition("Example", "Invalid`1", 1, generic.BaseType, new[] { "T" }));
        Reject<ArgumentException>(() => new TypeDefinition("Example", "Invalid", 1, generic.BaseType, new[] { "T", "T" }));
        helper.Call(answerBody); helper.Call(identityBody.MakeConstructedReference(new SignatureType[] { PrimitiveType.Int32 })); helper.Return();
        var entryDefinition = new MethodDefinition("Main", PrimitiveMethodSignature.Int32(0, true));
        assembly.MainModule.Functions.Add(entryDefinition);
        assembly.EntryPoint = entryDefinition;
        var entry = MethodBuilder.ForDefinition(entryDefinition);
        var local = entry.DeclareLocal(type);
        entry.LoadDefault(type); entry.StoreLocal(local);
        entry.LoadLocalAddress(local); entry.Call(helper); entry.StoreField(field);
        entry.LoadLocal(local); entry.LoadField(field); entry.Return();
        return assembly;
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var assembly = Executable(); var builder = AssemblyBuilder.ForDefinition(assembly);
        var path = Path.Combine(output, "ManualStruct.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), builder.CoreLibrary));
        foreach (var (command, expected) in new[] { ("verify", 0), ("run", 42) })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout; var error = await stderr;
            if (process.ExitCode != expected) throw new Exception(command + ": " + process.ExitCode + " " + text + error);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new { verified = true, result = 42,
            scope = "manual assembly/type/field/function construction and entry-point assignment; helper call through same definitions in body builders; native write/load/execute; manual static type-method construction; manual root-class constructor, readonly field initialization and instance call; manual inherited interface relationship and virtual dispatch; direct generic type declaration and constructed call; definition-owned property/accessor association and body storage/clearing; arbitrary instruction editing and loaded editing remain pending" }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
