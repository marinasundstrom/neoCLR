using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExternalNativeSignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var a = new AssemblyBuilder(new("PayloadLibrary", new Version(1, 2, 0, 0)), core);
        var box = a.AddClass("Example", "Box");
        var value = box.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = box.AddConstructor(new[] { PrimitiveType.Int32 });
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var readA = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(a.WriteNativeAssembly(), core));
        var b = new AssemblyBuilder(new("HolderLibrary", new Version(1, 0, 0, 0)), core);
        var importedBox = b.ImportReference(readA.MainModule.Types.Single(), core);
        var holder = b.AddClass("Example", "Holder");
        var item = holder.AddField("Item", importedBox, FieldVisibility.Public);
        var holderCtor = holder.AddConstructor(new MethodSignature(PrimitiveType.Void, [importedBox]));
        holderCtor.LoadArgument(0); holderCtor.LoadArgument(1); holderCtor.StoreField(item); holderCtor.Return();
        var echo = holder.AddMethod("Echo", new MethodSignature(importedBox, [importedBox]));
        echo.LoadArgument(0); echo.Return();
        foreach (var binary in new[] { false, true })
        {
            var native = b.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var readB = AssemblyDefinition.ReadNativeAssembly(image);
            var reference = readB.MainModule.TypeReferences.Single();
            Check(reference.Module == readB.MainModule && reference.ResolutionScopeToken == readB.MainModule.AssemblyReferences.Single().MetadataToken, "scoped native type reference");
            var resolver = new Resolver(readA);
            Check(ReferenceEquals(reference.Resolve(resolver), readA.MainModule.Types.Single()), "exact external resolution");
            Reject<InvalidDataException>(() => reference.Resolve());
            Reject<InvalidDataException>(() => reference.Resolve(new Resolver(readB)));
            var readHolder = readB.MainModule.Types.Single();
            var readEcho = readHolder.Methods.Single(m => m.Name == "Echo");
            Check(readEcho.TryGetSignature(out var signature) && ReferenceEquals(signature!.ReturnType.ReferencedType, reference), "signature uses declared external reference");
            Check(readHolder.Fields.Single().TryGetSignature(out var fieldType) && fieldType == signature!.ReturnType, "field/method external signature identity");
            var output = new AssemblyBuilder(new("ExternalConsumer", new Version(1, 0, 0, 0)), core);
            Reject<InvalidDataException>(() => output.ImportReference(readEcho, core));
            Reject<InvalidDataException>(() => output.ImportReference(readHolder.Fields.Single(), core));
            var call = output.ImportReference(readEcho, core, resolver);
            var field = output.ImportReference(readHolder.Fields.Single(), core, resolver);
            var entry = output.AddFunction("Main"); output.EntryPoint = entry;
            entry.LoadConstant(42); entry.NewObject(output.ImportReference(readA.MainModule.Types.Single().Methods.Single(), core));
            entry.Call(call); entry.NewObject(output.ImportReference(readHolder.Methods.Single(m => m.Name == ".ctor"), core, resolver));
            entry.LoadField(field); entry.LoadField(output.ImportReference(readA.MainModule.Fields.Single(), core)); entry.Return();
            _ = RuntimeAssemblyContainer.WriteBinary(output.WriteNativeAssembly(), core);
            var context = new AssemblyLoadContext("external-native-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(a.Write())); context.LoadFromStream(new MemoryStream(b.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(output.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR three-assembly nominal signatures");
            }
            finally { context.Unload(); }
            var wrongVersion = new AssemblyBuilder(new("PayloadLibrary", new Version(2, 0, 0, 0)), core);
            wrongVersion.AddClass("Example", "Box");
            var wrong = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(wrongVersion.WriteNativeAssembly(), core));
            Reject<InvalidDataException>(() => output.ImportReference(readEcho, core, new Resolver(wrong)));
            var changed = new AssemblyBuilder(a.Identity, core);
            changed.AddClass("Example", "Box").AddField("Different", PrimitiveType.Int32);
            var changedRead = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(changed.WriteNativeAssembly(), core));
            Reject<InvalidDataException>(() => output.ImportReference(readEcho, core, new Resolver(changedRead)));
            var missing = new AssemblyBuilder(a.Identity, core); missing.AddClass("Example", "Different");
            var missingRead = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(missing.WriteNativeAssembly(), core));
            Reject<InvalidDataException>(() => reference.Resolve(new Resolver(missingRead)));
        }
    }
    private sealed class Resolver(AssemblyDefinition candidate) : IAssemblyResolver
    { public AssemblyDefinition? Resolve(AssemblyIdentity identity) => candidate; }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
