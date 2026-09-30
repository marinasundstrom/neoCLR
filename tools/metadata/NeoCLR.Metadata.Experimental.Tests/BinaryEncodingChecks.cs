using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class BinaryEncodingChecks
{
    internal static void Run()
    {
        byte[] json = "{\"x\":[null,true,false,-9223372036854775808,9223372036854775807,24,256,65536,4294967296,\"世界\"]}"u8.ToArray();
        var encoded = NativeBinaryCodec.Encode(json);
        Check(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(NativeBinaryCodec.Decode(encoded))), "binary values roundtrip");
        Check(NativeBinaryCodec.Encode("{\"x\":24}"u8).SequenceEqual(new byte[] { 0xa1, 0x61, 0x78, 0x18, 24 }), "RFC integer/text/map vector");
        foreach (var bytes in new byte[][] {
            [0xbf,0xff], [0x18,0], [0xc0,0], [0x40], [0x61,0xff], [0xa2,0x61,0x78,0,0x61,0x78,1],
            [0xa1,0,0], [0x80,0], [0xfb,0,0,0,0,0,0,0,0], [0x1b,255,255,255,255,255,255,255,255],
            [0x9a,255,255,255,255], Enumerable.Repeat((byte)0x81,65).Append((byte)0).ToArray()
        }) Reject(() => NativeBinaryCodec.Decode(bytes));
        for (int i = 0; i < encoded.Length; i++) { int length = i; Reject(() => NativeBinaryCodec.Decode(encoded.AsSpan(0, length))); }
        Reject(() => NativeBinaryCodec.Encode("1.5"u8));
        Reject(() => NativeBinaryCodec.Encode(Encoding.UTF8.GetBytes("\"\\ud800\"")));
        Reject(() => NativeBinaryCodec.Encode("{\"x\":1,\"x\":2}"u8));
        var nodes = new byte[262150]; nodes[0] = 0x9a; nodes[1] = 0; nodes[2] = 4; nodes[3] = 0; nodes[4] = 1;
        Reject(() => NativeBinaryCodec.Decode(nodes));
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("Binary", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main"); main.WriteConsoleLine("Hello 世界"); main.LoadConstant(0); main.Return(); graph.EntryPoint = main;
        var native = graph.WriteNativeAssembly(); var pe = RuntimeAssemblyContainer.WriteBinary(native, core);
        Check(RuntimeAssemblyContainer.Read(pe).SequenceEqual(native), "writer native bytes reconstructed");
        Check(RuntimeAssemblyContainer.ReadCliProjection(pe).Identity.Equals(graph.Identity), "binary container CLI projection");
        foreach (var pair in RuntimeContainerChecks.MalformedEnvelopes(pe)) Reject(() => RuntimeAssemblyContainer.Read(pair.Image));
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("unsupported binary encoding accepted");
    }
}
