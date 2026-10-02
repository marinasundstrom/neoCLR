using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ImportedFieldChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("FieldLibrary", new Version(1, 0, 0, 0)), core);
        var type = library.AddClass("Example", "Box");
        type.AddField("Private", PrimitiveType.Int32);
        type.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        type.AddField("ReadOnly", PrimitiveType.Int32, FieldVisibility.Public, isReadOnly: true);
        var ctor = type.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var nativeImage = RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core);
        foreach (var native in new[] { false, true })
        {
            var snapshot = native ? AssemblyDefinition.ReadNativeAssembly(nativeImage) : AssemblyDefinition.ReadAssembly(library.Write(), false);
            var owner = snapshot.MainModule.Types.Single(t => t.Name == "Box");
            var valueDefinition = owner.Fields.Single(f => f.Name == "Value");
            var app = new AssemblyBuilder(new("FieldConsumer", new Version(1, 0, 0, 0)), core);
            Reject<InvalidDataException>(() => app.ImportReference(type.Fields[1].Definition, core));
            Reject<InvalidDataException>(() => app.ImportReference(new FieldDefinition("Detached", 6, PrimitiveType.Int32), core));
            var value = app.ImportReference(valueDefinition, core);
            Check(ReferenceEquals(value, app.ImportReference(valueDefinition, core)), "field reference interning");
            var readonlyField = app.ImportReference(owner.Fields.Single(f => f.Name == "ReadOnly"), core);
            Reject<InvalidDataException>(() => app.ImportReference(owner.Fields.Single(f => f.Name == "Private"), core));
            Reject<InvalidDataException>(() => app.ImportReference(valueDefinition, new("WrongCore", new Version(1, 0, 0, 0))));
            var entry = app.AddFunction("Main"); app.EntryPoint = entry;
            var local = entry.DeclareLocal(value.DeclaringType);
            entry.NewObject(app.ImportReference(owner.Methods.Single(m => m.Name == ".ctor"), core)); entry.StoreLocal(local);
            entry.LoadLocal(local); entry.LoadConstant(42); entry.StoreField(value);
            entry.LoadLocal(local); entry.Emit(OpCode.Ldfld, readonlyField); entry.Emit(OpCode.Pop);
            entry.LoadLocal(local); entry.LoadField(value); entry.Return();
            if (native) _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
            else Reject<InvalidDataException>(() => app.WriteNativeAssembly());
            var context = new AssemblyLoadContext("field-import-" + native, isCollectible: true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR imported field load/store");
            }
            finally { context.Unload(); }
            var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core).AddFunction("Main");
            Reject<ArgumentException>(() => foreign.LoadField(value));
            Reject<ArgumentException>(() => entry.Emit(OpCode.Call, value));
            entry.ClearBody(); entry.LoadConstant(1); entry.LoadField(value); entry.Return();
            Reject<InvalidDataException>(() => app.Write());
            entry.ClearBody(); entry.LoadDefault(value.DeclaringType); entry.Emit(OpCode.Ldstr, "wrong"); entry.StoreField(value); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => app.Write());
            entry.ClearBody(); entry.LoadDefault(value.DeclaringType); entry.LoadConstant(1); entry.StoreField(readonlyField); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => app.Write());
            if (native)
            {
                var changed = new AssemblyBuilder(library.Identity, core);
                changed.AddClass("Example", "Box").AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
                var changedSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(changed.WriteNativeAssembly(), core));
                Reject<InvalidDataException>(() => app.ImportReference(changedSnapshot.MainModule.Types.Single().Fields.Single(), core));
            }
        }
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
