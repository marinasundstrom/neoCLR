using Operation = NeoCLR.Metadata.Experimental.Model.MethodBuilder.Operation;

namespace NeoCLR.Metadata.Experimental.Model;

// Body authoring owns mutation and operand validation. The builder supplies only
// declaration context; stored instructions, locals and labels remain on the definition.
internal sealed class MethodILGenerator(MethodBuilder bodyBuilder) : IILGenerator
{
    private AssemblyBuilder Assembly => bodyBuilder.Assembly;
    private TypeBuilder? DeclaringType => bodyBuilder.DeclaringType;
    private MethodSignature Signature => bodyBuilder.Signature;
    private MethodDefinition Definition => bodyBuilder.Definition;
    private List<MethodBuilder.Operation> Instructions => Definition.Body.Instructions;
    private List<LocalDefinition> locals => Definition.Body.LocalStorage;
    private List<BranchLabel> labels => Definition.Body.LabelStorage;
    public IReadOnlyList<LocalDefinition> Locals => Definition.Body.Locals;
    private static bool IsReferenceSignature(SignatureType type) => MethodBuilder.IsReferenceSignature(type);
    public void LoadConstant(float value) => Emit(OpCode.Ldc_R4, value);
    public void LoadConstant(double value) => Emit(OpCode.Ldc_R8, value);
    public void Emit(OpCode opCode, float operand)
    {
        if (opCode != OpCode.Ldc_R4) throw OperandError(opCode);
        Append(new("constantSingle", Value: BitConverter.SingleToInt32Bits(operand)));
    }
    public void Emit(OpCode opCode, double operand)
    {
        if (opCode != OpCode.Ldc_R8) throw OperandError(opCode);
        Append(new("constantDouble", LongValue: BitConverter.DoubleToInt64Bits(operand)));
    }
    public void LoadConstant(int value) => Emit(OpCode.Ldc_I4, value);
    public void ConvertToEnum(SignatureType enumType) => ConvertEnum(enumType, "enum.from");
    public void ConvertFromEnum(SignatureType enumType) => ConvertEnum(enumType, "enum.to");
    private void ConvertEnum(SignatureType type, string operation)
    {
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly);
        if (!Assembly.IsEnumSignature(type)) throw new ArgumentException("requires an owned or explicitly imported Int32 enum", nameof(type));
        Append(new(operation, Type: type));
    }


    public void WriteConsoleLine(string text)
    {
        ValidateLiteral(text);
        Append(new("console.line", Text: text));
    }

    public void WriteConsoleLine() => Append(new("console.write"));

    private static void ValidateLiteral(string text)
    {
        ArgumentNullException.ThrowIfNull(text);
        try
        {
            if (new System.Text.UTF8Encoding(false, true).GetByteCount(text) > 65536)
                throw new ArgumentException("string literal exceeds 64 KiB", nameof(text));
        }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid Unicode", nameof(text), error); }
    }

    public void LoadArgumentAddress(int index) => Emit(OpCode.Ldarga, index);
    public void LoadArgument(int index) => Emit(OpCode.Ldarg, index);

    public void StoreArgument(int index) => Emit(OpCode.Starg, index);

    public void Add() => Emit(OpCode.Add);

    public void Subtract() => Emit(OpCode.Sub);

    public void Multiply() => Emit(OpCode.Mul);

    public void Divide() => Emit(OpCode.Div);

    public void Remainder() => Emit(OpCode.Rem);

    public void BitwiseAnd() => Emit(OpCode.And);

    public void BitwiseOr() => Emit(OpCode.Or);

    public void BitwiseXor() => Emit(OpCode.Xor);

    public void ShiftLeft() => Emit(OpCode.Shl);

    public void ShiftRight() => Emit(OpCode.Shr);

    public void Call(MethodBuilder target) => Emit(OpCode.Call, target);

    public void Call(ImportedMethodReference target) => Emit(OpCode.Call, target);

    public void Call(NativeFunctionDefinition target) => Emit(OpCode.Call, target);

    public void Return() => Emit(OpCode.Ret);

    public void ClearBody() => Definition.Body.ClearInstructions();

    private void Append(Operation operation)
    {
        if (Instructions.Count >= 4096) throw new InvalidDataException("method instruction limit exceeded");
        Instructions.Add(operation);
    }

    public void LoadFieldAddress(ConstructedFieldReference field) => Emit(OpCode.Ldflda, field);

    public void LoadField(ConstructedFieldReference field) => Emit(OpCode.Ldfld, field);

    public void StoreField(ConstructedFieldReference field) => Emit(OpCode.Stfld, field);

    public void Emit(OpCode opCode, ConstructedFieldReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        ((SignatureType)operand.DeclaringType).ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        if (opCode == OpCode.Ldflda && operand.Definition.IsReadOnly) throw new ArgumentException("readonly field addresses are unsupported", nameof(operand));
        Append(new(opCode switch { OpCode.Ldfld => "field.load", OpCode.Stfld => "field.store", OpCode.Ldflda => "field.address", _ => throw OperandError(opCode) }, Field: operand.Definition, ConstructedField: operand));
    }

    public void NewObject(ConstructedMethodReference constructor) => Emit(OpCode.Newobj, constructor);

    public void Call(ConstructedMethodReference method) => Emit(OpCode.Call, method);

    public void CallVirtual(ConstructedMethodReference method) => Emit(OpCode.Callvirt, method);

    public void Emit(OpCode opCode, ConstructedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (opCode != (operand.Definition.IsConstructor ? OpCode.Newobj : operand.Definition.IsAbstract ? OpCode.Callvirt : OpCode.Call)) throw new ArgumentException("constructor requires Newobj, interface contract Callvirt, other members Call");
        if (!ReferenceEquals(operand.Definition.Assembly, Assembly)) throw new ArgumentException("constructed calls require an owned definition");
        foreach (var type in operand.DeclaringTypeArguments.Concat(operand.MethodArguments))
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(operand.Definition.IsConstructor ? "new.constructed" : operand.Definition.IsAbstract ? "call.virtual.constructed" : "call.constructed", Target: operand.Definition, ConstructedTarget: operand));
    }

    public void BindFunction(SignatureType functionType, MethodBuilder target) => Emit(OpCode.BindFunction, new FunctionBinding(functionType, target));
    public void BindFunction(SignatureType functionType, ConstructedMethodReference target) => Emit(OpCode.BindFunction, new FunctionBinding(functionType, target));

    public void Emit(OpCode opCode, FunctionBinding operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (opCode != OpCode.BindFunction || !ReferenceEquals(operand.Target.Assembly, Assembly)) throw new ArgumentException("binding requires BindFunction and an owned target");
        operand.FunctionType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        if (operand.ConstructedTarget is { } reference)
            foreach (var argument in reference.DeclaringTypeArguments.Concat(reference.MethodArguments))
                argument.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new("function.bind", Target: operand.Target, Type: operand.FunctionType, ConstructedTarget: operand.ConstructedTarget));
    }

    public void InvokeFunction(SignatureType functionType) => Emit(OpCode.Callvirt, functionType);

    public void Call(GenericMethodInstance method) => Emit(OpCode.Call, method);

    public void Emit(OpCode opCode, GenericMethodInstance operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (!ReferenceEquals(operand.Definition.Assembly, Assembly)) throw new ArgumentException("generic calls require an owned definition", nameof(operand));
        foreach (var type in operand.TypeArguments) type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        operand.Definition.ValidateMethodArguments(operand.TypeArguments, bodyBuilder);
        Append(new("call.generic", Target: operand.Definition, GenericTarget: operand));
    }

    public void NewObject(ImportedMethodReference constructor) => Emit(OpCode.Newobj, constructor);

    public void NewObject(ImportedConstructedMethodReference constructor) => Emit(OpCode.Newobj, constructor);

    public void Call(ImportedConstructedMethodReference method) => Emit(OpCode.Call, method);

    public void CallVirtual(ImportedConstructedMethodReference method) => Emit(OpCode.Callvirt, method);

    public void CallVirtual(ImportedMethodReference method) => Emit(OpCode.Callvirt, method);

    public void Emit(OpCode opCode, ImportedConstructedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly) || opCode != (operand.Definition.IsConstructor ? OpCode.Newobj : operand.Definition.RequiresVirtualDispatch ? OpCode.Callvirt : OpCode.Call))
            throw new ArgumentException("incorrect imported owner or dispatch opcode");
        foreach (var type in operand.DeclaringType.TypeArguments.Concat(operand.MethodArguments))
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(operand.Definition.IsConstructor ? "new.constructed" : operand.Definition.RequiresVirtualDispatch ? "call.virtual.constructed" : "call.constructed", Target: operand.Definition.Target,
            ConstructedTarget: operand.Target, Type: operand.Definition.IsStatic ? null : (SignatureType)operand.DeclaringType));
    }

    public void LoadField(ImportedFieldReference field) => Emit(OpCode.Ldfld, field);

    public void StoreField(ImportedFieldReference field) => Emit(OpCode.Stfld, field);

    public void Emit(OpCode opCode, ImportedFieldReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.Owner, Assembly) || operand.DeclaringType.GenericArity != 0)
            throw new ArgumentException("foreign or unconstructed imported field", nameof(operand));
        Append(new(opCode switch { OpCode.Ldfld => "field.import.load", OpCode.Stfld => "field.import.store", _ => throw OperandError(opCode) }, ImportedField: operand));
    }

    public void LoadField(ImportedConstructedFieldReference field) => Emit(OpCode.Ldfld, field);
    public void StoreField(ImportedConstructedFieldReference field) => Emit(OpCode.Stfld, field);
    public void Emit(OpCode opCode, ImportedConstructedFieldReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly)) throw new ArgumentException("foreign imported field", nameof(operand));
        ((SignatureType)operand.DeclaringType).ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(opCode switch { OpCode.Ldfld => "field.import.load", OpCode.Stfld => "field.import.store", _ => throw OperandError(opCode) },
            ImportedField: operand.Definition, ImportedConstructedField: operand, Type: operand.DeclaringType));
    }

    public void Call(ImportedGenericMethodReference method) => Emit(OpCode.Call, method);

    public void Emit(OpCode opCode, ImportedGenericMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(operand));
        foreach (var type in operand.TypeArguments)
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new("call.generic", Target: operand.Target.Definition, GenericTarget: operand.Target));
    }

    public void CallVirtual(MethodBuilder target) => Emit(OpCode.Callvirt, target);

    public void Emit(OpCode opCode, SignatureType elementType)
    {
        ArgumentNullException.ThrowIfNull(elementType);
        if (opCode == OpCode.Isinst)
        {
            if (elementType.Primitive is PrimitiveType.Void || elementType.ByReferenceElement is not null || elementType.IsSelf || elementType.FunctionSignature is not null)
                throw new ArgumentException("Isinst requires a storage type or scoped generic parameter", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            if (!IsReferenceSignature(elementType)) _ = Assembly.CoreObjectType;
            Append(new("reference.test", Type: elementType)); return;
        }
        if (opCode == OpCode.Box)
        {
            if (elementType.Primitive is PrimitiveType.Void || elementType.ByReferenceElement is not null || elementType.IsSelf || elementType.FunctionSignature is not null)
                throw new ArgumentException("Box requires a storage value or scoped generic parameter", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            _ = Assembly.CoreObjectType;
            Append(new("object.box", Type: elementType)); return;
        }
        if (opCode == OpCode.UnboxAny)
        {
            if (elementType.Primitive is PrimitiveType.Void || elementType.ByReferenceElement is not null || elementType.IsSelf || elementType.FunctionSignature is not null)
                throw new ArgumentException("UnboxAny requires a storage type or scoped generic parameter", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            Append(new("object.unbox", Type: elementType)); return;
        }
        if (opCode == OpCode.Castclass)
        {
            if (!IsReferenceSignature(elementType)) throw new ArgumentException("Castclass requires a reference target", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            Append(new("reference.cast", Type: elementType)); return;
        }
        if (opCode == OpCode.Callvirt)
        {
            if (elementType.FunctionSignature is null) throw new ArgumentException("Callvirt requires a Function signature", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            Append(new("function.invoke", Type: elementType)); return;
        }
        if (opCode is OpCode.Initobj or OpCode.Ldobj or OpCode.Stobj)
        {
            if (elementType.Primitive == PrimitiveType.Void) throw new ArgumentException("addressed operation requires a non-Void type", nameof(elementType));
            elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
            Append(new(opCode switch { OpCode.Ldobj => "object.load", OpCode.Stobj => "object.store", _ => "local.initialize" }, Type: elementType));
            return;
        }
        _ = SignatureType.ArrayOf(elementType);
        elementType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(opCode switch
        {
            OpCode.Newarr => "array.new",
            OpCode.ReserveArray => "array.reserve",
            OpCode.Ldelem => "array.load",
            OpCode.Stelem => "array.store",
            _ => throw OperandError(opCode)
        }, Type: elementType));
    }

    public void IsInstance(SignatureType target) => Emit(OpCode.Isinst, target);

    public void IsNull() => Emit(OpCode.ReferenceIsNull);

    public void UnboxAny(SignatureType target) => Emit(OpCode.UnboxAny, target);

    public void Box(SignatureType type) => Emit(OpCode.Box, type);

    public void CastReference(SignatureType target) => Emit(OpCode.Castclass, target);

    public void NewArray(SignatureType elementType) => Emit(OpCode.Newarr, elementType);

    public void ReserveArray(SignatureType elementType) => Emit(OpCode.ReserveArray, elementType);

    public void LoadArrayElement(SignatureType elementType) => Emit(OpCode.Ldelem, elementType);

    public void StoreArrayElement(SignatureType elementType) => Emit(OpCode.Stelem, elementType);

    public void LoadArrayLength()
    {
        if (Instructions.Count > 4094) throw new InvalidDataException("instruction limit exceeded");
        Emit(OpCode.Ldlen); Emit(OpCode.Conv_I4);
    }

    public void Emit(OpCode opCode)
        => Append(new(opCode switch
        {
            OpCode.ReferenceIsNull => "reference.isnull",
            OpCode.Ldlen => "array.length",
            OpCode.Dup => "duplicate",
            OpCode.Neg => "negate",
            OpCode.Not => "complement",
            OpCode.Conv_R4 => "convertSingle",
            OpCode.Conv_R8 => "convertDouble",
            OpCode.Conv_I8 => "convert64",
            OpCode.Conv_I4 => "convert32",
            OpCode.Conv_U1 => "convertByte",
            OpCode.Conv_I1 => "convertSByte",
            OpCode.Conv_I2 => "convertInt16",
            OpCode.Conv_U2 => "convertUInt16",
            OpCode.Conv_U4 => "convertUInt32",
            OpCode.Conv_U8 => "convertUInt64",
            OpCode.Div_Un => "divide.unsigned",
            OpCode.Rem_Un => "remainder.unsigned",
            OpCode.Shr_Un => "shift.right.unsigned",
            OpCode.Conv_R_Un => "convertUnsignedDouble",

            OpCode.Pop => "pop",
            OpCode.Ceq => "equal",
            OpCode.Clt => "less",
            OpCode.Clt_Un => "less.unordered",
            OpCode.Cgt_Un => "greater.unordered",
            OpCode.Cgt => "greater",
            OpCode.Shl => "shift.left",
            OpCode.Shr => "shift.right",
            OpCode.And => "and",
            OpCode.Or => "or",
            OpCode.Xor => "xor",
            OpCode.Rem => "remainder",
            OpCode.Div => "divide",
            OpCode.Add => "add",
            OpCode.Sub => "subtract",
            OpCode.Mul => "multiply",
            OpCode.Ret => "return",
            _ => throw OperandError(opCode)
        }));

    public void Emit(OpCode opCode, int operand)
        => Append(new(opCode switch
        {
            OpCode.Ldc_I4 => "constant",
            OpCode.Ldarg => "argument",
            OpCode.Ldarga => "argument.address",
            OpCode.Starg => "argument.store",
            OpCode.Ldloca => "local.address",
            OpCode.Ldloc => "local.load",
            OpCode.Stloc => "local.store",
            _ => throw OperandError(opCode)
        }, operand));

    public void Emit(OpCode opCode, long operand)
    {
        if (opCode != OpCode.Ldc_I8) throw OperandError(opCode);
        Append(new("constant64", LongValue: operand));
    }

    public void Emit(OpCode opCode, string operand)
    {
        if (opCode is not (OpCode.Ldstr or OpCode.Fail)) throw OperandError(opCode);
        ValidateLiteral(operand);
        Append(new(opCode == OpCode.Fail ? "fail" : "string", Text: operand));
    }

    public void Emit(OpCode opCode, TypeBuilder receiverType, MethodBuilder target)
    {
        ArgumentNullException.ThrowIfNull(target);
        if (opCode != (target.IsStatic ? OpCode.Call : OpCode.Callvirt)) throw OperandError(opCode);
        CallConstrained(receiverType, target);
    }

    public void CallConstrained(TypeBuilder receiverType, MethodBuilder target)
    {
        ValidateConstrainedOperands(Assembly, receiverType, target);
        Append(new("call.constrained", Target: target, Type: receiverType, ConstrainedOwner: receiverType));
    }

    public void Emit(OpCode opCode, SignatureType implementingType, MethodBuilder target)
    {
        if (opCode != OpCode.Call) throw OperandError(opCode);
        CallConstrained(implementingType, target);
    }

    public void CallConstrained(SignatureType implementingType, MethodBuilder target)
    {
        ValidateOpenConstrainedOperands(bodyBuilder, implementingType, target);
        Append(new("call.constrained", Target: target, Type: implementingType));
    }

    public void Emit(OpCode opCode, SignatureType implementingType, ImportedMethodReference target)
    {
        if (opCode != OpCode.Call) throw OperandError(opCode);
        CallConstrained(implementingType, target);
    }
    public void CallConstrained(SignatureType implementingType, ImportedMethodReference target)
    {
        ValidateExternalConstrainedOperands(bodyBuilder, implementingType, target);
        Append(new("call.constrained", Target: target.Target, Type: implementingType, ConstrainedReference: target));
    }
    public void Emit(OpCode opCode, SignatureType implementingType, ImportedConstructedMethodReference target)
    {
        ArgumentNullException.ThrowIfNull(target);
        if (opCode != (target.Definition.IsStatic ? OpCode.Call : OpCode.Callvirt)) throw OperandError(opCode);
        CallConstrained(implementingType, target);
    }
    public void CallConstrained(SignatureType implementingType, ImportedConstructedMethodReference target)
    {
        ArgumentNullException.ThrowIfNull(target);
        ValidateExternalConstructedConstrainedOperands(bodyBuilder, implementingType, target);
        Append(new("call.constrained", Target: target.Definition.Target, Type: implementingType, ConstructedTarget: target.Target, ConstrainedConstructedReference: target));
    }
    internal static void ValidateExternalConstructedConstrainedOperands(MethodBuilder caller, SignatureType implementingType, ImportedConstructedMethodReference target)
    {
        ArgumentNullException.ThrowIfNull(implementingType);
        if (!ReferenceEquals(target.Definition.Owner, caller.Assembly) || !target.Definition.IsInterfaceMethod || target.MethodArguments.Count != 0 ||
            implementingType.MethodParameterIndex is not { } index || index >= caller.Signature.GenericParameterNames.Count ||
            !caller.InterfaceConstraints.Any(c => c.ParameterIndex == index && caller.Assembly.SatisfiesConstrainedBound(caller.Definition.ConstraintSignature(c.InterfaceType), target.DeclaringType, implementingType)))
            throw new ArgumentException("constructed constrained call requires a matching method interface bound");
        foreach (var argument in target.DeclaringType.TypeArguments) argument.ValidateOwner(caller.Assembly, caller.Signature.GenericParameterNames.Count, caller.DeclaringType?.GenericParameterNames.Count ?? 0);
        _ = caller.Assembly.ExternalInterfaceMethods(target.DeclaringType).ToArray();
    }

    internal static void ValidateExternalConstrainedOperands(MethodBuilder caller, SignatureType implementingType, ImportedMethodReference target)
    {
        ArgumentNullException.ThrowIfNull(implementingType); ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(target.Owner, caller.Assembly) || !target.IsStatic || !target.IsInterfaceMethod ||
            target.Signature.GenericParameterNames.Count != 0 || target.DeclaringReference is not { GenericArity: 0 } owner ||
            implementingType.MethodParameterIndex is not { } index || index >= caller.Signature.GenericParameterNames.Count ||
            !caller.InterfaceConstraints.Any(c => c.ParameterIndex == index && caller.Assembly.SatisfiesInterfaceBound(caller.Definition.ConstraintSignature(c.InterfaceType), owner)))
            throw new ArgumentException("external constrained call requires a bounded method parameter and completed static interface target");
        _ = caller.Assembly.ExternalInterfaceMethods(owner).ToArray();
    }

    internal static void ValidateOpenConstrainedOperands(MethodBuilder caller, SignatureType implementingType, MethodBuilder target)
    {
        ArgumentNullException.ThrowIfNull(implementingType);
        ArgumentNullException.ThrowIfNull(target);
        if (implementingType.MethodParameterIndex is not { } index || index >= caller.Signature.GenericParameterNames.Count ||
            !ReferenceEquals(target.Assembly, caller.Assembly) || !target.IsStatic || !target.IsAbstract ||
            target.Signature.GenericParameterNames.Count != 0 || target.DeclaringType is not { IsInterface: true, GenericParameterNames.Count: 0 } owner ||
            !caller.InterfaceConstraints.Any(c => c.ParameterIndex == index && caller.Definition.ConstraintSignature(c.InterfaceType).ClassType is { } bound && bound.ConformsTo(owner)))
            throw new ArgumentException("open constrained call requires a method parameter with an owned interface bound and a static nongeneric contract");
    }

    internal static SignatureType SubstituteConstrainedSelf(SignatureType type, SignatureType implementingType) => type.IsSelf ? implementingType
        : type.FunctionSignature is { } function ? function.Substitute(t => SubstituteConstrainedSelf(t, implementingType))
        : type.ByReferenceElement is { } byref ? SignatureType.ByReference(SubstituteConstrainedSelf(byref, implementingType))
        : type.ArrayElement is { } element ? SignatureType.ArrayOf(SubstituteConstrainedSelf(element, implementingType))
        : type.ImportedType is { } imported ? imported.Substitute(t => SubstituteConstrainedSelf(t, implementingType))
        : type.GenericInstance is { } generic ? generic.Definition.MakeGenericInstance(generic.TypeArguments.Select(t => SubstituteConstrainedSelf(t, implementingType)).ToArray()) : type;

    internal static void ValidateConstrainedOperands(AssemblyBuilder assembly, TypeBuilder receiverType, MethodBuilder target)
    {
        ArgumentNullException.ThrowIfNull(receiverType);
        ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(receiverType.Assembly, assembly) || receiverType.IsStatic || receiverType.IsInterface || (!target.IsStatic && !receiverType.IsValueType) ||
            receiverType.GenericParameterNames.Count != 0 || !ReferenceEquals(target.Assembly, assembly) ||
            !target.IsAbstract || target.Signature.GenericParameterNames.Count != 0 ||
            target.DeclaringType!.GenericParameterNames.Count != 0 || !receiverType.ConformsTo(target.DeclaringType))
            throw new ArgumentException("constrained call requires an owned nongeneric implementation and interface; instance receivers must be value types");
    }

    public void Fail(string message) => Emit(OpCode.Fail, message);

    public void Emit(OpCode opCode, MethodBuilder operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (operand.Signature.GenericParameterNames.Count != 0 || operand.DeclaringType?.GenericParameterNames.Count > 0) throw new ArgumentException("generic calls require an instantiation", nameof(operand));
        if (opCode == OpCode.Callvirt)
        {
            if (!operand.IsAbstract || !ReferenceEquals(operand.Assembly, Assembly))
                throw new ArgumentException("callvirt requires an owned interface method", nameof(operand));
            Append(new("call.virtual", Target: operand));
        }
        else if (opCode == OpCode.Newobj)
        {
            if (!operand.IsConstructor) throw new ArgumentException("newobj requires a constructor", nameof(operand));
            Append(new("new.object", Target: operand));
        }
        else
        {
            RequireCall(opCode);
            if (operand.IsConstructor) throw new ArgumentException("constructor chaining is unsupported", nameof(operand));
            Append(new("call", Target: operand));
        }
    }

    public void Emit(OpCode opCode, ImportedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (opCode != (operand.IsConstructor ? OpCode.Newobj : operand.RequiresVirtualDispatch ? OpCode.Callvirt : OpCode.Call)) throw new ArgumentException("wrong dispatch opcode", nameof(opCode));
        if (!ReferenceEquals(operand.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(operand));
        if (operand.Signature.GenericParameterNames.Count != 0 || operand.Target.DeclaringType?.GenericParameterNames.Count > 0) throw new ArgumentException("generic import must be instantiated", nameof(operand));
        Append(new(operand.IsConstructor ? "new.object" : operand.RequiresVirtualDispatch ? "call.virtual" : "call", Target: operand.Target, Type: operand.IsStatic ? null : operand.Target.NativeImportPrimitiveOwner is { } primitive ? (SignatureType)primitive : (SignatureType)operand.DeclaringReference!));
    }

    public void Emit(OpCode opCode, NativeFunctionDefinition operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (operand.Library.ModuleName != "System" || !operand.TryGetStaticInt32Signature(out _))
            throw new InvalidDataException("native call requires a static Int32 System function");
        Append(new("native.call", NativeTarget: operand));
    }

    public void Emit(OpCode opCode, bool operand)
    {
        if (opCode != OpCode.Ldc_Bool) throw OperandError(opCode);
        Append(new("boolean", operand ? 1 : 0));
    }

    public void Emit(OpCode opCode, FieldBuilder operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.DeclaringType.Assembly, Assembly)) throw new ArgumentException("field belongs to another output", nameof(operand));
        if (operand.DeclaringType.GenericParameterNames.Count > 0 && !ReferenceEquals(DeclaringType, operand.DeclaringType))
            throw new ArgumentException("generic definition fields require the declaring type scope");
        if (opCode == OpCode.Ldflda && operand.IsReadOnly) throw new ArgumentException("readonly field addresses are unsupported", nameof(operand));
        Append(new(opCode switch { OpCode.Ldfld => "field.load", OpCode.Stfld => "field.store", OpCode.Ldflda => "field.address", _ => throw OperandError(opCode) }, Field: operand));
    }

    public void Duplicate() => Emit(OpCode.Dup);

    public void NewObject(MethodBuilder constructor) => Emit(OpCode.Newobj, constructor);

    public void LoadFieldAddress(FieldBuilder field) => Emit(OpCode.Ldflda, field);

    public void LoadField(FieldBuilder field) => Emit(OpCode.Ldfld, field);

    public void StoreField(FieldBuilder field) => Emit(OpCode.Stfld, field);

    private static ArgumentException OperandError(OpCode opCode)
        => new($"Unsupported opcode or operand kind: {opCode}", nameof(opCode));

    private static void RequireCall(OpCode opCode)
    {
        if (opCode != OpCode.Call) throw OperandError(opCode);
    }

    public BranchLabel DefineLabel()
    {
        if (labels.Count >= 4096) throw new InvalidDataException("label limit exceeded");
        var label = new BranchLabel(bodyBuilder, labels.Count); labels.Add(label); return label;
    }

    public void MarkLabel(BranchLabel label)
    {
        CheckLabel(label);
        if (Instructions.Any(i => i.Op == "label" && i.Value == label.Index)) throw new ArgumentException("label already marked", nameof(label));
        Append(new("label", label.Index));
    }

    public void Emit(OpCode opCode, BranchLabel label)
    {
        CheckLabel(label);
        Append(new(opCode switch { OpCode.Br => "branch", OpCode.Brtrue => "branch.true", OpCode.Brfalse => "branch.false", _ => throw OperandError(opCode) }, label.Index));
    }

    private void CheckLabel(BranchLabel label)
    {
        ArgumentNullException.ThrowIfNull(label);
        if (!ReferenceEquals(label.Method, bodyBuilder)) throw new ArgumentException("label belongs to another method", nameof(label));
    }

    public void LoadLocalAddress(LocalDefinition local) => Emit(OpCode.Ldloca, local);

    public void InitializeObject(SignatureType type) => Emit(OpCode.Initobj, type);

    public void LoadDefault(SignatureType type)
    {
        if (Instructions.Count > 4093) throw new InvalidDataException("instruction limit exceeded");
        var local = DeclareLocal(type);
        LoadLocalAddress(local); InitializeObject(type); LoadLocal(local);
    }

    public LocalDefinition DeclareInt32Local() => DeclareLocal(PrimitiveType.Int32);

    public LocalDefinition DeclareLocal(PrimitiveType type)
    {
        if (!Enum.IsDefined(type) || type == PrimitiveType.Void) throw new ArgumentException("unsupported local type", nameof(type));
        if (locals.Count >= 256) throw new InvalidDataException("local limit exceeded");
        var local = new LocalDefinition(bodyBuilder, locals.Count, type); locals.Add(local); return local;
    }

    public LocalDefinition DeclareLocal(TypeBuilder type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.IsStatic || !ReferenceEquals(type.Assembly, Assembly)) throw new ArgumentException("local requires an owned root class", nameof(type));
        if (locals.Count >= 256) throw new InvalidDataException("local limit exceeded");
        var local = new LocalDefinition(bodyBuilder, locals.Count, type); locals.Add(local); return local;
    }

    public LocalDefinition DeclareLocal(SignatureType type)
    {
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        if (type.Primitive == PrimitiveType.Void) throw new ArgumentException("local cannot be Void", nameof(type));
        if (locals.Count >= 256) throw new InvalidDataException("local limit exceeded");
        var local = new LocalDefinition(bodyBuilder, locals.Count, type); locals.Add(local); return local;
    }

    public void LoadLocal(LocalDefinition local) => Emit(OpCode.Ldloc, local);

    public void StoreLocal(LocalDefinition local) => Emit(OpCode.Stloc, local);

    public void Emit(OpCode opCode, LocalDefinition local)
    {
        ArgumentNullException.ThrowIfNull(local);
        if (opCode is not (OpCode.Ldloc or OpCode.Ldloca or OpCode.Stloc) || !ReferenceEquals(local.Method, bodyBuilder))
            throw new ArgumentException("invalid local operand", nameof(local));
        Emit(opCode, local.Index);
    }

    public void LoadObject(SignatureType type) => Emit(OpCode.Ldobj, type);

    public void StoreObject(SignatureType type) => Emit(OpCode.Stobj, type);
}
