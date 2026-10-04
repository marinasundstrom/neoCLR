using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class OpenConstrainedCallChecks
{
    internal static void Run()
    {
        External();
        PrimitiveDesignation();
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
    private static void External()
    {
        var core = new AssemblyIdentity("System.Runtime", new Version(10, 0, 0, 0));
        var contracts = new AssemblyBuilder(new("OpenExternalContracts", new Version(1, 0, 0, 0)), core);
        var comparable = contracts.AddGenericInterface("Example", "Comparable", ["T"]);
        comparable.AddInterfaceMethod("Compare", new(PrimitiveType.Int32, [SignatureType.TypeParameter(0)]));
        var number = contracts.AddInterface("Example", "Number");
        number.AddBaseInterface(comparable.MakeGenericInstance(PrimitiveType.Int32));
        number.AddInterfaceMethod("Add", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Int32]), isStatic: true);
        var dependency = RuntimeAssemblyContainer.WriteBinary(contracts);
        var graph = new AssemblyBuilder(new("OpenExternalConsumer", new Version(1, 0, 0, 0)), core);
        var bound = graph.CreateInterfaceReference(contracts.Identity, core, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(dependency)), "Example", "Number");
        var add = graph.CreateMethodReference(bound, "Add", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Int32]), isStatic: true);
        var compareOwner = graph.CreateInterfaceReference(contracts.Identity, core, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(dependency)), "Example", "Comparable`1", 1);
        var compare = graph.CreateMethodReference(compareOwner, "Compare", new(PrimitiveType.Int32, [SignatureType.TypeParameter(0)]));
        graph.AddInterfaceConversion(bound, compareOwner.MakeGenericInstance(PrimitiveType.Int32));
        graph.CompleteInterfaceReference(compareOwner);
        graph.CompleteInterfaceReference(bound);
        var implementation = graph.AddValueType("Example", "Count");
        implementation.AddInterfaceImplementation(bound);
        var body = implementation.AddMethod("Add", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Int32])).GetILGenerator();
        body.LoadArgument(0); body.LoadArgument(1); body.Add(); body.Return();
        var comparison = implementation.AddInstanceMethod("Compare", new(PrimitiveType.Int32, [PrimitiveType.Int32])).GetILGenerator();
        comparison.LoadArgument(1); comparison.Return();
        var sum = graph.AddFunction("Sum", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Int32], ["T"]));
        Reject(() => sum.GetILGenerator().CallConstrained(SignatureType.MethodParameter(0), add));
        sum.AddInterfaceConstraint(0, bound);
        var il = sum.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.Emit(OpCode.Call, SignatureType.MethodParameter(0), add); il.Return();
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        var check = graph.AddFunction("Check", new(PrimitiveType.Int32, [SignatureType.MethodParameter(0)], ["T"]));
        check.AddInterfaceConstraint(0, bound);
        var checkIl = check.GetILGenerator();
        checkIl.LoadArgumentAddress(0); checkIl.LoadConstant(20); checkIl.LoadConstant(22);
        checkIl.Call(sum.MakeGenericInstance(SignatureType.MethodParameter(0)));
        checkIl.Emit(OpCode.Callvirt, SignatureType.MethodParameter(0), compare.MakeConstructedReference([PrimitiveType.Int32]));
        checkIl.Return();
        var entry = main.GetILGenerator(); entry.LoadDefault(implementation); entry.Call(check.MakeGenericInstance(implementation)); entry.Return(); graph.EntryPoint = main;
        var context = new System.Runtime.Loader.AssemblyLoadContext("external-open-call", true);
        try
        {
            var loadedContract = context.LoadFromStream(new MemoryStream(contracts.Write()));
            context.Resolving += (_, name) => name.Name == loadedContract.GetName().Name ? loadedContract : null;
            if ((int)context.LoadFromStream(new MemoryStream(graph.Write())).EntryPoint!.Invoke(null, null)! != 42) throw new Exception("external open CLI dispatch");
        }
        finally { context.Unload(); }
        if (Environment.GetEnvironmentVariable("NEOCLR_OPEN_CALL_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path + ".external.neox", RuntimeAssemblyContainer.WriteBinary(graph));
            File.WriteAllBytes(path + ".contracts.neox", dependency);
        }
    }
    private static void PrimitiveDesignation()
    {
        var core = new AssemblyIdentity("System.Runtime", new Version(10, 0, 0, 0));
        var graph = new AssemblyBuilder(new("PrimitiveConsumer", new Version(1, 0, 0, 0)), core);
        ImportedTypeReference Create(string name)
        {
            var provider = new AssemblyBuilder(new(name, new Version(1, 0, 0, 0)), core);
            provider.AddValueType("System", "Int32").SetNativePrimitive(PrimitiveType.Int32);
            var image = RuntimeAssemblyContainer.WriteBinary(provider);
            return graph.CreateValueTypeReference(provider.Identity, core, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "System", "Int32");
        }
        var first = Create("FirstNumericOwner");
        graph.SetNativePrimitive(first, PrimitiveType.Int32);
        graph.SetNativePrimitive(first, PrimitiveType.Int32);
        Reject(() => graph.SetNativePrimitive(first, PrimitiveType.Double));
        var foreign = new AssemblyBuilder(new("ForeignConsumer", new Version(1, 0, 0, 0)), core);
        Reject(() => foreign.SetNativePrimitive(first, PrimitiveType.Int32));
        var second = Create("SecondNumericOwner");
        try { graph.SetNativePrimitive(second, PrimitiveType.Int32); }
        catch (InvalidDataException) { return; }
        throw new Exception("duplicate numeric ownership accepted");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (ArgumentException) { return; }
        throw new Exception("invalid open constrained operand accepted");
    }
}
