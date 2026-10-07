using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class UnitRepresentationChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("UnitImplementation", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var unit = graph.AddValueType("System", "Void");
        unit.SetNativePrimitive(PrimitiveType.Void);
        SignatureType unitValue = unit;
        if (unitValue.Primitive is not null) throw new Exception("owned unit collapsed to CLI void");
        var callback = SignatureType.Function(new MethodSignature(unitValue, []));
        if (callback.FunctionSignature!.NoResult) throw new Exception("unit callback lost its value result");
        var value = graph.AddFunction("Unit", new MethodSignature(unitValue, []));
        value.GetILGenerator().LoadDefault(unitValue); value.GetILGenerator().Return();
        var apply = graph.AddFunction("Apply", new MethodSignature(unitValue, [callback]));
        apply.GetILGenerator().LoadArgument(0); apply.GetILGenerator().InvokeFunction(callback); apply.GetILGenerator().Return();
        var parameter = SignatureType.MethodParameter(0);
        var identity = graph.AddFunction("Identity", new MethodSignature(parameter, [parameter], ["T"]));
        identity.GetILGenerator().LoadArgument(0);
        identity.GetILGenerator().Return();
        var entry = graph.AddFunction("Main"); graph.EntryPoint = entry;
        var il = entry.GetILGenerator(); il.BindFunction(callback, value); il.Call(apply); il.Call(identity.MakeGenericInstance(unitValue)); il.Emit(OpCode.Pop); il.LoadConstant(42); il.Return();
        var image = RuntimeAssemblyContainer.WriteBinary(graph);
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var declaration = loaded.MainModule.Types.Single(t => t.Name == "Void");
        if (declaration.NativePrimitive != PrimitiveType.Void) throw new Exception("unit designation lost");
        var context = new MetadataLoadContext([loaded]);
        var assembly = context.Resolve(loaded.Identity);
        var roundTrip = assembly.GetModules().Single().GetFunctions().Single(f => f.Name == "Apply");
        var function = (FunctionTypeInfo)roundTrip.GetParameters().Single().ParameterType;
        if (function.NoResult) throw new Exception("unit callback round trip lost value");
        if (!ReferenceEquals(function.ReturnType, roundTrip.ReturnType)) throw new Exception("unit result identity differs inside callback");
        graph.EntryPoint = null;
        var dependencyImage = RuntimeAssemblyContainer.WriteBinary(graph);
        loaded = AssemblyDefinition.ReadNativeAssembly(dependencyImage);
        var consumer = new AssemblyBuilder(new("UnitConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var externalUnit = consumer.CreateValueTypeReference(graph.Identity, graph.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(dependencyImage)), "System", "Void");
        consumer.SetNativePrimitive(externalUnit, PrimitiveType.Void);
        SignatureType externalValue = externalUnit;
        var echo = consumer.AddFunction("Echo", new MethodSignature(externalValue, [externalValue]));
        echo.GetILGenerator().LoadArgument(0);
        echo.GetILGenerator().Return();
        var consumerEntry = consumer.AddFunction("Main");
        consumer.EntryPoint = consumerEntry;
        var consumerIL = consumerEntry.GetILGenerator();
        consumerIL.LoadDefault(externalValue);
        consumerIL.Call(echo);
        consumerIL.Emit(OpCode.Pop);
        consumerIL.LoadConstant(42);
        consumerIL.Return();
        var consumerImage = RuntimeAssemblyContainer.WriteBinary(consumer);
        var consumerSnapshot = AssemblyDefinition.ReadNativeAssembly(consumerImage);
        var importedContext = new MetadataLoadContext([loaded, consumerSnapshot]);
        var importedEcho = importedContext.Resolve(consumerSnapshot.Identity).GetModules().Single().GetFunctions().Single(f => f.Name == "Echo");
        if (!ReferenceEquals(importedEcho.ReturnType, importedContext.Resolve(loaded.Identity).GetTypes().Single(t => t.Name == "Void")))
            throw new Exception("imported unit lost canonical owner");
        if (Environment.GetEnvironmentVariable("NEOCLR_UNIT_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path, image);
            File.WriteAllBytes(path + ".consumer", consumerImage);
            File.WriteAllBytes(path + ".library", dependencyImage);
            File.WriteAllText(path + ".neoil", ".module System\n.references ()\n");
        }
        var manualGraph = new AssemblyBuilder(new("ManualUnit", new(1, 0, 0, 0)), graph.CoreLibrary);
        var manual = new TypeDefinition("System", "Void", 0x109,
            manualGraph.Definition.MainModule.ImportReference(graph.CoreLibrary, "System", "ValueType"));
        manual.SetNativePrimitive(PrimitiveType.Void);
        manualGraph.Definition.MainModule.Types.Add(manual);
        var manualSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(manualGraph));
        if (manualSnapshot.MainModule.Types.Single().NativePrimitive != PrimitiveType.Void)
            throw new Exception("manual unit designation lost");
        var invalid = graph.AddValueType("Other", "Void");
        try { invalid.SetNativePrimitive(PrimitiveType.Void); throw new Exception("lookalike unit accepted"); }
        catch (ArgumentException) { }
        unit.AddField("Storage", PrimitiveType.Int32);
        try { RuntimeAssemblyContainer.WriteBinary(graph); throw new Exception("fielded unit accepted"); }
        catch (ArgumentException) { }
        catch (InvalidDataException) { }
    }
}
