namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An exact metadata identity for explicit resolution, not CLR binding policy or signature trust.</summary>
public sealed class AssemblyIdentity : IEquatable<AssemblyIdentity>
{
    /// <summary>Constructs a normalized key-token identity while retaining ordinal name/culture and flags.</summary>
    /// <param name="name">Nonempty simple name.</param>
    /// <param name="version">Four components, each 0–65535.</param>
    /// <param name="culture">Metadata culture string; empty means neutral.</param>
    /// <param name="publicKeyToken">Empty for unsigned, or exactly 16 hexadecimal characters.</param>
    /// <param name="flags">Assembly metadata flags excluding the public-key representation bit (1).</param>
    /// <exception cref="ArgumentNullException">Name, version, culture or token is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/version/token or public-key representation flag.</exception>
    public AssemblyIdentity(string name, Version version, string culture = "", string publicKeyToken = "", uint flags = 0)
    {
        ArgumentNullException.ThrowIfNull(name); ArgumentNullException.ThrowIfNull(version);
        ArgumentNullException.ThrowIfNull(culture); ArgumentNullException.ThrowIfNull(publicKeyToken);
        if (name.Length == 0 || new[] { version.Major, version.Minor, version.Build, version.Revision }.Any(n => n is < 0 or > 65535) ||
            (publicKeyToken.Length != 0 && (publicKeyToken.Length != 16 || !publicKeyToken.All(Uri.IsHexDigit))) || (flags & 1) != 0)
            throw new ArgumentException("invalid exact assembly identity");
        Name = name; Version = version; Culture = culture;
        PublicKeyToken = publicKeyToken.ToLowerInvariant(); Flags = flags;
    }
    /// <summary>Gets the exact simple name.</summary>
    public string Name { get; }
    /// <summary>Gets the four-part version.</summary>
    public Version Version { get; }
    /// <summary>Gets the culture string; empty is neutral.</summary>
    public string Culture { get; }
    /// <summary>Gets a lowercase token, or empty for unsigned identity.</summary>
    public string PublicKeyToken { get; }
    /// <summary>Gets retained flags excluding the full-key/token representation bit.</summary>
    public uint Flags { get; }
    /// <summary>Compares exact name, version, culture, normalized token and retained flags.</summary>
    /// <param name="other">Candidate identity or null.</param>
    /// <returns>True for an exact match.</returns>
    public bool Equals(AssemblyIdentity? other) => other is not null && Name == other.Name && Version == other.Version &&
        Culture == other.Culture && PublicKeyToken == other.PublicKeyToken && Flags == other.Flags;
    /// <summary>Compares another identity; other object types are unequal.</summary>
    /// <param name="obj">Candidate object.</param>
    /// <returns>Whether identities match.</returns>
    public override bool Equals(object? obj) => obj is AssemblyIdentity other && Equals(other);
    /// <summary>Returns a process-local hash consistent with equality.</summary>
    /// <returns>A nonpersistent hash code.</returns>
    public override int GetHashCode() => HashCode.Combine(Name, Version, Culture, PublicKeyToken, Flags);
}

/// <summary>Host-supplied metadata dependency lookup; no implicit filesystem or runtime loading.</summary>
public interface IAssemblyResolver
{
    /// <summary>Finds an already read assembly or reads one under the host's explicit policy.</summary>
    /// <param name="identity">Requested exact identity.</param>
    /// <returns>A candidate snapshot, or null when absent. The reference rechecks its identity.</returns>
    AssemblyDefinition? Resolve(AssemblyIdentity identity);
}

/// <summary>An owned physical AssemblyRef row in a consuming module.</summary>
public sealed class AssemblyReference
{
    internal AssemblyReference(ModuleDefinition module, uint token, AssemblyIdentity identity)
    { Module = module; MetadataToken = token; Identity = identity; }
    /// <summary>Gets the consuming module, not the referenced module.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets this image's AssemblyRef token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the requested exact metadata identity.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Requests a dependency and rejects missing or mismatched candidates.</summary>
    /// <param name="resolver">Explicit host resolver; results are not cached by this reference.</param>
    /// <returns>The candidate whose exact identity matches.</returns>
    /// <exception cref="ArgumentNullException">Resolver is null.</exception>
    /// <exception cref="InvalidDataException">No candidate or a mismatched identity was returned.</exception>
    /// <remarks>Host resolver exceptions propagate; this does not verify signatures or resolve types.</remarks>
    public AssemblyDefinition Resolve(IAssemblyResolver resolver)
    {
        ArgumentNullException.ThrowIfNull(resolver);
        var assembly = resolver.Resolve(Identity);
        if (assembly is null) throw new InvalidDataException("assembly dependency not found: " + Identity.Name);
        if (!Identity.Equals(assembly.Identity)) throw new InvalidDataException("assembly dependency identity mismatch: " + Identity.Name);
        return assembly;
    }
}
