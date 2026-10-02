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
        var staticLibrary = new AssemblyBuilder(new("StaticNative", new Version(1, 0, 0, 0)), core);
        var staticType = staticLibrary.AddType("Example", "Math");
        var staticMethod = staticType.AddMethod("Echo", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        staticMethod.LoadArgument(0); staticMethod.Return();
        var privateMethod = staticType.AddMethod("Hidden", new MethodSignature(PrimitiveType.Void, []), MethodVisibility.Private);
        privateMethod.Return();
        var staticRead = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(staticLibrary.WriteNativeAssembly(), core));
        var readType = staticRead.MainModule.Types.Single();
        Check(readType.MetadataToken == 0x02000002 && readType.Attributes == 0x100181 && !readType.IsValueType, "static type origin/category");
        Check(staticRead.MainModule.Functions.Count == 0 && readType.Methods.Count == 2 && ReferenceEquals(readType, readType.Methods[0].DeclaringType), "static method ownership");
        Check((readType.Methods[1].Attributes & 7) == 1, "private member access retained");
        Check(ReferenceEquals(readType, readType.ToReference().Resolve()), "native type reference identity");
        var staticImport = consumer.ImportReference(readType.Methods[0], core);
        Check(staticImport.DeclaringTypeName == "Math" && staticImport.Namespace == "Example", "native static method import");
        var staticCall = consumer.AddFunction("StaticCall"); staticCall.LoadConstant(42); staticCall.Call(staticImport); staticCall.Return();
        _ = consumer.WriteNativeAssembly();
        Reject<InvalidOperationException>(() => staticRead.MainModule.Types.Add(new TypeDefinition("Example", "Extra", 0x100181, null)));
        var instances = new AssemblyBuilder(new("InstanceNative", new Version(1, 0, 0, 0)), core);
        var instanceType = instances.AddClass("Example", "Calculator");
        var constructor = instanceType.AddConstructor(Array.Empty<PrimitiveType>()); constructor.Return();
        var value = instanceType.AddInstanceMethod("Value", new MethodSignature(PrimitiveType.Int32, []));
        value.LoadConstant(42); value.Return();
        foreach (var primitive in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Boolean, PrimitiveType.String })
            instanceType.AddField(primitive.ToString(), primitive, FieldVisibility.Public, isReadOnly: primitive == PrimitiveType.String);
        var instanceRead = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(instances.WriteNativeAssembly(), core));
        var instanceDefinition = instanceRead.MainModule.Types.Single();
        foreach (var field in instanceDefinition.Fields)
        {
            Check(field.TryGetPrimitiveType(out var primitive) && primitive.ToString() == field.Name, "native primitive field signature");
            Check(ReferenceEquals(field, instanceRead.MainModule.GetFieldDefinition(field.MetadataToken)) && ReferenceEquals(field.DeclaringType, instanceDefinition), "canonical field identity/owner");
            Check((field.Attributes & 7) == 6 && ((field.Attributes & 0x20) != 0) == (primitive == PrimitiveType.String), "field access/readonly flags");
            Reject<NotSupportedException>(() => field.GetSignature());
            Reject<InvalidOperationException>(() => field.Name = "Changed");
        }
        Check(instanceDefinition.Fields[0].MetadataToken == 0x04000001, "native field origin token");
        var ctorDefinition = instanceDefinition.Methods.Single(m => m.Name == ".ctor");
        var valueDefinition = instanceDefinition.Methods.Single(m => m.Name == "Value");
        Check(instanceDefinition.Attributes == 0x100001 && !ctorDefinition.IsStatic && (ctorDefinition.Attributes & 0x1800) == 0x1800, "instance and constructor attributes");
        Check(valueDefinition.TryGetSignature(out var valueSignature) && valueSignature!.ReturnType.Primitive == PrimitiveType.Int32, "instance logical signature");
        Check(!valueDefinition.TryGetStaticInt32Signature(out _, out _) && !valueDefinition.TryGetStaticPrimitiveSignature(out _) && !valueDefinition.TryGetStaticValueSignature(out _), "static helpers reject instance methods");
        var ctorImport = consumer.ImportReference(ctorDefinition, core);
        var valueImport = consumer.ImportReference(valueDefinition, core);
        Check(ctorImport.IsConstructor && !valueImport.IsStatic && valueImport.DeclaringTypeName == "Calculator", "instance import contract");
        var instanceCall = consumer.AddFunction("InstanceCall"); instanceCall.NewObject(ctorImport); instanceCall.Call(valueImport); instanceCall.Return();
        _ = consumer.WriteNativeAssembly();
        instances.AddClass("Example", "UnsupportedStorage").AddField("Other", SignatureType.Function(new MethodSignature(PrimitiveType.Int32, [])));
        var unsupportedStorageImage = RuntimeAssemblyContainer.WriteBinary(instances.WriteNativeAssembly(), core);
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(unsupportedStorageImage));
        library.AddValueType("Example", "Unsupported");
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core)));
        var generic = new AssemblyBuilder(new("Generic", new Version(1, 0, 0, 0)), core);
        var identity = generic.AddGenericInterface("Example", "Unsupported", ["T"]);
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
