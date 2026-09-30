namespace NeoCLR.Metadata.Experimental;

/// <summary>An owned, locally validated reference profile, independent of compiler or runtime objects.</summary>
/// <remarks>Local validity does not establish that catalog declarations exist or that an assembly can execute.</remarks>
public sealed class MetadataProfileDocument
{
    internal MetadataProfileDocument(IReadOnlyList<MetadataSection> sections, TypeExpression root,
        SignatureContext context, ReferenceBindings bindings, IReadOnlyList<StructuralMemberReference> members,
        bool hasMemberTable)
    {
        Sections = sections;
        Root = root;
        Context = context;
        Bindings = bindings;
        Members = members;
        HasMemberTable = hasMemberTable;
        UnknownOptionalSections = Array.AsReadOnly(sections.Where(s => s.Kind > 4 || (s.Kind == 4 && s.Version != 1)).ToArray());
    }
    /// <summary>Gets all original immutable sections in input order, including opaque optional data.</summary>
    public IReadOnlyList<MetadataSection> Sections { get; }
    /// <summary>Gets the section-3 structural signature root.</summary>
    public TypeExpression Root { get; }
    /// <summary>Gets local generic binder counts and Self permission.</summary>
    public SignatureContext Context { get; }
    /// <summary>Gets section-2 nominal references and declaring owners.</summary>
    public ReferenceBindings Bindings { get; }
    /// <summary>Gets decoded schema-1 structural members, empty when absent.</summary>
    public IReadOnlyList<StructuralMemberReference> Members { get; }
    /// <summary>Gets whether a supported schema-1 member table is present, including an empty table.</summary>
    public bool HasMemberTable { get; }
    /// <summary>Gets unrecognized optional sections preserved without interpreting their payloads.</summary>
    public IReadOnlyList<MetadataSection> UnknownOptionalSections { get; }

    /// <summary>Resolves the root against explicit host declarations.</summary>
    /// <param name="catalog">Authoritative catalog; do not mutate during resolution.</param>
    /// <returns>Catalog-scoped structural identity.</returns>
    /// <exception cref="ArgumentNullException">Catalog is null.</exception>
    /// <exception cref="InvalidDataException">References, declaration kinds, arities or owners do not resolve.</exception>
    public ResolvedTypeIdentity ResolveType(IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog) =>
        StructuralIdentity.Resolve(Root, Context, Bindings, catalog);

    /// <summary>Resolves the owner and derives supported member contracts, even if the list is empty.</summary>
    /// <param name="catalog">Authoritative catalog; do not mutate during resolution.</param>
    /// <returns>Owned read-only member descriptors in table order.</returns>
    /// <exception cref="ArgumentNullException">Catalog is null.</exception>
    /// <exception cref="InvalidDataException">Owner declarations cannot be resolved.</exception>
    public IReadOnlyList<StructuralMemberDescriptor> ResolveMembers(IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog) =>
        StructuralMembers.Resolve(Members, Root, Context, Bindings, catalog);
}

/// <summary>Reads and constructs NEOX 0.1 reference profiles for future compiler adapters.</summary>
/// <remarks>Not a PE reader, assembly builder, local section-1 profile reader or executable loader.</remarks>
public static class MetadataProfile
{
    private static readonly IReadOnlyDictionary<ushort, ushort> Schemas = new Dictionary<ushort, ushort>
    { [2] = 1, [3] = 1, [4] = 1 };

