using System.Reflection;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericMethodConstraintChecks
{
    internal static void Run()
    {
        ExternalBounds();
        foreach (var detached in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("MethodBounds" + detached, new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var bound = graph.AddInterface("Example", "Number");
            var implementation = graph.AddValueType("Example", "Count");
            implementation.AddInterfaceImplementation(bound);
            var signature = new MethodSignature(PrimitiveType.Int32, [], ["T"]);
            MethodBuilder method;
            if (detached)
            {
                var definition = new MethodDefinition("Answer", signature);
                definition.AddInterfaceConstraint(0, bound.Definition);
                graph.Definition.MainModule.Functions.Add(definition);
                method = MethodBuilder.ForDefinition(definition);
            }
            else
            {
                method = graph.AddFunction("Answer", signature);
                method.AddInterfaceConstraint(0, bound);
            }
            method.GetILGenerator().LoadConstant(42); method.GetILGenerator().Return();
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(0, bound));
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(1, bound));
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(0, implementation));
            Reject<ArgumentException>(() => method.MakeGenericInstance(PrimitiveType.Int32));
            Reject<ArgumentException>(() => method.MakeGenericInstance(SignatureType.MethodParameter(0)));
            var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
            main.GetILGenerator().Call(method.MakeGenericInstance(implementation)); main.GetILGenerator().Return();
            graph.EntryPoint = main;
            var cli = graph.Write();
            var loaded = Assembly.Load(cli);
            if ((int)loaded.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("CLI bounded generic execution");
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
            foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, expectedExtended: false), AssemblyDefinition.ReadNativeAssembly(native), AssemblyDefinition.ReadAssembly(projected, expectedExtended: false) })
            {
                var definition = snapshot.MainModule.Functions.Single(m => m.Name == "Answer");
                if (definition.InterfaceConstraints.Single().InterfaceType.Name != "Number") throw new Exception("method constraint lost");
                var context = new MetadataLoadContext([snapshot]);
                var view = context.Resolve(definition);
                var parameter = (MethodGenericParameterTypeInfo)view.GetGenericArguments().Single();
                if (!ReferenceEquals(parameter.GetInterfaceConstraints().Single(), context.Resolve(definition.InterfaceConstraints.Single().InterfaceType))) throw new Exception("method bound facade");
                Reject<InvalidOperationException>(() => definition.AddInterfaceConstraint(0, definition.InterfaceConstraints[0].InterfaceType));
                var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
                Reject<NotSupportedException>(() => consumer.ImportReference(definition, graph.CoreLibrary));
            }
            var json = graph.WriteNativeAssembly();
            foreach (var bad in new[] { -1, 1 })
            {
                var tree = JsonNode.Parse(json)!;
                var function = tree["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Answer")!;
                function["generic_constraints"]![0]!["parameter"] = bad;
                Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(tree.ToJsonString())));
            }
            var duplicate = JsonNode.Parse(json)!;
            var rows = duplicate["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Answer")!["generic_constraints"]!.AsArray();
            rows.Add(rows[0]!.DeepClone());
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(duplicate.ToJsonString())));
            if (Environment.GetEnvironmentVariable("NEOCLR_METHOD_BOUND_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
            var extraBound = graph.AddInterface("Example", "Other");
            method.AddInterfaceConstraint(0, extraBound);
            Reject<InvalidDataException>(() => graph.Write());
        }
    }
    private static void ExternalBounds()
    {
        foreach (var manual in new[] { false, true })
        {
            var core = new AssemblyIdentity("System.Runtime", new Version(10, 0, 0, 0));
            var contracts = new AssemblyBuilder(new("BoundContracts" + manual, new Version(1, 0, 0, 0)), core);
            contracts.AddInterface("Example", "Number");
            var contractImage = RuntimeAssemblyContainer.WriteBinary(contracts);
            var graph = new AssemblyBuilder(new("ExternalBounds" + manual, new Version(1, 0, 0, 0)), core);
            var bound = graph.CreateInterfaceReference(contracts.Identity, core, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(contractImage)), "Example", "Number");
            var method = graph.AddFunction("Answer", new(PrimitiveType.Int32, [], ["T"]));
            if (manual) method.Definition.AddInterfaceConstraint(0, graph.Definition.MainModule.ImportReference(contracts.Identity, "Example", "Number"));
            else method.AddInterfaceConstraint(0, bound);
            method.GetILGenerator().LoadConstant(42); method.GetILGenerator().Return();
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(0, bound));
            Reject<InvalidDataException>(() => graph.Write());
            graph.CompleteInterfaceReference(bound);
            var implementation = graph.AddValueType("Example", "Count");
            implementation.AddInterfaceImplementation(bound);
            var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
            main.GetILGenerator().Call(method.MakeGenericInstance(implementation)); main.GetILGenerator().Return();
            graph.EntryPoint = main;
            var cli = graph.Write();
            var context = new System.Runtime.Loader.AssemblyLoadContext("bounds", true);
            try
            {
                var dependency = context.LoadFromStream(new MemoryStream(contracts.Write()));
                context.Resolving += (_, name) => name.Name == dependency.GetName().Name ? dependency : null;
                var assembly = context.LoadFromStream(new MemoryStream(cli));
                if ((int)assembly.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("external CLI method bound execution");
            }
            finally { context.Unload(); }
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, expectedExtended: false), AssemblyDefinition.ReadNativeAssembly(native) })
            {
                var definition = snapshot.MainModule.Functions.Single(m => m.Name == "Answer");
                var dependency = AssemblyDefinition.ReadNativeAssembly(contractImage);
                var load = new MetadataLoadContext([snapshot, dependency]);
                var parameter = (MethodGenericParameterTypeInfo)load.Resolve(definition).GetGenericArguments().Single();
                if (!ReferenceEquals(parameter.GetInterfaceConstraints().Single(), load.Resolve(dependency.MainModule.Types.Single(t => t.Name == "Number").ToReference())))
                    throw new Exception("external bound canonical resolution");
                var missing = new MetadataLoadContext([snapshot]);
                Reject<InvalidDataException>(() => ((MethodGenericParameterTypeInfo)missing.Resolve(definition).GetGenericArguments().Single()).GetInterfaceConstraints());
                foreach (var wrongVersion in new[] { false, true })
                {
                    var wrong = new AssemblyBuilder(wrongVersion ? new AssemblyIdentity(contracts.Identity.Name, new Version(2, 0, 0, 0)) : contracts.Identity, core);
                    wrong.AddClass("Example", "Number");
                    var wrongLoad = new MetadataLoadContext([snapshot, AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(wrong))]);
                    Reject<InvalidDataException>(() => ((MethodGenericParameterTypeInfo)wrongLoad.Resolve(definition).GetGenericArguments().Single()).GetInterfaceConstraints());
                }
            }
            if (Environment.GetEnvironmentVariable("NEOCLR_EXTERNAL_BOUND_ARTIFACT") is { } path)
            {
                File.WriteAllBytes(path, native);
                File.WriteAllBytes(path + ".contracts.neox", contractImage);
            }
        }
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
