using System.Buffers.Binary;

namespace NeoCLR.Metadata.Experimental;

/// <summary>Reads and writes NEOX local/reference-profile signature payloads; no dependency resolution or assignability.</summary>
public static class StructuralSignature
{
    private static readonly string[] Kinds = ["", "int32", "string", "bool", "unit", "type_parameter",
        "method_parameter", "self", "array", "tuple", "function", "union", "intersection", "nullable", "array_ref", "nominal"];
    private static readonly string[] Modes = ["value", "ref", "readonly_ref", "out", "out_when_true"];

    /// <summary>Decodes and validates a complete payload into an owned immutable syntax tree.</summary>
    /// <param name="payload">Signature bytes, not a complete NEOX or PE image.</param>
    /// <param name="allowReferences">Allow nominal nodes for section 3; does not validate the reference table.</param>
    /// <returns>The root syntax tree and local binder context.</returns>
    /// <exception cref="InvalidDataException">Malformed/unsupported data, invalid context/shape or resource limit.</exception>
    public static (TypeExpression Root, SignatureContext Context) Read(ReadOnlySpan<byte> payload, bool allowReferences = false)
    {
        if (payload.Length < 5 || payload.Length > MetadataEnvelope.MaxImageSize) throw Invalid("invalid signature size");
        var reader = new Reader(payload);
        int types = reader.U16(payload.Length), methods = reader.U16(payload.Length), self = reader.Byte(payload.Length);
        if (types > 256 || methods > 256 || self > 1) throw Invalid("invalid signature context");
        var context = new SignatureContext(types, methods, self == 1);
        var root = reader.Node(payload.Length, 0);
        if (reader.Position != payload.Length) throw Invalid("trailing signature bytes");
        // Shared validation of in-memory and decoded trees, including canonical payloads.
        if (!Write(root, context, allowReferences).AsSpan().SequenceEqual(payload)) throw Invalid("noncanonical signature");
        return (root, context);
    }

    /// <summary>Validates and encodes a complete signature, preserving operand order and duplicates.</summary>
    /// <param name="root">Root syntax tree.</param>
    /// <param name="context">Local binder context.</param>
    /// <param name="allowReferences">Allow nominal nodes with local indices 1–256; reference resolution is separate.</param>
    /// <returns>A new caller-owned byte array.</returns>
    /// <exception cref="ArgumentNullException">Root or context is null.</exception>
    /// <exception cref="InvalidDataException">Unknown kind/mode, invalid shape/context, or resource limit.</exception>
    /// <remarks>Limits: depth 32 with root at zero, 4096 nodes, arity 256, and 1 MiB payload. This does not authorize execution.</remarks>
    public static byte[] Write(TypeExpression root, SignatureContext context, bool allowReferences = false)
    {
        ArgumentNullException.ThrowIfNull(root);
        ArgumentNullException.ThrowIfNull(context);
        int remaining = 4096;
        byte[] Encode(TypeExpression value, int depth)
        {
            if (depth > 32 || --remaining < 0) throw Invalid("signature resource limit");
            int tag = Array.IndexOf(Kinds, value.Kind);
            if (tag < 1 || (tag == 15 && !allowReferences)) throw Invalid("unknown or unsupported signature kind");
            var children = value.Children;
            if (value.Kind != "function" && (value.NoResult || value.Modes.Count != 0)) throw Invalid("unexpected Function contract");
            if (value.Kind is not ("type_parameter" or "method_parameter" or "nominal") && value.Index != 0)
                throw Invalid("unexpected index");
            using var payload = new MemoryStream();
            using var writer = new BinaryWriter(payload);
            void Child(TypeExpression child) => writer.Write(Encode(child, depth + 1));
            switch (value.Kind)
            {
                case "type_parameter": case "method_parameter":
                    int bound = value.Kind == "type_parameter" ? context.TypeParameters : context.MethodParameters;
                    if (value.Index < 0 || value.Index >= bound) throw Invalid("generic parameter outside binder");
                    writer.Write((ushort)value.Index);
                    break;
                case "self":
                    if (!context.SelfAllowed) throw Invalid("Self outside contract context");
                    break;
                case "array": case "array_ref": case "nullable":
                    if (children.Count != 1) throw Invalid("unary type requires one child");
                    Child(children[0]);
                    break;
                case "tuple": case "union": case "intersection":
                    int minimum = value.Kind == "tuple" ? 1 : 2;
                    if (children.Count < minimum || children.Count > 256) throw Invalid("invalid compound arity");
                    writer.Write((ushort)children.Count);
                    foreach (var child in children) Child(child);
                    break;
                case "nominal":
                    if (value.Index < 1 || value.Index > 256 || children.Count > 256) throw Invalid("invalid nominal index/arity");
                    writer.Write((ushort)value.Index);
                    writer.Write((ushort)children.Count);
                    foreach (var child in children) Child(child);
                    break;
                case "function":
                    if (children.Count == 0 || value.Modes.Count != children.Count - 1) throw Invalid("invalid Function shape");
                    if (value.NoResult && children[^1].Kind != "unit") throw Invalid("no-result Function requires unit result");
                    if (value.Modes.Contains("out_when_true") && (children[^1].Kind != "bool" || value.NoResult))
                        throw Invalid("conditional output requires bool result");
                    writer.Write((byte)0); // managed calling convention
                    writer.Write((byte)(value.NoResult ? 1 : 0));
                    writer.Write((ushort)value.Modes.Count);
                    for (int i = 0; i < value.Modes.Count; i++)
                    {
                        int mode = Array.IndexOf(Modes, value.Modes[i]);
                        if (mode < 0) throw Invalid("unknown parameter mode");
                        writer.Write((byte)mode);
                        Child(children[i]);
                    }
                    Child(children[^1]);
                    break;
            }
            if (tag <= 7 && children.Count != 0) throw Invalid("leaf type has children");
            using var result = new MemoryStream();
            using var output = new BinaryWriter(result);
            output.Write((byte)tag);
            output.Write((uint)payload.Length);
            payload.WriteTo(result);
            return result.ToArray();
        }
        using var stream = new MemoryStream();
        using var header = new BinaryWriter(stream);
        header.Write((ushort)context.TypeParameters);
        header.Write((ushort)context.MethodParameters);
        header.Write((byte)(context.SelfAllowed ? 1 : 0));
        header.Write(Encode(root, 0));
        if (stream.Length > MetadataEnvelope.MaxImageSize) throw Invalid("signature size limit");
        return stream.ToArray();
    }

