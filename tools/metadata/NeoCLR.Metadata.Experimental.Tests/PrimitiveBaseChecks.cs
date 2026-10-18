using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class PrimitiveBaseChecks
{
    internal static void Run()
    {
        var identity = new AssemblyIdentity("PrimitiveCore", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(identity, identity);
        var root = graph.AddNativeObjectRoot();
        var foundation = graph.AddClass("System", "ValueType", root);
        foundation.SetAbstractClass();
        foreach (var primitive in new[] { PrimitiveType.SByte, PrimitiveType.Byte, PrimitiveType.Int16,
            PrimitiveType.UInt16, PrimitiveType.Int32, PrimitiveType.UInt32, PrimitiveType.Int64,
            PrimitiveType.UInt64, PrimitiveType.IntPtr, PrimitiveType.UIntPtr, PrimitiveType.Single,
            PrimitiveType.Double, PrimitiveType.Boolean, PrimitiveType.RuntimeTypeHandle, PrimitiveType.Void })
        {
            var value = graph.AddValueType("System", primitive.ToString());
            value.SetNativePrimitive(primitive);
            value.SetValueTypeBase(foundation);
            Reject<InvalidOperationException>(() => value.SetValueTypeBase(foundation));
        }
        var enumBase = graph.AddClass("System", "Enum", foundation);
        enumBase.SetAbstractClass();
        var choice = graph.AddEnum("System", "Choice");
        choice.SetEnumBase(enumBase);
        choice.AddEnumMember("One", 1);
        Reject<InvalidOperationException>(() => choice.SetEnumBase(enumBase));
        graph.AddClass("System.Runtime.CompilerServices", "ReferenceAssemblyAttribute", root);
        var loaded = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph));
        if (loaded.MainModule.Types.Single(t => t.Name == "Choice").BaseType?.Name != "Enum")
            throw new Exception("local enum ancestry lost");
        foreach (var value in loaded.MainModule.Types.Where(t => t.NativePrimitive is not null))
            if (!value.IsValueType || value.BaseType?.Name != "ValueType")
                throw new Exception("primitive ancestry/category lost");
        var consumer = new AssemblyBuilder(new("ClosedConsumer", new Version(1, 0, 0, 0)), identity);
        var objectReference = consumer.ImportReference(loaded.MainModule.Types.Single(t => t.Name == "Object"), identity);
        consumer.SetNativeObjectRoot(objectReference);
        var toString = consumer.CreateObjectSlotReference("ToString", new(PrimitiveType.String, []));
        consumer.CompleteClassBaseReference(objectReference, [toString], 0);
        var closed = consumer.AddClosedClass("Example", "Closed");
        closed.SetBaseType(objectReference);
        var display = closed.AddOverride("ToString", new MethodSignature(PrimitiveType.String, []));
        display.GetILGenerator().Emit(OpCode.Ldstr, "closed");
        display.Return();
        var closedRead = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(consumer));
        if (closedRead.MainModule.Types.Single().BaseType?.Name != "Object")
            throw new Exception("closed root Object ancestry lost");
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), identity);
        var foreign = other.AddValueType("System", "Int32");
        foreign.SetNativePrimitive(PrimitiveType.Int32);
        Reject<ArgumentException>(() => foreign.SetValueTypeBase(foundation));
        Reject<InvalidOperationException>(() => graph.AddClass("Example", "Reference").SetValueTypeBase(foundation));
        Console.WriteLine("PASS intrinsic primitive ValueType ancestry");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
