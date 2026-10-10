using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeInitAccessorChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeInit", new Version(1, 0, 0, 0)), core);
        var box = library.AddClass("Example", "Box");
        var field = box.AddField("stored", PrimitiveType.Int32, isReadOnly: true);
        var get = box.AddInstanceMethod("get_Value", new(PrimitiveType.Int32, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        var set = box.AddInstanceMethod("set_Value", new(PrimitiveType.Void, [PrimitiveType.Int32]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(field); set.Return();
        var property = box.AddProperty("Value", PrimitiveType.Int32, get, set, isInitOnly: true);
        Check(property.Definition.IsInitOnly, "authored initialization contract");
        byte[]? consumerImage = null;
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            var imported = read.MainModule.Properties.Single();
            Check(imported.IsInitOnly && imported.SetMethod is not null, "native accessor contract round trip");
            var view = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([read]).Resolve(read.Identity).GetTypes().Single().GetProperties().Single();
            Check(view.IsInitOnly, "introspection preserves init contract");
            Check(read.Write().SequenceEqual(image), "opaque image preserved");
            var consumer = new AssemblyBuilder(new("InitConsumer", new Version(1, 0, 0, 0)), core);
            var helper = consumer.AddType("Example", "Helpers");
            var apply = helper.AddMethod("Apply", new(PrimitiveType.Int32, [consumer.ImportReference(imported.DeclaringType, core)]));
            apply.LoadArgument(0); apply.LoadConstant(43); apply.Call(consumer.ImportReference(imported.SetMethod!, core));
            apply.LoadArgument(0); apply.Call(consumer.ImportReference(imported.GetMethod!, core)); apply.Return();
            consumerImage = consumer.Write();
        }
        Reject(() => box.AddProperty("NoSetter", PrimitiveType.Int32, get, isInitOnly: true));
        var staticSetter = box.AddMethod("Static", new(PrimitiveType.Void, [PrimitiveType.Int32])); staticSetter.Return();
        Reject(() => box.AddProperty("Static", PrimitiveType.Int32, setter: staticSetter, isInitOnly: true));
        Check(AssemblyDefinition.ReadAssembly(library.Write(), expectedExtended: false).MainModule.Properties.Single().IsInitOnly, "CLI reader init contract");
        var context = new System.Runtime.Loader.AssemblyLoadContext("init-accessors", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(library.Write()));
            var setter = assembly.GetType("Example.Box")!.GetProperty("Value")!.SetMethod!;
            Check(setter.ReturnParameter.GetRequiredCustomModifiers().Single().FullName == "System.Runtime.CompilerServices.IsExternalInit", "CLI init modreq");
            var instance = System.Runtime.CompilerServices.RuntimeHelpers.GetUninitializedObject(setter.DeclaringType!);
            setter.Invoke(instance, [42]);
            Check((int)setter.DeclaringType!.GetProperty("Value")!.GetValue(instance)! == 42, "CLR reflection invokes init setter outside construction");
            var consumer = context.LoadFromStream(new MemoryStream(consumerImage!));
            Check((int)consumer.GetType("Example.Helpers")!.GetMethod("Apply")!.Invoke(null, [instance])! == 43, "imported CLI call retains required modifier");
        }
        finally { context.Unload(); }
        var illegal = box.AddInstanceMethod("Mutate", new(PrimitiveType.Void, [PrimitiveType.Int32]));
        illegal.LoadArgument(0); illegal.LoadArgument(1); illegal.StoreField(field); illegal.Return();
        try { library.WriteNativeAssembly(); throw new Exception("ordinary readonly mutation was accepted"); }
        catch (InvalidDataException error) { Check(error.Message.Contains("readonly field"), "ordinary methods retain readonly restriction"); }
        Console.WriteLine("PASS native init accessor authoring, containers and invalid shapes");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    { try { action(); } catch (ArgumentException) { return; } throw new Exception("expected ArgumentException"); }
}
