namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A method-owned symbolic branch destination, independent of encoded offsets.</summary>
public sealed class BranchLabel
{
    internal BranchLabel(MethodBuilder method, int index) { Method = method; Index = index; }
    /// <summary>Gets the owning method.</summary>
    public MethodBuilder Method { get; }
    internal int Index { get; }
}

public sealed partial class MethodBuilder
{
    private readonly List<BranchLabel> labels = [];
    /// <summary>Creates an unmarked branch destination. ClearBody retains label handles.</summary>
    /// <returns>A label owned by this method.</returns>
    /// <exception cref="InvalidDataException">4096-label limit exceeded.</exception>
    public BranchLabel DefineLabel()
    {
        if (labels.Count >= 4096) throw new InvalidDataException("label limit exceeded");
        var label = new BranchLabel(this, labels.Count); labels.Add(label); return label;
    }
    /// <summary>Marks a label at the current instruction position.</summary>
    /// <param name="label">An unmarked label owned by this method.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign or already marked label.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void MarkLabel(BranchLabel label)
    {
        CheckLabel(label);
        if (Instructions.Any(i => i.Op == "label" && i.Value == label.Index)) throw new ArgumentException("label already marked", nameof(label));
        Append(new("label", label.Index));
    }
    /// <summary>Appends a branch. Conditional branches consume a Boolean comparison result.</summary>
    /// <param name="opCode">Br, Brtrue or Brfalse.</param>
    /// <param name="label">Destination owned by this method; it must be marked before writing.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign label or incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, BranchLabel label)
    {
        CheckLabel(label);
        Append(new(opCode switch { OpCode.Br => "branch", OpCode.Brtrue => "branch.true", OpCode.Brfalse => "branch.false", _ => throw OperandError(opCode) }, label.Index));
    }
    private void CheckLabel(BranchLabel label)
    {
        ArgumentNullException.ThrowIfNull(label);
        if (!ReferenceEquals(label.Method, this)) throw new ArgumentException("label belongs to another method", nameof(label));
    }
    internal Dictionary<int, int> LabelPositions()
        => Instructions.Select((instruction, index) => (instruction, index)).Where(p => p.instruction.Op == "label")
            .ToDictionary(p => p.instruction.Value, p => p.index);

    private sealed record FlowState(PrimitiveType[] Stack, bool[] Assigned);

    internal void Validate()
    {
        if (Instructions.Count == 0) throw new InvalidDataException("empty body");
        var positions = LabelPositions();
        // Check reference bounds even in unreachable code.
        foreach (var instruction in Instructions)
        {
            if (instruction.Op is "branch" or "branch.true" or "branch.false" && !positions.ContainsKey(instruction.Value))
                throw new InvalidDataException("unmarked branch label");
            if (instruction.Op is "local.load" or "local.store" && (instruction.Value < 0 || instruction.Value >= locals.Count))
                throw new InvalidDataException("local outside declarations");
            if (instruction.Op == "argument" && (instruction.Value < 0 || instruction.Value >= ParameterCount))
                throw new InvalidDataException("argument outside signature");
        }
        MaxStack = 0;
        var states = new FlowState?[Instructions.Count];
        var work = new Queue<int>();
        states[0] = new([], new bool[locals.Count]); work.Enqueue(0);
        while (work.TryDequeue(out var index))
        {
            var state = states[index]!;
            var stack = state.Stack.ToList(); var assigned = (bool[])state.Assigned.Clone();
            var instruction = Instructions[index];
            // Preserve native primitive identity across stack operations and joins.
            void Pop(PrimitiveType type = PrimitiveType.Int32)
            {
                if (stack.Count == 0 || stack[^1] != type) throw new InvalidDataException("evaluation stack type mismatch or underflow");
                stack.RemoveAt(stack.Count - 1);
            }
            switch (instruction.Op)
            {
                case "label": case "console.line": break;
                case "pop":
                    if (stack.Count == 0) throw new InvalidDataException("evaluation stack underflow");
                    stack.RemoveAt(stack.Count - 1); break;
                case "constant64": stack.Add(PrimitiveType.Int64); break;
                case "convert32": case "convert64":
                    if (stack.Count == 0 || stack[^1] is not (PrimitiveType.Int32 or PrimitiveType.Int64))
                        throw new InvalidDataException("integer conversion requires Int32 or Int64");
                    stack[^1] = instruction.Op == "convert64" ? PrimitiveType.Int64 : PrimitiveType.Int32; break;
                case "constant": stack.Add(PrimitiveType.Int32); break;
                case "argument": stack.Add(Signature.ParameterTypes[instruction.Value]); break;
                case "boolean": stack.Add(PrimitiveType.Boolean); break;
                case "local.load":
                    if (!assigned[instruction.Value]) throw new InvalidDataException("local loaded before store on some path");
                    stack.Add(locals[instruction.Value].Type); break;
                case "local.store": Pop(locals[instruction.Value].Type); assigned[instruction.Value] = true; break;
                case "add": case "subtract": case "multiply": case "less": case "greater":
                    if (stack.Count == 0 || stack[^1] is not (PrimitiveType.Int32 or PrimitiveType.Int64))
                        throw new InvalidDataException("integer operands required");
                    var integerType = stack[^1]; Pop(integerType); Pop(integerType);
                    stack.Add(instruction.Op is "less" or "greater" ? PrimitiveType.Boolean : integerType); break;
                case "equal":
                    if (stack.Count == 0) throw new InvalidDataException("evaluation stack underflow");
                    var equalityType = stack[^1]; Pop(equalityType); Pop(equalityType); stack.Add(PrimitiveType.Boolean); break;
                case "call":
                    for (int i = instruction.Target!.ParameterCount - 1; i >= 0; i--) Pop(instruction.Target.Signature.ParameterTypes[i]);
                    if (instruction.Target.ReturnsValue) stack.Add(instruction.Target.Signature.ReturnType);
                    break;
                case "native.call":
                    if (!instruction.NativeTarget!.TryGetStaticInt32Signature(out var count)) throw new InvalidDataException("invalid native call");
                    for (int i = 0; i < count; i++) Pop();
                    stack.Add(PrimitiveType.Int32); break;
                case "branch.true": case "branch.false": Pop(PrimitiveType.Boolean); break;
                case "branch": break;
                case "return":
                    if (ReturnsValue) Pop(Signature.ReturnType);
                    if (stack.Count != 0) throw new InvalidDataException("invalid return stack");
                    continue;
                default: throw new InvalidDataException("unsupported instruction");
            }
            MaxStack = Math.Max(MaxStack, stack.Count);
            void Merge(int target)
            {
                if (target >= Instructions.Count) throw new InvalidDataException("reachable fallthrough outside method");
                var next = new FlowState(stack.ToArray(), (bool[])assigned.Clone());
                if (states[target] is { } previous)
                {
                    if (!previous.Stack.SequenceEqual(next.Stack)) throw new InvalidDataException("incompatible branch stack");
                    for (int i = 0; i < next.Assigned.Length; i++) next.Assigned[i] &= previous.Assigned[i];
                    if (previous.Assigned.SequenceEqual(next.Assigned)) return;
                }
                states[target] = next; work.Enqueue(target);
            }
            if (instruction.Op is "branch" or "branch.true" or "branch.false") Merge(positions[instruction.Value]);
            if (instruction.Op != "branch") Merge(index + 1);
        }
        for (int i = 0; i < states.Length; i++)
            if (states[i] is null && Instructions[i].Op != "label")
                throw new InvalidDataException("unreachable instruction");
    }
}
