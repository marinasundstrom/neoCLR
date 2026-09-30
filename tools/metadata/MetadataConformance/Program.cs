using NeoCLR.Metadata.Experimental;

var schemas = new Dictionary<ushort, ushort> { [1] = 1, [2] = 1, [3] = 1, [4] = 1, [7] = 2 };
var empty = new Dictionary<ushort, ushort>();
try
{
    if (args.Length == 0) throw new ArgumentException("selftest | emit path | roundtrip input output | reject input");
    switch (args[0])
    {
        case "assembly-model":
            AssemblyModelChecks.Run(args[1], args[2], args[3], args[4]);
            break;
        case "artifact-vectors":
            ArtifactChecks.Run(Read(args[1]));
            break;
        case "profile-vectors":
            ProfileChecks.Run(Read(args[1]));
            break;
        case "member-vectors":
            MemberChecks.Run(Read(args[1]));
            break;
        case "reference-vectors":
            ReferenceChecks.Run(Read(args[1]));
            break;
        case "signature-roundtrip":
            var signature = StructuralSignature.Read(Read(args[1]), args.Length > 3 && args[3] == "references");
            File.WriteAllBytes(args[2], StructuralSignature.Write(signature.Root, signature.Context,
                args.Length > 3 && args[3] == "references"));
            break;
        case "signature-reject":
            try { StructuralSignature.Read(Read(args[1]), args.Length > 2 && args[2] == "references"); }
            catch (InvalidDataException) { Console.WriteLine("rejected"); return 0; }
            throw new Exception("malformed signature accepted");
        case "signature-emit":
            var root = new TypeExpression("function", [
                new("array", [new("tuple", [new("int32"), new("string")])]),
                new("intersection", [new("type_parameter"), new("self")]),
                new("union", [new("nullable", [new("method_parameter")]), new("unit")])
            ], modes: ["value", "readonly_ref"]);
            File.WriteAllBytes(args[1], StructuralSignature.Write(root, new SignatureContext(1, 1, true)));
            break;
        case "signature-selftest":
            SignatureChecks();
            Console.WriteLine("signature writer, ownership and boundary checks passed");
            break;
        case "roundtrip":
            File.WriteAllBytes(args[2], MetadataEnvelope.Write(MetadataEnvelope.Read(Read(args[1]), schemas), schemas));
            break;
        case "reject":
            try { MetadataEnvelope.Read(Read(args[1]), schemas); }
            catch (InvalidDataException) { Console.WriteLine("rejected"); return 0; }
            throw new Exception("malformed input accepted");
        case "emit":
            File.WriteAllBytes(args[1], MetadataEnvelope.Write(
                [new MetadataSection(7, 2, true, "abc"u8), new MetadataSection(60000, 9, false, [0, 255, 0])], schemas));
            break;
        case "selftest":
            var golden = Convert.FromHexString("4E454F5800000100010000002300000007000100000000002000000003000000616263");
            Check(MetadataEnvelope.Write(MetadataEnvelope.Read(golden, empty), empty).SequenceEqual(golden), "golden");
            var source = new byte[] { 1, 2, 3 };
            var section = new MetadataSection(7, 2, true, source);
            source[0] = 8;
            var copy = section.GetPayload();
            copy[0] = 9;
            Check(section.GetPayload()[0] == 1, "constructor and accessor isolation");
            var encoded = MetadataEnvelope.Write([section], schemas);
            var decoded = MetadataEnvelope.Read(encoded, schemas);
            encoded[^1] = 77;
            Check(decoded[0].GetPayload()[^1] == 3, "reader buffer isolation");
            Reject(() => MetadataEnvelope.Read(MetadataEnvelope.Write([section], schemas), empty));
            Reject(() => MetadataEnvelope.Read(MetadataEnvelope.Write([section], schemas), new Dictionary<ushort, ushort> { [7] = 1 }));
            Reject(() => MetadataEnvelope.Write([section], empty));
            Reject(() => MetadataEnvelope.Write([section, section], schemas));
            Reject(() => MetadataEnvelope.Write(Enumerable.Repeat(section, 65).ToArray(), schemas));
            Reject(() => MetadataEnvelope.Write([new MetadataSection(1, 1, false, new byte[MetadataEnvelope.MaxImageSize])], empty));
            Reject(() => MetadataEnvelope.Write(new MetadataSection[] { null! }, schemas));
            var optional = new MetadataSection(7, 99, false, [1, 0, 2]);
            Check(MetadataEnvelope.Read(MetadataEnvelope.Write([optional], empty), empty)[0].Version == 99, "optional schema preservation");
            var maximum = new MetadataSection(1, 1, false, new byte[MetadataEnvelope.MaxImageSize - 32]);
            Check(MetadataEnvelope.Read(MetadataEnvelope.Write([maximum], empty), empty)[0].PayloadLength == maximum.PayloadLength, "size boundary");
            Console.WriteLine("library conformance and ownership checks passed");
            break;
        default: throw new ArgumentException("unknown operation");
    }
    return 0;
}
catch (Exception error)
{
    Console.Error.WriteLine(error.Message);
    return 1;
}

