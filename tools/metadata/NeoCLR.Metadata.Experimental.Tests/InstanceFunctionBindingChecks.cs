using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class InstanceFunctionBindingChecks
{
    internal static AssemblyBuilder Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("InstanceCallbacks", new Version(1, 0, 0, 0)), core);
        var counter = graph.AddClass("Example", "Counter");
        var value = counter.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = counter.AddConstructor([PrimitiveType.Int32]);
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var add = counter.AddInstanceMethod("Add", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        add.LoadArgument(0); add.LoadArgument(0); add.LoadField(value); add.LoadArgument(1); add.Emit(OpCode.Add); add.StoreField(value);
        add.LoadArgument(0); add.LoadField(value); add.Return();
        var shape = SignatureType.Function(add.Signature);
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        var instance = il.DeclareLocal(counter);
        var callback = il.DeclareLocal(shape);
        il.LoadConstant(40); il.NewObject(ctor); il.StoreLocal(instance);
        il.LoadLocal(instance); il.BindFunction(shape, add); il.StoreLocal(callback);
        il.LoadLocal(callback); il.LoadConstant(1); il.InvokeFunction(shape); il.Emit(OpCode.Pop);
        // A second binding must retain the same mutable receiver, not a copy.
        il.LoadLocal(instance); il.Emit(OpCode.BindFunction, new FunctionBinding(shape, add));
        il.LoadConstant(1); il.InvokeFunction(shape); il.Emit(OpCode.Pop);
        var done = il.DefineLabel();
        il.Emit(OpCode.Ldc_Bool, true); il.Emit(OpCode.Brtrue, done); il.LoadConstant(0); il.Return(); il.MarkLabel(done);
        il.LoadLocal(instance); il.LoadField(value); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        if (!Equals(42, Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null)))
            throw new Exception("receiver-bound callbacks lost shared state");
        _ = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var target = graph.Types.Single().Methods.Single(m => m.Name == "Add");
        var shape = SignatureType.Function(target.Signature);
        var main = graph.EntryPoint!;
        main.ClearBody(); main.BindFunction(shape, target); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
        Reject(() => graph.Write());
        main.ClearBody(); main.LoadConstant(42); main.BindFunction(shape, target); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
        Reject(() => graph.WriteNativeAssembly());
        static void Reject(Action action)
        {
            try { action(); } catch (InvalidDataException) { return; }
            throw new Exception("invalid bound receiver accepted");
        }
    }
}
