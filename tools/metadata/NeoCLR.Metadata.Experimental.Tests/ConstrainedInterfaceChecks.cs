using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ConstrainedInterfaceChecks
{
    internal static void Run()
    {
        RunCase(authored: true);
        RunCase(authored: false);
    }

    private static void RunCase(bool authored)
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("ConstrainedValue", new Version(1, 0, 0, 0)), core);
        var contract = graph.AddInterface("Example", "Counter");
        var next = contract.AddInterfaceMethod("Next", new(PrimitiveType.Int32, []));
        var value = graph.AddValueType("Example", "ValueCounter");
        // Manual definitions and builder convenience methods share attachment validation.
        if (authored) value.Definition.Interfaces.Add(new InterfaceImplementation(contract.Definition.ToReference()));
        else value.AddInterfaceImplementation(contract);
        var field = value.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = value.AddConstructor([PrimitiveType.Int32]);
        var init = ctor.GetILGenerator(); init.LoadArgument(0); init.LoadArgument(1); init.StoreField(field); init.Return();
        var implementation = value.AddInstanceMethod("Next", new(PrimitiveType.Int32, []));
        var body = implementation.GetILGenerator();
        body.LoadArgument(0); body.LoadArgument(0); body.LoadField(field); body.LoadConstant(1); body.Add(); body.StoreField(field);
        body.LoadArgument(0); body.LoadField(field); body.Return();
        var genericContract = graph.AddGenericInterface("Example", "Reader", ["T"]);
        genericContract.AddInterfaceMethod("Read", new(SignatureType.TypeParameter(0), []));
        var genericValue = graph.AddGenericValueType("Example", "ValueReader", ["T"]);
        var constructedContract = genericContract.MakeGenericInstance(SignatureType.TypeParameter(0));
        if (authored) genericValue.Definition.Interfaces.Add(new InterfaceImplementation(genericContract.Definition.ToReference(), [SignatureType.TypeParameter(0)]));
        else genericValue.AddInterfaceImplementation(constructedContract);
        var genericField = genericValue.AddField("Value", SignatureType.TypeParameter(0));
        var genericRead = genericValue.AddInstanceMethod("Read", new(SignatureType.TypeParameter(0), [])).GetILGenerator();
        genericRead.LoadArgument(0); genericRead.LoadField(genericField); genericRead.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        var original = il.DeclareLocal(value); var copy = il.DeclareLocal(value);
        il.LoadConstant(40); il.NewObject(ctor); il.StoreLocal(original);
        il.LoadLocal(original); il.StoreLocal(copy);
        il.LoadLocalAddress(original);
        if (authored) il.CallConstrained(value, next); else il.Emit(OpCode.Callvirt, value, next);
        il.Emit(OpCode.Pop);
        il.LoadLocalAddress(original); il.CallConstrained(value, next); Check(42);
        il.LoadLocalAddress(copy); il.CallConstrained(value, next); Check(41);
        il.LoadLocalAddress(original); il.LoadField(field); il.Return();
        void Check(int expected)
        {
            var success = il.DefineLabel(); il.LoadConstant(expected); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brtrue, success);
            il.LoadConstant(99); il.Return(); il.MarkLabel(success);
        }
        try { il.CallConstrained(contract, next); throw new Exception("reference receiver admitted"); } catch (ArgumentException) { }
        try { il.CallConstrained(genericValue, next); throw new Exception("open generic receiver admitted"); } catch (ArgumentException) { }
        var image = graph.Write();
        var context = new AssemblyLoadContext("constrained-value", true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(image));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("constrained mutation/copy failed");
            if (loaded.GetType("Example.ValueCounter")!.GetInterfaces().Single().Name != "Counter") throw new Exception("CLI interface relationship missing");
        }
        finally { context.Unload(); }
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        if (snapshot.MainModule.Types.Single(t => t.Name == "ValueCounter").Interfaces.Count != 1) throw new Exception("native relationship missing");
        if (snapshot.MainModule.Types.Single(t => t.Name == "ValueReader`1").Interfaces.Single().TypeArguments.Single().TypeParameterIndex != 0)
            throw new Exception("generic value relationship lost owner parameter");
        var metadata = new MetadataLoadContext([snapshot]);
        var views = metadata.Resolve(snapshot.Identity).GetTypes();
        if (!ReferenceEquals(views.Single(t => t.Name == "Counter"), views.Single(t => t.Name == "ValueCounter").GetInterfaces().Single()))
            throw new Exception("value interface view lost canonical identity");
        if (Environment.GetEnvironmentVariable("NEOCLR_CONSTRAINED_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
        main.ClearBody();
        var wrong = il.DeclareLocal(PrimitiveType.Int32);
        il.LoadConstant(42); il.StoreLocal(wrong); il.LoadLocalAddress(wrong); il.CallConstrained(value, next); il.Return();
        try { graph.Write(); throw new Exception("wrong managed receiver admitted"); } catch (InvalidDataException) { }
        // Ordinary interface calls must not treat an unboxed value as an object.
        main.ClearBody(); il.LoadConstant(40); il.NewObject(ctor); il.CallVirtual(next); il.Return();
        try { graph.Write(); throw new Exception("unboxed interface receiver admitted"); } catch (InvalidDataException) { }
        main.ClearBody(); il.LoadConstant(0); il.Return();
        graph.AddValueType("Example", "Incomplete").AddInterfaceImplementation(contract);
        try { graph.Write(); throw new Exception("missing implementation admitted"); }
        catch (InvalidDataException error) when (error.Message.Contains("missing public interface implementation")) { }
    }
}
