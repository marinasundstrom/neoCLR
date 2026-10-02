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
    private List<BranchLabel> labels => Definition.Body.LabelStorage;
    /// <summary>Creates an unmarked branch destination. ClearBody retains label handles.</summary>
    /// <returns>A label owned by this method.</returns>
    /// <exception cref="InvalidDataException">4096-label limit exceeded.</exception>
    public BranchLabel DefineLabel() => GetILGenerator().DefineLabel();
    /// <summary>Marks a label at the current instruction position.</summary>
    /// <param name="label">An unmarked label owned by this method.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign or already marked label.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void MarkLabel(BranchLabel label) => GetILGenerator().MarkLabel(label);
    /// <summary>Appends a branch. Conditional branches consume a Boolean comparison result.</summary>
    /// <param name="opCode">Br, Brtrue or Brfalse.</param>
    /// <param name="label">Destination owned by this method; it must be marked before writing.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign label or incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, BranchLabel label) => GetILGenerator().Emit(opCode, label);

    internal Dictionary<int, int> LabelPositions()
        => Instructions.Select((instruction, index) => (instruction, index)).Where(p => p.instruction.Op == "label")
            .ToDictionary(p => p.instruction.Value, p => p.index);

    private readonly record struct BodyValueType(PrimitiveType Primitive, TypeBuilder? Class = null, SignatureType? ArrayElement = null, bool NativeLength = false, int? MethodParameter = null, int? AddressedLocal = null, int? TypeParameter = null, GenericTypeInstance? GenericInstance = null, ImportedTypeReference? ImportedType = null, SignatureType? ByReferenceElement = null, int? AddressedParameter = null, bool ConstructionReceiver = false, FunctionSignature? Function = null)
    {
        internal static BodyValueType Receiver(TypeBuilder owner) => owner.IsValueType ? SignatureType.ByReference(owner.OpenSignature) : owner.OpenSignature;
        public static implicit operator BodyValueType(PrimitiveType type) => new(type);
        public static implicit operator BodyValueType(SignatureType type) => type.FunctionSignature is { } function ? new(PrimitiveType.Void, Function: function) : type.ByReferenceElement is { } target ? new(PrimitiveType.Void, ByReferenceElement: target) : type.ImportedType is { } imported ? new(PrimitiveType.Void, ImportedType: imported) : type.GenericInstance is { } instance ? new(PrimitiveType.Void, GenericInstance: instance) : type.TypeParameterIndex is { } ordinal ? new(PrimitiveType.Void, TypeParameter: ordinal) : type.MethodParameterIndex is { } index ? new(PrimitiveType.Void, MethodParameter: index) : type.ArrayElement is { } element ? new(PrimitiveType.Void, ArrayElement: element) : type.ClassType is { } c ? new(PrimitiveType.Void, c) : new(type.Primitive!.Value);
    }
    private BodyValueType ArgumentType(int index)
    {
        if (!IsStatic && index == 0) return BodyValueType.Receiver(DeclaringType!) with { ConstructionReceiver = IsConstructor && DeclaringType!.IsValueType };
        int parameter = index - (IsStatic ? 0 : 1);
        BodyValueType type = Signature.ParameterTypes[parameter];
        return type.ByReferenceElement is null ? type : type with { AddressedParameter = parameter };
    }
    private static BodyValueType LocalType(LocalDefinition local) => local.SignatureType;
    private sealed record FlowState(BodyValueType[] Stack, bool[] Assigned);

    internal void Validate()
    {
        if (Instructions.Count == 0) throw new InvalidDataException("empty body");
        var positions = LabelPositions();
        // Check reference bounds even in unreachable code.
        foreach (var instruction in Instructions)
        {
            if (instruction.Op is "branch" or "branch.true" or "branch.false" && !positions.ContainsKey(instruction.Value))
                throw new InvalidDataException("unmarked branch label");
            if (instruction.Op is "local.load" or "local.store" or "local.address" && (instruction.Value < 0 || instruction.Value >= locals.Count))
                throw new InvalidDataException("local outside declarations");
            if (instruction.Op is "argument" or "argument.store" && (instruction.Value < 0 || instruction.Value >= ArgumentCount))
                throw new InvalidDataException("argument outside signature");
        }
        if (!IsStatic && Instructions.Any(i => i.Op == "argument.store" && i.Value == 0))
            throw new InvalidDataException("receiver stores are unsupported");
        MaxStack = 0;
        var states = new FlowState?[Instructions.Count];
        var work = new Queue<int>();
        var initiallyAssigned = new bool[locals.Count + ParameterCount + (IsConstructor && DeclaringType!.IsValueType ? DeclaringType.Fields.Count : 0)];
        for (int i = 0; i < ParameterCount; i++) initiallyAssigned[locals.Count + i] = !Signature.OutParameters.Contains(i);
        states[0] = new([], initiallyAssigned); work.Enqueue(0);
        while (work.TryDequeue(out var index))
        {
            var state = states[index]!;
            var stack = state.Stack.ToList(); var assigned = (bool[])state.Assigned.Clone();
            var instruction = Instructions[index];
            // Preserve native primitive identity across stack operations and joins.
            void Pop(BodyValueType type, bool output = false)
            {
                if (stack.Count > 0 && stack[^1].ConstructionReceiver)
                    throw new InvalidDataException("construction receiver cannot escape or be used for ordinary access");
                if (type.ByReferenceElement is { } target && stack.Count > 0)
                {
                    var referenceValue = stack[^1];
                    int? slot = referenceValue.AddressedLocal ?? (referenceValue.AddressedParameter is { } parameter ? locals.Count + parameter : null);
                    var actualTarget = referenceValue.AddressedLocal is { } local ? locals[local].SignatureType : referenceValue.ByReferenceElement;
                    if (actualTarget != target || slot is { } index && !output && !assigned[index])
                        throw new InvalidDataException("ref call requires initialized storage of the exact type");
                    stack.RemoveAt(stack.Count - 1);
                    return;
                }
                var constructedConformance = stack.Count > 0 && stack[^1].GenericInstance is { } instance &&
                    (type.Class is { IsInterface: true } interfaceType && instance.ConformsTo(interfaceType) ||
                     type.GenericInstance is { Definition.IsInterface: true } interfaceInstance && instance.ConformsTo(interfaceInstance));
                var importedConformance = stack.Count > 0 && stack[^1].ImportedType is { } importedActual && type.ImportedType is { } importedTarget &&
                    Assembly.HasNativeInterfaceConversion(importedActual, importedTarget);
                if (stack.Count == 0 || stack[^1] != type && !importedConformance && !constructedConformance && !(type.Class is { IsInterface: true } contract && stack[^1].Class is { } actual && actual.ConformsTo(contract)) && !(type.GenericInstance is { Definition.IsInterface: true } constructed && stack[^1].Class is { } concrete && concrete.ConformsTo(constructed))) throw new InvalidDataException("evaluation stack type mismatch or underflow");
                stack.RemoveAt(stack.Count - 1);
            }
            switch (instruction.Op)
            {
                case "label": case "console.line": break;
                case "duplicate":
                    if (stack.Count == 0) throw new InvalidDataException("evaluation stack underflow");
                    stack.Add(stack[^1]); break;
                case "field.import.load":
                case "field.import.store":
                    var importedField = instruction.ImportedField!;
                    if (instruction.Op == "field.import.store")
                    {
                        if (importedField.IsReadOnly) throw new InvalidDataException("cannot store an external readonly field");
                        Pop(instruction.ImportedConstructedField?.FieldType ?? importedField.FieldType);
                    }
                    Pop(instruction.Type ?? (SignatureType)importedField.DeclaringType);
                    if (instruction.Op == "field.import.load") stack.Add(instruction.ImportedConstructedField?.FieldType ?? importedField.FieldType);
                    break;
                case "field.load":
                case "field.store":
                    if (instruction.Op == "field.store")
                    {
                        if (instruction.Field!.IsReadOnly && (!IsConstructor || !ReferenceEquals(DeclaringType, instruction.Field.DeclaringType)))
                            throw new InvalidDataException("readonly field requires its declaring constructor");
                        Pop(instruction.ConstructedField?.FieldType ?? instruction.Field.FieldType);
                    }
                    SignatureType fieldOwner = instruction.ConstructedField is { } fieldReference
                        ? fieldReference.DeclaringType : instruction.Field!.DeclaringType.OpenSignature;
                    if (stack.Count > 0 && stack[^1].ConstructionReceiver)
                    {
                        if (!ReferenceEquals(instruction.Field!.DeclaringType, DeclaringType)) throw new InvalidDataException("constructor requires an owned field");
                        var fieldSlot = locals.Count + ParameterCount + instruction.Field.Index;
                        if (instruction.Op == "field.load" && !assigned[fieldSlot]) throw new InvalidDataException("constructor field read before assignment");
                        if (instruction.Op == "field.store") assigned[fieldSlot] = true;
                        stack.RemoveAt(stack.Count - 1);
                    }
                    else if (instruction.Field!.DeclaringType.IsValueType && stack.Count > 0 && stack[^1].AddressedLocal is { } receiverLocal)
                    {
                        if (!assigned[receiverLocal] || locals[receiverLocal].SignatureType != fieldOwner)
                            throw new InvalidDataException("value-type field receiver requires an initialized local of the exact owner type");
                        stack.RemoveAt(stack.Count - 1);
                    }
                    else if (instruction.Field.DeclaringType.IsValueType && stack.Count > 0 && stack[^1].ByReferenceElement == fieldOwner)
                    {
                        if (stack[^1].AddressedParameter is { } parameter && !assigned[locals.Count + parameter])
                            throw new InvalidDataException("value field receiver parameter is uninitialized");
                        stack.RemoveAt(stack.Count - 1);
                    }
                    else
                    {
                        if (instruction.Field.DeclaringType.IsValueType && instruction.Op == "field.store")
                            throw new InvalidDataException("value-type field store requires an addressed local receiver");
                        Pop(fieldOwner);
                    }
                    if (instruction.Op == "field.load") stack.Add(instruction.ConstructedField?.FieldType ?? instruction.Field!.FieldType);
                    break;
                case "array.new":
                case "array.reserve":
                    Pop(PrimitiveType.Int32); stack.Add(SignatureType.ArrayOf(instruction.Type!)); break;
                case "array.load":
                case "array.store":
                    if (instruction.Op == "array.store") Pop(instruction.Type!);
                    Pop(PrimitiveType.Int32); Pop(SignatureType.ArrayOf(instruction.Type!));
                    if (instruction.Op == "array.load") stack.Add(instruction.Type!);
                    break;
                case "array.length":
                    if (stack.Count == 0 || stack[^1].ArrayElement is null) throw new InvalidDataException("array length requires a vector");
                    stack[^1] = new(PrimitiveType.Void, NativeLength: true); break;
                case "new.constructed":
                    var constructor = instruction.ConstructedTarget!;
                    for (int i = constructor.Signature.ParameterTypes.Count - 1; i >= 0; i--) Pop(constructor.Signature.ParameterTypes[i]);
                    stack.Add(instruction.Type ?? (SignatureType)constructor.Definition.DeclaringType!.MakeGenericInstance(constructor.DeclaringTypeArguments.ToArray())); break;
                case "new.object":
                    for (int i = instruction.Target!.ParameterCount - 1; i >= 0; i--) Pop(instruction.Target.Signature.ParameterTypes[i]);
                    stack.Add(instruction.Type ?? instruction.Target.DeclaringType!.OpenSignature); break;
                case "pop":
                    if (stack.Count == 0) throw new InvalidDataException("evaluation stack underflow");
                    stack.RemoveAt(stack.Count - 1); break;
                case "negate":
                case "complement":
                    if (stack.Count == 0 || stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64))
                        throw new InvalidDataException("unary integer operation requires Int32 or Int64");
                    break;
                case "string": stack.Add(PrimitiveType.String); break;
                case "console.write": Pop(PrimitiveType.String); break;
                case "constant64": stack.Add(PrimitiveType.Int64); break;
                case "convert32":
                case "convert64":
                    if (stack.Count == 0 || (stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64) && !(instruction.Op == "convert32" && stack[^1].NativeLength)))
                        throw new InvalidDataException("integer conversion requires Int32/Int64 or array length for conv.i4");
                    stack[^1] = instruction.Op == "convert64" ? PrimitiveType.Int64 : PrimitiveType.Int32; break;
                case "constant": stack.Add(PrimitiveType.Int32); break;
                case "argument.store":
                    if (ArgumentType(instruction.Value).ByReferenceElement is not null)
                        throw new InvalidDataException("managed-reference argument rebinding is unsupported");
                    Pop(ArgumentType(instruction.Value)); break;
                case "argument": stack.Add(ArgumentType(instruction.Value)); break;
                case "boolean": stack.Add(PrimitiveType.Boolean); break;
                case "local.address": stack.Add(new BodyValueType(PrimitiveType.Void, AddressedLocal: instruction.Value)); break;
                case "object.load":
                case "object.store":
                    if (instruction.Op == "object.store") Pop(instruction.Type!);
                    if (stack.Count == 0) throw new InvalidDataException("object operation requires a managed reference");
                    var address = stack[^1];
                    if (address.ConstructionReceiver) throw new InvalidDataException("whole construction receiver access unsupported");
                    if (address.AddressedLocal is { } addressed)
                    {
                        if (locals[addressed].SignatureType != instruction.Type)
                            throw new InvalidDataException("object operation address type mismatch");
                        if (instruction.Op == "object.load" && !assigned[addressed])
                            throw new InvalidDataException("indirect local loaded before store on some path");
                        if (instruction.Op == "object.store") assigned[addressed] = true;
                    }
                    else
                    {
                        if (address.ByReferenceElement is null || address.ByReferenceElement != instruction.Type)
                            throw new InvalidDataException("object operation requires an exact managed-reference target");
                        if (address.AddressedParameter is { } parameter)
                        {
                            if (instruction.Op == "object.load" && !assigned[locals.Count + parameter])
                                throw new InvalidDataException("out parameter read before assignment");
                            if (instruction.Op == "object.store") assigned[locals.Count + parameter] = true;
                        }
                    }
                    stack.RemoveAt(stack.Count - 1);
                    if (instruction.Op == "object.load") stack.Add(instruction.Type!);
                    break;
                case "local.initialize":
                    if (stack.Count == 0 || stack[^1].AddressedLocal is not { } initialized || locals[initialized].SignatureType != instruction.Type)
                        throw new InvalidDataException("initobj requires an owned local address of the exact type");
                    stack.RemoveAt(stack.Count - 1); assigned[initialized] = true; break;
                case "local.load":
                    if (!assigned[instruction.Value]) throw new InvalidDataException("local loaded before store on some path");
                    stack.Add(LocalType(locals[instruction.Value])); break;
                case "local.store": Pop(LocalType(locals[instruction.Value])); assigned[instruction.Value] = true; break;
                case "shift.left":
                case "shift.right":
                    Pop(PrimitiveType.Int32);
                    if (stack.Count == 0 || stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64))
                        throw new InvalidDataException("shift requires an Int32/Int64 value and Int32 count");
                    break;
                case "and":
                case "or":
                case "xor":
                    if (stack.Count == 0 || stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64 or PrimitiveType.Boolean))
                        throw new InvalidDataException("bitwise operands require matching integer or Boolean types");
                    var bitwiseType = stack[^1]; Pop(bitwiseType); Pop(bitwiseType); stack.Add(bitwiseType); break;
                case "add":
                case "subtract":
                case "multiply":
                case "divide":
                case "remainder":
                case "less":
                case "greater":
                    if (stack.Count == 0 || stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64))
                        throw new InvalidDataException("integer operands required");
                    var integerType = stack[^1]; Pop(integerType); Pop(integerType);
                    stack.Add(instruction.Op is "less" or "greater" ? PrimitiveType.Boolean : integerType); break;
                case "equal":
                    if (stack.Count == 0 || stack[^1].Primitive is not (PrimitiveType.Int32 or PrimitiveType.Int64 or PrimitiveType.Boolean))
                        throw new InvalidDataException("equality requires numeric or Boolean operands");
                    var equalityType = stack[^1]; Pop(equalityType); Pop(equalityType); stack.Add(PrimitiveType.Boolean); break;
                case "reference.cast":
                    if (stack.Count == 0 || stack[^1] is not { ConstructionReceiver: false, ByReferenceElement: null, AddressedLocal: null, AddressedParameter: null } value ||
                        !(value.ArrayElement is not null || value.Class is { IsValueType: false, IsStatic: false } || value.GenericInstance?.Definition is { IsValueType: false, IsStatic: false } || value.ImportedType is { IsValueType: false }))
                        throw new InvalidDataException("reference cast requires a reference value");
                    stack.RemoveAt(stack.Count - 1); stack.Add(instruction.Type!); break;
                case "function.bind":
                    stack.Add(instruction.Type!); MaxStack = Math.Max(MaxStack, stack.Count + 1); break;
                case "function.invoke":
                    var function = instruction.Type!.FunctionSignature!;
                    for (int i = function.ParameterTypes.Count - 1; i >= 0; i--) Pop(function.ParameterTypes[i]);
                    Pop(instruction.Type);
                    if (!function.NoResult) stack.Add(function.ReturnType);
                    break;
                case "call.virtual.constructed":
                case "call.virtual":
                case "call":
                case "call.constructed":
                case "call.generic":
                    var callSignature = instruction.ConstructedTarget?.Signature ?? instruction.GenericTarget?.Signature ?? instruction.Target!.Signature;
                    var outputs = new List<int>();
                    for (int i = instruction.Target!.ParameterCount - 1; i >= 0; i--)
                    {
                        bool output = callSignature.OutParameters.Contains(i);
                        if (output && stack.Count > 0)
                        {
                            if (stack[^1].AddressedLocal is { } slot) outputs.Add(slot);
                            if (stack[^1].AddressedParameter is { } parameter) outputs.Add(locals.Count + parameter);
                        }
                        Pop(callSignature.ParameterTypes[i], output);
                    }
                    if (!instruction.Target.IsStatic)
                    {
                        var receiver = instruction.Type ?? (instruction.ConstructedTarget is { } reference ? (SignatureType)reference.Definition.DeclaringType!.MakeGenericInstance(reference.DeclaringTypeArguments.ToArray()) : instruction.Target.DeclaringType!.OpenSignature);
                        Pop(instruction.Target.DeclaringType!.IsValueType ? SignatureType.ByReference(receiver) : receiver);
                    }
                    // Receiver and ref input preconditions precede all output assignments.
                    foreach (var slot in outputs) assigned[slot] = true;
                    if (instruction.Target.ReturnsValue) stack.Add(callSignature.ReturnType);
                    break;
                case "native.call":
                    if (!instruction.NativeTarget!.TryGetStaticInt32Signature(out var count)) throw new InvalidDataException("invalid native call");
                    for (int i = 0; i < count; i++) Pop(PrimitiveType.Int32);
                    stack.Add(PrimitiveType.Int32); break;
                case "branch.true": case "branch.false": Pop(PrimitiveType.Boolean); break;
                case "branch": break;
                case "fail":
                    if (stack.Count != 0) throw new InvalidDataException("terminal failure requires an empty stack");
                    MaxStack = Math.Max(MaxStack, 1); // CLI diagnostic/exception construction.
                    continue;
                case "return":
                    if (IsConstructor && DeclaringType!.IsValueType && assigned.Skip(locals.Count + ParameterCount).Any(value => !value))
                        throw new InvalidDataException("value constructor must assign every field on every normal return");
                    if (Signature.OutParameters.Any(i => !assigned[locals.Count + i]))
                        throw new InvalidDataException("out parameter must be assigned on every normal return");
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
