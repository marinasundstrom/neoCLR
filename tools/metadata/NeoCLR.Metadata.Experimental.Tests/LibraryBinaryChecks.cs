using System.Text;
using System.Buffers.Binary;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;

internal static class LibraryBinaryChecks
{
    internal static void Run()
    {
        byte[] values = "[-9223372036854775808,9223372036854775807,9223372036854775808,18446744073709551615]"u8.ToArray();
        var binary = NativeBinaryCodec.Encode(values, true);
        Check(JsonNode.DeepEquals(JsonNode.Parse(values), JsonNode.Parse(NativeBinaryCodec.Decode(binary, true))), "integer endpoints");
        Reject(() => NativeBinaryCodec.Encode(values));
        Reject(() => NativeBinaryCodec.Decode(binary));
        Reject(() => NativeBinaryCodec.Decode([0x3b,255,255,255,255,255,255,255,255], true));
        foreach (var value in new[] { "18446744073709551616", "-9223372036854775809", "1.5" })
            Reject(() => NativeBinaryCodec.Encode(Encoding.UTF8.GetBytes(value), true));
        foreach (var invalid in new byte[][] { [0xbf,0xff], [0x18,0], [0xc0,0], [0x40], [0x61,255],
            [0xa2,0x61,120,0,0x61,120,1], [0x80,0], [0xfb,0,0,0,0,0,0,0,0] })
            Reject(() => NativeBinaryCodec.Decode(invalid, true));
        var nodes = new byte[2097156]; nodes[0] = 0x9a; nodes[2] = 0x1f; nodes[3] = 255; nodes[4] = 255;
        // 2,097,151 child integers + one array = exactly the schema-3 item budget.
        _ = NativeBinaryCodec.Decode(nodes, true);
        Reject(() => NativeBinaryCodec.Decode(nodes));
        var excess = new byte[2097157]; excess[0] = 0x9a; excess[2] = 0x20;
        Reject(() => NativeBinaryCodec.Decode(excess, true));
        var largest = new byte[8 * 1024 * 1024 - 32];
        largest[0] = 0x7a;
        BinaryPrimitives.WriteUInt32BigEndian(largest.AsSpan(1, 4), (uint)(largest.Length - 5));
        largest.AsSpan(5).Fill((byte)'x');
        _ = NativeBinaryCodec.Decode(largest, true);
        Reject(() => NativeBinaryCodec.Decode(new byte[8 * 1024 * 1024 - 31], true));
        Reject(() => NativeBinaryCodec.Encode(new byte[32 * 1024 * 1024 + 1], true));
        Reject(() => NativeBinaryCodec.Encode(new byte[4 * 1024 * 1024 + 1]));
        var deep = Enumerable.Repeat((byte)0x81,65).Append((byte)0).ToArray();
        Reject(() => NativeBinaryCodec.Decode(deep, true));
        byte[] json = Encoding.UTF8.GetBytes("{\"format\":5,\"name\":\"Library\",\"functions\":[],\"padding\":\"" + new string('x', 1024 * 1024) + "\"}");
        var image = NativeModuleContainer.WriteLibraryBinary(json);
        Check(image[18] == 3, "small libraries retain schema 3");
        Check(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(NativeModuleContainer.Read(image))), "large container values");
        Reject(() => MetadataEnvelope.Read(image, new Dictionary<ushort, ushort> { [256] = 3 }));
        Reject(() => NativeModuleContainer.WriteBinary(json));
        var downgraded = image.ToArray(); downgraded[18] = 2;
        Reject(() => NativeModuleContainer.Read(downgraded));
        var unknown = image.ToArray(); unknown[18] = 5;
        Reject(() => NativeModuleContainer.Read(unknown));
    }
    internal static void RoundTripFile(string path)
    {
        var original = File.ReadAllBytes(path);
        var binary = NativeModuleContainer.WriteLibraryBinary(original);
        Check(JsonNode.DeepEquals(JsonNode.Parse(original), JsonNode.Parse(NativeModuleContainer.Read(binary))), "complete library JSON values");
        Console.WriteLine($"PASS complete library values: {original.Length} JSON bytes, {binary.Length} binary bytes");
    }
    internal static void EmitFixture(string path)
    {
        const string json = """
            {"format":5,"name":"FloatingBits","functions":[
              {"name":"NegativeZero","parameters":[],"returns":"Double","locals":[],"body":[{"op":"ldc.r8","arg":{"bits":9223372036854775808}},{"op":"ret"}]},
              {"name":"NegativeNaN","parameters":[],"returns":"Double","locals":[],"body":[{"op":"ldc.r8","arg":{"bits":18444492273895866369}},{"op":"ret"}]},
              {"name":"MaxBits","parameters":[],"returns":"Double","locals":[],"body":[{"op":"ldc.r8","arg":{"bits":18446744073709551615}},{"op":"ret"}]}]}
            """;
        File.WriteAllBytes(path, NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json)));
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid library profile accepted");
    }
}
