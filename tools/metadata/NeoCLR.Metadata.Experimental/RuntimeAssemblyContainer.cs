using System.Buffers.Binary;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;
using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental;

/// <summary>Experimental PE/#Neo transport for native format-5 metadata and bodies.</summary>
/// <remarks>The native payload is authoritative. CLI metadata is a reference-only projection;
/// its throwing bodies are never used by neoCLR. Binding detects modification, not authenticity.</remarks>
public static class RuntimeAssemblyContainer
{
    /// <summary>Maximum library PE size, 16 MiB. Schema-3/4 envelopes are limited to 8/16 MiB.</summary>
    public const int MaxLibraryImageSize = 16 * 1024 * 1024;
    private static readonly IReadOnlyDictionary<ushort, ushort> Schemas = new Dictionary<ushort, ushort> { [256] = 1 };

    /// <summary>Builds an unsigned PE32 reference projection with a required native execution section.</summary>
    /// <param name="nativeImage">API-produced native format-5 JSON, subject to the 1 MiB envelope limit.</param>
    /// <param name="coreLibrary">Explicit .NET reference core supplying Object and ReferenceAssemblyAttribute.</param>
    /// <returns>A new PE image for native loading and .NET metadata inspection.</returns>
    /// <exception cref="InvalidDataException">Unsupported declarations, exceeded bounds or invalid container.</exception>
    /// <exception cref="ArgumentNullException">Core identity is null.</exception>
    public static byte[] Write(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary)
        => WriteCore(nativeImage, coreLibrary, binary: false);

    /// <summary>Builds a PE/#Neo container with schema-2 binary native metadata (bounded CBOR).</summary>
    /// <param name="nativeImage">API-produced format-5 JSON used only during host emission.</param>
    /// <param name="coreLibrary">Explicit .NET reference core identity.</param>
    /// <returns>Owned PE bytes; runtime decoding requires no JSON parsing.</returns>
    /// <exception cref="ArgumentNullException">Core identity is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported input, non-integer numeric data or exceeded limits.</exception>
    /// <remarks>Schema 1 remains supported by Read. Schema 2 is a provisional binary object encoding, not indexed CLI tables.</remarks>
    public static byte[] WriteBinary(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary)
        => WriteCore(nativeImage, coreLibrary, binary: true);

    /// <summary>Builds a binary native PE directly from an authored graph and its validated CLI reference projection.</summary>
    /// <param name="assembly">The complete output graph, including completed external interface contracts.</param>
    /// <returns>A PE with authoritative native metadata and non-executable CLI reference bodies.</returns>
    /// <exception cref="ArgumentNullException">The graph is null.</exception>
    /// <exception cref="InvalidDataException">The graph, dependency contracts or encoding are invalid.</exception>
    /// <remarks>No dependency reader is consulted. Use this overload for external interface declarations;
    /// the native-bytes overload cannot reconstruct external method contracts for its CLI projection.
    /// Selects schema 3 when Double literal bits require UInt64; otherwise retains schema 2.
    /// The application host-image size limit remains unchanged.</remarks>
    public static byte[] WriteBinary(AssemblyBuilder assembly)
    {
        ArgumentNullException.ThrowIfNull(assembly);
        var native = assembly.WriteNativeAssembly();
        var projection = assembly.WriteReferenceImage();
        return WriteCore(native, assembly.CoreLibrary, binary: true, projection, library: assembly.RequiresWideNumericPayload);
    }

    /// <summary>Builds a schema-3 library PE, selecting schema 4 when its payload exceeds 8 MiB.</summary>
    /// <param name="assembly">The authored graph and complete external contracts; no dependency reader is consulted.</param>
    /// <returns>Owned PE bytes, at most 16 MiB; schema-3/4 envelopes allow 8/16 MiB.</returns>
    /// <exception cref="ArgumentNullException">The graph is null.</exception>
    /// <exception cref="InvalidDataException">Invalid graph, unsupported declarations or exceeded profile bounds.</exception>
    /// <remarks>Host JSON is bounded to 32 MiB, binary nodes to 2,097,152 and depth to 64.
    /// Declaration row/signature bounds remain unchanged. Schema 4 changes only the envelope byte budget; older readers reject it.</remarks>
    public static byte[] WriteLibraryBinary(AssemblyBuilder assembly)
    {
        ArgumentNullException.ThrowIfNull(assembly);
        return WriteCore(assembly.WriteNativeLibraryAssembly(), assembly.CoreLibrary, binary: true,
            assembly.WriteReferenceImage(), library: true);
    }

