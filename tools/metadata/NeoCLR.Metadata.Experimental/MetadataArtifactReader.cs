using System.Buffers.Binary;
using System.Security.Cryptography;
using System.Text;

namespace NeoCLR.Metadata.Experimental;

/// <summary>An inspected bounded PE artifact; not an executable assembly or resolved catalog.</summary>
public sealed class MetadataArtifact
{
    internal MetadataArtifact(MetadataProfileDocument? profile) => Profile = profile;
    /// <summary>Gets whether the image has a verified experimental marker and locally valid reference profile.</summary>
    public bool IsExtended => Profile is not null;
    /// <summary>Gets the owned reference profile, or null for explicitly permitted ordinary CLI input.</summary>
    public MetadataProfileDocument? Profile { get; }
}

/// <summary>Recognizes experimental neoCLR metadata in bounded unsigned IL-only PE32 images.</summary>
public static class MetadataArtifactReader
{
    /// <summary>Maximum input image length, 4 MiB.</summary>
    public const int MaxImageSize = 4 * 1024 * 1024;

    /// <summary>Validates the container and marker binding, then reads the required reference profile.</summary>
    /// <param name="image">Complete PE bytes; do not mutate during this call.</param>
    /// <param name="expectedExtended">True by default: reject images with both extension markers absent. False permits ordinary CLI classification.</param>
    /// <returns>An owned profile for recognized extended input, otherwise an ordinary classification only.</returns>
    /// <exception cref="InvalidDataException">Unsupported/malformed PE, missing or inconsistent markers, bad stream binding or invalid reference profile.</exception>
    /// <remarks>Digest checking is consistency, not authentication, IL verification, declaration binding or an execution guard. No image is loaded or executed.</remarks>
    public static MetadataArtifact Read(ReadOnlySpan<byte> image, bool expectedExtended = true)
    {
        var envelope = ReadEnvelope(image, expectedExtended);
        return new(envelope is null ? null : MetadataProfile.Read(envelope));
    }

    internal static byte[]? ReadEnvelope(ReadOnlySpan<byte> image, bool expectedExtended = true)
    {
        if (image.Length > MaxImageSize) throw Invalid("image exceeds 4 MiB limit");
        var metadata = ContainerMetadata(image.ToArray());
        var data = new Bytes(metadata);
        if (!data.Take(0, 4).SequenceEqual("BSJB"u8)) throw Invalid("invalid metadata signature");
        long versionLength = data.U32(12);
        if (versionLength > 256 || versionLength % 4 != 0) throw Invalid("unsupported metadata version length");
        var version = data.Take(16, versionLength);
        long position = 16 + versionLength;
        int count = data.U16(position + 2);
        if (count > 16) throw Invalid("too many metadata streams");
        position += 4;
        var entries = new List<(string Name, long Offset, long Size)>();
        var names = new HashSet<string>(StringComparer.Ordinal);
        for (int i = 0; i < count; i++)
        {
            long offset = data.U32(position), size = data.U32(position + 4);
            position += 8;
            long start = position;
            var name = new StringBuilder();
            bool terminated = false;
            for (int n = 0; n < 32; n++)
            {
                byte value = data.Take(position++, 1)[0];
                if (value == 0) { terminated = true; break; }
                if (value > 127) throw Invalid("non-ASCII stream name");
                name.Append((char)value);
            }
            if (!terminated) throw Invalid("unterminated stream name");
            long end = start + Align(position - start, 4);
            if (Nonzero(data.Take(position, end - position))) throw Invalid("invalid stream-name padding");
            position = end;
            string key = name.ToString();
            if (key.Length == 0 || !names.Add(key)) throw Invalid("empty or duplicate stream name");
            entries.Add((key, offset, size));
        }
        var streams = new Dictionary<string, byte[]>(StringComparer.Ordinal);
        for (int i = 0; i < entries.Count; i++)
        {
            var entry = entries[i];
            if (entry.Offset < position || entry.Offset % 4 != 0) throw Invalid("invalid metadata stream offset");
            streams.Add(entry.Name, data.Take(entry.Offset, entry.Size).ToArray());
            foreach (var other in entries.Take(i))
                if (Overlap(entry.Offset, entry.Size, other.Offset, other.Size)) throw Invalid("overlapping metadata streams");
        }
        bool marked = version.StartsWith("neoCLR."u8);
        bool hasStream = streams.TryGetValue("#Neo", out var payload);
        if (!marked && !hasStream)
        {
            if (expectedExtended) throw Invalid("expected extended artifact; marker and #Neo missing");
            return null;
        }
        if (!marked) throw Invalid("unmarked #Neo transport is not a recognized artifact");
        if (!hasStream) throw Invalid("required #Neo stream missing from marked artifact");
        int terminator = version.IndexOf((byte)0);
        if (terminator < 0 || Nonzero(version[terminator..])) throw Invalid("invalid recognition marker termination");
        var content = version[..terminator];
        ReadOnlySpan<byte> prefix = "neoCLR.NEOX.0.1;sha256="u8;
        if (!content.StartsWith(prefix)) throw Invalid("unsupported recognition version");
        var claimed = content[prefix.Length..];
        if (claimed.Length != 64) throw Invalid("invalid binding digest");
        foreach (byte value in claimed)
            if (!(value is >= (byte)'0' and <= (byte)'9' or >= (byte)'a' and <= (byte)'f')) throw Invalid("invalid binding digest");
        using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        digest.AppendData("neoCLR experimental metadata binding 0.1\0"u8);
        Span<byte> number = stackalloc byte[4];
        foreach (var stream in streams.OrderBy(pair => pair.Key, StringComparer.Ordinal))
        {
            var name = Encoding.ASCII.GetBytes(stream.Key);
            BinaryPrimitives.WriteUInt16LittleEndian(number, (ushort)name.Length);
            digest.AppendData(number[..2]);
            digest.AppendData(name);
            BinaryPrimitives.WriteUInt32LittleEndian(number, (uint)stream.Value.Length);
            digest.AppendData(number);
            digest.AppendData(stream.Value);
        }
        var actual = Encoding.ASCII.GetBytes(Convert.ToHexString(digest.GetHashAndReset()).ToLowerInvariant());
        if (!claimed.SequenceEqual(actual)) throw Invalid("metadata binding mismatch; aware rewrite required");
        var neo = new Bytes(payload!);
        long length = neo.U32(12);
        if (length < 16 || length > payload!.Length || payload.Length - length > 3 || Nonzero(neo.Take(length, payload.Length - length)))
            throw Invalid("invalid #Neo padding/length");
        return neo.Take(0, length).ToArray();
    }

