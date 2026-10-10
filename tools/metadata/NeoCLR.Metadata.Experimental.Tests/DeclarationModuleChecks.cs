using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class DeclarationModuleChecks
{
    internal static void Run(string? output)
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new AssemblyIdentity("Package", new Version(1, 0, 0, 0)), core);
        builder.DefineModule("Example.Empty");
        var math = builder.DefineModule("Example.Math");
        if (!ReferenceEquals(math, builder.DefineModule("Example.Math"))) throw new Exception("unstable module");
        var item = builder.DefineModule("Example.Data").AddClass("Item");
        item.AddNestedValueType("Nested");
        item.AddField("Value", PrimitiveType.Int32);
        var owned = item.AddMethod("Owned", new MethodSignature(PrimitiveType.Int32, []));
        owned.LoadConstant(7); owned.Return();
        builder.DefineModule("Example.Math.Advanced").AddClass("Child");
        builder.DefineModule("").AddClass("Global");
        math.AddConstant("Pi", Math.PI);
        var f = math.AddFunction("Read", new MethodSignature(PrimitiveType.Int32, []));
        f.LoadConstant(42); f.Return(); builder.EntryPoint = f;
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteLibraryBinary(builder), NativeModuleContainer.WriteLibraryBinary(builder.WriteNativeAssembly()) })
        {
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            var modules = read.GetModules();
            if (!modules.Select(m => m.Name).SequenceEqual(new[] { "", "Example.Data", "Example.Empty", "Example.Math", "Example.Math.Advanced" }) || modules.Any(m => m.IsProjection)) throw new Exception("module table round trip");
            if (modules[2].GetMembers().Count != 0 || modules[3].GetMembers().Count != 2 || modules[1].GetMembers().Count != 1) throw new Exception("module membership");
            foreach (var m in modules)
                if (m.GetMembers().Any(member => !ReferenceEquals(member.DeclaringModule, m))) throw new Exception("module owner");
            var other = new AssemblyBuilder(new AssemblyIdentity("DifferentPackage", new Version(1, 0, 0, 0)), core);
            other.DefineModule("Example.Math");
            var catalog = new MetadataLoadContext([read, AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(other.WriteNativeAssembly()))]);
            if (catalog.GetModules().Count(m => m.Name == "Example.Math") != 2) throw new Exception("same path across assemblies merged");
            CheckViews(catalog, read);
            var sameSnapshotContext = new MetadataLoadContext([read]);
            if (ReferenceEquals(catalog.Resolve(modules[3]), sameSnapshotContext.Resolve(modules[3]))) throw new Exception("module view leaked across contexts");
            var foreign = AssemblyDefinition.ReadNativeAssembly(image);
            try { catalog.Resolve(foreign.GetModules()[0]); throw new Exception("foreign snapshot admitted"); }
            catch (InvalidDataException) { }
            try { catalog.Resolve((DeclarationModuleDefinition)null!); throw new Exception("null module admitted"); }
            catch (ArgumentNullException) { }
        }
        var json = JsonNode.Parse(builder.WriteNativeAssembly())!;
        void Bad(Action<JsonNode> mutate)
        {
            var copy = json.DeepClone(); mutate(copy["assemblies"]![0]!["declaration_modules"]!);
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(copy.ToJsonString())); }
            catch (InvalidDataException) { return; }
            throw new Exception("invalid module table accepted");
        }
        Bad(table => table["version"] = 2);
        Bad(table => table["names"]!.AsArray().Add("Example.Math"));
        Bad(table => table["names"]!.AsArray().Clear());
        Bad(table => table["names"]!.AsArray().Add("Bad..Path"));
        json["assemblies"]![0]!.AsObject().Remove("declaration_modules");
        var legacy = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json.ToJsonString())));
        if (legacy.GetModules().Any(m => !m.IsProjection) || legacy.GetModules().Count != 4) throw new Exception("legacy projection");
        var legacyContext = new MetadataLoadContext([legacy]);
        CheckViews(legacyContext, legacy);
        foreach (var (name, moduleNames) in new[] {
            ("System.Runtime", new[] { "System", "System.Networking" }),
            ("Acme.CoffeeMaker", new[] { "Acme.CoffeeMaker", "Acme.CoffeeMaker.Factories" }) })
        {
            var package = new AssemblyBuilder(new(name, new(1, 0, 0, 0)), core);
            foreach (var moduleName in moduleNames) package.DefineModule(moduleName);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(package));
            var context = new MetadataLoadContext([snapshot]);
            if (!context.GetModules().Select(module => module.Name).SequenceEqual(moduleNames) ||
                context.GetModules().Any(module => module.Assembly.Name != name || module.GetMembers().Count != 0))
                throw new Exception("assembly packaging changed flat module names or synthesized parents");
        }
        if (output is not null) File.WriteAllBytes(output, builder.WriteNativeAssembly());
        Console.WriteLine("PASS logical module ownership, canonical context views, empty/global/child modules, foreign snapshots, legacy projection and invalid manifests");
    }

    private static void CheckViews(MetadataLoadContext catalog, AssemblyDefinition snapshot)
    {
        var assembly = catalog.Resolve(snapshot.Identity);
        var views = assembly.GetModules();
        if (!views.Select(m => m.Name).SequenceEqual(snapshot.GetModules().Select(m => m.Name))) throw new Exception("view ordering");
        foreach (var definition in snapshot.GetModules())
        {
            var view = views.Single(m => m.Name == definition.Name);
            if (!ReferenceEquals(view, catalog.Resolve(definition)) || !ReferenceEquals(view.Assembly, assembly) || view.IsProjection != definition.IsProjection)
                throw new Exception("canonical module identity");
            if (!view.GetMembers().Select(m => m.FullName).SequenceEqual(definition.GetMembers().Select(m => m.FullName))) throw new Exception("direct member projection");
            foreach (var member in view.GetMembers())
            {
                if (!ReferenceEquals(member.Module, view) || !ReferenceEquals(member.Assembly, assembly))
                    throw new Exception("member ownership");
                if (member.Function is { } function && !ReferenceEquals(function, view.GetFunctions().Single(f => f.Name == member.Name)))
                    throw new Exception("function view identity");
                if (member.Type is { } type && !ReferenceEquals(type, assembly.GetTypes().Single(t => t.FullName == member.FullName)))
                    throw new Exception("type view identity");
            }
            try { ((IList<AssemblyMemberInfo>)view.GetMembers()).Clear(); throw new Exception("mutable module members"); }
            catch (NotSupportedException) { }
        }
        var item = assembly.GetTypes().Single(type => type.Name == "Item");
        var nested = assembly.GetTypes().Single(type => type.Name == "Nested");
        if (!ReferenceEquals(item.Module, nested.Module) || item.Module.Name != "Example.Data" ||
            item.MetadataScopeName != snapshot.MainModule.Name || nested.MetadataScopeName != snapshot.MainModule.Name ||
            !item.Module.GetTypes().Contains(nested) || item.Module.GetMembers().Any(member => member.Name == "Nested"))
            throw new Exception("nested type logical ownership and physical token scope");
        var owned = item.GetMethods().Single();
        if (!ReferenceEquals(owned.Module, item.Module) || owned.MetadataScopeName != snapshot.MainModule.Name ||
            item.Module.GetFunctions().Count != 0) throw new Exception("type-owned method leaked into module functions");
        foreach (var member in assembly.GetMembers())
            if (!ReferenceEquals(member.Module, views.Single(m => m.Name == member.Namespace))) throw new Exception("aggregate owner identity");
        var mathViews = catalog.GetModules().Where(m => m.Name == "Example.Math").ToArray();
        if (mathViews.Distinct().Count() != mathViews.Length) throw new Exception("same-name assembly views merged");
        if (!ReferenceEquals(views[0], assembly.GetModules()[0])) throw new Exception("unstable module view");
        try { ((IList<ModuleInfo>)views).Clear(); throw new Exception("mutable module list"); }
        catch (NotSupportedException) { }
    }
}