    private static byte[] WriteCore(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary, bool binary, byte[]? projection = null, bool library = false)
    {
        ArgumentNullException.ThrowIfNull(coreLibrary);
        if (!binary && nativeImage.Length > MetadataEnvelope.MaxImageSize - 32)
            throw new InvalidDataException("native payload exceeds execution envelope limit");
        var definition = library ? NativeAssemblyDefinition.ReadLibraryAssembly(nativeImage) : NativeAssemblyDefinition.ReadAssembly(nativeImage);
        ushort schema = library ? (ushort)3 : binary ? (ushort)2 : (ushort)1;
        byte[] payloadBytes = binary ? NativeBinaryCodec.Encode(nativeImage, library, expandedLibrary: library) : nativeImage.ToArray();
        if (library && payloadBytes.Length > 8 * 1024 * 1024 - 32) schema = 4;
        int envelopeLimit = schema == 4 ? 16 * 1024 * 1024 : library ? 8 * 1024 * 1024 : MetadataEnvelope.MaxImageSize;
        var envelope = MetadataEnvelope.WriteCore([new MetadataSection(256, schema, true, payloadBytes, envelopeLimit)],
            new Dictionary<ushort, ushort> { [256] = schema }, envelopeLimit);
        var image = projection ?? definition.CreateReferenceAssembly(coreLibrary);
        using var pe = new PEReader(new MemoryStream(image, writable: false));
        var headers = pe.PEHeaders;
        var metadata = pe.GetMetadata().GetContent().ToArray();
        int position = 16 + I32(metadata, 12);
        int count = BinaryPrimitives.ReadUInt16LittleEndian(metadata.AsSpan(position + 2));
        position += 4;
        var streams = new SortedDictionary<string, byte[]>(StringComparer.Ordinal);
        for (int i = 0; i < count; i++)
        {
            int offset = I32(metadata, position), size = I32(metadata, position + 4);
            int start = position + 8, end = Array.IndexOf(metadata, (byte)0, start);
            streams.Add(Encoding.ASCII.GetString(metadata, start, end - start), metadata.AsSpan(offset, size).ToArray());
            position = start + Align(end - start + 1, 4);
        }
        streams.Add("#Neo", envelope);
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        hash.AppendData("neoCLR experimental metadata binding 0.1\0"u8);
        foreach (var (name, payload) in streams)
        {
            var encoded = Encoding.ASCII.GetBytes(name);
            byte[] length = new byte[4];
            BinaryPrimitives.WriteUInt16LittleEndian(length, (ushort)encoded.Length);
            hash.AppendData(length.AsSpan(0, 2)); hash.AppendData(encoded);
            BinaryPrimitives.WriteInt32LittleEndian(length, payload.Length);
            hash.AppendData(length); hash.AppendData(payload);
        }
        var marker = Encoding.ASCII.GetBytes("neoCLR.NEOX.0.1;sha256=" + Convert.ToHexString(hash.GetHashAndReset()).ToLowerInvariant() + "\0");
        int versionSize = Align(marker.Length, 4);
        int directorySize = streams.Sum(s => 8 + Align(s.Key.Length + 1, 4));
        int payloadStart = 16 + versionSize + 4 + directorySize;
        byte[] root = new byte[payloadStart + streams.Sum(s => Align(s.Value.Length, 4))];
        metadata.AsSpan(0, 12).CopyTo(root);
        Put(root, 12, versionSize); marker.CopyTo(root, 16);
        BinaryPrimitives.WriteUInt16LittleEndian(root.AsSpan(18 + versionSize), (ushort)streams.Count);
        position = 20 + versionSize;
        foreach (var (name, payload) in streams)
        {
            Put(root, position, payloadStart); Put(root, position + 4, payload.Length);
            Encoding.ASCII.GetBytes(name).CopyTo(root, position + 8);
            position += 8 + Align(name.Length + 1, 4);
            payload.CopyTo(root, payloadStart); payloadStart += Align(payload.Length, 4);
        }
        int peOffset = I32(image, 0x3c), optional = peOffset + 24;
        int row = optional + 224 + headers.SectionHeaders.Length * 40;
        if (row + 40 > headers.PEHeader!.SizeOfHeaders || image.AsSpan(row, 40).ContainsAnyExcept((byte)0))
            throw new InvalidDataException("no free PE section header");
        int raw = Align(image.Length, headers.PEHeader.FileAlignment);
        int rawSize = Align(root.Length, headers.PEHeader.FileAlignment);
        int rva = Align(headers.SectionHeaders.Max(s => s.VirtualAddress + Math.Max(s.VirtualSize, s.SizeOfRawData)), headers.PEHeader.SectionAlignment);
        byte[] result = new byte[raw + rawSize]; image.CopyTo(result, 0); root.CopyTo(result, raw);
        ".neometa"u8.CopyTo(result.AsSpan(row));
        Put(result, row + 8, root.Length); Put(result, row + 12, rva);
        Put(result, row + 16, rawSize); Put(result, row + 20, raw); Put(result, row + 36, 0x40000040);
        BinaryPrimitives.WriteUInt16LittleEndian(result.AsSpan(peOffset + 6), (ushort)(headers.SectionHeaders.Length + 1));
        Put(result, optional + 8, checked(I32(image, optional + 8) + rawSize));
        Put(result, optional + 56, Align(rva + root.Length, headers.PEHeader.SectionAlignment));
        Put(result, headers.CorHeaderStartOffset + 8, rva); Put(result, headers.CorHeaderStartOffset + 12, root.Length);
        _ = Read(result);
        return result;
    }

