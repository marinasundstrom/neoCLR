namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned physical CLI MemberRef, separate from method definitions and runtime binding.</summary>
public sealed class MemberReference
{
    private readonly byte[] signature;
    internal MemberReference(ModuleDefinition module, AssemblyDefinition.MemberReferenceRow row)
    {
        Module = module;
        MetadataToken = row.Token;
        ParentToken = row.ParentToken;
        Name = row.Name;
        signature = row.Signature;
    }
    /// <summary>Gets the consuming module snapshot.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the physical MemberRef token in the consuming module.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the physical MemberRefParent token, without resolving it.</summary>
    public uint ParentToken { get; }
    /// <summary>Gets the referenced metadata name.</summary>
    public string Name { get; }
    /// <summary>Copies the original signature blob, including unsupported signatures.</summary>
    /// <returns>New owned signature bytes.</returns>
    public byte[] GetSignature() => (byte[])signature.Clone();
    /// <summary>Resolves the static primitive/vector/no-result and unconstrained generic method subset through a nominal TypeDef or TypeRef parent.</summary>
    /// <param name="resolver">Explicit assembly resolver required by external TypeRef scopes.</param>
    /// <returns>The unique matching method in the resolved type's owned snapshot.</returns>
    /// <exception cref="InvalidDataException">Unsupported signature/parent, missing or mismatched dependency, or absent/ambiguous method.</exception>
    /// <remarks>Does not compare module-local signature tokens as cross-module identities. Field references, constrained generic/instance methods,
    /// inherited lookup, access checks, TypeSpec, ModuleRef, MethodDef-parent varargs and global MemberRefs are not resolved.
    /// Host resolver failures propagate. No resolution result is cached.</remarks>
    public MethodDefinition ResolveMethod(IAssemblyResolver? resolver = null)
    {
        if (!MethodDefinition.TryDecodeStaticValueSignature(signature, out var decoded,
                signature.Length > 1 && signature[0] == 0x10 && signature[1] is > 0 and <= 32 ? signature[1] : 0))
            throw new InvalidDataException("unsupported member method signature");
        TypeDefinition owner = (ParentToken >> 24) switch
        {
            0x02 => Module.GetTypeDefinition(ParentToken) ?? throw new InvalidDataException("missing member TypeDef parent"),
            0x01 => (Module.TypeReferences.FirstOrDefault(type => type.MetadataToken == ParentToken)
                ?? throw new InvalidDataException("missing member TypeRef parent")).Resolve(resolver),
            _ => throw new InvalidDataException("unsupported member method parent")
        };
        var matches = owner.Methods.Where(method => method.Name == Name &&
            (method.TryGetStaticValueSignature(out var candidate) || method.TryGetStaticGenericValueSignature(out candidate)) && candidate!.Matches(decoded!)).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("member method missing or ambiguous: " + Name);
        return matches[0];
    }
}
