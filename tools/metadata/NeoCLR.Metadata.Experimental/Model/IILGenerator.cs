namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Method-body authoring contract for the metadata library, independent of compiler backend interfaces.</summary>
/// <remarks>Appends to the owning authored definition. Not thread-safe. Operand ownership, opcode support,
/// stack validation and target-specific limits match the existing MethodBuilder operations.
/// This does not edit loaded opaque bodies or provide instruction insertion/reordering.</remarks>
public interface IILGenerator
{
    /// <summary>Gets declared locals in slot order. ClearBody retains them.</summary>
    IReadOnlyList<LocalDefinition> Locals { get; }
    /// <summary>Appends an Int32 constant.</summary>
    /// <param name="value">Constant value.</param>
    void LoadConstant(int value);
    /// <summary>Appends a native System.Console.WriteLine call with a constant UTF-8 string.</summary>
    /// <param name="text">Unicode text, at most 64 KiB when UTF-8 encoded.</param>
    /// <exception cref="ArgumentNullException">Text is null.</exception>
    /// <exception cref="ArgumentException">Invalid Unicode or text exceeds the limit.</exception>
    /// <remarks>Native emission only; ordinary CLI Write rejects this operation. Does not alter the surrounding primitive stack.</remarks>
    void WriteConsoleLine(string text);
    /// <summary>Consumes a String stack value and writes it through native System.Console.WriteLine.</summary>
    /// <remarks>Native-only bootstrap; discards bundled System's inhabited Void result. Ordinary CLI output rejects this operation.</remarks>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or stack mismatch when writing.</exception>
    void WriteConsoleLine();
    /// <summary>Appends a parameter load; bounds are checked at Write.</summary>
    /// <param name="index">Argument slot index; instance receiver is zero and declared parameters start at one.</param>
    void LoadArgument(int index);
    /// <summary>Stores a value into a by-value argument slot in this invocation.</summary>
    /// <param name="index">Argument slot index; instance declared parameters start at one. Bounds and exact type are checked when writing.</param>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or invalid index/stack type when writing.</exception>
    /// <remarks>Does not update caller storage. Receiver stores and by-reference parameters are unsupported.</remarks>
    void StoreArgument(int index);
    /// <summary>Appends matching-width Int32/Int64 addition.</summary>
    void Add();
    /// <summary>Appends matching-width Int32/Int64 subtraction.</summary>
    void Subtract();
    /// <summary>Appends matching-width Int32/Int64 multiplication.</summary>
    void Multiply();
    /// <summary>Appends matching-width signed Int32/Int64 division, truncating toward zero.</summary>
    /// <remarks>Zero and minimum-value divided by -1 fault at execution, not when writing.</remarks>
    void Divide();
    /// <summary>Appends matching-width signed Int32/Int64 remainder, with the dividend's sign.</summary>
    /// <remarks>Zero faults at execution, not when writing. Native minimum/-1 faults; CLI follows the host CLR edge behavior.</remarks>
    void Remainder();
    /// <summary>Appends bitwise AND of matching Int32/Int64 or Boolean operands.</summary>
    void BitwiseAnd();
    /// <summary>Appends bitwise OR of matching Int32/Int64 or Boolean operands.</summary>
    void BitwiseOr();
    /// <summary>Appends bitwise XOR of matching Int32/Int64 or Boolean operands.</summary>
    void BitwiseXor();
    /// <summary>Appends an Int32/Int64 left shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    void ShiftLeft();
    /// <summary>Appends a sign-extending Int32/Int64 right shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    void ShiftRight();
    /// <summary>Appends a call; foreign methods are imported during Write.</summary>
    /// <param name="target">Local or external builder method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    void Call(MethodBuilder target);
    /// <summary>Appends a call to an imported read-only method contract.</summary>
    /// <param name="target">Reference imported by this method's assembly builder.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Foreign reference, open generic definition, or a contract requiring Callvirt.</exception>
    void Call(ImportedMethodReference target);
    /// <summary>Calls a static Int32 function selected from an explicitly loaded native System inventory.</summary>
    /// <param name="target">Owned System function whose parameters and result are Int32.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="InvalidDataException">Module is not System or the callable signature is unsupported.</exception>
    /// <remarks>Native-only bootstrap. The host must supply the matching System assembly to neoCLR;
    /// no assembly revision or image digest is encoded. Ordinary CLI output rejects this operation.</remarks>
    void Call(NativeFunctionDefinition target);
    /// <summary>Appends return with the declared stack shape; no values may remain afterward.</summary>
    void Return();
    /// <summary>Clears instructions for editing before another Write; local declarations and handles are retained.</summary>
    void ClearBody();
    /// <summary>Loads a field from an exactly matching constructed receiver.</summary>
    void LoadField(ConstructedFieldReference field);
    /// <summary>Stores a field on an exactly matching constructed receiver.</summary>
    void StoreField(ConstructedFieldReference field);
    /// <summary>Appends Ldfld or Stfld with a constructed field reference.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">Owned reference, valid in the caller's parameter scope.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign definition or out-of-scope argument.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; receiver, value and readonly checks run on write.</exception>
    void Emit(OpCode opCode, ConstructedFieldReference operand);
    /// <summary>Allocates and invokes a constructor on a constructed generic class.</summary>
    /// <param name="constructor">Owned constructed constructor reference.</param>
    /// <exception cref="ArgumentException">Not a constructor, foreign owner or invalid caller scope.</exception>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void NewObject(ConstructedMethodReference constructor);
    /// <summary>Calls a method on a constructed generic owner.</summary>
    void Call(ConstructedMethodReference method);
    /// <summary>Dispatches an owned constructed interface method.</summary>
    /// <param name="method">A constructed interface contract from this assembly.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Foreign owner, noninterface method or invalid caller scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    void CallVirtual(ConstructedMethodReference method);
    /// <summary>Appends Call, Callvirt or Newobj with a constructed owner/method reference.</summary>
    /// <param name="opCode">Newobj for constructors, Callvirt for interface contracts, Call otherwise.</param>
    /// <param name="operand">Owned reference, valid in the caller's type/method scope.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Foreign owner, wrong opcode or invalid caller scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    void Emit(OpCode opCode, ConstructedMethodReference operand);
    /// <summary>Pushes a Function value bound to an owned static method.</summary>
    /// <param name="functionType">The exact structural shape.</param>
    /// <param name="target">Owned nongeneric static target.</param>
    /// <exception cref="ArgumentException">Invalid target, shape, foreign owner or scope.</exception>
    /// <exception cref="ArgumentNullException">An operand is null.</exception>
    void BindFunction(SignatureType functionType, MethodBuilder target);
    /// <summary>Emits a checked Function binding; consumes no receiver and pushes the callable value.</summary>
    /// <param name="opCode">BindFunction.</param>
    /// <param name="operand">The exact static binding owned by this assembly.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign target or invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, FunctionBinding operand);
    /// <summary>Consumes a Function receiver followed by its arguments, then pushes its result if any.</summary>
    /// <param name="functionType">Exact structural Function type.</param>
    /// <exception cref="ArgumentException">Not a Function or invalid generic scope/owner.</exception>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validity is checked on write.</exception>
    void InvokeFunction(SignatureType functionType);
    /// <summary>Appends a call to an instantiated generic method in this output.</summary>
    /// <param name="method">Owned instantiation with arguments valid in the caller's generic scope.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Foreign definition or out-of-scope type arguments.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack shape is validated on write.</exception>
    void Call(GenericMethodInstance method);
    /// <summary>Appends a generic call using an explicit typed reference.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">Instantiation; see Call(GenericMethodInstance).</param>
    /// <exception cref="ArgumentException">Wrong opcode, foreign definition or invalid generic scope.</exception>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, GenericMethodInstance operand);
    /// <summary>Allocates through an imported nongeneric constructor; consumes its parameters and returns the declaring value or reference type.</summary>
    /// <param name="constructor">A constructor reference owned by this output assembly.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is not a constructor, belongs to another output, or has invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">The instruction limit is exceeded; stack validity is checked on write.</exception>
    void NewObject(ImportedMethodReference constructor);
    /// <summary>Allocates through an imported constructor on a constructed generic owner.</summary>
    /// <param name="constructor">A constructor reference owned by this output assembly.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is not a constructor, belongs to another output, or has invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">The instruction limit is exceeded; stack validity is checked on write.</exception>
    void NewObject(ImportedConstructedMethodReference constructor);
    /// <summary>Appends a direct imported call on a constructed nominal owner.</summary>
    void Call(ImportedConstructedMethodReference method);
    /// <summary>Appends interface dispatch to an imported constructed contract.</summary>
    void CallVirtual(ImportedConstructedMethodReference method);
    /// <summary>Appends interface dispatch to an imported nongeneric contract.</summary>
    void CallVirtual(ImportedMethodReference method);
    /// <summary>Emits Newobj for constructors, Call for concrete/static methods or Callvirt for interface contracts.</summary>
    /// <param name="opCode">Opcode matching the imported member dispatch kind.</param>
    /// <param name="operand">Reference owned by the current output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign consumer, or invalid caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    void Emit(OpCode opCode, ImportedConstructedMethodReference operand);
    /// <summary>Loads an imported field on a constructed receiver.</summary>
    void LoadField(ImportedConstructedFieldReference field);
    /// <summary>Stores an imported field on a constructed receiver; readonly checks run on write.</summary>
    void StoreField(ImportedConstructedFieldReference field);
    /// <summary>Appends Ldfld/Stfld; rejects null, foreign, wrong-opcode or out-of-scope operands. Stack checks run on write.</summary>
    void Emit(OpCode opCode, ImportedConstructedFieldReference operand);

