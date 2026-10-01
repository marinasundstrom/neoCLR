namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Supported logical instruction codes for the bounded CLI/native body writers.</summary>
/// <remarks>Numeric enum values are not serialized opcodes. This is not the complete CLI or neoIL instruction set.</remarks>
public enum OpCode
{
    /// <summary>Pushes an Int32 constant; requires an Int32 operand.</summary>
    Ldc_I4,
    /// <summary>Loads a declared primitive argument by zero-based index; requires an Int32 operand.</summary>
    Ldarg,
    /// <summary>Adds two matching Int32 or Int64 values.</summary>
    Add,
    /// <summary>Subtracts two matching Int32 or Int64 values.</summary>
    Sub,
    /// <summary>Multiplies two matching Int32 or Int64 values.</summary>
    Mul,
    /// <summary>Calls a typed method reference; requires a supported method operand.</summary>
    Call,
    /// <summary>Returns with the method's declared stack shape.</summary>
    Ret,
    /// <summary>Loads a declared primitive local; requires a slot index or owned local.</summary>
    Ldloc,
    /// <summary>Stores a declared primitive local; requires a slot index or owned local.</summary>
    Stloc,
    /// <summary>Compares matching Int32, Int64 or Boolean values for equality, pushing Boolean.</summary>
    Ceq,
    /// <summary>Compares two matching signed Int32 or Int64 values for less-than, pushing Boolean.</summary>
    Clt,
    /// <summary>Compares two matching signed Int32 or Int64 values for greater-than, pushing Boolean.</summary>
    Cgt,
    /// <summary>Branches unconditionally to a BranchLabel.</summary>
    Br,
    /// <summary>Consumes Boolean and branches when true.</summary>
    Brtrue,
    /// <summary>Consumes Boolean and branches when false.</summary>
    Brfalse,
    /// <summary>Pushes a Boolean constant; requires a Boolean operand.</summary>
    Ldc_Bool,
    /// <summary>Discards the top evaluation-stack value.</summary>
    Pop,
    /// <summary>Pushes an Int64 constant; requires a long operand.</summary>
    Ldc_I8,
    /// <summary>Converts Int32/Int64 to signed Int64, sign-extending Int32.</summary>
    Conv_I8,
    /// <summary>Converts Int32/Int64 to Int32, retaining the low 32 bits.</summary>
    Conv_I4,
    /// <summary>Negates Int32/Int64 with wrapping signed overflow.</summary>
    Neg,
    /// <summary>Complements every bit of Int32/Int64, preserving width.</summary>
    Not,
    /// <summary>Pushes a Unicode string literal; requires a string operand.</summary>
    Ldstr,
    /// <summary>Stores into a declared argument by zero-based index; requires an Int32 operand.</summary>
    Starg,
    /// <summary>Divides matching signed Int32/Int64 values, truncating toward zero; zero and minimum/-1 fault at execution.</summary>
    Div,
    /// <summary>Computes signed Int32/Int64 remainder; zero faults at execution, and minimum/-1 follows the target runtime.</summary>
    Rem,
    /// <summary>Bitwise AND of matching Int32/Int64 or Boolean operands.</summary>
    And,
    /// <summary>Bitwise OR of matching Int32/Int64 or Boolean operands.</summary>
    Or,
    /// <summary>Bitwise XOR of matching Int32/Int64 or Boolean operands.</summary>
    Xor,
    /// <summary>Shifts an Int32/Int64 value left by an Int32 count.</summary>
    Shl,
    /// <summary>Arithmetically shifts an Int32/Int64 value right by an Int32 count.</summary>
    Shr
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends an operand-free arithmetic, comparison, stack or return instruction.</summary>
    /// <param name="opCode">Add, Sub, Mul, Div, Rem, And, Or, Xor, Shl, Shr, Ceq, Clt, Cgt, Pop, Conv_I4, Conv_I8, Neg, Not or Ret.</param>
    /// <exception cref="ArgumentException">Unknown opcode or an opcode requiring an operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    /// <remarks>Stack and return-flow validation remains deferred until writing. Rejected emission does not change the body.</remarks>
    public void Emit(OpCode opCode)
        => Append(new(opCode switch {
            OpCode.Neg => "negate", OpCode.Not => "complement", OpCode.Conv_I8 => "convert64", OpCode.Conv_I4 => "convert32", OpCode.Pop => "pop", OpCode.Ceq => "equal", OpCode.Clt => "less", OpCode.Cgt => "greater",
            OpCode.Shl => "shift.left", OpCode.Shr => "shift.right", OpCode.And => "and", OpCode.Or => "or", OpCode.Xor => "xor", OpCode.Rem => "remainder", OpCode.Div => "divide", OpCode.Add => "add", OpCode.Sub => "subtract", OpCode.Mul => "multiply", OpCode.Ret => "return",
            _ => throw OperandError(opCode)
        }));

    /// <summary>Appends an Int32 constant, argument-index or local-index instruction.</summary>
    /// <param name="opCode">Ldc_I4, Ldarg, Starg, Ldloc or Stloc.</param>
    /// <param name="operand">Signed constant, or zero-based argument/local index validated when writing.</param>
    /// <exception cref="ArgumentException">Unknown opcode or opcode incompatible with an Int32 operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, int operand)
        => Append(new(opCode switch {
            OpCode.Ldc_I4 => "constant", OpCode.Ldarg => "argument", OpCode.Starg => "argument.store",
            OpCode.Ldloc => "local.load", OpCode.Stloc => "local.store", _ => throw OperandError(opCode)
        }, operand));

    /// <summary>Appends an exact signed Int64 constant.</summary>
    /// <param name="opCode">Ldc_I8; other opcodes reject.</param>
    /// <param name="operand">Constant value, including Int64 extrema.</param>
    /// <exception cref="ArgumentException">Incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, long operand)
    {
        if (opCode != OpCode.Ldc_I8) throw OperandError(opCode);
        Append(new("constant64", LongValue: operand));
    }

    /// <summary>Appends a string literal shared by CLI and native emission.</summary>
    /// <param name="opCode">Ldstr; other opcodes reject.</param>
    /// <param name="operand">Non-null valid Unicode text, at most 64 KiB in UTF-8; empty text is supported.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Incorrect opcode, invalid Unicode or oversized literal.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    /// <remarks>Rejects unpaired UTF-16 surrogates instead of substituting replacement characters.</remarks>
    public void Emit(OpCode opCode, string operand)
    {
        if (opCode != OpCode.Ldstr) throw OperandError(opCode);
        ValidateLiteral(operand);
        Append(new("string", Text: operand));
    }

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