static byte[] Read(string path)
{
    using var input = File.OpenRead(path);
    if (input.Length > MetadataEnvelope.MaxImageSize) throw new InvalidDataException("image too large");
    byte[] bytes = new byte[(int)input.Length];
    input.ReadExactly(bytes);
    return bytes;
}
static void Check(bool condition, string message)
{
    if (!condition) throw new Exception(message);
}
static void Reject(Action action)
{
    try { action(); }
    catch (InvalidDataException) { return; }
    throw new Exception("invalid writer or schema input accepted");
}

static void SignatureChecks()
{
    var context = new SignatureContext();
    var integer = new TypeExpression("int32");
    var children = new[] { integer, new TypeExpression("unit") };
    var modes = new[] { "ref" };
    var function = new TypeExpression("function", children, modes: modes);
    children[0] = new TypeExpression("string");
    modes[0] = "out";
    Check(function.Children[0].Kind == "int32" && function.Modes[0] == "ref", "owned node lists");
    foreach (var node in new TypeExpression[] {
        new("unknown"), new("self"), new("type_parameter"), new("method_parameter"),
        new("tuple"), new("union", [integer]), new("array"), new("nominal", index: 1),
        new("int32", [integer]), new("int32", index: 1), new("int32", noResult: true),
        new("function", [integer, integer]),
        new("function", [integer, integer], modes: ["out_when_true"]),
        new("function", [integer], noResult: true),
        new("function", [integer, integer], modes: ["unknown"])
    }) Reject(() => StructuralSignature.Write(node, context));
    var deep = integer;
    for (int i = 0; i < 32; i++) deep = new("array", [deep]);
    StructuralSignature.Read(StructuralSignature.Write(deep, context));
    deep = new("array", [deep]);
    Reject(() => StructuralSignature.Write(deep, context));
    var wide = new TypeExpression("tuple", Enumerable.Repeat(integer, 256).ToArray());
    StructuralSignature.Read(StructuralSignature.Write(wide, context));
    var tooMany = new TypeExpression("tuple", Enumerable.Repeat(wide, 16).ToArray());
    Reject(() => StructuralSignature.Write(tooMany, context));
    foreach (var mode in new[] { "value", "ref", "readonly_ref", "out", "out_when_true" })
    {
        var node = new TypeExpression("function", [integer, new("bool")], modes: [mode]);
        var bytes = StructuralSignature.Write(node, context);
        var read = StructuralSignature.Read(bytes);
        Check(read.Root.Modes[0] == mode && read.Root.Children[^1].Kind == "bool", "Function contract");
    }
    var unit = new TypeExpression("function", [new("unit")]);
    var absent = new TypeExpression("function", [new("unit")], noResult: true);
    Check(!StructuralSignature.Write(unit, context).SequenceEqual(StructuralSignature.Write(absent, context)), "unit versus no result");
}
