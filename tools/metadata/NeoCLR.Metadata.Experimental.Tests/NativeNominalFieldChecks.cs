using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeNominalFieldChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NominalFields", new Version(1, 0, 0, 0)), core);
        var holder = library.AddClass("Example", "Holder");
        var box = library.AddClass("Example", "Box");
        var item = holder.AddField("Item", box, FieldVisibility.Public);
        holder.AddField("Original", box, FieldVisibility.Public, isReadOnly: true);
        box.AddField("Owner", holder, FieldVisibility.Public);
        var value = box.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var boxCtor = box.AddConstructor(new[] { PrimitiveType.Int32 });
        boxCtor.LoadArgument(0); boxCtor.LoadArgument(1); boxCtor.StoreField(value); boxCtor.Return();
        var holderCtor = holder.AddConstructor(new MethodSignature(PrimitiveType.Void, new SignatureType[] { box }));
        holderCtor.LoadArgument(0); holderCtor.LoadArgument(1); holderCtor.StoreField(item); holderCtor.Return();
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var bytes = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var read = AssemblyDefinition.ReadNativeAssembly(bytes);
            var readHolder = read.MainModule.Types.Single(t => t.Name == "Holder");
            var readBox = read.MainModule.Types.Single(t => t.Name == "Box");
            var readItem = readHolder.Fields[0];
            Check(readItem.TryGetSignature(out var signature) && ReferenceEquals(signature!.ReferencedType!.Resolve(), readBox), "forward nominal field identity");
            Check(readBox.Fields[0].TryGetSignature(out var back) && ReferenceEquals(back!.ReferencedType!.Resolve(), readHolder), "cyclic nominal field identity");
            Check(readHolder.Methods[0].TryGetSignature(out var ctorSignature) && ctorSignature!.ParameterTypes[0] == signature, "field/method signature identity");
            Check(!readItem.TryGetPrimitiveType(out var primitive) && primitive == PrimitiveType.Void, "nominal field is not primitive");
            Reject<NotSupportedException>(() => readItem.GetSignature());
            Reject<InvalidOperationException>(() => readItem.Name = "Changed");
            var app = new AssemblyBuilder(new("NominalFieldConsumer", new Version(1, 0, 0, 0)), core);
            var imported = app.ImportReference(readItem, core);
            var importedValue = app.ImportReference(readBox.Fields.Single(f => f.Name == "Value"), core);
            Check(ReferenceEquals(imported.FieldType.ImportedType, importedValue.DeclaringType), "interned imported field storage");
            var entry = app.AddFunction("Main"); app.EntryPoint = entry;
            var local = entry.DeclareLocal(imported.DeclaringType);
            entry.LoadConstant(1); entry.NewObject(app.ImportReference(readBox.Methods[0], core));
            entry.NewObject(app.ImportReference(readHolder.Methods[0], core)); entry.StoreLocal(local);
            entry.LoadLocal(local); entry.LoadConstant(42); entry.NewObject(app.ImportReference(readBox.Methods[0], core)); entry.StoreField(imported);
            entry.LoadLocal(local); entry.LoadField(imported); entry.LoadField(importedValue); entry.Return();
            _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
            var context = new AssemblyLoadContext("nominal-fields-" + binary, isCollectible: true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR nominal field store/load");
            }
            finally { context.Unload(); }
            entry.ClearBody(); entry.LoadDefault(imported.DeclaringType); entry.LoadConstant(7); entry.StoreField(imported); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => app.Write());
            entry.ClearBody(); entry.LoadDefault(imported.DeclaringType); entry.LoadDefault(imported.DeclaringType); entry.StoreField(imported); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => app.WriteNativeAssembly());
            var readOnly = app.ImportReference(readHolder.Fields[1], core);
            entry.ClearBody(); entry.LoadDefault(imported.DeclaringType); entry.LoadDefault(imported.FieldType); entry.StoreField(readOnly); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => app.WriteNativeAssembly());
            Check(read.Write().SequenceEqual(bytes), "native image roundtrip");
        }
        var cli = AssemblyDefinition.ReadAssembly(library.Write(), false);
        Check(!cli.MainModule.Types.Single(t => t.Name == "Holder").Fields[0].TryGetSignature(out var unsupported) && unsupported is null, "CLI nominal field decoding remains explicit unsupported profile");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
