namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Supported logical instruction codes for the bounded CLI/native body writers.</summary>
/// <remarks>Numeric enum values are not serialized opcodes. This is not the complete CLI or neoIL instruction set.</remarks>
public enum OpCode
{
    /// <summary>Pushes an Int32 constant; requires an Int32 operand.</summary>
    Ldc_I4,
    /// <summary>Loads an Int32 argument by zero-based index; requires an Int32 operand.</summary>
    Ldarg,
    /// <summary>Adds two Int32 values.</summary>
    Add,
    /// <summary>Subtracts two Int32 values.</summary>
    Sub,
    /// <summary>Multiplies two Int32 values.</summary>
    Mul,
    /// <summary>Calls a typed method reference; requires a supported method operand.</summary>
    Call,
    /// <summary>Returns with the method's declared stack shape.</summary>
    Ret,
    /// <summary>Loads an Int32 local; requires a slot index or owned local.</summary>
    Ldloc,
    /// <summary>Stores an Int32 local; requires a slot index or owned local.</summary>
    Stloc,
    /// <summary>Compares two Int32 values for equality, pushing Boolean.</summary>
    Ceq,
    /// <summary>Compares two signed Int32 values for less-than, pushing Boolean.</summary>
    Clt,
    /// <summary>Compares two signed Int32 values for greater-than, pushing Boolean.</summary>
    Cgt,
    /// <summary>Branches unconditionally to a BranchLabel.</summary>
    Br,
    /// <summary>Consumes Boolean and branches when true.</summary>
    Brtrue,
    /// <summary>Consumes Boolean and branches when false.</summary>
    Brfalse,
    /// <summary>Pushes a Boolean constant; requires a Boolean operand.</summary>
    Ldc_Bool
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends an operand-free arithmetic, comparison or return instruction.</summary>
    /// <param name="opCode">Add, Sub, Mul, Ceq, Clt, Cgt or Ret.</param>
    /// <exception cref="ArgumentException">Unknown opcode or an opcode requiring an operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    /// <remarks>Stack and return-flow validation remains deferred until writing. Rejected emission does not change the body.</remarks>
    public void Emit(OpCode opCode)
        => Append(new(opCode switch {
            OpCode.Ceq => "equal", OpCode.Clt => "less", OpCode.Cgt => "greater",
            OpCode.Add => "add", OpCode.Sub => "subtract", OpCode.Mul => "multiply", OpCode.Ret => "return",
            _ => throw OperandError(opCode)
        }));

    /// <summary>Appends an Int32 constant, argument-index or local-index instruction.</summary>
    /// <param name="opCode">Ldc_I4, Ldarg, Ldloc or Stloc.</param>
    /// <param name="operand">Signed constant, or zero-based argument/local index validated when writing.</param>
    /// <exception cref="ArgumentException">Unknown opcode or opcode incompatible with an Int32 operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, int operand)
        => Append(new(opCode switch {
            OpCode.Ldc_I4 => "constant", OpCode.Ldarg => "argument",
            OpCode.Ldloc => "local.load", OpCode.Stloc => "local.store", _ => throw OperandError(opCode)
        }, operand));

    /// <summary>Appends a call to a local or external builder method.</summary>
    /// <param name="opCode">Call.</param>
    /// <param name="operand">Method with a supported signature; external identity/core contracts are checked when writing.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Opcode is not Call.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, MethodBuilder operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        Append(new("call", Target: operand));
    }

    /// <summary>Appends a call to an owned imported read-only method reference.</summary>
    /// <param name="opCode">Call.</param>
    /// <param name="operand">Reference imported by this output assembly builder.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Opcode is not Call or the reference belongs to another builder.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, ImportedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (!ReferenceEquals(operand.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(operand));
        Append(new("call", Target: operand.Target));
    }

    /// <summary>Appends a native-only call to a loaded static Int32 System declaration.</summary>
    /// <param name="opCode">Call.</param>
    /// <param name="operand">Owned native System function with an admitted Int32 signature.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Opcode is not Call.</exception>
    /// <exception cref="InvalidDataException">Unsupported module/signature or instruction limit exceeded.</exception>
    /// <remarks>Same matching-System runtime requirement as Call(NativeFunctionDefinition). Ordinary CLI output rejects this instruction.</remarks>
    public void Emit(OpCode opCode, NativeFunctionDefinition operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (operand.Library.ModuleName != "System" || !operand.TryGetStaticInt32Signature(out _))
            throw new InvalidDataException("native call requires a static Int32 System function");
        Append(new("native.call", NativeTarget: operand));
    }

    /// <summary>Pushes a Boolean constant for branch conditions.</summary>
    /// <param name="opCode">Ldc_Bool.</param>
    /// <param name="operand">The Boolean value.</param>
    /// <exception cref="ArgumentException">Incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, bool operand)
    {
        if (opCode != OpCode.Ldc_Bool) throw OperandError(opCode);
        Append(new("boolean", operand ? 1 : 0));
    }

    private static ArgumentException OperandError(OpCode opCode)
        => new($"Unsupported opcode or operand kind: {opCode}", nameof(opCode));
    private static void RequireCall(OpCode opCode)
    {
        if (opCode != OpCode.Call) throw OperandError(opCode);
    }
}
