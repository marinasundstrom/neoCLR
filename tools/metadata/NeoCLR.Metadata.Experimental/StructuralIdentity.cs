namespace NeoCLR.Metadata.Experimental;

/// <summary>A resolved structural equality key within compatible host catalogs; not a persistent identifier.</summary>
public sealed class ResolvedTypeIdentity : IEquatable<ResolvedTypeIdentity>
{
    private readonly byte[] key;
    internal ResolvedTypeIdentity(byte[] key) => this.key = key;
    /// <summary>Compares the complete resolved key, not a hash or display name.</summary>
    /// <param name="other">Other identity, or null.</param>
    /// <returns>True only for equal resolved structure and declaration scopes.</returns>
    public bool Equals(ResolvedTypeIdentity? other) => other is not null && key.AsSpan().SequenceEqual(other.key);
    /// <summary>Compares another resolved identity; other object kinds are unequal.</summary>
    /// <param name="obj">Candidate object.</param>
    /// <returns>Whether the full identity is equal.</returns>
    public override bool Equals(object? obj) => obj is ResolvedTypeIdentity other && Equals(other);
    /// <summary>Returns a process-local hash consistent with equality, not a persistent fingerprint.</summary>
    /// <returns>The hash code.</returns>
    public override int GetHashCode() { var hash = new HashCode(); hash.AddBytes(key); return hash.ToHashCode(); }
}

/// <summary>Resolves signature structure against explicitly supplied authoritative declarations.</summary>
public static class StructuralIdentity
{
    /// <summary>Validates references/owners and builds a normalized, catalog-scoped equality key.</summary>
    /// <param name="root">Signature tree.</param>
    /// <param name="context">Local binder counts and Self permission.</param>
    /// <param name="bindings">Reference indices and declaring owners.</param>
    /// <param name="catalog">Authoritative definitions, including all referenced dependencies. Do not mutate during the call.</param>
    /// <returns>A full structural key without consuming-image reference indices.</returns>
    /// <exception cref="ArgumentNullException">Any argument is null.</exception>
    /// <exception cref="InvalidDataException">Invalid syntax, missing/wrong-kind definitions, owner/arity mismatch or unsupported Self contract.</exception>
    /// <remarks>Flattens like unions/intersections, removes duplicates and ignores operand order. Does not perform assignability, subtype reduction, execution or PE resolution.</remarks>
    public static ResolvedTypeIdentity Resolve(TypeExpression root, SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog)
    {
        ArgumentNullException.ThrowIfNull(root); ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(bindings); ArgumentNullException.ThrowIfNull(catalog);
        StructuralSignature.Write(root, context, allowReferences: true); // bound recursion before key construction
        ReferenceTable.Validate(bindings);
        if ((context.TypeParameters != 0 && bindings.TypeOwner == 0) ||
            (context.MethodParameters != 0 && bindings.MethodOwner == 0) || context.SelfAllowed != (bindings.SelfOwner != 0))
            throw new InvalidDataException("signature context lacks matching owner");
        MetadataDefinition Definition(MetadataReference reference)
        {
            if (!catalog.TryGetValue(reference, out var value) || value is null) throw new InvalidDataException("unresolved definition");
            bool method = reference.Token >> 24 == 6;
            if ((method && value.Kind != "method") || (!method && value.Kind is not ("type" or "interface")))
                throw new InvalidDataException("catalog definition kind mismatch");
            if (value.Arity < 0 || value.Arity > 256) throw new InvalidDataException("invalid catalog arity");
            if (method)
            {
                if (value.Owner is not { } owner) throw new InvalidDataException("method requires declaring type");
                ReferenceTable.ValidateReference(owner);
                if (owner.Token >> 24 != 2) throw new InvalidDataException("method requires declaring type");
            }
            else if (value.Owner is not null) throw new InvalidDataException("unexpected catalog owner");
            return value;
        }
        foreach (var reference in bindings.References) Definition(reference);
        MetadataReference? Owner(int index) => index == 0 ? null : bindings.References[index - 1];
        var typeOwner = Owner(bindings.TypeOwner);
        var methodOwner = Owner(bindings.MethodOwner);
        var selfOwner = Owner(bindings.SelfOwner);
        foreach (var (owner, arity) in new[] { (typeOwner, context.TypeParameters), (methodOwner, context.MethodParameters) })
            if (owner is { } reference && Definition(reference).Arity != arity) throw new InvalidDataException("binder arity disagrees with definition");
        if (methodOwner is { } methodRef)
        {
            var definition = Definition(methodRef);
            Definition(definition.Owner!.Value);
            if (definition.Owner != typeOwner) throw new InvalidDataException("method/type owner mismatch");
        }
        if (selfOwner is { } selfRef && (Definition(selfRef).Kind != "interface" || Definition(selfRef).Arity != 0))
            throw new InvalidDataException("Self owner must be a nongeneric interface contract");

        KeyNode Build(TypeExpression node)
        {
            var children = node.Children.Select(Build).ToArray();
            MetadataReference? reference = null;
            int ordinal = 0;
            switch (node.Kind)
            {
                case "nominal":
                    if (node.Index > bindings.References.Count) throw new InvalidDataException("nominal reference outside table");
                    reference = bindings.References[node.Index - 1];
                    if (reference.Value.Token >> 24 != 2) throw new InvalidDataException("nominal reference is not a type");
                    if (Definition(reference.Value).Arity != children.Length) throw new InvalidDataException("nominal generic argument count mismatch");
                    break;
                case "type_parameter": reference = typeOwner; ordinal = node.Index; break;
                case "method_parameter": reference = methodOwner; ordinal = node.Index; break;
                case "self": reference = selfOwner; break;
                case "union": case "intersection":
                    var flat = children.SelectMany(c => c.Kind == node.Kind ? c.Children : new[] { c }).ToList();
                    flat.Sort((a, b) => a.Bytes.AsSpan().SequenceCompareTo(b.Bytes));
                    var distinct = new List<KeyNode>();
                    foreach (var child in flat)
                        if (distinct.Count == 0 || !distinct[^1].Bytes.AsSpan().SequenceEqual(child.Bytes)) distinct.Add(child);
                    children = distinct.ToArray();
                    break;
            }
            return new KeyNode(node.Kind, children, reference, ordinal, node.Modes, node.NoResult);
        }
        return new ResolvedTypeIdentity(Build(root).Bytes);
    }

    internal static ResolvedTypeIdentity NativeUnsignedResult() =>
        new(new KeyNode("intrinsic_native_uint", [], null, 0, [], false).Bytes);

    // Private canonical equality representation, intentionally not a serialization API.
    private sealed class KeyNode
    {
        public string Kind { get; }
        public KeyNode[] Children { get; }
        public byte[] Bytes { get; }
        public KeyNode(string kind, KeyNode[] children, MetadataReference? reference, int ordinal,
            IReadOnlyList<string> modes, bool noResult)
        {
            Kind = kind; Children = children;
            using var output = new MemoryStream();
            using var writer = new BinaryWriter(output);
            writer.Write(kind); writer.Write(reference.HasValue);
            if (reference is { } value)
            {
                writer.Write(value.Assembly.ToByteArray()); writer.Write(value.Module.ToByteArray()); writer.Write(value.Token);
            }
            writer.Write(ordinal); writer.Write(noResult); writer.Write(modes.Count);
            foreach (var mode in modes) writer.Write(mode);
            writer.Write(children.Length);
            foreach (var child in children) { writer.Write(child.Bytes.Length); writer.Write(child.Bytes); }
            Bytes = output.ToArray();
        }
    }
}
