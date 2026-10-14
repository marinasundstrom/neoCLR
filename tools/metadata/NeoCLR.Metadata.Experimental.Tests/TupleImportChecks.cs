using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class TupleImportChecks
{
    private sealed class Resolver(AssemblyDefinition definition) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => definition.Identity.Equals(identity) ? definition : null;
    }

    internal static void Run(string output)
    {
        AuthoredTupleChecks.Run(output);
        var producer = JsonNode.Parse(NativeModuleContainer.Read(File.ReadAllBytes(Path.Combine(output, "AuthoredTuples.neox"))))!;
        producer["entry"] = "";
        var image = NativeModuleContainer.WriteBinary(System.Text.Encoding.UTF8.GetBytes(producer.ToJsonString()));
        File.WriteAllBytes(Path.Combine(output, "TupleProducer.neox"), image);
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true));
        var snapshot = AssemblyDefinition.ReadExperimentalTupleAssembly(image);
        if (!snapshot.Write().SequenceEqual(image)) throw new Exception("immutable image changed");
        var echo = snapshot.MainModule.Functions.Single(m => m.Name == "Echo");
        if (!echo.TryGetSignature(out var signature) || signature!.ReturnType.Tuple?.ElementTypes.Count != 3 || signature.ReturnType.Tuple.ElementTypes[0].Tuple?.ElementTypes.Count != 2)
            throw new Exception("ordered tuple snapshot lost");
        if (signature.ReturnType != signature.ParameterTypes[0]) throw new Exception("snapshot identity lost");
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var consumer = new AssemblyBuilder(new("TupleConsumer", new Version(1, 0, 0, 0)), core);
        Reject(() => consumer.ImportReference(echo, core));
        foreach (var name in new[] { "Echo", "EchoClosed", "EchoNamed" })
        {
            var imported = consumer.ImportExperimentalTupleReference(snapshot.MainModule.Functions.Single(m => m.Name == name), core);
            var wrapper = consumer.AddFunction(name, imported.Signature);
            wrapper.LoadArgument(0); wrapper.Call(imported); wrapper.Return();
        }
        // Importing once must not make the ordinary API admit tuples through its cache.
        Reject(() => consumer.ImportReference(echo, core));
        var main = consumer.AddFunction("Main");
        var callable = consumer.ImportExperimentalTupleReference(snapshot.MainModule.Functions.Single(m => m.Name == "EchoNamed"), core);
        CheckNames(callable.Signature);
        var tuple = callable.Signature.ReturnType;
        var pair = tuple.Tuple!.ElementTypes[0];
        var original = main.DeclareLocal(tuple);
        var il = main.GetILGenerator();
        main.LoadConstant(40); il.Emit(OpCode.Ldstr, "payload"); il.NewTuple(pair);
        main.LoadConstant(2); main.LoadConstant(0); il.NewTuple(tuple);
        main.Call(callable); main.StoreLocal(original);
        main.LoadLocal(original); main.LoadConstant(7); il.StoreTupleElement(tuple, 1);
        il.LoadTupleElement(tuple, 1);
        main.LoadLocal(original); il.LoadTupleElement(tuple, 0); il.LoadTupleElement(pair, 0);
        main.Emit(OpCode.Add);
        main.LoadLocal(original); il.LoadTupleElement(tuple, 1); main.Emit(OpCode.Add);
        main.LoadConstant(7); main.Emit(OpCode.Sub);
        main.Return(); consumer.EntryPoint = main;
        var consumerImage = consumer.WriteNativeAssembly();
        File.WriteAllBytes(Path.Combine(output, "TupleConsumer.neox"), NativeModuleContainer.WriteBinary(consumerImage));
        Reject(() => consumer.Write());
        var reread = AssemblyDefinition.ReadExperimentalTupleAssembly(NativeModuleContainer.WriteBinary(consumerImage));
        if (!reread.MainModule.Functions.Single(m => m.Name == "Echo").TryGetSignature(out var copied) || copied!.ReturnType != signature.ReturnType)
            throw new Exception("imported tuple signature changed");
        if (!reread.MainModule.Functions.Single(m => m.Name == "EchoNamed").TryGetSignature(out var namedCopy)) throw new Exception("named echo missing");
        CheckNames(namedCopy!);
        var third = new AssemblyBuilder(new("TupleThird", new Version(1, 0, 0, 0)), core);
        var external = reread.MainModule.Functions.Single(m => m.Name == "EchoClosed");
        Reject(() => third.ImportExperimentalTupleReference(external, core));
        var resolved = third.ImportExperimentalTupleReference(external, core, new Resolver(snapshot));
        if (resolved.Signature.ReturnType.Tuple?.ElementTypes[0].ImportedType?.TypeArguments.Count != 1)
            throw new Exception("resolved external generic tuple element lost");
        var json = JsonNode.Parse(NativeModuleContainer.Read(image))!;
        foreach (var invalid in new JsonNode[] { new JsonArray(), new JsonArray("Void"), new JsonArray(new JsonObject { ["MethodTypeParameter"] = 0 }),
            JsonNode.Parse("""{"elements":["Int32"],"names":[1]}""")!,
            JsonNode.Parse("""{"elements":["Int32"],"names":["x","y"]}""")!,
            JsonNode.Parse("""{"elements":["Int32"],"names":["x"],"extra":true}""")! })
        {
            var changed = json.DeepClone();
            changed["functions"]![0]!["returns"] = new JsonObject { ["Tuple"] = invalid.DeepClone() };
            Reject(() => AssemblyDefinition.ReadExperimentalTupleAssembly(NativeModuleContainer.WriteBinary(System.Text.Encoding.UTF8.GetBytes(changed.ToJsonString()))));
        }
        CheckInvalidOperations(core);
        Console.WriteLine("PASS native tuple snapshots, imports and value operations");
    }
    private static void CheckNames(MethodSignature signature)
    {
        if (signature.ReturnType != signature.ParameterTypes[0] ||
            !signature.ReturnType.Tuple!.ElementNames.SequenceEqual(new string?[] { "payload", "offset", null }) ||
            !signature.ParameterTypes[0].Tuple!.ElementNames.SequenceEqual(new string?[] { "data", "position", "unused" }) ||
            !signature.ReturnType.Tuple.ElementTypes[0].Tuple!.ElementNames.SequenceEqual(new[] { "count", "text" }))
            throw new Exception("per-signature names lost across snapshot/import");
    }
    private static void CheckInvalidOperations(AssemblyIdentity core)
    {
        foreach (var operation in new[] { "construct", "store", "receiver" })
        {
            var builder = new AssemblyBuilder(new("InvalidTupleOps", new Version(1, 0, 0, 0)), core);
            var tuple = builder.CreateExperimentalTuple(PrimitiveType.Int32, PrimitiveType.String);
            var main = builder.AddFunction("Main");
            var il = main.GetILGenerator();
            Reject(() => il.NewTuple(PrimitiveType.Int32));
            Reject(() => il.LoadTupleElement(tuple, -1));
            Reject(() => il.StoreTupleElement(tuple, 2));
            var foreign = new AssemblyBuilder(new("ForeignTuple", new Version(1, 0, 0, 0)), core).CreateExperimentalTuple(PrimitiveType.Int32);
            Reject(() => il.NewTuple(foreign));
            if (operation == "receiver")
            {
                main.LoadConstant(1); il.LoadTupleElement(tuple, 0);
            }
            else
            {
                main.LoadConstant(1);
                if (operation == "construct") main.LoadConstant(2);
                else il.Emit(OpCode.Ldstr, "text");
                il.NewTuple(tuple);
                if (operation == "store") { main.LoadConstant(2); il.StoreTupleElement(tuple, 1); }
                main.Emit(OpCode.Pop); main.LoadConstant(0);
            }
            main.Return();
            Reject(() => builder.WriteNativeAssembly());
        }
    }

    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is InvalidDataException or ArgumentException or NotSupportedException) { return; }
        throw new Exception("unsupported tuple operation accepted");
    }
}
