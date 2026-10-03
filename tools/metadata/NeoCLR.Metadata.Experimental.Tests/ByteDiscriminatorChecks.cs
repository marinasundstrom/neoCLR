using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class ByteDiscriminatorChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("ByteDiscriminator", new Version(1, 0, 0, 0)), core);
        var tagged = graph.AddValueType("Example", "Tagged");
        var tag = tagged.AddField("Tag", PrimitiveType.Byte, FieldVisibility.Public);
        var ctor = tagged.AddConstructor([PrimitiveType.Byte]);
        var init = ctor.GetILGenerator(); init.LoadArgument(0); init.LoadArgument(1); init.StoreField(tag); init.Return();
        var getter = tagged.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Byte, []));
        var read = getter.GetILGenerator(); read.LoadArgument(0); read.LoadField(tag); read.Return();
        var entry = graph.AddFunction("Main"); graph.EntryPoint = entry;
        var il = entry.GetILGenerator();
        foreach (var (input, expected) in new[] { (-1, 255), (0, 0), (42, 42), (255, 255), (256, 0), (298, 42) })
        {
            il.LoadConstant(input); il.Emit(OpCode.Conv_U1); Verify(expected);
        }
        il.Emit(OpCode.Ldc_I8, -1L); il.Emit(OpCode.Conv_U1); Verify(255);
        var local = il.DeclareLocal(PrimitiveType.Byte);
        il.LoadConstant(298); il.StoreLocal(local); il.LoadLocal(local); Verify(42);
        var value = il.DeclareLocal(tagged);
        il.LoadConstant(255); il.NewObject(ctor); il.StoreLocal(value);
        il.LoadLocalAddress(value); il.Call(getter); Verify(255);
        var array = il.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Byte));
        il.LoadConstant(1); il.NewArray(PrimitiveType.Byte); il.StoreLocal(array);
        il.LoadLocal(array); il.LoadConstant(0); il.LoadConstant(298); il.StoreArrayElement(PrimitiveType.Byte);
        il.LoadLocal(array); il.LoadConstant(0); il.LoadArrayElement(PrimitiveType.Byte); il.Return();
        void Verify(int expected)
        {
            var correct = il.DefineLabel(); il.LoadConstant(expected); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brtrue, correct);
            il.LoadConstant(99); il.Return(); il.MarkLabel(correct);
        }
        var image = graph.Write();
        var context = new AssemblyLoadContext("byte-discriminator", true);
        try { Check(Equals(42, context.LoadFromStream(new MemoryStream(image)).EntryPoint!.Invoke(null, null)), "CLR byte storage/conversion"); }
        finally { context.Unload(); }
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(image, false), AssemblyDefinition.ReadNativeAssembly(native) })
        {
            var metadata = new MetadataLoadContext([snapshot]);
            var type = metadata.Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Tagged");
            Check(type.GetFields().Single().FieldType is PrimitiveTypeInfo { Kind: PrimitiveType.Byte }, "byte field signature");
            if (snapshot.IsNative)
                Check(type.GetMethods().Single().ReturnType is PrimitiveTypeInfo { Kind: PrimitiveType.Byte }, "native byte result signature");
            var importer = new AssemblyBuilder(new("ByteImporter", new Version(1, 0, 0, 0)), core);
            Check(importer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Tagged").Methods.Single(m => m.Name == "Read"), core).Signature.ReturnType.Primitive == PrimitiveType.Byte, "imported byte result signature");
        }
        if (Environment.GetEnvironmentVariable("NEOCLR_BYTE_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
        var invalid = new AssemblyBuilder(new("InvalidByte", new Version(1, 0, 0, 0)), core);
        var output = invalid.AddFunction("Ref", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(PrimitiveType.Byte)]));
        output.GetILGenerator().Return();
        var body = invalid.AddFunction("WrongAddress").GetILGenerator();
        var integer = body.DeclareLocal(PrimitiveType.Int32); body.LoadConstant(42); body.StoreLocal(integer);
        body.LoadLocalAddress(integer); body.Call(output); body.LoadConstant(0); body.Return();
        try { invalid.Write(); throw new Exception("ref Int32 admitted as ref Byte"); } catch (InvalidDataException) { }
    }
    static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