    private static InvalidDataException Invalid(string message) => new(message);

    private ref struct Reader(ReadOnlySpan<byte> data)
    {
        private readonly ReadOnlySpan<byte> data = data;
        public int Position;
        private int nodes;
        private ReadOnlySpan<byte> Take(int count, int limit)
        {
            if (count > limit - Position) throw Invalid("truncated type payload");
            var value = data.Slice(Position, count);
            Position += count;
            return value;
        }
        public byte Byte(int limit) => Take(1, limit)[0];
        public ushort U16(int limit) => BinaryPrimitives.ReadUInt16LittleEndian(Take(2, limit));
        private uint U32(int limit) => BinaryPrimitives.ReadUInt32LittleEndian(Take(4, limit));
        public TypeExpression Node(int limit, int depth)
        {
            if (depth > 32 || ++nodes > 4096) throw Invalid("signature resource limit");
            int tag = Byte(limit);
            uint length = U32(limit);
            if (length > limit - Position) throw Invalid("invalid type node length");
            int end = Position + (int)length;
            if (tag < 1 || tag >= Kinds.Length) throw Invalid("unknown type node");
            string kind = Kinds[tag];
            var children = new List<TypeExpression>();
            var modes = new List<string>();
            int index = 0;
            bool noResult = false;
            switch (kind)
            {
                case "type_parameter": case "method_parameter": index = U16(end); break;
                case "array": case "array_ref": case "nullable": children.Add(Node(end, depth + 1)); break;
                case "tuple": case "union": case "intersection": case "nominal":
                    if (kind == "nominal") index = U16(end);
                    int count = U16(end);
                    if (count > 256) throw Invalid("signature arity limit");
                    for (int i = 0; i < count; i++) children.Add(Node(end, depth + 1));
                    break;
                case "function":
                    int convention = Byte(end), flags = Byte(end), parameters = U16(end);
                    if (convention != 0 || flags > 1 || parameters > 256) throw Invalid("invalid Function convention/flags/arity");
                    noResult = flags == 1;
                    for (int i = 0; i < parameters; i++)
                    {
                        int mode = Byte(end);
                        if (mode >= Modes.Length) throw Invalid("unknown parameter mode");
                        modes.Add(Modes[mode]);
                        children.Add(Node(end, depth + 1));
                    }
                    children.Add(Node(end, depth + 1));
                    break;
            }
            if (Position != end) throw Invalid("trailing type payload");
            return new TypeExpression(kind, children, index, modes, noResult);
        }
    }
}