    /// <summary>Validates the PE, recognition digest, envelope and supported native declarations.</summary>
    /// <param name="image">Complete unsigned PE32 image, at most 4 MiB for schema 1/2 or 16 MiB for schema 3/4.</param>
    /// <returns>Owned native format-5 JSON: original schema-1 bytes or reconstructed schema-2/3/4 values.</returns>
    /// <exception cref="InvalidDataException">Missing/changed binding, malformed container or unsupported required schema/declarations.</exception>
    /// <remarks>Accepts schema-1 JSON, schema-2 CBOR and schema-3/4 library CBOR. Does not execute, resolve dependencies, compare CLI declarations or verify native bodies.</remarks>
    public static byte[] Read(ReadOnlySpan<byte> image)
    {
        var envelope = MetadataArtifactReader.ReadEnvelope(image, maxImageSize: MaxLibraryImageSize)!;
        IReadOnlyList<MetadataSection> sections;
        try { sections = MetadataEnvelope.Read(envelope, Schemas); }
        catch (InvalidDataException)
        {
            try { sections = MetadataEnvelope.Read(envelope, new Dictionary<ushort, ushort> { [256] = 2 }); }
            catch (InvalidDataException)
            {
                try { sections = MetadataEnvelope.ReadCore(envelope, new Dictionary<ushort, ushort> { [256] = 3 }, 8 * 1024 * 1024); }
                catch (InvalidDataException)
                { sections = MetadataEnvelope.ReadCore(envelope, new Dictionary<ushort, ushort> { [256] = 4 }, 16 * 1024 * 1024); }
            }
        }
        var execution = sections.SingleOrDefault(s => s.Kind == 256);
        if (execution is null || !execution.Required || execution.Version is not (1 or 2 or 3 or 4))
            throw new InvalidDataException("required native execution section missing or unsupported");
        if (execution.Version < 3 && image.Length > MetadataArtifactReader.MaxImageSize)
            throw new InvalidDataException("legacy native PE exceeds 4 MiB");
        var native = execution.Version == 1 ? execution.GetPayload() : NativeBinaryCodec.Decode(execution.Payload, library: execution.Version >= 3, expandedLibrary: execution.Version == 4);
        _ = execution.Version >= 3 ? NativeAssemblyDefinition.ReadLibraryAssembly(native) : NativeAssemblyDefinition.ReadAssembly(native);
        return native;
    }

    /// <summary>Reads the physical CLI reference declarations from a validated runtime container.</summary>
    /// <param name="image">Complete PE/#Neo bytes.</param>
    /// <returns>An owned Cecil-style snapshot preserving the original container bytes.</returns>
    /// <exception cref="InvalidDataException">Invalid transport, native declarations or CLI metadata.</exception>
    /// <remarks>Profile is null: the structural reference profile is not used. CLI declarations are
    /// a tooling projection, not runtime authority. No semantic equivalence check is performed;
    /// use Write to generate both views from the same native snapshot.</remarks>
    public static AssemblyDefinition ReadCliProjection(ReadOnlySpan<byte> image)
    {
        _ = Read(image);
        return AssemblyDefinition.ReadRuntimeProjection(image);
    }

    private static int Align(int value, int alignment) => checked(value + alignment - 1) & ~(alignment - 1);
    private static int I32(byte[] image, int offset) => BinaryPrimitives.ReadInt32LittleEndian(image.AsSpan(offset));
    private static void Put(byte[] image, int offset, int value) => BinaryPrimitives.WriteInt32LittleEndian(image.AsSpan(offset), value);
}
