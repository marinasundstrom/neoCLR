using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ClassVirtualChecks
{
    private static AssemblyBuilder Create(bool manual)
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("VirtualClasses" + Guid.NewGuid().ToString("N"), new(1, 0, 0, 0)),
            new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var reader = graph.AddInterface("Example", "Reader");
        var contract = reader.AddInterfaceMethod("Read", new(PrimitiveType.Int32, []));
        var parent = graph.AddClass("Example", "Base"); parent.SetAbstractClass(); parent.AddInterfaceImplementation(reader);
        var field = parent.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var init = parent.AddConstructor([PrimitiveType.Int32]);
        var il = init.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(field); il.Return();
        MethodBuilder Slot(TypeBuilder owner, string name, MethodSignature signature, ushort flags)
        {
            if (!manual) return flags == 0x546 ? owner.AddAbstractMethod(name, signature) : flags == 0x146
                ? owner.AddVirtualMethod(name, signature) : owner.AddOverride(name, signature);
            var definition = new MethodDefinition(name, flags, signature); owner.Definition.Methods.Add(definition);
            return MethodBuilder.ForDefinition(definition);
        }
        var read = Slot(parent, "Read", new(PrimitiveType.Int32, []), 0x546);
        var set = Slot(parent, "Set", new(PrimitiveType.Void, [PrimitiveType.Int32]), 0x146);
        il = set.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(field); il.Return();
        var child = graph.AddClass("Example", "Derived", parent);
        var ctor = child.AddConstructor([]);
        il = ctor.GetILGenerator(); il.LoadArgument(0); il.LoadConstant(7); il.Call(init); il.Return();
        var childRead = Slot(child, "Read", new(PrimitiveType.Int32, []), 0x46);
        il = childRead.GetILGenerator(); il.LoadArgument(0); il.LoadField(field); il.Return();
        var childSet = Slot(child, "Set", new(PrimitiveType.Void, [PrimitiveType.Int32]), 0x46);
        il = childSet.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.Call(set); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator(); var item = il.DeclareLocal(child); il.NewObject(ctor); il.StoreLocal(item);
        il.LoadLocal(item); il.CallVirtual(contract); il.Emit(OpCode.Pop);
        il.LoadLocal(item); il.LoadConstant(42); il.CallVirtual(set);
        il.LoadLocal(item); il.CallVirtual(read); il.Return();
        return graph;
    }
    internal static void Run()
    {
        InvalidContracts();
        foreach (var manual in new[] { false, true })
        {
            var graph = Create(manual);
            var cli = System.Reflection.Assembly.Load(graph.Write());
            var child = cli.GetType("Example.Derived")!; var parent = child.BaseType!;
            var item = Activator.CreateInstance(child)!;
            var contract = cli.GetType("Example.Reader")!.GetMethod("Read")!;
            if ((int)contract.Invoke(item, null)! != 7) throw new Exception("CLI abstract interface dispatch failed");
            parent.GetMethod("Set")!.Invoke(item, [42]);
            if ((int)parent.GetMethod("Read")!.Invoke(item, null)! != 42) throw new Exception("CLI virtual base dispatch failed");
            if (parent.GetMethod("Set")!.IsFinal || child.GetMethod("Read")!.GetBaseDefinition().DeclaringType != parent)
                throw new Exception("CLI slot flags changed");
            var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
            var views = new MetadataLoadContext([snapshot]).Assemblies.Single().GetTypes();
            var abstractSlot = views.Single(t => t.Name == "Base").GetMethods().Single(m => m.Name == "Read");
            var implementation = views.Single(t => t.Name == "Derived").GetMethods().Single(m => m.Name == "Read");
            if (!abstractSlot.IsAbstract || !abstractSlot.IsVirtual || !abstractSlot.IsNewSlot ||
                implementation.IsAbstract || !implementation.IsVirtual || implementation.IsNewSlot)
                throw new Exception("native abstract/override slot flags lost");
        }
    }
    private static void RejectGraph(AssemblyBuilder graph)
    {
        foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
        {
            try { write(); } catch (InvalidDataException) { continue; }
            throw new Exception("invalid virtual hierarchy accepted");
        }
    }

    private static void InvalidContracts()
    {
        var missing = Create(false);
        missing.AddClass("Example", "Missing", missing.Types.Single(t => t.Name == "Base"));
        RejectGraph(missing);
        var abstractBody = Create(false);
        abstractBody.Types.Single(t => t.Name == "Base").Methods.Single(m => m.Name == "Read").GetILGenerator().LoadConstant(1);
        RejectGraph(abstractBody);
        foreach (var overrideSlot in new[] { false, true })
        {
            var invalid = Create(false);
            var parent = invalid.Types.Single(t => t.Name == "Derived");
            var child = invalid.AddClass("Example", "Invalid", parent);
            var method = overrideSlot ? child.AddOverride("Read", new(PrimitiveType.Boolean, []))
                : child.AddVirtualMethod("Read", new(PrimitiveType.Int32, []));
            var il = method.GetILGenerator(); if (overrideSlot) il.Emit(OpCode.Ldc_Bool, true); else il.LoadConstant(0); il.Return();
            RejectGraph(invalid);
        }
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        foreach (var manual in new[] { false, true })
        {
            var path = Path.Combine(directory, manual ? "Manual.dll" : "Builder.dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create(manual)));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                start.ArgumentList.Add(command); start.ArgumentList.Add(path);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
                try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
                if (process.ExitCode != (command == "run" ? 42 : 0) || (await stderr).Length != 0)
                    throw new Exception((await stdout) + (await stderr));
            }
        }
        Console.WriteLine("PASS class virtual/abstract slots, overrides and interface dispatch: CLR and native return 42");
    }
}