    /// <summary>Loads a primitive, nominal or vector field from its exact external receiver type.</summary>
    void LoadField(ImportedFieldReference field);
    /// <summary>Stores a primitive, nominal or vector field on its exact external receiver type; readonly stores fail validation.</summary>
    void StoreField(ImportedFieldReference field);
    /// <summary>Appends Ldfld or Stfld with an imported field operand.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">Reference owned by this output builder.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or foreign reference.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack and readonly checks run on write.</exception>
    void Emit(OpCode opCode, ImportedFieldReference operand);
    /// <summary>Appends a call to an instantiated imported generic method.</summary>
    /// <param name="method">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Reference belongs to another output or arguments exceed caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    void Call(ImportedGenericMethodReference method);
    /// <summary>Appends a generic imported call using the raw typed operand API.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, consuming owner or caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, ImportedGenericMethodReference operand);
    /// <summary>Appends virtual dispatch to an owned nongeneric interface method.</summary>
    /// <param name="target">A public abstract interface instance method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Foreign, generic or noninterface target.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void CallVirtual(MethodBuilder target);
    /// <summary>Appends a typed vector operation or addressed-local operation.</summary>
    /// <param name="opCode">Newarr, ReserveArray, Ldelem, Stelem, Initobj, Ldobj, Stobj, Castclass, or Callvirt for a Function signature.</param>
    /// <param name="elementType">Supported non-Void signature type; vector operations require scalar elements.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, unsupported element or foreign owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    void Emit(OpCode opCode, SignatureType elementType);
    /// <summary>Consumes a reference and pushes the same object through a checked target reference signature.</summary>
    /// <param name="target">Owned or imported nominal reference or vector type.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Target is not a supported reference or belongs to another assembly.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; input stack validation occurs on write.</exception>
    /// <remarks>CLI uses castclass; native uses castclass and its verifier/runtime conversion contract. This does not box values.</remarks>
    void CastReference(SignatureType target);
    /// <summary>Consumes an Int32 length and creates a default-initialized reference vector.</summary>
    /// <param name="elementType">Supported scalar element; see Emit(OpCode, SignatureType).</param>
    void NewArray(SignatureType elementType);
    /// <summary>Consumes an Int32 length and reserves checked uninitialized vector elements.</summary>
    /// <param name="elementType">Supported scalar element, including caller-scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Unsupported element or invalid owner scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on native write.</exception>
    /// <remarks>Native-only operation. Reads before writes fault at runtime. Executable CLI writing rejects this operation; PE/#Neo reference projections remain supported.</remarks>
    void ReserveArray(SignatureType elementType);
    /// <summary>Consumes an array and Int32 index, then pushes the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    void LoadArrayElement(SignatureType elementType);
    /// <summary>Consumes an array, Int32 index and element, then stores the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    void StoreArrayElement(SignatureType elementType);
    /// <summary>Consumes an array and loads its length normalized to Int32 using ldlen and conv.i4.</summary>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    void LoadArrayLength();
    /// <summary>Appends an operand-free arithmetic, comparison, stack or return instruction.</summary>
    /// <param name="opCode">Add, Sub, Mul, Div, Rem, And, Or, Xor, Shl, Shr, Ceq, Clt, Cgt, Dup, Pop, Conv_I4, Conv_I8, Neg, Not, Ldlen or Ret.</param>
    /// <exception cref="ArgumentException">Unknown opcode or an opcode requiring an operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    /// <remarks>Stack and return-flow validation remains deferred until writing. Rejected emission does not change the body.</remarks>
    void Emit(OpCode opCode);
    /// <summary>Appends an Int32 constant, argument-index or local-index instruction.</summary>
    /// <param name="opCode">Ldc_I4, Ldarg, Starg, Ldloc, Ldloca or Stloc.</param>
    /// <param name="operand">Signed constant, or zero-based argument/local index validated when writing.</param>
    /// <exception cref="ArgumentException">Unknown opcode or opcode incompatible with an Int32 operand.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, int operand);
    /// <summary>Appends an exact signed Int64 constant.</summary>
    /// <param name="opCode">Ldc_I8; other opcodes reject.</param>
    /// <param name="operand">Constant value, including Int64 extrema.</param>
    /// <exception cref="ArgumentException">Incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, long operand);
    /// <summary>Appends a string literal or terminal failure shared by CLI and native emission.</summary>
    /// <param name="opCode">Ldstr or Fail; other opcodes reject.</param>
    /// <param name="operand">Non-null valid Unicode text, at most 64 KiB in UTF-8; empty text is supported.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Incorrect opcode, invalid Unicode or oversized literal.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    /// <remarks>Rejects unpaired UTF-16 surrogates instead of substituting replacement characters.</remarks>
    void Emit(OpCode opCode, string operand);
    /// <summary>Terminates the invocation with a literal message. Requires an empty stack; no normal return or output assignment follows.</summary>
    /// <param name="message">Valid Unicode diagnostic, at most 64 KiB UTF-8.</param>
    /// <exception cref="ArgumentNullException">Message is null.</exception>
    /// <exception cref="ArgumentException">Message contains invalid Unicode or exceeds the UTF-8 size limit.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; body-flow errors are reported when writing.</exception>
    /// <remarks>Native execution produces UserFault without guest exception handling. CLI execution throws InvalidOperationException, which CLR callers can catch.</remarks>
    void Fail(string message);
    /// <summary>Appends a call or allocation using a local or external builder method.</summary>
    /// <param name="opCode">Call, Callvirt or Newobj; Callvirt currently requires an owned interface method.</param>
    /// <param name="operand">Method with a supported signature; external identity/core contracts are checked when writing.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or constructor usage.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, MethodBuilder operand);
    /// <summary>Appends a call to an owned imported read-only method reference.</summary>
    /// <param name="opCode">Newobj for a constructor; otherwise Call, or Callvirt when RequiresVirtualDispatch is true.</param>
    /// <param name="operand">Reference imported by this output assembly builder.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong dispatch opcode, an uninstantiated generic definition, or a reference from another builder.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, ImportedMethodReference operand);
    /// <summary>Appends a native-only call to a loaded static Int32 System declaration.</summary>
    /// <param name="opCode">Call.</param>
    /// <param name="operand">Owned native System function with an admitted Int32 signature.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Opcode is not Call.</exception>
    /// <exception cref="InvalidDataException">Unsupported module/signature or instruction limit exceeded.</exception>
    /// <remarks>Same matching-System runtime requirement as Call(NativeFunctionDefinition). Ordinary CLI output rejects this instruction.</remarks>
    void Emit(OpCode opCode, NativeFunctionDefinition operand);
    /// <summary>Pushes a Boolean constant for branch conditions.</summary>
    /// <param name="opCode">Ldc_Bool.</param>
    /// <param name="operand">The Boolean value.</param>
    /// <exception cref="ArgumentException">Incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, bool operand);
    /// <summary>Appends a field load/store using an owned output-field handle.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">An instance field declared in this output assembly.</param>
    /// <exception cref="ArgumentNullException">Field is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign field or generic definition field outside its declaring type.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, FieldBuilder operand);
    /// <summary>Duplicates the top stack value.</summary>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validity is checked on write.</exception>
    void Duplicate();
    /// <summary>Allocates an object and invokes its constructor, consuming the declared arguments.</summary>
    /// <param name="constructor">Class or value constructor.</param>
    /// <exception cref="ArgumentException">Not a constructor.</exception>
    /// <exception cref="ArgumentNullException">Constructor is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void NewObject(MethodBuilder constructor);
    /// <summary>Consumes a receiver and loads its field value.</summary>
    /// <param name="field">Owned instance field.</param>
    /// <exception cref="ArgumentException">Foreign field.</exception>
    /// <exception cref="ArgumentNullException">Field is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void LoadField(FieldBuilder field);
    /// <summary>Consumes a receiver followed by a value and stores the field.</summary>
    /// <param name="field">Owned instance field.</param>
    /// <exception cref="ArgumentException">Foreign field.</exception>
    /// <exception cref="ArgumentNullException">Field is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void StoreField(FieldBuilder field);
    /// <summary>Creates an unmarked branch destination. ClearBody retains label handles.</summary>
    /// <returns>A label owned by this method.</returns>
    /// <exception cref="InvalidDataException">4096-label limit exceeded.</exception>
    BranchLabel DefineLabel();
    /// <summary>Marks a label at the current instruction position.</summary>
    /// <param name="label">An unmarked label owned by this method.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign or already marked label.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void MarkLabel(BranchLabel label);
    /// <summary>Appends a branch. Conditional branches consume a Boolean comparison result.</summary>
    /// <param name="opCode">Br, Brtrue or Brfalse.</param>
    /// <param name="label">Destination owned by this method; it must be marked before writing.</param>
    /// <exception cref="ArgumentNullException">Label is null.</exception>
    /// <exception cref="ArgumentException">Foreign label or incorrect opcode.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, BranchLabel label);
    /// <summary>Loads an owned local address, including an as-yet uninitialized local.</summary>
    /// <param name="local">Local belonging to this method.</param>
    /// <exception cref="ArgumentException">Foreign local.</exception>
    /// <exception cref="ArgumentNullException">Null local.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void LoadLocalAddress(LocalDefinition local);
    /// <summary>Initializes an addressed local using its exact type; validates on write.</summary>
    /// <param name="type">Non-Void supported signature type in the current generic scope.</param>
    /// <exception cref="ArgumentNullException">Null type.</exception>
    /// <exception cref="ArgumentException">Void, foreign owner or out-of-scope parameter.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded or invalid stack on write.</exception>
    void InitializeObject(SignatureType type);
    /// <summary>Pushes a default value using a fresh scratch local and typed initialization.</summary>
    /// <param name="type">Supported non-Void value type, including scoped method parameters.</param>
    /// <remarks>Uses one local and three instructions. ClearBody retains the scratch local.</remarks>
    /// <exception cref="ArgumentNullException">Null type.</exception>
    /// <exception cref="ArgumentException">Void, foreign owner or out-of-scope parameter.</exception>
    /// <exception cref="InvalidDataException">Local or instruction limit exceeded.</exception>
    void LoadDefault(SignatureType type);
    /// <summary>Declares an Int32 local. Loads require a store or typed initialization on every reachable path.</summary>
    /// <returns>A local handle owned by this method.</returns>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    LocalDefinition DeclareInt32Local();
    /// <summary>Declares a typed primitive local. ClearBody retains the slot and type.</summary>
    /// <param name="type">Int32, Int64, Boolean or String; Void is not a local type.</param>
    /// <returns>A stable local handle owned by this method.</returns>
    /// <exception cref="ArgumentException">Type is Void or an invalid enum value.</exception>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    LocalDefinition DeclareLocal(PrimitiveType type);
    /// <summary>Declares a local holding an instance of a root class in this output assembly.</summary>
    /// <param name="type">Nonstatic class declaration owned by the output assembly.</param>
    /// <returns>A stable local handle; ClearBody retains its declared class identity.</returns>
    /// <exception cref="ArgumentNullException">Class is null.</exception>
    /// <exception cref="ArgumentException">Class is static or belongs to another assembly.</exception>
    /// <exception cref="InvalidDataException">The method already has 256 locals.</exception>
    /// <remarks>Loads require a store or typed initialization on every path. No inheritance, boxing or external class locals are admitted.</remarks>
    LocalDefinition DeclareLocal(TypeBuilder type);
    /// <summary>Declares a primitive, owned root-class or vector local with exact type identity.</summary>
    /// <param name="type">Non-Void type; all classes must belong to this output.</param>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Void or foreign class type.</exception>
    /// <exception cref="InvalidDataException">Local limit exceeded.</exception>
    LocalDefinition DeclareLocal(SignatureType type);
    /// <summary>Loads a local previously stored or initialized in this body.</summary>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Local belongs to another method.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void LoadLocal(LocalDefinition local);
    /// <summary>Stores the top value into a local of the same declared type.</summary>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Local belongs to another method.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void StoreLocal(LocalDefinition local);
    /// <summary>Appends a typed local load or store. Stack/initialization validation occurs when writing.</summary>
    /// <param name="opCode">Ldloc, Ldloca or Stloc.</param>
    /// <param name="local">Local owned by this method.</param>
    /// <exception cref="ArgumentNullException">Local is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    void Emit(OpCode opCode, LocalDefinition local);
    /// <summary>Consumes an initialized owned local address or managed-reference parameter and loads its value.</summary>
    /// <param name="type">Exact non-Void local type, including scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid type, ownership or generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; address, type and definite-assignment checks run on write.</exception>
    void LoadObject(SignatureType type);
    /// <summary>Consumes an owned local address or managed-reference parameter followed by its value; local stores establish definite assignment.</summary>
    /// <param name="type">Exact non-Void local type, including scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid type, ownership or generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; address and type checks run on write.</exception>
    void StoreObject(SignatureType type);
}
