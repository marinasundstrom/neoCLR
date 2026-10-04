using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ConstructedFunctionBindingChecks
{
    internal static AssemblyBuilder Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("ConstructedCallbacks", new Version(1, 0, 0, 0)), core);
        var parameter = SignatureType.TypeParameter(0);
        var contract = graph.AddGenericInterface("Example", "Reader", ["T"]);
        var readContract = contract.AddInterfaceMethod("Read", new(parameter, []));
        var box = graph.AddGenericClass("Example", "Box", ["T"]);
        box.AddInterfaceImplementation(contract.MakeGenericInstance(parameter));
        var field = box.AddField("Value", parameter);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        var il = ctor.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(field); il.Return();
        var read = box.AddInstanceMethod("Read", new(parameter, []));
        il = read.GetILGenerator(); il.LoadArgument(0); il.LoadField(field); il.Return();
        var shape = SignatureType.Function(new MethodSignature(parameter, []));
        var bind = box.AddInstanceMethod("Bind", new(shape, []));
        il = bind.GetILGenerator(); il.LoadArgument(0); il.BindFunction(shape, read.MakeConstructedReference([parameter])); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator();
        var instance = il.DeclareLocal(box.MakeGenericInstance(PrimitiveType.Int32));
        var first = il.DeclareLocal(PrimitiveType.Int32);
        var closedShape = SignatureType.Function(new MethodSignature(PrimitiveType.Int32, []));
        il.LoadConstant(21); il.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32])); il.StoreLocal(instance);
        il.LoadLocal(instance); il.Call(bind.MakeConstructedReference([PrimitiveType.Int32])); il.InvokeFunction(closedShape); il.StoreLocal(first);
        il.LoadLocal(instance); il.CastReference(contract.MakeGenericInstance(PrimitiveType.Int32));
        il.Emit(OpCode.BindFunction, new FunctionBinding(closedShape, readContract.MakeConstructedReference([PrimitiveType.Int32])));
        il.InvokeFunction(closedShape); il.LoadLocal(first); il.Add(); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        if (!Equals(42, Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null))) throw new Exception("constructed/interface callback execution");
        _ = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var read = graph.Types.Single(t => t.Name == "Box`1").Methods.Single(m => m.Name == "Read");
        var wrong = SignatureType.Function(new MethodSignature(PrimitiveType.String, []));
        try { _ = new FunctionBinding(wrong, read.MakeConstructedReference([PrimitiveType.Int32])); }
        catch (ArgumentException) { return; }
        throw new Exception("incorrect constructed callback signature accepted");
    }
}
