namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>The canonical authored storage for a method's instructions, locals and labels.</summary>
/// <remarks>Use MethodBuilder.ForDefinition for typed/raw emission. Arbitrary instruction editing and loaded body decoding remain pending.</remarks>
public sealed class MethodBodyDefinition
{
    internal MethodBodyDefinition(MethodDefinition method)
    {
        Method = method;
        Locals = LocalStorage.AsReadOnly();
        Labels = LabelStorage.AsReadOnly();
    }
    internal List<MethodBuilder.Operation> Instructions { get; } = [];
    internal List<LocalDefinition> LocalStorage { get; } = [];
    internal List<BranchLabel> LabelStorage { get; } = [];
    /// <summary>Gets the exact authored method owning this body.</summary>
    public MethodDefinition Method { get; }
    /// <summary>Gets a live read-only view of declared local slots.</summary>
    public IReadOnlyList<LocalDefinition> Locals { get; }
    /// <summary>Gets a live read-only view of allocated symbolic branch labels.</summary>
    public IReadOnlyList<BranchLabel> Labels { get; }
    /// <summary>Removes instructions and label marks while preserving local declarations and label handles.</summary>
    /// <remarks>Preserved labels must be marked again before use. Subsequent writing revalidates the body.</remarks>
    public void ClearInstructions() => Instructions.Clear();
}

public sealed partial class MethodDefinition
{
    private MethodBodyDefinition? authoredBody;
    /// <summary>Gets the canonical authored body shared by emission helpers.</summary>
    /// <exception cref="NotSupportedException">This is a loaded snapshot; body materialization is pending.</exception>
    /// <remarks>Abstract contracts and internal calls may expose an empty body, but writing rejects instructions or locals on them. Internal calls also reject labels.</remarks>
    public MethodBodyDefinition Body => AuthoredSignature is null
        ? throw new NotSupportedException("loaded method bodies are not materialized yet")
        : authoredBody ??= new MethodBodyDefinition(this);
}
