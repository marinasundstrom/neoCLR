using System.Buffers.Binary;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExpandedLibraryChecks
{
    internal static void Run()
    {
        var json = Encoding.UTF8.GetBytes("{\"format\":5,\"name\":\"Expanded\",\"functions\":[],\"padding\":\"" + new string('x', 8 * 1024 * 1024) + "\"}");
        var envelope = NativeModuleContainer.WriteLibraryBinary(json);
        Check(envelope[18] == 4, "large library must declare schema 4");
        Check(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(NativeModuleContainer.Read(envelope))), "expanded round trip");
        Reject(() => NativeBinaryCodec.Encode(json, library: true));
        var downgraded = envelope.ToArray(); downgraded[18] = 3;
        Reject(() => NativeModuleContainer.Read(downgraded));
        Reject(() => MetadataEnvelope.ReadCore(envelope, new Dictionary<ushort, ushort> { [256] = 3 }, 16 * 1024 * 1024));
        var largest = new byte[16 * 1024 * 1024 - 32];
        largest[0] = 0x7a;
        BinaryPrimitives.WriteUInt32BigEndian(largest.AsSpan(1, 4), (uint)(largest.Length - 5));
        largest.AsSpan(5).Fill((byte)'x');
        _ = NativeBinaryCodec.Decode(largest, library: true, expandedLibrary: true);
        Reject(() => NativeBinaryCodec.Decode(new byte[largest.Length + 1], library: true, expandedLibrary: true));
        Reject(() => NativeBinaryCodec.Encode(new byte[32 * 1024 * 1024 + 1], library: true, expandedLibrary: true));
        var excessNodes = new byte[2097157]; excessNodes[0] = 0x9a; excessNodes[2] = 0x20;
        Reject(() => NativeBinaryCodec.Decode(excessNodes, library: true, expandedLibrary: true));
        Reject(() => NativeBinaryCodec.Decode(Enumerable.Repeat((byte)0x81, 65).Append((byte)0).ToArray(), library: true, expandedLibrary: true));

        var graph = new AssemblyBuilder(new("ExpandedLibrary", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var targetMethod = graph.AddFunction("Padding" + new string('x', 512));
        targetMethod.GetILGenerator().LoadConstant(0); targetMethod.GetILGenerator().Return();
        for (int i = 0; i < 16; i++)
        {
            var unused = graph.AddFunction("Unused" + i);
            for (int j = 0; j < 512; j++)
            {
                unused.GetILGenerator().Call(targetMethod);
                unused.GetILGenerator().Emit(OpCode.Pop);
            }
            unused.GetILGenerator().LoadConstant(0); unused.GetILGenerator().Return();
        }
        var answer = graph.AddFunction("Answer"); answer.GetILGenerator().LoadConstant(42); answer.GetILGenerator().Return();
        var library = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        Check(AssemblyDefinition.ReadNativeAssembly(library).MainModule.Functions.Count == 18, "expanded PE definitions");
        var app = new AssemblyBuilder(new("ExpandedConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var target = app.CreateFunctionReference(graph.Identity, graph.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(library)), "", "Answer", new(PrimitiveType.Int32, []));
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        entry.GetILGenerator().Call(target); entry.GetILGenerator().Return();
        if (Environment.GetEnvironmentVariable("NEOCLR_EXPANDED_LIBRARY_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path, library);
            File.WriteAllBytes(path + ".app", RuntimeAssemblyContainer.WriteBinary(app));
            File.WriteAllText(path + ".neoil", ".module System\n.references ()\n");
            File.WriteAllBytes(path + ".neox", envelope);
            File.WriteAllBytes(path + ".downgraded.neox", downgraded);
        }
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid expanded library accepted");
    }
}
