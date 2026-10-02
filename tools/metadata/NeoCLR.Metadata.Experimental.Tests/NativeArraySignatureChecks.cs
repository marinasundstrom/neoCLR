using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeArraySignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeArrays", new Version(1, 0, 0, 0)), core);
        var box = library.AddClass("Example", "Box");
        var value = box.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = box.AddConstructor(new[] { PrimitiveType.Int32 });
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var holder = library.AddClass("Example", "Holder");
        var boxes = SignatureType.ArrayOf(box);
        var items = holder.AddField("Items", boxes, FieldVisibility.Public);
        var holderCtor = holder.AddConstructor(new MethodSignature(PrimitiveType.Void, [boxes]));
        holderCtor.LoadArgument(0); holderCtor.LoadArgument(1); holderCtor.StoreField(items); holderCtor.Return();
        var echo = holder.AddMethod("Echo", new MethodSignature(boxes, [boxes])); echo.LoadArgument(0); echo.Return();
        foreach (var element in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Boolean, PrimitiveType.String })
        {
            var array = SignatureType.ArrayOf(element);
            var method = holder.AddMethod("Echo" + element, new MethodSignature(array, [array])); method.LoadArgument(0); method.Return();
        }
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            var readBox = read.MainModule.Types.Single(t => t.Name == "Box");
            var readHolder = read.MainModule.Types.Single(t => t.Name == "Holder");
            var readEcho = readHolder.Methods.Single(m => m.Name == "Echo");
            Check(readEcho.TryGetSignature(out var signature) && ReferenceEquals(signature!.ReturnType.ArrayElement!.ReferencedType!.Resolve(), readBox), "nominal vector identity");
            Check(readHolder.Fields.Single().TryGetSignature(out var fieldSignature) && signature!.ReturnType == fieldSignature, "field and method array type identity");
            Check(!readEcho.TryGetStaticValueSignature(out _) && !readEcho.TryGetStaticPrimitiveSignature(out _), "primitive vector helper rejects nominal vector");
            foreach (var method in readHolder.Methods.Where(m => m.Name.StartsWith("Echo") && m.Name != "Echo"))
                Check(method.TryGetStaticValueSignature(out var primitive) && primitive!.ReturnType.ArrayElement!.Primitive is not null &&
                    !method.TryGetStaticPrimitiveSignature(out _), "primitive array helper recognizes vectors only");
            var output = new AssemblyBuilder(new("NativeArrayConsumer", new Version(1, 0, 0, 0)), core);
            Reject<ArgumentException>(() => output.AddFunction("Unimported", signature!));
            var field = output.ImportReference(readHolder.Fields.Single(), core);
            var element = field.FieldType.ArrayElement!;
            var entry = output.AddFunction("Main"); output.EntryPoint = entry;
            var arrayLocal = entry.DeclareLocal(field.FieldType);
            entry.LoadConstant(1); entry.NewArray(element); entry.StoreLocal(arrayLocal);
            entry.LoadLocal(arrayLocal); entry.LoadConstant(0); entry.LoadConstant(42);
            entry.NewObject(output.ImportReference(readBox.Methods.Single(), core)); entry.StoreArrayElement(element);
            entry.LoadLocal(arrayLocal); entry.Call(output.ImportReference(readEcho, core));
            entry.NewObject(output.ImportReference(readHolder.Methods.Single(m => m.Name == ".ctor"), core));
            entry.LoadField(field); entry.LoadConstant(0); entry.LoadArrayElement(element);
            entry.LoadField(output.ImportReference(readBox.Fields.Single(), core)); entry.Return();
            _ = RuntimeAssemblyContainer.WriteBinary(output.WriteNativeAssembly(), core);
            var context = new AssemblyLoadContext("native-array-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(output.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR native array imports");
            }
            finally { context.Unload(); }
            entry.ClearBody(); entry.LoadConstant(1); entry.NewArray(PrimitiveType.Int32); entry.Call(output.ImportReference(readEcho, core)); entry.Emit(OpCode.Pop); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => output.WriteNativeAssembly());
            entry.ClearBody(); entry.LoadDefault(field.DeclaringType); entry.LoadConstant(1); entry.NewArray(PrimitiveType.Int32); entry.StoreField(field); entry.LoadConstant(0); entry.Return();
            Reject<InvalidDataException>(() => output.Write());
            Check(read.Write().SequenceEqual(image), "opaque array image roundtrip");
        }
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
