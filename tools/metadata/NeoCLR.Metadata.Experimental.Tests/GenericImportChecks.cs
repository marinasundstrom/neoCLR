using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericImportChecks
{
    internal static void Run()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("GenericLibrary", new Version(1, 0, 0, 0)), core);
        var parameter = SignatureType.MethodParameter(0);
        var method = library.AddType("Example", "Algorithms").AddMethod("First",
            new MethodSignature(parameter, [SignatureType.ArrayOf(parameter)], ["T"]));
        method.LoadArgument(0); method.LoadConstant(0); method.LoadArrayElement(parameter); method.Return();
        var overload = method.DeclaringType!.AddMethod("First",
            new MethodSignature(parameter, [SignatureType.ArrayOf(parameter)], ["T", "U"]));
        overload.LoadArgument(0); overload.LoadConstant(0); overload.LoadArrayElement(parameter); overload.Return();
        var image = library.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        var definition = snapshot.MainModule.Methods.Single(m => m.GenericArity == 1);
        Check(definition.TryGetStaticGenericValueSignature(out var signature) && signature!.GenericParameterNames.Count == 1 && signature.ReturnType == parameter, "scoped generic signature");
        Check(!definition.TryGetStaticValueSignature(out _) && !definition.TryGetStaticPrimitiveSignature(out _), "nongeneric decoders stay narrow");
        var app = new AssemblyBuilder(new("GenericConsumer", new Version(1, 0, 0, 0)), core);
        var imported = app.ImportReference(definition, core);
        var arguments = new SignatureType[] { PrimitiveType.Int32 };
        var call = imported.MakeGenericInstance(arguments); arguments[0] = PrimitiveType.Int64;
        Check(call.TypeArguments[0].Primitive == PrimitiveType.Int32 && call.Signature.ReturnType.Primitive == PrimitiveType.Int32, "copied substituted argument");
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        entry.LoadConstant(1); entry.NewArray(PrimitiveType.Int32); entry.Duplicate();
        entry.LoadConstant(0); entry.LoadConstant(42); entry.StoreArrayElement(PrimitiveType.Int32);
        entry.Emit(OpCode.Call, call); entry.Return();
        Reject<ArgumentException>(() => entry.Call(imported));
        Reject<ArgumentException>(() => imported.MakeGenericInstance());
        Reject<ArgumentException>(() => imported.MakeGenericInstance(PrimitiveType.Void));
        Reject<ArgumentException>(() => entry.Call(imported.MakeGenericInstance(parameter)));
        Reject<ArgumentException>(() => entry.Call(imported.MakeGenericInstance(SignatureType.TypeParameter(0))));
        Check(imported.MakeGenericInstance(SignatureType.ArrayOf(PrimitiveType.Int32)).Signature.ParameterTypes[0].ArrayElement?.ArrayElement?.Primitive == PrimitiveType.Int32, "nested imported substitution");
        Reject<ArgumentNullException>(() => imported.MakeGenericInstance(null!));
        Reject<ArgumentException>(() => entry.Emit(OpCode.Add, call));
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => other.AddFunction("Main").Call(call));
        var foreignType = other.AddClass("", "Foreign");
        Reject<ArgumentException>(() => imported.MakeGenericInstance(foreignType));
        var owned = app.AddClass("", "Item");
        var field = owned.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var constructor = owned.AddConstructor([]); constructor.Return();
        var roundtrip = app.AddType("", "Checks").AddMethod("Roundtrip", 0, true);
        var firstItem = imported.MakeGenericInstance(owned);
        roundtrip.LoadConstant(1); roundtrip.NewArray(owned); roundtrip.Duplicate(); roundtrip.LoadConstant(0);
        roundtrip.NewObject(constructor); roundtrip.Duplicate(); roundtrip.LoadConstant(42); roundtrip.StoreField(field);
        roundtrip.StoreArrayElement(owned); roundtrip.Call(firstItem); roundtrip.LoadField(field); roundtrip.Return();
        var forward = app.AddFunction("Forward", new MethodSignature(parameter, [SignatureType.ArrayOf(parameter)], ["T"]));
        forward.LoadArgument(0); forward.Call(imported.MakeGenericInstance(parameter)); forward.Return();
        var appImage = app.Write();
        var read = AssemblyDefinition.ReadAssembly(appImage, false);
        Check(ReferenceEquals(read.MainModule.MemberReferences.Single(m => m.Name == "First").ResolveMethod(new Resolver(snapshot)), definition), "generic MemberRef resolution");
        var context = new AssemblyLoadContext("generic-import", isCollectible: true);
        try
        {
            context.LoadFromStream(new MemoryStream(image));
            var loaded = context.LoadFromStream(new MemoryStream(appImage));
            Check((int)loaded.EntryPoint!.Invoke(null, null)! == 42, "CLR executes imported MethodSpec");
            Check((int)loaded.GetType("Checks")!.GetMethod("Roundtrip")!.Invoke(null, null)! == 42, "CLR retains consumer-owned generic argument identity");
        }
        finally { context.Unload(); }
        _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        Check(projection.MainModule.Methods.Single(m => m.GenericArity == 1).TryGetStaticGenericValueSignature(out _), "native projection retains generic signature");
    }
    private sealed class Resolver(AssemblyDefinition assembly) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => assembly.Identity.Equals(identity) ? assembly : null;
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
