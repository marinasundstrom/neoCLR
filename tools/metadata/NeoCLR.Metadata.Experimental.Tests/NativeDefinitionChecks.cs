using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeDefinitionChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("DirectNative", new Version(2, 3, 4, 5)), core);
        var echo = library.AddFunction("Exämple", "Echo", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        echo.LoadArgument(0); echo.Return();
        var flag = library.AddFunction("Exämple", "Echo", new MethodSignature(PrimitiveType.Boolean, [PrimitiveType.Boolean]));
        flag.LoadArgument(0); flag.Return();
        var notify = library.AddFunction("Notify", returnsValue: false); notify.Return();
        var main = library.AddFunction("Main"); main.LoadConstant(42); main.Return(); library.EntryPoint = main;
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var expected = (byte[])image.Clone();
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            Array.Clear(image);
            Check(read.IsNative && read.Identity.Equals(library.Identity), "native identity");
            Check(read.MainModule.Mvid == Guid.Empty && read.MainModule.Types.Count == 0, "no invented module id or container type");
            Check(read.MainModule.Functions.Count == 4 && read.MainModule.Methods.Count == 4, "all namespace functions");
            var method = read.MainModule.Functions[0];
            Check(method.Namespace == "Exämple" && method.Name == "Echo" && method.DeclaringType is null, "namespace ownership");
            Check(ReferenceEquals(method, read.MainModule.GetMethodDefinition(method.MetadataToken)), "canonical lookup");
            Check(method.TryGetSignature(out var signature) && signature!.ReturnType.Primitive == PrimitiveType.Int32 && signature.ParameterTypes.Single().Primitive == PrimitiveType.Int32, "typed signature");
            Check(method.TryGetStaticInt32Signature(out var count, out var result) && count == 1 && result, "existing signature helper");
            Check(read.MainModule.Functions[1].TryGetStaticPrimitiveSignature(out var boolean) && boolean!.ReturnType == PrimitiveType.Boolean, "Boolean remains distinct");
            Check(read.MainModule.Functions[2].TryGetSignature(out var noResult) && noResult!.ReturnType.Primitive == PrimitiveType.Void, "no-result signature");
            Check(ReferenceEquals(read.EntryPoint, read.MainModule.Functions[3]), "native entry identity");
            Reject<NotSupportedException>(() => method.GetSignature());
            Reject<NotSupportedException>(() => { _ = method.Body; });
            Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(method));
            Reject<InvalidOperationException>(() => read.EntryPoint = null);
            Reject<InvalidOperationException>(() => read.MainModule.Functions.Add(new MethodDefinition("Extra", new MethodSignature(PrimitiveType.Void, []))));
            Check(read.Write().SequenceEqual(expected), "owned opaque image roundtrip");
            var output = read.Write(); Array.Clear(output); Check(read.Write().SequenceEqual(expected), "write does not expose snapshot buffer");
            Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(expected[..^1]));
        }
        Check(echo.Definition.TryGetSignature(out var authored) && ReferenceEquals(authored, echo.Definition.AuthoredSignature), "authored signature shares the same API");
        var cli = AssemblyDefinition.ReadAssembly(library.Write(), expectedExtended: false);
        Check(!cli.IsNative && cli.MainModule.Functions[0].TryGetSignature(out var cliSignature) && cliSignature!.ReturnType.Primitive == PrimitiveType.Int32, "ordinary CLI reader shares signature query");
        var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var dependency = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        var imported = consumer.ImportReference(dependency.MainModule.Functions[0], core);
        var call = consumer.AddFunction("Call"); call.LoadConstant(42); call.Call(imported); call.Return();
        Check(imported.Namespace == "Exämple" && imported.Name == "Echo", "native namespace import");
        var reread = AssemblyDefinition.ReadNativeAssembly(dependency.Write());
        Check(ReferenceEquals(imported, consumer.ImportReference(reread.MainModule.Functions[0], core)), "identical native snapshot import");
        var importedCall = consumer.AddFunction("ImportedCall"); importedCall.LoadConstant(42); importedCall.Call(imported); importedCall.Return();
        var conflict = new AssemblyBuilder(library.Identity, core);
        var changed = conflict.AddFunction("Exämple", "Echo", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        changed.LoadConstant(0); changed.Return();
        var changedSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(conflict.WriteNativeAssembly(), core));
        Reject<InvalidDataException>(() => consumer.ImportReference(changedSnapshot.MainModule.Functions[0], core));
        Reject<InvalidDataException>(() => consumer.ImportReference(dependency.MainModule.Functions[0], new("WrongCore", new Version(1, 0, 0, 0))));
        var caller = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(consumer.WriteNativeAssembly(), core));
        var reference = caller.MainModule.AssemblyReferences.Single(r => r.Identity.Equals(library.Identity));
        Check(ReferenceEquals(reference.Resolve(new Resolver(dependency)), dependency), "existing exact resolver accepts native definitions");
        Reject<InvalidDataException>(() => reference.Resolve(new Resolver(null)));
        Reject<InvalidDataException>(() => reference.Resolve(new Resolver(caller)));
        library.AddType("Example", "Unsupported");
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core)));
        var generic = new AssemblyBuilder(new("Generic", new Version(1, 0, 0, 0)), core);
        var identity = generic.AddFunction("Identity", new MethodSignature(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]));
        identity.LoadArgument(0); identity.Return();
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(generic.WriteNativeAssembly(), core)));
    }
    private sealed class Resolver(AssemblyDefinition? assembly) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => assembly;
    }
    private static void Check(bool result, string message) { if (!result) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
