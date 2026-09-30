using System.Buffers.Binary;
using System.Diagnostics;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class RuntimeContainerChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("ContainerTest", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main"); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        var native = graph.WriteNativeAssembly();
        var image = RuntimeAssemblyContainer.Write(native, core);
        if (!RuntimeAssemblyContainer.Read(image).SequenceEqual(native)) throw new Exception("payload roundtrip");
        var read = RuntimeAssemblyContainer.ReadCliProjection(image);
        if (!read.Identity.Equals(graph.Identity) || read.MainModule.Functions.Single().Name != "Main" || !read.Write().SequenceEqual(image))
            throw new Exception("CLI projection snapshot");
        var detached = RuntimeAssemblyContainer.Read(image); Array.Clear(detached);
        if (!RuntimeAssemblyContainer.Read(image).SequenceEqual(native)) throw new Exception("owned payload");
        Reject(graph.Write());
        Reject(image[..^1]);
        var changed = image.ToArray();
        using var pe = new PEReader(new MemoryStream(image));
        var root = pe.GetMetadata().GetContent().ToArray();
        int rootStart = image.AsSpan().LastIndexOf(root);
        changed[rootStart + root.Length - 4] ^= 1; Reject(changed);
        foreach (var pair in MalformedEnvelopes(image)) Reject(pair.Image);
    }

    // Recompute the consistency digest so runtime/schema checks cannot hide behind digest failure.
    internal static IEnumerable<(string Name, byte[] Image)> MalformedEnvelopes(byte[] image)
    {
        foreach (var change in new (string Name, int Offset, byte Value)[] {
            ("UnknownRequired", 16, 1), ("WrongSchema", 18, 3), ("OptionalExecution", 20, 0),
            ("BadRange", 24, 31), ("BadVersion", 6, 2) })
        {
            var mutated = image.ToArray();
            using var pe = new PEReader(new MemoryStream(image));
            var root = pe.GetMetadata().GetContent().ToArray();
            int rootStart = image.AsSpan().LastIndexOf(root);
            int versionSize = BinaryPrimitives.ReadInt32LittleEndian(root.AsSpan(12));
            int position = 20 + versionSize;
            int count = BinaryPrimitives.ReadUInt16LittleEndian(root.AsSpan(18 + versionSize));
            var streams = new SortedDictionary<string, (int Offset, int Size)>(StringComparer.Ordinal);
            for (int i = 0; i < count; i++)
            {
                int offset = BinaryPrimitives.ReadInt32LittleEndian(root.AsSpan(position));
                int size = BinaryPrimitives.ReadInt32LittleEndian(root.AsSpan(position + 4));
                int start = position + 8, end = Array.IndexOf(root, (byte)0, start);
                streams.Add(Encoding.ASCII.GetString(root, start, end - start), (offset, size));
                position = start + ((end - start + 4) & ~3);
            }
            root[streams["#Neo"].Offset + change.Offset] = change.Value;
            using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            hash.AppendData("neoCLR experimental metadata binding 0.1\0"u8);
            foreach (var (name, range) in streams)
            {
                byte[] number = new byte[4]; var encoded = Encoding.ASCII.GetBytes(name);
                BinaryPrimitives.WriteUInt16LittleEndian(number, (ushort)encoded.Length);
                hash.AppendData(number.AsSpan(0, 2)); hash.AppendData(encoded);
                BinaryPrimitives.WriteInt32LittleEndian(number, range.Size); hash.AppendData(number);
                hash.AppendData(root.AsSpan(range.Offset, range.Size));
            }
            Encoding.ASCII.GetBytes(Convert.ToHexString(hash.GetHashAndReset()).ToLowerInvariant()).CopyTo(root, 16 + "neoCLR.NEOX.0.1;sha256=".Length);
            root.CopyTo(mutated, rootStart);
            yield return (change.Name, mutated);
        }
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Directory.CreateDirectory(directory);
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("ContainerRuntime", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main"); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        var image = RuntimeAssemblyContainer.Write(graph.WriteNativeAssembly(), core);
        await Check("Valid", image, 0, "verify"); await Check("Run", image, 42, "run");
        var binary = RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core);
        File.WriteAllBytes(Path.Combine(directory, "Module.neo.json"), graph.WriteNativeAssembly());
        await Check("Binary", binary, 0, "verify"); await Check("BinaryRun", binary, 42, "run");
        foreach (var pair in MalformedEnvelopes(binary)) await Check("Binary" + pair.Name, pair.Image, 1, "verify");
        // A larger bounded fixture for the load-only comparison; unused methods keep execution constant.
        for (int i = 0; i < 64; i++)
        {
            var method = graph.AddFunction("Extra" + i); method.LoadConstant(0);
            for (int j = 0; j < 64; j++) { method.LoadConstant(j); method.Add(); }
            method.Return();
        }
        var largeNative = graph.WriteNativeAssembly();
        File.WriteAllBytes(Path.Combine(directory, "Large.neo.json"), largeNative);
        File.WriteAllBytes(Path.Combine(directory, "LargeJson.dll"), RuntimeAssemblyContainer.Write(largeNative, core));
        File.WriteAllBytes(Path.Combine(directory, "LargeBinary.dll"), RuntimeAssemblyContainer.WriteBinary(largeNative, core));
        await Check("Ordinary", graph.Write(), 1, "verify");
        var invalidBody = System.Text.Json.Nodes.JsonNode.Parse(graph.WriteNativeAssembly())!;
        invalidBody["functions"]![0]!["body"] = new System.Text.Json.Nodes.JsonArray(
            new System.Text.Json.Nodes.JsonObject { ["op"] = "future.op" });
        await Check("InvalidBinaryBody", RuntimeAssemblyContainer.WriteBinary(Encoding.UTF8.GetBytes(invalidBody.ToJsonString()), core), 1, "verify");
        await Check("InvalidNativeBody", RuntimeAssemblyContainer.Write(Encoding.UTF8.GetBytes(invalidBody.ToJsonString()), core), 1, "verify");
        var damaged = image.ToArray();
        // A payload-specific damage always changes a bound stream, not arbitrary PE padding.
        using (var pe = new PEReader(new MemoryStream(image)))
        {
            var root = pe.GetMetadata().GetContent().ToArray();
            damaged = image.ToArray(); damaged[image.AsSpan().LastIndexOf(root) + root.Length - 4] ^= 1;
        }
        await Check("Damaged", damaged, 1, "verify");
        foreach (var pair in MalformedEnvelopes(image)) await Check(pair.Name, pair.Image, 1, "verify");
        async Task Check(string name, byte[] bytes, int expected, string command)
        {
            var path = Path.Combine(directory, name + ".dll"); File.WriteAllBytes(path, bytes);
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var output = process.StandardOutput.ReadToEndAsync(); var error = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await process.WaitForExitAsync(timeout.Token); }
            catch (OperationCanceledException) { process.Kill(true); throw; }
            var text = await output + await error;
            if (process.ExitCode != expected) throw new Exception(name + ": " + text);
            Console.WriteLine($"PASS PE runtime {name}: {expected}");
        }
    }
    private static void Reject(byte[] image)
    {
        try { RuntimeAssemblyContainer.Read(image); }
        catch (InvalidDataException) { return; }
        throw new Exception("invalid runtime container accepted");
    }
}
