namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable Int32 local-slot identity owned by one method builder.</summary>
public sealed class LocalDefinition
{
    internal LocalDefinition(MethodBuilder method, int index) { Method = method; Index = index; }
    /// <summary>Gets the owning method.</summary>
    public MethodBuilder Method { get; }
    /// <summary>Gets the zero-based slot index.</summary>
    public int Index { get; }
}

public sealed partial class MethodBuilder
{
    private readonly List<LocalDefinition> locals = [];
    /// <summary>Gets Int32 locals in slot order. ClearBody preserves these declarations.</summary>
    public IReadOnlyList<LocalDefinition> Locals => locals.AsReadOnly();
    /// <summary>Declares an Int32 local. Loads require an earlier store in the linear body.</summary>
    /// <returns>A local handle owned by this method.</returns>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    public LocalDefinition DeclareInt32Local()
    {
        if (locals.Count >= 256) throw new InvalidDataException("local limit exceeded");
        var local = new LocalDefinition(this, locals.Count); locals.Add(local); return local;
    }
    /// <summary>Loads a local previously stored in this body.</summary>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Local belongs to another method.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void LoadLocal(LocalDefinition local) => Emit(OpCode.Ldloc, local);
    /// <summary>Stores the top Int32 stack value into a local.</summary>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Local belongs to another method.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void StoreLocal(LocalDefinition local) => Emit(OpCode.Stloc, local);
    /// <summary>Appends a typed local load or store. Stack/initialization validation occurs when writing.</summary>
    /// <param name="opCode">Ldloc or Stloc.</param>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, LocalDefinition local)
    {
        ArgumentNullException.ThrowIfNull(local);
        if (opCode is not (OpCode.Ldloc or OpCode.Stloc) || !ReferenceEquals(local.Method, this))
            throw new ArgumentException("invalid local operand", nameof(local));
        Emit(opCode, local.Index);
    }
}
