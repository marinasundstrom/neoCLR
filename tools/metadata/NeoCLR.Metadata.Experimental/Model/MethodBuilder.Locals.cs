namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive local-slot identity owned by one method builder.</summary>
public sealed class LocalDefinition
{
    internal LocalDefinition(MethodBuilder method, int index, PrimitiveType type) { Method = method; Index = index; Type = type; }
    /// <summary>Gets the owning method.</summary>
    public MethodBuilder Method { get; }
    /// <summary>Gets the zero-based slot index.</summary>
    public int Index { get; }
    /// <summary>Gets the declared Int32, Int64 or Boolean slot type.</summary>
    public PrimitiveType Type { get; }
}

public sealed partial class MethodBuilder
{
    private readonly List<LocalDefinition> locals = [];
    /// <summary>Gets primitive locals in slot order. ClearBody preserves these declarations.</summary>
    public IReadOnlyList<LocalDefinition> Locals => locals.AsReadOnly();
    /// <summary>Declares an Int32 local. Loads require a store on every reachable path.</summary>
    /// <returns>A local handle owned by this method.</returns>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    public LocalDefinition DeclareInt32Local() => DeclareLocal(PrimitiveType.Int32);
    /// <summary>Declares a typed primitive local. ClearBody retains the slot and type.</summary>
    /// <param name="type">Int32, Int64 or Boolean; Void is not a local type.</param>
    /// <returns>A stable local handle owned by this method.</returns>
    /// <exception cref="ArgumentException">Type is Void or an invalid enum value.</exception>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    public LocalDefinition DeclareLocal(PrimitiveType type)
    {
        if (type is not (PrimitiveType.Int32 or PrimitiveType.Int64 or PrimitiveType.Boolean)) throw new ArgumentException("unsupported local type", nameof(type));
        if (locals.Count >= 256) throw new InvalidDataException("local limit exceeded");
        var local = new LocalDefinition(this, locals.Count, type); locals.Add(local); return local;
    }
    /// <summary>Loads a local previously stored in this body.</summary>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Local belongs to another method.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void LoadLocal(LocalDefinition local) => Emit(OpCode.Ldloc, local);
    /// <summary>Stores the top value into a local of the same declared type.</summary>
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
