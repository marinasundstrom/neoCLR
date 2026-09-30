using NeoCLR.Metadata.Experimental;

var schemas = new Dictionary<ushort, ushort> { [1] = 1, [2] = 1, [3] = 1, [4] = 1, [7] = 2 };
var empty = new Dictionary<ushort, ushort>();
try
{
    if (args.Length == 0) throw new ArgumentException("selftest | emit path | roundtrip input output | reject input");
    switch (args[0])
    {
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