    private static byte[] ContainerMetadata(byte[] image)
    {
        var data = new Bytes(image);
        if (!data.Take(0, 2).SequenceEqual("MZ"u8)) throw Invalid("invalid DOS signature");
        long pe = data.U32(0x3c);
        if (!data.Take(pe, 4).SequenceEqual("PE\0\0"u8)) throw Invalid("invalid PE signature");
        int count = data.U16(pe + 6), optionalSize = data.U16(pe + 20);
        long optional = pe + 24;
        if (count is < 1 or > 16 || optionalSize != 224 || data.U16(optional) != 0x10b) throw Invalid("bounded PE32 required");
        if (data.U32(optional + 92) != 16) throw Invalid("unsupported PE directory count");
        long directories = optional + 96;
        if (Nonzero(data.Take(directories + 32, 8)) || data.U32(optional + 64) != 0) throw Invalid("signed/checksummed images unsupported");
        long sectionAlignment = data.U32(optional + 32), fileAlignment = data.U32(optional + 36);
        if (fileAlignment < 512 || fileAlignment > 65536 || !PowerOfTwo(fileAlignment) ||
            sectionAlignment < fileAlignment || sectionAlignment > 65536 || !PowerOfTwo(sectionAlignment)) throw Invalid("unsupported PE alignment");
        long table = optional + optionalSize, headers = data.U32(optional + 60);
        if (headers > image.Length || table + count * 40 > headers) throw Invalid("invalid section table range");
        var sections = new List<(long Rva, long VirtualSize, long Raw, long Size)>();
        for (int i = 0; i < count; i++)
        {
            long row = table + i * 40;
            long virtualSize = data.U32(row + 8), rva = data.U32(row + 12), size = data.U32(row + 16), raw = data.U32(row + 20);
            if (raw < headers || raw % fileAlignment != 0 || size % fileAlignment != 0 || rva % sectionAlignment != 0 || rva < Align(headers, sectionAlignment))
                throw Invalid("invalid section alignment/range");
            data.Take(raw, size);
            foreach (var other in sections)
                if (Overlap(rva, Math.Max(virtualSize, size), other.Rva, Math.Max(other.VirtualSize, other.Size)) || Overlap(raw, size, other.Raw, other.Size))
                    throw Invalid("overlapping PE sections");
            sections.Add((rva, virtualSize, raw, size));
        }
        if (sections.Max(s => s.Raw + s.Size) != image.Length) throw Invalid("PE overlays unsupported");
        long Map(long rva, long size)
        {
            foreach (var section in sections)
                if (section.Rva <= rva && rva - section.Rva <= section.Size && size <= section.Size - (rva - section.Rva))
                    return section.Raw + rva - section.Rva;
            throw Invalid("RVA outside file-backed sections");
        }
        long cliSize = data.U32(directories + 14 * 8 + 4);
        if (cliSize != 72) throw Invalid("unsupported CLI header size");
        long cli = Map(data.U32(directories + 14 * 8), cliSize);
        if (data.U32(cli) != 72 || data.U32(cli + 16) != 1 || Nonzero(data.Take(cli + 32, 8)) || Nonzero(data.Take(cli + 64, 8)))
            throw Invalid("only unsigned IL-only fixture images supported");
        long metadataSize = data.U32(cli + 12);
        return data.Take(Map(data.U32(cli + 8), metadataSize), metadataSize).ToArray();
    }

    private sealed class Bytes(byte[] data)
    {
        public ReadOnlySpan<byte> Take(long offset, long size)
        {
            if (offset < 0 || size < 0 || offset > data.Length || size > data.Length - offset) throw Invalid("truncated container range");
            return data.AsSpan((int)offset, (int)size);
        }
        public ushort U16(long offset) => BinaryPrimitives.ReadUInt16LittleEndian(Take(offset, 2));
        public uint U32(long offset) => BinaryPrimitives.ReadUInt32LittleEndian(Take(offset, 4));
    }
    private static bool Nonzero(ReadOnlySpan<byte> bytes) { foreach (byte value in bytes) if (value != 0) return true; return false; }
    private static bool PowerOfTwo(long value) => value > 0 && (value & (value - 1)) == 0;
    private static long Align(long value, long alignment) => (value + alignment - 1) & ~(alignment - 1);
    private static bool Overlap(long a, long n, long b, long m) => Math.Max(a, b) < Math.Min(a + n, b + m);
    private static InvalidDataException Invalid(string message) => new(message);
}
