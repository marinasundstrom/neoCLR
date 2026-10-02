using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeNominalSignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NominalLibrary", new Version(1, 0, 0, 0)), core);
        var factory = library.AddType("Example", "Factory");
        var box = library.AddClass("Example", "Box");
        var other = library.AddClass("Other", "Box");
        var field = box.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = box.AddConstructor(new[] { PrimitiveType.Int32 });
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(field); ctor.Return();
        var create = factory.AddMethod("Create", new MethodSignature(box, [PrimitiveType.Int32]));
        create.LoadArgument(0); create.NewObject(ctor); create.Return();
        var echo = box.AddInstanceMethod("Echo", new MethodSignature(box, [box]));
        echo.LoadArgument(1); echo.Return();
        var distinct = factory.AddMethod("Other", new MethodSignature(other, [other]));
        distinct.LoadArgument(0); distinct.Return();
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var bytes = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(bytes);
            var readBox = snapshot.MainModule.Types.Single(t => t.Namespace == "Example" && t.Name == "Box");
            var readCreate = snapshot.MainModule.Methods.Single(m => m.Name == "Create");
            Check(readCreate.TryGetSignature(out var signature), "nominal signature available");
            Check(signature!.ReturnType.ClassType is null && signature.ReturnType.ImportedType is null &&
                ReferenceEquals(signature.ReturnType.ReferencedType!.Resolve(), readBox), "immutable canonical definition reference");
            Check(signature.ReturnType.ToString() == "Example.Box", "nominal diagnostic name");
            Check(!readCreate.TryGetStaticPrimitiveSignature(out var primitive) && primitive is null &&
                !readCreate.TryGetStaticValueSignature(out var value) && value is null &&
                !readCreate.TryGetStaticInt32Signature(out _, out _), "primitive helpers reject nominal results");
            var readOther = snapshot.MainModule.Methods.Single(m => m.Name == "Other");
            Check(readOther.TryGetSignature(out var otherSignature) && otherSignature!.ParameterTypes[0].ReferencedType!.Resolve().Namespace == "Other", "same short name distinct identity");
            Check(!readOther.TryGetStaticPrimitiveSignature(out _) && !readOther.TryGetStaticValueSignature(out _), "primitive helpers reject nominal parameters");
            var readEcho = readBox.Methods.Single(m => m.Name == "Echo");
            Check(readEcho.TryGetSignature(out var echoSignature) && echoSignature!.ReturnType == signature.ReturnType &&
                echoSignature.ParameterTypes[0] == signature.ReturnType, "same nominal type has equal signature identity");
            var app = new AssemblyBuilder(new("NominalConsumer", new Version(1, 0, 0, 0)), core);
            Reject<ArgumentException>(() => app.AddFunction("Unimported", signature));
            var importedCreate = app.ImportReference(readCreate, core);
            var importedEcho = app.ImportReference(readBox.Methods.Single(m => m.Name == "Echo"), core);
            Check(ReferenceEquals(importedCreate.Signature.ReturnType.ImportedType, importedEcho.Signature.ParameterTypes[0].ImportedType), "output reference interning");
            var main = app.AddFunction("Main"); app.EntryPoint = main;
            var local = main.DeclareLocal(importedCreate.Signature.ReturnType);
            main.LoadConstant(42); main.Call(importedCreate); main.StoreLocal(local);
            main.LoadLocal(local); main.LoadLocal(local); main.Call(importedEcho);
            main.LoadField(app.ImportReference(readBox.Fields.Single(), core)); main.Return();
            _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
            var context = new AssemblyLoadContext("nominal-" + binary, isCollectible: true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR nominal imported signature execution");
            }
            finally { context.Unload(); }
            Check(snapshot.Write().SequenceEqual(bytes), "opaque native roundtrip");
        }
        var vector = SignatureType.ArrayOf(box);
        var arrayMethod = library.AddFunction("ArrayIdentity", new MethodSignature(vector, [vector]));
        arrayMethod.LoadArgument(0); arrayMethod.Return();
        var vectorImage = RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core);
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(vectorImage));
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