    /// <summary>Reads mandatory reference/signature sections and checks local uses and member owner shapes.</summary>
    /// <param name="image">Complete envelope; do not mutate during the call.</param>
    /// <returns>An immutable owned document retaining section order and unknown optional payloads.</returns>
    /// <exception cref="InvalidDataException">Invalid framing, unsupported requirements, incomplete/mixed profiles or malformed/local-invalid payloads.</exception>
    public static MetadataProfileDocument Read(ReadOnlySpan<byte> image)
    {
        var sections = MetadataEnvelope.Read(image, Schemas);
        if (sections.Any(s => s.Kind == 1)) throw new InvalidDataException("cannot mix local and reference signature profiles");
        MetadataSection Required(ushort kind)
        {
            var section = sections.SingleOrDefault(s => s.Kind == kind);
            if (section is null || section.Version != 1 || !section.Required)
                throw new InvalidDataException("reference profile requires mandatory schema-1 reference and signature sections");
            return section;
        }
        var bindings = ReferenceTable.Read(Required(2).Payload);
        var (root, context) = StructuralSignature.Read(Required(3).Payload, allowReferences: true);
        if ((context.TypeParameters != 0 && bindings.TypeOwner == 0) ||
            (context.MethodParameters != 0 && bindings.MethodOwner == 0) || context.SelfAllowed != (bindings.SelfOwner != 0))
            throw new InvalidDataException("signature context lacks matching owner");
        void ValidateUses(TypeExpression node)
        {
            if (node.Kind == "nominal" && (node.Index > bindings.References.Count ||
                bindings.References[node.Index - 1].Token >> 24 != 2))
                throw new InvalidDataException("nominal reference must identify a local type entry");
            foreach (var child in node.Children) ValidateUses(child);
        }
        ValidateUses(root);
        var memberSection = sections.SingleOrDefault(s => s.Kind == 4 && s.Version == 1);
        IReadOnlyList<StructuralMemberReference> members = Array.AsReadOnly(Array.Empty<StructuralMemberReference>());
        if (memberSection is not null)
        {
            if (!memberSection.Required) throw new InvalidDataException("supported member table must be mandatory");
            members = StructuralMembers.Read(memberSection.Payload);
            foreach (var member in members) StructuralMembers.ValidateShape(member, root);
        }
        return new(sections, root, context, bindings, members, memberSection is not null);
    }

    /// <summary>Constructs a fresh locally validated profile with canonical section order 2, 3, optional 4, extensions.</summary>
    /// <param name="root">Signature root.</param>
    /// <param name="context">Generic/Self context.</param>
    /// <param name="bindings">Local references and owners; caller supplies correctly remapped indices.</param>
    /// <param name="members">Null omits section 4; an empty list emits a mandatory empty table.</param>
    /// <param name="optionalSections">Optional opaque extensions with kinds greater than 4; null means none.</param>
    /// <returns>An owned document ready for writing or explicit catalog resolution.</returns>
    /// <exception cref="ArgumentNullException">Root, context or bindings is null.</exception>
    /// <exception cref="InvalidDataException">Invalid payloads, uses, shapes, extension kinds/requirements, duplicate kinds or size limits.</exception>
    /// <remarks>Does not infer reference remapping or prove that opaque extensions remain meaningful after edits.</remarks>
    public static MetadataProfileDocument Create(TypeExpression root, SignatureContext context, ReferenceBindings bindings,
        IReadOnlyList<StructuralMemberReference>? members = null, IReadOnlyList<MetadataSection>? optionalSections = null)
    {
        var sections = new List<MetadataSection>
        {
            new(2, 1, true, ReferenceTable.Write(bindings)),
            new(3, 1, true, StructuralSignature.Write(root, context, allowReferences: true))
        };
        if (members is not null) sections.Add(new(4, 1, true, StructuralMembers.Write(members)));
        if (optionalSections is not null)
        {
            if (optionalSections.Count > MetadataEnvelope.MaxSections - sections.Count) throw new InvalidDataException("too many sections");
            foreach (var section in optionalSections)
            {
                if (section is null || section.Required || section.Kind <= 4) throw new InvalidDataException("extensions must be optional and outside reserved kinds 1-4");
                sections.Add(section);
            }
        }
        return Read(MetadataEnvelope.Write(sections, Schemas));
    }

    /// <summary>Writes the unchanged owned document, preserving section order and opaque optional bytes.</summary>
    /// <param name="document">Document returned by Read or Create.</param>
    /// <returns>A fresh complete envelope; a read/write round-trip is byte-identical.</returns>
    /// <exception cref="ArgumentNullException">Document is null.</exception>
    public static byte[] Write(MetadataProfileDocument document)
    {
        ArgumentNullException.ThrowIfNull(document);
        return MetadataEnvelope.Write(document.Sections, Schemas);
    }
}
