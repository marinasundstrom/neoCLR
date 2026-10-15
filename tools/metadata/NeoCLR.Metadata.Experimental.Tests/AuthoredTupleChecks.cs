using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredTupleChecks
{
    internal static void Run(string output)
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new("AuthoredTuples", new Version(1, 0, 0, 0)), core);
        SignatureType[] elements = [PrimitiveType.Int32, PrimitiveType.String];
        var pair = builder.CreateExperimentalTuple(elements);
        elements[0] = PrimitiveType.Boolean;
        if (pair.Tuple!.ElementTypes[0] != (SignatureType)PrimitiveType.Int32) throw new Exception("mutable input retained");
        if (pair != builder.CreateExperimentalTuple(PrimitiveType.Int32, PrimitiveType.String) || pair.GetHashCode() != builder.CreateExperimentalTuple(PrimitiveType.Int32, PrimitiveType.String).GetHashCode()) throw new Exception("tuple identity lost");
        if (pair == builder.CreateExperimentalTuple(PrimitiveType.String, PrimitiveType.Int32)) throw new Exception("order lost");
        var nested = builder.CreateExperimentalTuple(pair, PrimitiveType.Int32, PrimitiveType.Int32);
        if (nested.Tuple!.ElementTypes.Count != 3 || nested.Tuple.ElementTypes[0] != pair) throw new Exception("nesting or multiplicity lost");
        var echo = builder.AddFunction("Echo", new MethodSignature(nested, [nested]));
        echo.LoadArgument(0); echo.Return();
        var main = builder.AddFunction("Main");
        main.GetILGenerator().LoadTypeToken(nested); main.GetILGenerator().Emit(OpCode.Pop);
        main.LoadConstant(42); main.Return(); builder.EntryPoint = main;
        var image = builder.WriteNativeAssembly();
        using var json = JsonDocument.Parse(image);
        var signature = json.RootElement.GetProperty("functions").EnumerateArray().Single(f => f.GetProperty("returns").ValueKind == JsonValueKind.Object).GetProperty("returns");
        if (signature.GetProperty("Tuple").GetArrayLength() != 3 || signature.GetProperty("Tuple")[0].GetProperty("Tuple").GetArrayLength() != 2) throw new Exception("native tuple encoding lost");
        Directory.CreateDirectory(output);
        File.WriteAllBytes(Path.Combine(output, "AuthoredTuples.neox"), NativeModuleContainer.WriteBinary(image));
        Reject(() => builder.Write());
        Reject(() => builder.CreateExperimentalTuple());
        Reject(() => builder.CreateExperimentalTuple((SignatureType)null!));
        var box = builder.AddGenericClass("", "Box", ["T"]);
        var closed = builder.CreateExperimentalTuple(box.MakeGenericInstance(PrimitiveType.Int32));
        if (closed.Tuple!.ElementTypes[0].GenericInstance is null) throw new Exception("closed generic lost");
        Reject(() => builder.CreateExperimentalTuple(box.MakeGenericInstance(SignatureType.TypeParameter(0))));
        var closedEcho = builder.AddFunction("EchoClosed", new MethodSignature(closed, [closed]));
        closedEcho.LoadArgument(0); closedEcho.Return();
        string?[] names = ["count", "text"];
        var namedPair = builder.CreateExperimentalTuple([PrimitiveType.Int32, PrimitiveType.String], names);
        names[0] = "changed";
        if (namedPair.Tuple!.ElementNames[0] != "count" || namedPair != pair || namedPair.GetHashCode() != pair.GetHashCode()) throw new Exception("names affected identity or retained mutable input");
        var named = builder.CreateExperimentalTuple([namedPair, PrimitiveType.Int32, PrimitiveType.Int32], ["payload", "offset", null]);
        var renamed = builder.CreateExperimentalTuple([pair, PrimitiveType.Int32, PrimitiveType.Int32], ["data", "position", "unused"]);
        var namedEcho = builder.AddFunction("EchoNamed", new MethodSignature(named, [renamed]));
        namedEcho.LoadArgument(0); namedEcho.Return();
        foreach (var invalidNames in new string?[][] { ["only"], ["", null], ["x", "x"], ["Item2", null], ["x\0y", null], [new string('é', 513), null], ["\ud800", null] })
            Reject(() => builder.CreateExperimentalTuple([PrimitiveType.Int32, PrimitiveType.Int32], invalidNames));
        File.WriteAllBytes(Path.Combine(output, "AuthoredTuples.neox"), NativeModuleContainer.WriteBinary(builder.WriteNativeAssembly()));
        Reject(() => builder.CreateExperimentalTuple(Enumerable.Repeat<SignatureType>(PrimitiveType.Int32, 257).ToArray()));
        foreach (SignatureType invalid in new SignatureType[] { PrimitiveType.Void, PrimitiveType.RuntimeTypeHandle, SignatureType.TypeParameter(0), SignatureType.ArrayOf(PrimitiveType.Int32), SignatureType.ByReference(PrimitiveType.Int32) }) Reject(() => builder.CreateExperimentalTuple(invalid));
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core);
        Reject(() => other.AddFunction("Foreign", new MethodSignature(pair, [pair])));
        Reject(() => builder.CreateExperimentalTuple(other.AddClass("", "Foreign")));
        SignatureType deep = PrimitiveType.Int32;
        for (var i = 0; i < 16; i++) deep = builder.CreateExperimentalTuple(deep);
        Reject(() => builder.CreateExperimentalTuple(deep));
        var invalidBody = new AssemblyBuilder(new("InvalidTupleBody", new Version(1, 0, 0, 0)), core);
        var first = invalidBody.CreateExperimentalTuple(PrimitiveType.Int32, PrimitiveType.String);
        var reversed = invalidBody.CreateExperimentalTuple(PrimitiveType.String, PrimitiveType.Int32);
        var wrong = invalidBody.AddFunction("Wrong", new MethodSignature(first, [reversed]));
        wrong.LoadArgument(0); wrong.Return();
        Reject(() => invalidBody.WriteNativeAssembly());
        Console.WriteLine("PASS authored structural tuple signatures");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is ArgumentException or InvalidDataException or NotSupportedException) { return; }
        throw new Exception("unsupported tuple accepted");
    }
}
