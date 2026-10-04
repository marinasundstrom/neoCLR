using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class OpenConstrainedCallChecks
{
    internal static void Run()
    {
        foreach (var self in new[] { false, true })
        foreach (var raw in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("OpenConstrained" + self + raw, new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var contract = graph.AddInterface("Example", "Number");
            var scalar = self ? PrimitiveType.Double : PrimitiveType.Int32;
            var contractType = self ? SignatureType.Self : (SignatureType)scalar;
            var add = contract.AddInterfaceMethod("Add", new(contractType, [contractType, contractType]), isStatic: true);
            var implementation = graph.AddValueType(self ? "System" : "Example", self ? "Double" : "Count");
            if (self) implementation.SetNativePrimitive(PrimitiveType.Double);
            var bound = raw ? graph.AddInterface("Example", "DerivedNumber") : contract;
            if (raw) bound.AddBaseInterface(contract);
            implementation.AddInterfaceImplementation(bound);
            var body = implementation.AddMethod("Add", new(scalar, [scalar, scalar])).GetILGenerator();
            body.LoadArgument(0); body.LoadArgument(1); body.Add(); body.Return();
            var parameter = SignatureType.MethodParameter(0);
            var value = self ? parameter : (SignatureType)scalar;
            var sum = graph.AddFunction("Sum", new(value, [value, value], ["T"]));
            var il = sum.GetILGenerator();
            Reject(() => il.CallConstrained(parameter, add));
            sum.AddInterfaceConstraint(0, bound);
            Reject(() => il.CallConstrained(SignatureType.MethodParameter(1), add));
            Reject(() => il.CallConstrained(SignatureType.TypeParameter(0), add));
            Reject(() => il.Emit(OpCode.Callvirt, parameter, add));
            il.LoadArgument(0); il.LoadArgument(1);
            if (raw) il.Emit(OpCode.Call, parameter, add); else il.CallConstrained(parameter, add);
            il.Return();
            var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
            var entry = main.GetILGenerator();
            if (self) { entry.LoadConstant(20.0); entry.LoadConstant(22.0); }
            else { entry.LoadConstant(20); entry.LoadConstant(22); }
            entry.Call(sum.MakeGenericInstance(self ? (SignatureType)PrimitiveType.Double : implementation));
            if (self) entry.Emit(OpCode.Conv_I4);
            entry.Return(); graph.EntryPoint = main;
            if (!self && (int)Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null)! != 42)
                throw new Exception("open constrained CLI execution");
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            _ = AssemblyDefinition.ReadNativeAssembly(native);
            if (Environment.GetEnvironmentVariable("NEOCLR_OPEN_CALL_ARTIFACT") is { } path)
                File.WriteAllBytes(path + (self ? ".self.neox" : ".neox"), native);
        }
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (ArgumentException) { return; }
        throw new Exception("invalid open constrained operand accepted");
    }
}
