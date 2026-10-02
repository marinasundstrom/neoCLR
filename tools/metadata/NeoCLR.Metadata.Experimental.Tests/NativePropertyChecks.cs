using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativePropertyChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeProperties", new Version(1, 0, 0, 0)), core);
        var box = library.AddClass("Example", "Box");
        var storage = box.AddField("stored", PrimitiveType.Int32);
        var ctor = box.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var get = box.AddInstanceMethod("Read", new(PrimitiveType.Int32, [])); get.LoadArgument(0); get.LoadField(storage); get.Return();
        var set = box.AddInstanceMethod("Write", new(PrimitiveType.Void, [PrimitiveType.Int32])); set.LoadArgument(0); set.LoadArgument(1); set.StoreField(storage); set.Return();
        var property = box.AddProperty("Value", PrimitiveType.Int32, get, set);
        var same = box.AddInstanceMethod("Self", new(box, Array.Empty<SignatureType>())); same.LoadArgument(0); same.Return();
        box.AddProperty("Same", box, same);
        var array = box.AddInstanceMethod("Array", new(SignatureType.ArrayOf(box), Array.Empty<SignatureType>())); array.LoadConstant(0); array.NewArray(box); array.Return();
        box.AddProperty("Items", SignatureType.ArrayOf(box), array);
        var staticType = library.AddType("Example", "Utility");
        var constant = staticType.AddMethod("Read", new(PrimitiveType.Int32, [])); constant.LoadConstant(42); constant.Return();
        staticType.AddProperty("Answer", PrimitiveType.Int32, constant);
        var sink = staticType.AddMethod("Write", new(PrimitiveType.Void, [PrimitiveType.String]), MethodVisibility.Private); sink.Return();
        staticType.AddProperty("Sink", PrimitiveType.String, setter: sink);
        Check(property.Definition.TryGetSignature(out var authored, out var authoredStatic) && authored!.Primitive == PrimitiveType.Int32 && !authoredStatic, "authored logical property signature");
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            var owner = read.MainModule.Types.Single(t => t.Name == "Box");
            var value = owner.Properties.Single(p => p.Name == "Value");
            Check(read.MainModule.Properties.Count == 5 && ReferenceEquals(value, read.MainModule.GetPropertyDefinition(value.MetadataToken)) && ReferenceEquals(value.DeclaringType, owner), "canonical property ownership");
            Check(ReferenceEquals(value.GetMethod, owner.Methods.Single(m => m.Name == "Read")) && ReferenceEquals(value.SetMethod, owner.Methods.Single(m => m.Name == "Write")), "canonical accessors");
            Check(value.TryGetPrimitiveSignature(out var primitive, out var isStatic) && primitive == PrimitiveType.Int32 && !isStatic && (value.GetMethod!.Attributes & 0x800) != 0, "primitive signature and accessor flags");
            Check(owner.Properties.Single(p => p.Name == "Same").TryGetSignature(out var nominal, out _) && ReferenceEquals(nominal!.ReferencedType!.Resolve(), owner), "nominal property type");
            Check(owner.Properties.Single(p => p.Name == "Items").TryGetSignature(out var vector, out _) && ReferenceEquals(vector!.ArrayElement!.ReferencedType!.Resolve(), owner), "array property type");
            Check(!owner.Properties.Single(p => p.Name == "Items").TryGetPrimitiveSignature(out _, out _), "primitive helper rejects array");
            var answer = read.MainModule.Properties.Single(p => p.Name == "Answer");
            Check(answer.TryGetSignature(out _, out isStatic) && isStatic && answer.SetMethod is null, "static readonly property");
            var sinkProperty = read.MainModule.Properties.Single(p => p.Name == "Sink");
            Check(sinkProperty.GetMethod is null && (sinkProperty.SetMethod!.Attributes & 7) == 1, "write-only private accessor preserved");
            Reject<NotSupportedException>(() => value.GetSignature());
            var app = new AssemblyBuilder(new("PropertyConsumer", new Version(1, 0, 0, 0)), core);
            var main = app.AddFunction("Main"); app.EntryPoint = main;
            var local = main.DeclareLocal(app.ImportReference(owner, core));
            main.NewObject(app.ImportReference(owner.Methods.Single(m => m.Name == ".ctor"), core)); main.StoreLocal(local);
            main.LoadLocal(local); main.LoadConstant(42); main.Call(app.ImportReference(value.SetMethod!, core));
            main.LoadLocal(local); main.Call(app.ImportReference(value.GetMethod!, core)); main.Return();
            _ = app.WriteNativeAssembly();
            var context = new AssemblyLoadContext("native-properties-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR imported property accessor execution");
            }
            finally { context.Unload(); }
            Check(read.Write().SequenceEqual(image), "opaque property image roundtrip");
        }
        var index = box.AddInstanceMethod("Index", new(PrimitiveType.Int32, [PrimitiveType.Int32])); index.LoadArgument(1); index.Return();
        var authoredIndex = box.AddProperty("Item", PrimitiveType.Int32, index);
        Check(authoredIndex.Definition.TryGetSignature(out _, out var authoredIndices, out _) && authoredIndices.Count == 1, "authored indexed signature");
        var writeAt = box.AddInstanceMethod("WriteAt", new(PrimitiveType.Void, [PrimitiveType.Int32, PrimitiveType.String])); writeAt.Return();
        box.AddProperty("WriteOnly", PrimitiveType.String, setter: writeAt);
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var indexedImage = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var read = AssemblyDefinition.ReadNativeAssembly(indexedImage);
            var indexed = read.MainModule.Properties.Single(p => p.Name == "Item");
            Check(indexed.TryGetSignature(out var valueType, out var indices, out var isStatic) &&
                valueType!.Primitive == PrimitiveType.Int32 && indices.Count == 1 && indices[0].Primitive == PrimitiveType.Int32 && !isStatic,
                "indexed logical signature");
            Check(!indexed.TryGetSignature(out _, out _) && !indexed.TryGetPrimitiveSignature(out _, out _), "non-indexed helpers reject indexer");
            Reject<NotSupportedException>(() => ((IList<SignatureType>)indices)[0] = PrimitiveType.String);
            Check(ReferenceEquals(indexed.GetMethod, indexed.DeclaringType.Methods.Single(m => m.Name == "Index")), "indexed accessor identity");
            var writeOnly = read.MainModule.Properties.Single(p => p.Name == "WriteOnly");
            Check(writeOnly.GetMethod is null && writeOnly.TryGetSignature(out var writtenType, out var writtenIndices, out _) &&
                writtenType!.Primitive == PrimitiveType.String && writtenIndices.Count == 1 && writtenIndices[0].Primitive == PrimitiveType.Int32,
                "write-only index excludes setter value");
            Check(read.Write().SequenceEqual(indexedImage), "indexed image roundtrip");
            var app = new AssemblyBuilder(new("IndexConsumer", new Version(1, 0, 0, 0)), core);
            var main = app.AddFunction("Main"); app.EntryPoint = main;
            main.NewObject(app.ImportReference(indexed.DeclaringType.Methods.Single(m => m.Name == ".ctor"), core));
            main.LoadConstant(42); main.Call(app.ImportReference(indexed.GetMethod!, core)); main.Return();
            _ = app.WriteNativeAssembly();
            var context = new AssemblyLoadContext("native-index-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR imported indexed accessor execution");
            }
            finally { context.Unload(); }
        }
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
