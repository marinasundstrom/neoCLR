namespace NeoCLR.Metadata.Experimental;

/// <summary>An immutable syntax tree for an experimental structural signature; not a resolved type identity.</summary>
public sealed class TypeExpression
{
    /// <summary>Copies a node's ordered children and parameter modes. Shape validation occurs in the codec.</summary>
    /// <param name="kind">Wire-kind spelling, such as int32, tuple, function or nominal.</param>
    /// <param name="children">Ordered children; a Function's result is last. Null means empty.</param>
    /// <param name="index">Generic ordinal or one-based nominal reference index; otherwise zero.</param>
    /// <param name="modes">Function parameter-mode spellings; null means empty.</param>
    /// <param name="noResult">Whether a Function has no result instead of an inhabited unit result.</param>
    /// <exception cref="ArgumentNullException">Kind is null.</exception>
    /// <exception cref="ArgumentException">A list contains null or exceeds local arity limits.</exception>
    public TypeExpression(string kind, IReadOnlyList<TypeExpression>? children = null,
        int index = 0, IReadOnlyList<string>? modes = null, bool noResult = false)
    {
        ArgumentNullException.ThrowIfNull(kind);
        if ((children?.Count ?? 0) > 257 || (modes?.Count ?? 0) > 256)
            throw new ArgumentException("local signature arity limit");
        var copiedChildren = children?.ToArray() ?? [];
        var copiedModes = modes?.ToArray() ?? [];
        if (copiedChildren.Any(c => c is null) || copiedModes.Any(m => m is null))
            throw new ArgumentException("null signature child or mode");
        Kind = kind;
        Children = Array.AsReadOnly(copiedChildren);
        Index = index;
        Modes = Array.AsReadOnly(copiedModes);
        NoResult = noResult;
    }
    /// <summary>Gets the case-sensitive wire-kind spelling.</summary>
    public string Kind { get; }
    /// <summary>Gets owned, read-only ordered children.</summary>
    public IReadOnlyList<TypeExpression> Children { get; }
    /// <summary>Gets the generic ordinal or local nominal reference index.</summary>
    public int Index { get; }
    /// <summary>Gets owned, read-only Function parameter modes.</summary>
    public IReadOnlyList<string> Modes { get; }
    /// <summary>Gets the Function's no-result flag.</summary>
    public bool NoResult { get; }
}

/// <summary>Local generic arities and Self permission, without resolved declaring identities.</summary>
public sealed class SignatureContext
{
    /// <summary>Creates a context with arities in the inclusive range 0–256.</summary>
    /// <param name="typeParameters">Declaring-type generic arity.</param>
    /// <param name="methodParameters">Declaring-method generic arity.</param>
    /// <param name="selfAllowed">Whether a symbolic Self node is permitted.</param>
    /// <exception cref="ArgumentOutOfRangeException">An arity is outside 0–256.</exception>
    public SignatureContext(int typeParameters = 0, int methodParameters = 0, bool selfAllowed = false)
    {
        if (typeParameters < 0 || typeParameters > 256) throw new ArgumentOutOfRangeException(nameof(typeParameters));
        if (methodParameters < 0 || methodParameters > 256) throw new ArgumentOutOfRangeException(nameof(methodParameters));
        TypeParameters = typeParameters;
        MethodParameters = methodParameters;
        SelfAllowed = selfAllowed;
    }
    /// <summary>Gets the local type-parameter count.</summary>
    public int TypeParameters { get; }
    /// <summary>Gets the local method-parameter count.</summary>
    public int MethodParameters { get; }
    /// <summary>Gets whether symbolic Self is permitted.</summary>
    public bool SelfAllowed { get; }
}
