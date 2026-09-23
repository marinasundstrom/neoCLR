using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;

// Bounded Result/Option profile, not a general CLI loader. Only supplied metadata is resolved.
static class UnionImport
{
    const string VoidResult = "System.Result<Void,System.OverflowError>";
    const string VoidOk = "System.Result.Ok<Void>";
    const string Overflow = "System.OverflowError";
    const string IntArray = "arrayref<Int32>";
    const string VoidOption = "System.Option<Void>";
    const string VoidSome = "System.Option.Some<Void>";
    const string Option = "System.Option<Int32>";
    const string Some = "System.Option.Some<Int32>";
    const string None = "System.Option.None";
    const string Carrier = "System.Result<Int32,System.OverflowError>";
    const string Ok = "System.Result.Ok<Int32>";
    const string Error = "System.Result.Error<System.OverflowError>";
    sealed record Slot(string Type, int Local = -1, int ConditionalOut = -1, int Argument = -1, MethodDefinition? Function = null, bool VirtualFunction = false, string? FunctionReceiver = null);
    sealed record State(List<Slot> Stack, bool[] Assigned);
    sealed record Call(string Name, string[] Arguments, string Result, int OutArgument = -1, string? Instruction = null, bool ConditionalOutput = false);

    public static void Write(string application, string core, string destination, bool collectionProfile = false, params string[] dependencies)
        => WriteImplementation(application, core, destination, collectionProfile, dependencies, null);

    public static void WriteLibrary(string application, string core, string destination, string owner)
        => WriteImplementation(application, core, destination, true, [], owner);

    static void WriteImplementation(string application, string core, string destination, bool collectionProfile, string[] dependencies, string? libraryOwner)
    {
        if (dependencies.Length > 8) throw new InvalidDataException("Library input limit exceeded.");
        GenericUnionBindings.Reset();
        TaskBindings.Reset(); CollectionBindings.Reset(); ReflectionBindings.Reset(); RuntimeServiceBindings.Reset();
        DelegateBindings.Reset();
        var inputs = new[] { application, core }.Concat(dependencies).ToArray();
        foreach (var path in inputs)
            if (new FileInfo(path).Length > 16 * 1024 * 1024) throw new InvalidDataException("Image exceeds profile limit.");
        foreach (var path in new[] { application }.Concat(dependencies)) VoidStorageValidation.Check(path);
        var errors = ClosureAudit.Inspect(inputs);
        if (errors.Length != 0) throw new InvalidDataException(string.Join("\n", errors));
        using var resolver = new ClosureAudit.SuppliedAssemblies();
        foreach (var path in inputs) resolver.Add(path);
        var app = resolver.Images.Single(a => a.Name.FullName == System.Reflection.AssemblyName.GetAssemblyName(application).FullName);
        var library = resolver.Images.Single(a => a.Name.FullName == System.Reflection.AssemblyName.GetAssemblyName(core).FullName);
        var guestLibraries = resolver.Images.Where(a => a != app && a != library).Select(a => a.MainModule).ToHashSet();
        if (guestLibraries.Append(app.MainModule).Any(module => module.Types.Any(t => t.Name == "<Module>" && t.Methods.Any(m => m.IsConstructor && m.IsStatic))))
            throw new InvalidDataException("Module initializers unsupported.");
        if (libraryOwner is null) IntrospectionHierarchy.RejectExternalProviders(guestLibraries.Append(app.MainModule));
        ApplicationTypes.Reset(guestLibraries.Append(app.MainModule).ToArray());
        var exports = libraryOwner is null ? [] : LibraryImplementation.Roots(app.MainModule, library.MainModule, libraryOwner);
        if (libraryOwner is not null) ApplicationTypes.SetLibraryScope(app.MainModule, libraryOwner);
        var entry = libraryOwner is null ? app.EntryPoint ?? throw new InvalidDataException("Missing entry point.") : null;
        string Name(MethodDefinition method) => libraryOwner is null ? MetadataIdentity.FunctionName(method)
            : exports.Contains(method) ? (method.DeclaringType.FullName == "System.Tasks.TaskOperators" ? method.DeclaringType.FullName : libraryOwner) + "." + LibraryImplementation.GenericName(method)
            : throw new InvalidDataException("Unexported implementation dependency: " + method.FullName);
        if (entry is not null && (entry.Parameters.Count != 0 || entry.ReturnType.MetadataType != MetadataType.Void))
            throw new InvalidDataException("Result profile requires a parameterless no-result entry.");
        if (collectionProfile) { InterfaceBindings.Validate(library.MainModule); CollectionBindings.Validate(library.MainModule); ReflectionBindings.Validate(library.MainModule); NativeMemoryBindings.Validate(library.MainModule); }
        bool InternalLibraryAccess(MethodDefinition target, MethodDefinition caller) => libraryOwner is not null
            && target.IsAssembly && target.Module == caller.Module
            && ApplicationTypes.IsLibrary(target.DeclaringType) && ApplicationTypes.IsLibrary(caller.DeclaringType);
        MethodDefinition? activeLibraryMethod = null;
        string ProfileType(TypeReference type, bool result = false)
        {
            if (libraryOwner is not null && type is ByReferenceType byref)
                return ProfileType(byref.ElementType) + "&";
            if (libraryOwner is not null && type is GenericParameter parameter)
            {
                if (!ApplicationTypes.IsLibraryParameter(parameter) && (parameter.Type != GenericParameterType.Method || parameter.Owner != activeLibraryMethod))
                    throw new InvalidDataException("Foreign generic parameter in library body.");
                return "T" + parameter.Position;
            }
            if (libraryOwner is not null && ParameterSnapshotBindings.Type(type) is { } snapshot) return snapshot;
            if (libraryOwner is not null && type is ArrayType { IsVector: true } array)
                return "arrayref<" + ProfileType(array.ElementType) + ">";
            if (libraryOwner is not null && type is GenericInstanceType propagation
                && propagation.ElementType.FullName == "System.Propagatable`3" && RuntimeSignatures.IsCore(propagation.Scope)
                && propagation.GenericArguments.Count == 3)
                return "System.Propagatable<" + string.Join(',', propagation.GenericArguments.Select(t => ProfileType(t))) + ">";
            var collection = CollectionBindings.Type(type, libraryOwner is null ? null : t => ProfileType(t));
            if (collection is not null && collectionProfile) return collection;
            return ApplicationTypes.Type(type) ?? InterfaceBindings.Type(type, libraryOwner is null ? null : t => ProfileType(t)) ?? NativeMemoryBindings.Type(type) ?? ReflectionBindings.Type(type) ?? DelegateBindings.Type(type) ?? ProcessBindings.ArrayType(type) ?? GenericUnionBindings.Type(type) ?? CalendarBindings.Type(type) ?? PrimitiveBindings.Type(type) ?? ResultBindings.Type(type) ?? Type(type, result);
        }
        if (libraryOwner is not null)
        {
            GenericUnionBindings.ParameterMap = parameter => ProfileType(parameter);
            ApplicationTypes.LibraryMap = type => ProfileType(type);
        }
        var output = new StringBuilder(entry is not null ? $".module ImportedUnion\n.entry {Name(entry)}\n" : "");
        if (libraryOwner is null)
            foreach (var module in new[] { app.MainModule }.Concat(guestLibraries))
                output.AppendLine(SourceMetadata.Assembly(module));
        var coercions = new Dictionary<string, (string Name, string Body)>();
        Call Coerce(Call call, string[] actual)
        {
            if (!actual.Where((t, i) => Converts(t, call.Arguments[i])).Any()) return call;
            var key = (call.Instruction ?? call.Name) + string.Join(',', call.Arguments) + string.Join(',', actual) + call.Result + call.OutArgument;
            if (!coercions.TryGetValue(key, out var helper))
            {
                var name = "RuntimeCliCall" + coercions.Count;
                var parameters = actual.Select((t, i) => (i == call.OutArgument ? (call.ConditionalOutput ? "out(true) " : "out ") : ApplicationTypes.IsLibraryUnion(call.Arguments[i]) && t == call.Arguments[i] + "&" ? "readonly " : "") + t + " arg" + i);
                var body = new StringBuilder($".function {name}({string.Join(',', parameters)}) -> {call.Result}\n");
                for (var i = 0; i < actual.Length; i++) body.AppendLine("ldarg arg" + i).Append(ConvertStack(actual[i], call.Arguments[i]));
                body.AppendLine(call.Instruction ?? $"call {call.Name}({string.Join(',', call.Arguments)})").AppendLine("ret\n.end");
                helper = (name, body.ToString()); coercions.Add(key, helper);
            }
            return call with { Name = helper.Name, Arguments = actual, Instruction = null };
        }
        var delegateAdapters = new Dictionary<string, string>();
        var mappings = new List<(MethodDefinition Method, int MethodId, int Offset)>();
        var pending = new Queue<MethodDefinition>(entry is not null ? [entry] : exports);
        var seen = new HashSet<MethodDefinition>();
        var instanceBodies = new Dictionary<MethodDefinition, string>();
        while (pending.TryDequeue(out var method))
        {
            if (!seen.Add(method)) continue;
            var methodId = seen.Count;
            if (seen.Count > 128) throw new InvalidDataException("Method limit exceeded.");
            if (libraryOwner is not null && !exports.Contains(method) && !ApplicationTypes.IsLibraryDependency(method.DeclaringType)) throw new InvalidDataException("Unexported library body.");
            activeLibraryMethod = libraryOwner is null ? null : method;
            if (libraryOwner is null) ApplicationTypes.CheckMethod(method);
            else LibraryImplementation.CheckMethod(method);
            foreach (var parameter in method.Parameters) ApplicationTypes.CheckAccess(parameter.ParameterType, method.Module);
            ApplicationTypes.CheckAccess(method.ReturnType, method.Module);
            if (method.HasBody)
                foreach (var variable in method.Body.Variables) ApplicationTypes.CheckAccess(variable.VariableType, method.Module);
            if (!method.HasBody || method.IsPInvokeImpl || method.IsInternalCall || method.Body.HasExceptionHandlers
                || method.Body.CodeSize > 65536 || method.Body.MaxStackSize > 256 || method.Body.Variables.Count > 256 || method.Body.Instructions.Count == 0
                || method.DeclaringType.Methods.Any(m => m.IsConstructor && m.IsStatic))
                throw new InvalidDataException("Unsupported body: " + method.FullName + " (" + method.Attributes + ")");
            foreach (var instruction in method.Body.Instructions)
            {
                switch (instruction.Operand)
                {
                    case MethodReference called:
                        if (called is GenericInstanceMethod genericCall)
                            foreach (var argument in genericCall.GenericArguments) ApplicationTypes.CheckAccess(argument, method.Module);
                        ApplicationTypes.CheckAccess(called.DeclaringType, method.Module);
                        ApplicationTypes.CheckAccess(called.ReturnType, method.Module);
                        foreach (var parameter in called.Parameters) ApplicationTypes.CheckAccess(parameter.ParameterType, method.Module);
                        break;
                    case FieldReference fieldReference:
                        ApplicationTypes.CheckAccess(fieldReference.DeclaringType, method.Module);
                        ApplicationTypes.CheckAccess(fieldReference.FieldType, method.Module);
                        break;
                    case TypeReference referencedType: ApplicationTypes.CheckAccess(referencedType, method.Module); break;
                }
            }
            if (libraryOwner is not null && (ErrorCarrierLibrary.Constructor(method, ProfileType) ?? GenericUnionLibrary.CaseConstructor(method, ProfileType)) is { } constructorBody)
            {
                instanceBodies[method] = constructorBody;
                ApplicationTypes.Expand(ProfileType, pending);
                continue;
            }
            var args = (method.HasThis ? new[] { ApplicationTypes.Receiver(method) } : Array.Empty<string>()).Concat(method.Parameters.Select(p => ProfileType(p.ParameterType))).ToArray();
            var valueConstructor = libraryOwner is null && method.IsConstructor && method.DeclaringType.IsValueType;
            var emitInstance = method.HasThis && !valueConstructor;
            var emitOwnedStatic = libraryOwner is not null && method.IsStatic && (WorkerBindings.IsName(method.DeclaringType.FullName) || AsyncBindings.IsName(method.DeclaringType.FullName) || method.DeclaringType.IsValueType || OpaqueLibrary.IsString(method.DeclaringType) || ArrayLibrary.IsMatched(method.DeclaringType) || DescriptorLibrary.IsProvider(method.DeclaringType));
            var result = ProfileType(method.ReturnType, true);
            var locals = method.Body.Variables.Select(v => ProfileType(v.VariableType)).ToArray();
            if (locals.Any(t => !(libraryOwner is not null && t is "Value" or ParameterSnapshotBindings.Vector) && !(libraryOwner is not null && method.GenericParameters.Concat(method.DeclaringType.GenericParameters).Any(p => t == "T" + p.Position)) && !WorkerBindings.IsName(t) && !EnumBindings.IsType(t) && !AsyncBindings.IsType(t) && !TaskBindings.IsType(t) && !ApplicationTypes.IsType(t) && !ManagedArrayBindings.IsType(t) && t != "System.Object" && !InterfaceBindings.IsInterface(t) && !NativeMemoryBindings.IsPointer(t) && !ReflectionBindings.IsType(t) && t != "arrayref<String>" && !DelegateBindings.IsType(t) && !GenericUnionBindings.IsType(t) && !CalendarBindings.IsReference(t) && !CalendarBindings.Types.Contains(t) && !PrimitiveBindings.Types.Contains(t) && !ResultBindings.IsType(t) && !CollectionBindings.IsReference(t) && t is not ("Boolean" or "Int32" or "Double" or "String" or IntArray or Carrier or Ok or Error or Option or Some or None or VoidOption or VoidSome or Overflow or "Void" or VoidResult or VoidOk)))
                throw new InvalidDataException("Unsupported local default in Result profile.");
            NormalizePatternBranches(method);
            var instructions = method.Body.Instructions.ToArray();
            var indexes = instructions.Select((i, n) => (i, n)).ToDictionary(p => p.i, p => p.n);
            var states = new Dictionary<int, State>();
            var work = new Queue<int>();
            var bodies = new Dictionary<int, string>();
            Merge(0, new([], locals.Select(t => !method.GenericParameters.Concat(method.DeclaringType.GenericParameters).Any(p => t == "T" + p.Position) && method.Body.InitLocals && t != Carrier && t != Option && t != VoidOption && t != VoidResult && t != "String" && !NeedsInitialization(t)).ToArray()));
            var visits = 0;
            while (work.TryDequeue(out var index))
            {
                if (++visits > 100000) throw new InvalidDataException("Control-flow analysis limit exceeded.");
                var input = states[index];
                var stack = input.Stack.ToList(); var assigned = (bool[])input.Assigned.Clone();
                var instruction = instructions[index]; var code = new StringBuilder(); var successors = new List<int>();
                var assignmentEdges = new Dictionary<int, bool[]>();
                if (stack.Any(slot => slot.Function is not null) && instruction.OpCode.Code is not (Code.Newobj or Code.Nop))
                    throw new InvalidDataException("Function address must be consumed by an admitted delegate constructor.");
                if (stack.Any(slot => slot.ConditionalOut >= 0) && instruction.OpCode.Code is not (Code.Nop or Code.Brtrue or Code.Brtrue_S or Code.Brfalse or Code.Brfalse_S))
                    throw new InvalidDataException("Conditional extraction output must be tested before use.");
                void Push(Slot slot) { stack.Add(slot); if (stack.Count > method.Body.MaxStackSize) throw new InvalidDataException("Declared maxstack exceeded."); }
                Slot Pop() { if (stack.Count == 0) throw new InvalidDataException($"Input stack underflow in {method.FullName} at {instruction.Offset:x4}."); var top = stack[^1]; stack.RemoveAt(stack.Count - 1); return top; }
                Slot Expect(string type)
                {
                    var top = Pop();
                    if (ManagedArrayBindings.IsType(top.Type) && ManagedArrayBindings.IsType(type) && top.Type != type)
                        throw new InvalidDataException("Mutable array conversions require identical element types.");
                    if (!(CollectionBindings.Assignable(PrimitiveBindings.Stack(top.Type), PrimitiveBindings.Stack(type))
                        || (type == "System.Runtime.CompilerServices.ITaskAwaiter" && TaskBindings.IsType(top.Type) && top.Type.StartsWith("System.Tasks.Task<")) || ReflectionBindings.Assignable(top.Type, type) || ApplicationTypes.Assignable(top.Type, type)))
                        throw new InvalidDataException($"Input stack type mismatch in {method.FullName} at IL_{instruction.Offset:x4}: expected {type}, found {top.Type}.");
                    return top;
                }
                int Local(int n) { if (n < 0 || n >= locals.Length) throw new InvalidDataException("Invalid local index."); return n; }
                void Load(int n)
                {
                    Local(n);
                    if (!assigned[n]) throw new InvalidDataException($"Read of uninitialized or unsupported default local {n} in {method.FullName} at instruction {index}.");
                    Push(new(locals[n] == "Boolean" ? "Int32" : PrimitiveBindings.Stack(locals[n])));
                    code.AppendLine($"ldloc local{n}");
                    if (locals[n] == "Boolean") code.Append(BooleanBindings.Convert("Boolean", "Int32"));
                }
                Slot ConvertTop(string type)
                {
                    if (stack.Count > 0 && stack[^1].Type == "FaultNull"
                        && (WorkerBindings.IsName(type) || TaskBindings.IsType(type) || AsyncBindings.IsType(type) || ApplicationTypes.IsReference(type)))
                    {
                        var nullValue = Pop();
                        var key = "DefaultReference" + Convert.ToHexString(Encoding.UTF8.GetBytes(type));
                        delegateAdapters[key] = $".function {key}() -> {type}\n.local {type} value\nldloca value\ninitobj {type}\nldloc value\nret\n.end\n";
                        code.AppendLine($"call {key}()");
                        return nullValue;
                    }
                    if (stack.Count == 0 || !Converts(stack[^1].Type, type)) return Expect(type);
                    var value = Pop(); code.Append(ConvertStack(value.Type, type)); return value;
                }
                Slot Argument(string type)
                {
                    return stack.Count > 0 && Converts(stack[^1].Type, type) ? Pop() : Expect(type);
                }
                void Store(int n) { Local(n); ConvertTop(locals[n]); assigned[n] = true; code.AppendLine($"stloc local{n}"); }
                void Arg(int n)
                {
                    if (n < 0 || n >= args.Length) throw new InvalidDataException("Invalid parameter index.");
                    Push(new(args[n] == "Boolean" ? "Int32" : PrimitiveBindings.Stack(args[n]), Argument: args[n].EndsWith('&') || n == 0 && LibraryImplementation.IsByValueReceiver(method) ? n : -1));
                    code.AppendLine($"ldarg {n}");
                    if (args[n] == "Boolean") code.Append(BooleanBindings.Convert("Boolean", "Int32"));
                }
                bool NumericOperands()
                {
                    if (stack.Count < 2) return false;
                    var left = PrimitiveBindings.Stack(stack[^2].Type);
                    var right = PrimitiveBindings.Stack(stack[^1].Type);
                    return left == right && left is "Int32" or "Int64" or "Double" or "IntPtr" or "UIntPtr";
                }
                int Target() => instruction.Operand is Instruction target && indexes.TryGetValue(target, out var n)
                    ? n : throw new InvalidDataException("Invalid branch target.");
                var terminates = false;
                switch (instruction.OpCode.Code)
                {
                    case Code.Nop: break;
                    case Code.Initobj:
                        var initializedType = ProfileType((TypeReference)instruction.Operand);
                        var address = Expect(initializedType + "&");
                        if (address.Local < 0) throw new InvalidDataException("Only local initialization is admitted.");
                        if (initializedType is Carrier or Option or VoidOption or VoidResult || NeedsInitialization(initializedType))
                        {
                            if (assigned[address.Local]) throw new InvalidDataException("Resetting an initialized carrier is unsupported.");
                            // A CLI carrier default is not a valid union value. Keep it unreadable
                            // until an actual carrier assignment, rather than inventing a case.
                            code.AppendLine("pop");
                        }
                        else
                        {
                            code.AppendLine((ApplicationTypes.IsType(initializedType) || ManagedArrayBindings.IsReference(initializedType) || CalendarBindings.Types.Contains(initializedType) || GenericUnionBindings.IsType(initializedType)) ? "initobj " + initializedType : Default(initializedType) + "\nstobj " + initializedType);
                            assigned[address.Local] = true;
                        }
                        break;
                    case Code.Ldtoken:
                        if (!collectionProfile || instruction.Operand is not TypeReference tokenType) throw new InvalidDataException("Only admitted type tokens supported.");
                        var token = ProfileType(tokenType is TypeSpecification ? tokenType : tokenType.Resolve() ?? tokenType);
                        Push(new("System.RuntimeTypeHandle")); code.AppendLine("ldtoken " + token); break;
                    case Code.Box:
                        var boxedType = ProfileType((TypeReference)instruction.Operand);
                        ConvertTop(boxedType); Push(new("System.Object")); code.AppendLine("box " + boxedType); break;
                    case Code.Isinst:
                        var testedTarget = ProfileType((TypeReference)instruction.Operand);
                        var testedSource = Pop().Type;
                        if (!ManagedArrayBindings.IsReference(testedSource) || !ManagedArrayBindings.IsReference(testedTarget))
                            throw new InvalidDataException("Only reference type tests are supported.");
                        if (ManagedArrayBindings.IsType(testedSource) && ManagedArrayBindings.IsType(testedTarget)
                            && testedSource != testedTarget)
                            throw new InvalidDataException("Mutable array tests require identical element types.");
                        Push(new(testedTarget)); code.AppendLine("isinst " + testedTarget); break;
                    case Code.Castclass:
                        var castTarget = ProfileType((TypeReference)instruction.Operand);
                        var castSource = Pop().Type;
                        if (!ManagedArrayBindings.IsReference(castSource) || !ManagedArrayBindings.IsReference(castTarget))
                            throw new InvalidDataException("Unsupported reference cast.");
                        if (ManagedArrayBindings.IsType(castSource) && ManagedArrayBindings.IsType(castTarget)
                            && castSource != castTarget)
                            throw new InvalidDataException("Mutable array casts require identical element types.");
                        Push(new(castTarget)); code.AppendLine("castclass " + castTarget); break;
                    case Code.Ldnull: Push(new("FaultNull")); break;
                    case Code.Throw:
                        Expect("FaultNull");
                        code.AppendLine("fault \"Guest program reached a terminal failure\"");
                        terminates = true; break;
                    case Code.Newarr:
                        var element = ProfileType((TypeReference)instruction.Operand);
                        if (!ManagedArrayBindings.Defaultable(element)) throw new InvalidDataException("Unsupported vector allocation element.");
                        Expect("Int32"); Push(new("arrayref<" + element + ">")); code.AppendLine("newarr " + element); break;
                    case Code.Ldlen:
                        var lengthArray = Pop().Type;
                        if (!ManagedArrayBindings.IsType(lengthArray)) throw new InvalidDataException("Unsupported vector length receiver.");
                        Push(new("UIntPtr")); code.AppendLine("ldlen"); break;
                    case Code.Ldelema:
                        var addressElement = ProfileType((TypeReference)instruction.Operand);
                        Expect("Int32"); Expect("arrayref<" + addressElement + ">");
                        Push(new(addressElement + "&")); code.AppendLine("ldelema " + addressElement); break;
                    case Code.Ldelem_I1: case Code.Ldelem_U1: case Code.Ldelem_I2: case Code.Ldelem_U2:
                    case Code.Ldelem_I4: case Code.Ldelem_U4: case Code.Ldelem_I8: case Code.Ldelem_I:
                    case Code.Ldelem_R4: case Code.Ldelem_R8: case Code.Ldelem_Any:
                        Expect("Int32"); var loadedArray = Pop().Type;
                        if (!ManagedArrayBindings.IsType(loadedArray)) throw new InvalidDataException("Unsupported array load.");
                        var loadedElement = loadedArray[9..^1];
                        ManagedArrayBindings.CheckElement(instruction.OpCode.Code, loadedElement,
                            instruction.Operand is TypeReference loadToken ? ProfileType(loadToken) : null);
                        Push(new(loadedElement == "Boolean" ? "Int32" : PrimitiveBindings.Stack(loadedElement))); code.AppendLine("ldelem " + loadedElement);
                        if (loadedElement == "Boolean") code.Append(BooleanBindings.Convert("Boolean", "Int32"));
                        break;
                    case Code.Stelem_I1: case Code.Stelem_I2: case Code.Stelem_I4: case Code.Stelem_I8:
                    case Code.Stelem_I: case Code.Stelem_R4: case Code.Stelem_R8: case Code.Stelem_Any:
                        if (stack.Count < 3 || !ManagedArrayBindings.IsType(stack[^3].Type)) throw new InvalidDataException("Unsupported array store.");
                        var storedElement = stack[^3].Type[9..^1];
                        ManagedArrayBindings.CheckElement(instruction.OpCode.Code, storedElement,
                            instruction.Operand is TypeReference storeToken ? ProfileType(storeToken) : null);
                        ConvertTop(storedElement); Expect("Int32"); Expect("arrayref<" + storedElement + ">");
                        code.AppendLine("stelem " + storedElement); break;
                    case Code.Ldelem_Ref:
                        Expect("Int32"); var referenceArray = Pop().Type;
                        if (!ManagedArrayBindings.IsType(referenceArray) || !ManagedArrayBindings.IsReference(referenceArray[9..^1])) throw new InvalidDataException("Unsupported reference vector.");
                        var referenceElement = referenceArray[9..^1];
                        Push(new(referenceElement)); code.AppendLine("ldelem " + referenceElement); break;
                    case Code.Stelem_Ref:
                        if (stack.Count < 3 || !ManagedArrayBindings.IsType(stack[^3].Type)) throw new InvalidDataException("Unsupported reference vector store.");
                        var storedArray = stack[^3].Type;
                        if (!ManagedArrayBindings.IsReference(storedArray[9..^1])) throw new InvalidDataException("Unsupported reference vector store.");
                        ConvertTop(storedArray[9..^1]); Expect("Int32"); Expect(storedArray);
                        code.AppendLine("stelem " + storedArray[9..^1]); break;
                    case Code.Ldfld:
                        if (!collectionProfile) throw new InvalidDataException("Native fields require target profile.");
                        var readField = (FieldReference)instruction.Operand;
                        var appRead = ApplicationTypes.Field(readField, method, ProfileType);
                        if (appRead is not null)
                        {
                            var receiver = Pop().Type;
                            if (!ApplicationTypes.Assignable(receiver, appRead.Owner) && receiver != appRead.Owner + "&") throw new InvalidDataException("Invalid application field receiver.");
                            Push(new(appRead.Type == "Boolean" ? "Int32" : PrimitiveBindings.Stack(appRead.Type)));
                            if ((PrimitiveLibrary.IsMatched(readField.DeclaringType.Resolve()) || OpaqueLibrary.IsString(readField.DeclaringType.Resolve()) || ArrayLibrary.IsMatched(readField.DeclaringType.Resolve())))
                            {
                                if (receiver.EndsWith('&')) code.AppendLine("ldobj " + appRead.Type);
                            }
                            else code.AppendLine($"ldfld {appRead.Owner}::{appRead.Name}");
                            if (appRead.Type == "Boolean") code.Append(BooleanBindings.Convert("Boolean", "Int32"));
                            break;
                        }
                        throw new InvalidDataException("Unsupported runtime field read.");
                    case Code.Stfld:
                        if (!collectionProfile) throw new InvalidDataException("Native fields require target profile.");
                        var writeField = (FieldReference)instruction.Operand;
                        if ((PrimitiveLibrary.IsMatched(writeField.DeclaringType.Resolve()) || OpaqueLibrary.IsString(writeField.DeclaringType.Resolve()) || ArrayLibrary.IsMatched(writeField.DeclaringType.Resolve())))
                            throw new InvalidDataException("Primitive library backing storage is readonly.");
                        var appWrite = ApplicationTypes.Field(writeField, method, ProfileType);
                        if (appWrite is not null)
                        {
                            ConvertTop(appWrite.Type); Expect(appWrite.Owner + (appWrite.ValueOwner ? "&" : ""));
                            code.AppendLine($"stfld {appWrite.Owner}::{appWrite.Name}");
                            if (appWrite.ValueOwner) code.AppendLine("pop");
                            break;
                        }
                        throw new InvalidDataException("Unsupported runtime field write.");
                    case Code.Ldobj:
                        var copiedToken = (TypeReference)instruction.Operand;
                        if (libraryOwner is null || !copiedToken.IsValueType || copiedToken.Resolve()?.IsValueType != true || !ApplicationTypes.IsLibrary(copiedToken))
                            throw new InvalidDataException("Only matched library value loads are admitted.");
                        var copiedType = ProfileType(copiedToken);
                        if (LibraryImplementation.IsByValueReceiver(method) && GenericUnionLibrary.IsMatched(method.DeclaringType)
                            && stack.Count > 0 && stack[^1].Type == copiedType && stack[^1].Argument == 0)
                        {
                            Pop(); Push(new(copiedType)); break;
                        }
                        var copiedAddress = Expect(copiedType + "&");
                        if (copiedAddress.Local >= 0 && !assigned[copiedAddress.Local])
                            throw new InvalidDataException("Read through uninitialized value address.");
                        Push(new(copiedType)); code.AppendLine("ldobj " + copiedType); break;
                    case Code.Stobj:
                        var storedOutput = ProfileType((TypeReference)instruction.Operand);
                        ConvertTop(storedOutput);
                        var outputAddress = Expect(storedOutput + "&");
                        if (libraryOwner is null || outputAddress.Argument != 1 || method.Parameters.Count != 1
                            || !GenericUnionLibrary.IsConditionalOutput(method, method.Parameters[0]))
                            throw new InvalidDataException("Only checked union output stores are admitted.");
                        code.AppendLine("stobj " + storedOutput); break;
                    case Code.Ldind_I4:
                        Expect("Int32*"); Push(new("Int32")); code.AppendLine("ldobj Int32"); break;
                    case Code.Stind_I4:
                        Expect("Int32"); Expect("Int32*"); code.AppendLine("stobj Int32"); break;
                    case Code.Ldsfld:
                        var field = ((FieldReference)instruction.Operand).Resolve();
                        if (field is null || field.Module != method.Module || field.FullName != "System.Unit System.Unit::Value"
                            || !field.IsStatic || !field.IsInitOnly || !field.DeclaringType.IsValueType
                            || field.DeclaringType.Fields.Any(f => !f.IsStatic)
                            || field.DeclaringType.Methods.Any(m => m.IsConstructor && m.IsStatic))
                            throw new InvalidDataException("Only the empty Raven Unit literal field is admitted.");
                        Push(new("Void")); code.AppendLine("ldvoid"); break;
                    case Code.Ldstr:
                        var value = (string)instruction.Operand;
                        if (value.Length > 65536) throw new InvalidDataException("String limit exceeded.");
                        Push(new("String")); code.AppendLine("ldstr " + JsonSerializer.Serialize(value)); break;
                    case Code.Ldc_I8:
                        Push(new("Int64")); code.AppendLine($"ldc.i8 {instruction.Operand}"); break;
                    case Code.Ldc_R4:
                        Push(new("Double")); code.AppendLine("ldc.r4 " + ((float)instruction.Operand).ToString("R", System.Globalization.CultureInfo.InvariantCulture)); break;
                    case Code.Conv_I1: case Code.Conv_U1: case Code.Conv_I2: case Code.Conv_U2:
                    case Code.Conv_I4: case Code.Conv_U4: case Code.Conv_I8: case Code.Conv_U8:
                    case Code.Conv_I: case Code.Conv_U: case Code.Conv_R4: case Code.Conv_R8: case Code.Conv_R_Un:
                        var converted = Pop();
                        if (EnumBindings.IsType(converted.Type)) { code.Append(EnumBindings.Convert(converted.Type, "Int32")); converted = new("Int32"); }
                        if (converted.Type is not ("Int32" or "Int64" or "IntPtr" or "UIntPtr" or "Double"))
                            throw new InvalidDataException("Unsupported numeric conversion source.");
                        var convertedType = instruction.OpCode.Code switch {
                            Code.Conv_I8 or Code.Conv_U8 => "Int64", Code.Conv_I => "IntPtr", Code.Conv_U => "UIntPtr",
                            Code.Conv_R4 or Code.Conv_R8 or Code.Conv_R_Un => "Double", _ => "Int32"
                        };
                        Push(new(convertedType)); code.AppendLine(instruction.OpCode.Name); break;
                    case Code.Ldc_R8:
                        Push(new("Double"));
                        code.AppendLine("ldc.r8 " + ((double)instruction.Operand).ToString("R", System.Globalization.CultureInfo.InvariantCulture)); break;
                    case Code.Ldc_I4_M1: case Code.Ldc_I4_0: case Code.Ldc_I4_1: case Code.Ldc_I4_2:
                    case Code.Ldc_I4_3: case Code.Ldc_I4_4: case Code.Ldc_I4_5: case Code.Ldc_I4_6:
                    case Code.Ldc_I4_7: case Code.Ldc_I4_8: case Code.Ldc_I4_S: case Code.Ldc_I4:
                        var number = instruction.Operand is null ? (int)instruction.OpCode.Code - (int)Code.Ldc_I4_0 : Convert.ToInt32(instruction.Operand);
                        // Preserve a literal Boolean return in the conditional-output
                        // contract. An adapter call would erase the verifier's false-path
                        // proof even though CLI represents this literal as Int32.
                        if (libraryOwner is not null && method.Parameters.Count == 1
                            && GenericUnionLibrary.IsConditionalOutput(method, method.Parameters[0])
                            && number is 0 or 1 && instruction.Next?.OpCode.Code == Code.Ret)
                        { Push(new("Boolean")); code.AppendLine(number == 0 ? "ldc.bool false" : "ldc.bool true"); }
                        else { Push(new("Int32")); code.AppendLine($"ldc.i4 {number}"); }
                        break;
                    case Code.Ldarg_0: case Code.Ldarg_1: case Code.Ldarg_2: case Code.Ldarg_3: Arg((int)instruction.OpCode.Code - (int)Code.Ldarg_0); break;
                    case Code.Ldarg: case Code.Ldarg_S: Arg(((ParameterDefinition)instruction.Operand).Index + (method.HasThis ? 1 : 0)); break;
                    case Code.Ldarga: case Code.Ldarga_S:
                        var parameter = ((ParameterDefinition)instruction.Operand).Index + (method.HasThis ? 1 : 0);
                        if (parameter < 0 || parameter >= args.Length || !(ApplicationTypes.IsType(args[parameter]) || EnumBindings.IsType(args[parameter]) || PrimitiveBindings.IsReceiver(args[parameter]) || CalendarBindings.Types.Contains(args[parameter]) || ErrorBindings.IsType(args[parameter]) || GenericUnionBindings.IsType(args[parameter])))
                            throw new InvalidDataException("Only admitted primitive argument addresses supported.");
                        Push(new(args[parameter] + "&", Argument: parameter)); code.AppendLine($"ldarga {parameter}"); break;
                    case Code.Ldloc_0: case Code.Ldloc_1: case Code.Ldloc_2: case Code.Ldloc_3: Load((int)instruction.OpCode.Code - (int)Code.Ldloc_0); break;
                    case Code.Ldloc: case Code.Ldloc_S: Load(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Stloc_0: case Code.Stloc_1: case Code.Stloc_2: case Code.Stloc_3: Store((int)instruction.OpCode.Code - (int)Code.Stloc_0); break;
                    case Code.Stloc: case Code.Stloc_S: Store(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Ldloca: case Code.Ldloca_S:
                        var local = Local(((VariableDefinition)instruction.Operand).Index);
                        Push(new(locals[local] + "&", local)); code.AppendLine($"ldloca local{local}"); break;
                    case Code.Ldftn:
                    case Code.Ldvirtftn:
                        var functionReference = (MethodReference)instruction.Operand;
                        ApplicationTypes.CheckMethod(functionReference);
                        var functionTarget = ClosureAudit.ResolveMethod(functionReference) ?? throw new InvalidDataException("Unresolved delegate target.");
                        var virtualFunction = instruction.OpCode.Code == Code.Ldvirtftn;
                        // A bounded generic library callback retains its constructed receiver.
                        // The adapter itself is emitted once on the generic type definition.
                        var genericLibraryCallback = libraryOwner is not null && !virtualFunction
                            && functionReference.DeclaringType is GenericInstanceType
                            && ApplicationTypes.IsLibrary(functionReference.DeclaringType)
                            && functionTarget.HasThis && !functionTarget.HasParameters
                            && functionTarget.ReturnType.MetadataType == MetadataType.Void
                            && ApplicationTypes.Matches(functionReference, functionTarget);
                        if (!ApplicationTypes.IsModule(functionTarget.Module) || (!genericLibraryCallback && functionReference.FullName != functionTarget.FullName)
                            || functionTarget.IsConstructor || (!functionTarget.HasBody && !virtualFunction)
                            || functionTarget.DeclaringType.IsValueType && functionTarget.HasThis)
                            throw new InvalidDataException("Only static or class application delegate targets are admitted: " + functionReference.FullName + "; " + instruction.OpCode);
                        if (!functionTarget.IsPublic && functionTarget.DeclaringType != method.DeclaringType
                            && !(functionTarget.IsAssembly && functionTarget.Module == method.Module))
                            throw new InvalidDataException("Nonpublic cross-type delegate target unsupported.");
                        if (virtualFunction)
                        {
                            if (!functionTarget.HasThis) throw new InvalidDataException("Virtual function address requires receiver.");
                            var receiverType = ApplicationTypes.Receiver(functionTarget);
                            Expect(receiverType);
                            var checkName = "CheckDelegateReceiver_" + MetadataIdentity.TypeName(functionTarget.DeclaringType).Replace('.', '_');
                            delegateAdapters[checkName] = $".function {checkName}({receiverType}) -> void\n.local {receiverType} empty\nldloca empty\ninitobj {receiverType}\nldarg 0\nldloc empty\nref.eq\nbrfalse Valid\nfault \"null delegate receiver\"\nValid:\nret\n.end\n";
                            code.AppendLine($"call {checkName}({receiverType})");
                        }
                        Push(new("FunctionAddress", Function: functionTarget, VirtualFunction: virtualFunction,
                            FunctionReceiver: genericLibraryCallback ? ApplicationTypes.Receiver(functionReference) : null)); break;
                    case Code.Newobj:
                        var constructor = (MethodReference)instruction.Operand;
                        var constructorDefinition = constructor.Resolve() ?? throw new InvalidDataException("Unresolved constructor.");
                        if (ArrayLibrary.IsMatched(constructorDefinition.DeclaringType)) throw new InvalidDataException("Managed arrays require intrinsic allocation.");
                        if (ApplicationTypes.IsLibrary(constructor.DeclaringType)
                            && constructor.DeclaringType.FullName == "System.String")
                            throw new InvalidDataException("Opaque library storage requires a runtime factory.");
                        if (ApplicationTypes.IsModule(constructorDefinition.Module))
                        {
                            ApplicationTypes.CheckMethod(constructor);
                            if (!constructorDefinition.IsPublic && constructorDefinition.DeclaringType != method.DeclaringType && !InternalLibraryAccess(constructorDefinition, method)) throw new InvalidDataException("Nonpublic application constructor unsupported.");
                            if (!constructorDefinition.IsConstructor || constructorDefinition.IsStatic || !ApplicationTypes.Matches(constructor, constructorDefinition)) throw new InvalidDataException("Invalid application constructor.");
                            var ctorArgs = constructor.Parameters.Select(p => ProfileType(ApplicationTypes.Close(p.ParameterType, constructor.DeclaringType))).ToArray();
                            var actualCtorArgs = new string[ctorArgs.Length];
                            for (var n = ctorArgs.Length - 1; n >= 0; n--)
                                actualCtorArgs[n] = Argument(ctorArgs[n]).Type;
                            var owner = ProfileType(constructor.DeclaringType);
                            pending.Enqueue(constructorDefinition); Push(new(owner));
                            Call constructionCall;
                            if (libraryOwner is null && constructorDefinition.DeclaringType.IsValueType)
                            {
                                var factory = "Create" + Name(constructorDefinition);
                                var body = new StringBuilder($".function {factory}({string.Join(',', ctorArgs)}) -> {owner}\n.local {owner} value\nldloca value\ninitobj {owner}\nldloca value\n");
                                for (var n = 0; n < ctorArgs.Length; n++) body.AppendLine($"ldarg {n}");
                                body.AppendLine($"call {Name(constructorDefinition)}({string.Join(',', new[] { owner + "&" }.Concat(ctorArgs))})\nldloc value\nret\n.end");
                                delegateAdapters[factory] = body.ToString();
                                constructionCall = new(factory, ctorArgs, owner);
                            }
                            else constructionCall = new("", ctorArgs, owner,
                                Instruction: $"newobj instance {owner}::.ctor({string.Join(',', ctorArgs)})");
                            constructionCall = Coerce(constructionCall, actualCtorArgs);
                            code.AppendLine(constructionCall.Instruction ?? $"call {constructionCall.Name}({string.Join(',', constructionCall.Arguments)})");
                            break;
                        }
                        if (constructorDefinition.Module != library.MainModule) throw new InvalidDataException("Only admitted library constructors supported.");
                        if (DelegateBindings.Type(constructor.DeclaringType) is { } delegateType)
                        {
                            DelegateBindings.Constructor(constructor, constructorDefinition);
                            var addressSlot = Pop();
                            var function = addressSlot.Function ?? throw new InvalidDataException("Delegate requires an admitted function address.");
                            if (function.HasThis) Expect(addressSlot.FunctionReceiver ?? ApplicationTypes.Receiver(function));
                            else Expect("FaultNull");
                            var signature = DelegateBindings.Signature(delegateType);
                            var targetArguments = function.Parameters.Select(p => ProfileType(p.ParameterType)).ToArray();
                            var targetResult = ProfileType(function.ReturnType, true);
                            if (!targetArguments.SequenceEqual(signature[..^1]) || (targetResult != signature[^1] && !(targetResult == "noresult" && signature[^1] == "Void")))
                                throw new InvalidDataException("Delegate target signature mismatch.");
                            if (!function.IsAbstract) pending.Enqueue(function);
                            if (function.HasThis)
                            {
                                var owner = ProfileType(function.DeclaringType);
                                var adapterName = "DelegateTarget_" + MetadataIdentity.FunctionName(function) + (addressSlot.VirtualFunction ? "Virtual" : "Direct");
                                var callInstruction = addressSlot.VirtualFunction ? "callvirt" : "call";
                                var body = new StringBuilder($".method instance {adapterName}({string.Join(',', targetArguments)}) -> {signature[^1]}\nldarg 0\n");
                                var adapterOwner = owner;
                                if (function.DeclaringType.IsInterface)
                                {
                                    adapterOwner = "Application." + adapterName;
                                    body.AppendLine($"ldfld {adapterOwner}::Target");
                                }
                                for (var n = 0; n < targetArguments.Length; n++) body.AppendLine($"ldarg {n + 1}");
                                body.AppendLine($"{callInstruction} instance {owner}::{ApplicationTypes.MethodName(function)}({string.Join(',', targetArguments)})");
                                if (targetResult == "noresult") body.AppendLine("ldvoid");
                                body.AppendLine("ret\n.end");
                                if (function.DeclaringType.IsInterface)
                                {
                                    delegateAdapters[adapterOwner] = $".type class {adapterOwner}\n.field Target {owner}\n" + body + ".end\n";
                                    code.AppendLine("newobj " + adapterOwner);
                                }
                                else ApplicationTypes.AddAdapter(owner, adapterName, body.ToString());
                                code.AppendLine($"delegate.bind {delegateType} = instance {addressSlot.FunctionReceiver ?? adapterOwner}::{adapterName}({string.Join(',', targetArguments)})");
                                Push(new(delegateType)); break;
                            }
                            var targetName = Name(function);
                            if (targetResult == "noresult")
                            {
                                var adapterName = "RuntimeDelegate" + targetName;
                                var body = new StringBuilder($".function {adapterName}({string.Join(',', targetArguments.Select((t, i) => t + " arg" + i))}) -> Void\n");
                                for (var i = 0; i < targetArguments.Length; i++) body.AppendLine("ldarg arg" + i);
                                body.AppendLine($"call {targetName}({string.Join(',', targetArguments)})").AppendLine("ldvoid\nret\n.end");
                                delegateAdapters[adapterName] = body.ToString(); targetName = adapterName;
                            }
                            code.AppendLine($"delegate.bind {delegateType} = {targetName}({string.Join(',', targetArguments)})");
                            Push(new(delegateType)); break;
                        }
                        if (collectionProfile && (WorkerBindings.Bind(constructor, constructorDefinition, true, libraryOwner is not null) ?? AsyncBindings.Bind(constructor, constructorDefinition, true) ?? TaskBindings.Bind(constructor, constructorDefinition, true, libraryOwner is not null)) is { } taskConstruction)
                        {
                            for (var n = taskConstruction.Arguments.Length - 1; n >= 0; n--) Argument(taskConstruction.Arguments[n]);
                            Push(new(taskConstruction.Result));
                            code.AppendLine(taskConstruction.Instruction);
                            break;
                        }
                        if (collectionProfile && MapBindings.Construct(constructor, constructorDefinition) is { } mapConstruction)
                        {
                            for (var n = mapConstruction.Arguments.Length - 1; n >= 0; n--) Argument(mapConstruction.Arguments[n]);
                            Push(new(mapConstruction.Result));
                            code.AppendLine(mapConstruction.Instruction);
                            break;
                        }
                        if (collectionProfile && CollectionBindings.IsArrayList(CollectionBindings.Type(constructor.DeclaringType, libraryOwner is null ? null : t => ProfileType(t))))
                        {
                            var signature = RuntimeSignatures.Match(constructor, constructorDefinition, t => CollectionBindings.Type(t, libraryOwner is null ? null : p => ProfileType(p)), allowOpenMethodParameters: libraryOwner is not null);
                            if (!constructorDefinition.IsConstructor || !constructor.HasThis || signature.Result != "noresult"
                                || signature.Args.Length > 1 || signature.Args.Any(p => p != "Int32"))
                                throw new InvalidDataException("Unsupported collection constructor.");
                            if (signature.Args.Length == 1) Expect("Int32");
                            var collectionOwner = CollectionBindings.Type(constructor.DeclaringType, libraryOwner is null ? null : t => ProfileType(t))!;
                            Push(new(collectionOwner));
                            code.AppendLine($"newobj instance {collectionOwner}::.ctor({string.Join(',', signature.Args)})");
                            break;
                        }
                        var construction = Construct(constructor, constructorDefinition);
                        var constructedArguments = new string[construction.Arguments.Length];
                        for (var n = construction.Arguments.Length - 1; n >= 0; n--) constructedArguments[n] = Argument(construction.Arguments[n]).Type;
                        construction = Coerce(construction, constructedArguments);
                        Push(new(construction.Result));
                        code.AppendLine($"call {construction.Name}({string.Join(',', construction.Arguments)})"); break;
                    case Code.Add: case Code.Sub: case Code.Mul:
                        var numericRight = Pop(); var numericLeft = Pop();
                        if (numericLeft.Type != numericRight.Type || numericLeft.Type is not ("Int32" or "Int64" or "Double")) throw new InvalidDataException("Unsupported arithmetic operands.");
                        Push(new(numericLeft.Type)); code.AppendLine(instruction.OpCode.Name); break;
                    case Code.Div: case Code.Div_Un: case Code.Rem: case Code.Rem_Un:
                        var divisor = Pop(); var dividend = Pop();
                        if (dividend.Type != divisor.Type || dividend.Type is not ("Int32" or "Int64" or "Double")
                            || dividend.Type == "Double" && instruction.OpCode.Code is (Code.Div_Un or Code.Rem_Un))
                            throw new InvalidDataException("Unsupported division/remainder operands.");
                        Push(new(dividend.Type)); code.AppendLine(instruction.OpCode.Name); break;
                    case Code.Shl: case Code.Shr: case Code.Shr_Un:
                        var shiftCount = Pop(); var shifted = Pop();
                        if (shiftCount.Type != "Int32" || shifted.Type is not ("Int32" or "Int64"))
                            throw new InvalidDataException("Unsupported shift operands.");
                        Push(new(shifted.Type)); code.AppendLine(instruction.OpCode.Name); break;
                    case Code.Or: case Code.And: case Code.Xor:
                        var bitsRight = Argument("Int32"); var bitsLeft = Argument("Int32");
                        var bits = Coerce(new("RuntimeBits" + instruction.OpCode.Code, ["Int32", "Int32"], "Int32"), [bitsLeft.Type, bitsRight.Type]);
                        Push(new("Int32")); code.AppendLine($"call {bits.Name}({string.Join(',', bits.Arguments)})"); break;
                    case Code.Not:
                        ConvertTop("Int32"); Push(new("Int32")); code.AppendLine("not"); break;
                    case Code.Cgt: case Code.Cgt_Un: case Code.Clt: case Code.Clt_Un:
                        if (instruction.OpCode.Code == Code.Cgt_Un && stack.Count >= 2
                            && stack[^1].Type == "FaultNull" && ManagedArrayBindings.IsReference(stack[^2].Type))
                        {
                            Pop(); Pop(); Push(new("Int32"));
                            code.AppendLine("ref.isnull\nldc.bool false\nceq").Append(BooleanBindings.Convert("Boolean", "Int32"));
                            break;
                        }
                        if (!NumericOperands()) throw new InvalidDataException("Unsupported comparison operands.");
                        Pop(); Pop(); Push(new("Int32"));
                        code.AppendLine(instruction.OpCode.Name).Append(BooleanBindings.Convert("Boolean", "Int32"));
                        break;
                    case Code.Ceq:
                        if (NumericOperands() || stack.Count >= 2 && stack[^1].Type == "Char" && stack[^2].Type == "Char")
                        {
                            Pop(); Pop(); Push(new("Int32"));
                            code.AppendLine("ceq").Append(BooleanBindings.Convert("Boolean", "Int32"));
                            break;
                        }
                        var right = Argument("Int32"); var left = Argument("Int32"); Push(new("Int32"));
                        var equality = Coerce(new("RuntimeEqual", ["Int32", "Int32"], "Int32"), [left.Type, right.Type]);
                        code.AppendLine($"call {equality.Name}({string.Join(',', equality.Arguments)})"); break;
                    case Code.Pop: if (Pop().Type != "FaultNull") code.AppendLine("pop"); break;
                    case Code.Dup: var top = Pop(); Push(top); Push(top); if (top.Type != "FaultNull") code.AppendLine("dup"); break;
                    case Code.Leave: case Code.Leave_S:
                        // Bodies with exception handlers are rejected above. A stackless
                        // leave outside protected regions is an ordinary control transfer.
                        if (stack.Count != 0) throw new InvalidDataException("leave requires an empty stack in this profile.");
                        goto case Code.Br;
                    case Code.Br: case Code.Br_S:
                        var branch = Target(); successors.Add(branch); code.AppendLine($"br M{methodId:x8}_IL_{instructions[branch].Offset:x4}"); terminates = true; break;
                    case Code.Beq: case Code.Beq_S: case Code.Bne_Un: case Code.Bne_Un_S:
                    case Code.Bgt: case Code.Bgt_S: case Code.Bgt_Un: case Code.Bgt_Un_S:
                    case Code.Blt: case Code.Blt_S: case Code.Blt_Un: case Code.Blt_Un_S:
                    case Code.Bge: case Code.Bge_S: case Code.Bge_Un: case Code.Bge_Un_S:
                    case Code.Ble: case Code.Ble_S: case Code.Ble_Un: case Code.Ble_Un_S:
                        if (!NumericOperands()) throw new InvalidDataException("Unsupported comparison branch operands.");
                        Pop(); Pop();
                        var comparisonTarget = Target(); successors.Add(comparisonTarget);
                        var comparisonOpcode = instruction.OpCode.Name;
                        if (comparisonOpcode.EndsWith(".s", StringComparison.Ordinal)) comparisonOpcode = comparisonOpcode[..^2];
                        code.AppendLine($"{comparisonOpcode} M{methodId:x8}_IL_{instructions[comparisonTarget].Offset:x4}");
                        break;
                    case Code.Brtrue: case Code.Brtrue_S: case Code.Brfalse: case Code.Brfalse_S:
                        var condition = Pop();
                        var referenceCondition = ManagedArrayBindings.IsReference(condition.Type);
                        if (referenceCondition) code.AppendLine("ref.isnull");
                        if (!referenceCondition && condition.Type is not ("Int32" or "Boolean")) throw new InvalidDataException("Invalid branch condition.");
                        var conditional = Target(); successors.Add(conditional);
                        if (condition.ConditionalOut >= 0 && conditional != index + 1)
                        {
                            var success = (bool[])assigned.Clone(); success[condition.ConditionalOut] = true;
                            assignmentEdges[instruction.OpCode.Code is Code.Brtrue or Code.Brtrue_S ? conditional : index + 1] = success;
                        }
                        code.AppendLine($"{((instruction.OpCode.Code is Code.Brtrue or Code.Brtrue_S) != referenceCondition ? "brtrue" : "brfalse")} M{methodId:x8}_IL_{instructions[conditional].Offset:x4}"); break;
                    case Code.Call:
                    case Code.Callvirt:
                        var reference = (MethodReference)instruction.Operand;
                        var targetMethod = ClosureAudit.ResolveMethod(reference) ?? throw new InvalidDataException("Unresolved call.");
                        Call call;
                        if (targetMethod.Module == app.MainModule || guestLibraries.Contains(targetMethod.Module))
                        {
                            if (libraryOwner is not null && reference is GenericInstanceMethod generic)
                            {
                                LibraryImplementation.CheckMethod(targetMethod);
                                if (instruction.OpCode.Code != Code.Call || generic.HasThis || generic.ExplicitThis
                                    || generic.CallingConvention != MethodCallingConvention.Generic
                                    || !exports.Contains(targetMethod) || !targetMethod.IsStatic
                                    || generic.ElementMethod.FullName != targetMethod.FullName
                                    || generic.GenericArguments.Count != targetMethod.GenericParameters.Count)
                                    throw new InvalidDataException("Invalid generic library call.");
                                string Closed(TypeReference type, bool result = false)
                                    => ProfileType(LibraryImplementation.Close(type, generic), result);
                                var closedArguments = targetMethod.Parameters.Select(p => Closed(p.ParameterType)).ToArray();
                                call = new(libraryOwner + "." + targetMethod.Name + "<" + string.Join(',', generic.GenericArguments.Select(t => ProfileType(t))) + ">",
                                    closedArguments, Closed(targetMethod.ReturnType, true));
                                pending.Enqueue(targetMethod);
                            }
                            else
                            {
                                ApplicationTypes.CheckMethod(reference); ApplicationTypes.CheckMethod(targetMethod);
                                if (reference.HasThis != targetMethod.HasThis || instruction.OpCode.Code == Code.Callvirt && targetMethod.IsStatic) throw new InvalidDataException("Invalid application call receiver.");
                                if (!targetMethod.IsPublic && targetMethod.DeclaringType != method.DeclaringType && !InternalLibraryAccess(targetMethod, method) && !(targetMethod.IsAssembly && targetMethod.Module == method.Module)
                                    && !(DescriptorLibrary.IsBaseConstructor(targetMethod) && method.IsConstructor && method.DeclaringType.BaseType?.Resolve() == targetMethod.DeclaringType))
                                    throw new InvalidDataException("Nonpublic cross-type call unsupported.");
                                if (!ApplicationTypes.Matches(reference, targetMethod)) throw new InvalidDataException("Resolved signature mismatch.");
                                if (!targetMethod.IsAbstract) pending.Enqueue(targetMethod);
                                var parameters = reference.Parameters.Select(p => ProfileType(ApplicationTypes.Close(p.ParameterType, reference.DeclaringType))).ToArray();
                                call = targetMethod.HasThis && (libraryOwner is not null || !(targetMethod.IsConstructor && targetMethod.DeclaringType.IsValueType))
                                    ? new("", new[] { ApplicationTypes.Receiver(reference) }.Concat(parameters).ToArray(), ProfileType(ApplicationTypes.Close(reference.ReturnType, reference.DeclaringType), true), Instruction: $"{(instruction.OpCode.Code == Code.Callvirt ? "callvirt" : "call")} instance {ProfileType(reference.DeclaringType)}::{ApplicationTypes.MethodName(targetMethod)}({string.Join(',', parameters)})" + (targetMethod.DeclaringType.IsValueType && targetMethod.ReturnType.MetadataType == MetadataType.Void ? "\npop" : ""))
                                    : new(libraryOwner is not null && targetMethod.IsStatic && (WorkerBindings.IsName(targetMethod.DeclaringType.FullName) || AsyncBindings.IsName(targetMethod.DeclaringType.FullName) || targetMethod.DeclaringType.IsValueType || DescriptorLibrary.IsProvider(targetMethod.DeclaringType)) ? ProfileType(reference.DeclaringType) + "::" + targetMethod.Name : Name(targetMethod), targetMethod.HasThis ? new[] { ApplicationTypes.Receiver(reference) }.Concat(parameters).ToArray() : parameters, ProfileType(ApplicationTypes.Close(reference.ReturnType, reference.DeclaringType), true));
                            }
                        }
                        else if (targetMethod.Module == library.MainModule)
                        {
                            if (targetMethod.IsConstructor && targetMethod.DeclaringType.FullName is "System.Object" or "System.ValueType" && targetMethod.Parameters.Count == 0 && method.IsConstructor && method.DeclaringType.BaseType?.FullName == targetMethod.DeclaringType.FullName && instruction.OpCode.Code == Code.Call)
                            { Expect(ApplicationTypes.Receiver(method)); code.AppendLine("pop"); break; }
                            if (libraryOwner is not null && reference.DeclaringType.FullName == "System.Runtime.CompilerServices.ValueStorage"
                                && reference.Name == "LeaveUnassigned")
                            {
                                if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference is not GenericInstanceMethod untouched
                                    || untouched.GenericArguments.Count != 1 || targetMethod.Parameters.Count != 1 || !targetMethod.Parameters[0].IsOut
                                    || method.Parameters.Count != 1 || !GenericUnionLibrary.IsConditionalOutput(method, method.Parameters[0]))
                                    throw new InvalidDataException("Only conditional union outputs may remain unassigned.");
                                var untouchedSignature = RuntimeSignatures.Match(reference, targetMethod, t => ProfileType(t), allowOpenMethodParameters: true);
                                if (untouchedSignature.Result != "noresult" || untouchedSignature.Args.Length != 1
                                    || untouchedSignature.Args[0] != args[1] || Expect(args[1]).Argument != 1)
                                    throw new InvalidDataException("Invalid conditional output address.");
                                var missReturn = instructions.Skip(index + 1).Where(i => i.OpCode.Code != Code.Nop).Take(2).ToArray();
                                if (missReturn.Length != 2 || missReturn[0].OpCode.Code != Code.Ldc_I4_0 || missReturn[1].OpCode.Code != Code.Ret)
                                    throw new InvalidDataException("Unassigned conditional output must return false immediately.");
                                code.AppendLine("pop"); break;
                            }
                            var runtimeService = libraryOwner is null ? null : RuntimeFailureBindings.Bind(reference, targetMethod, t => ProfileType(t)) ?? NativeAllocationBindings.Bind(reference, targetMethod, t => ProfileType(t)) ?? ParameterSnapshotBindings.Bind(reference, targetMethod, t => ProfileType(t)) ?? RuntimeServiceBindings.Bind(reference, targetMethod) ?? ValueStorageBindings.Bind(reference, targetMethod, t => ProfileType(t));
                            var checkedStorage = libraryOwner is null ? null : CheckedStorageBindings.Bind(reference, targetMethod, t => ProfileType(t));
                            var interfaceCall = collectionProfile ? InterfaceBindings.Bind(reference, targetMethod) : null;
                            var nativeCall = collectionProfile ? NativeMemoryBindings.Bind(reference, targetMethod) : null;
                            var reflectionCall = collectionProfile ? ReflectionBindings.Bind(reference, targetMethod) : null;
                            var queryCall = collectionProfile ? QueryBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt)
                                ?? OutcomeOperatorBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt) : null;
                            var arrayCallback = collectionProfile ? ArrayCallbackBindings.Bind(reference, targetMethod, libraryOwner is null ? null : t => ProfileType(t)) : null;
                            var delegateCall = DelegateBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt);
                            if (runtimeService is not null) call = new(runtimeService.Name, runtimeService.Arguments, runtimeService.Result, Instruction: runtimeService.Instruction);
                            else if (checkedStorage is not null) call = new(checkedStorage.Name, checkedStorage.Arguments, checkedStorage.Result, Instruction: checkedStorage.Instruction);
                            else if (interfaceCall is not null) call = new(interfaceCall.Name, interfaceCall.Arguments, interfaceCall.Result, Instruction: interfaceCall.Instruction);
                            else if (nativeCall is not null) call = new(nativeCall.Name, nativeCall.Arguments, nativeCall.Result);
                            else if (reflectionCall is not null) call = new(reflectionCall.Name, reflectionCall.Arguments, reflectionCall.Result);
                            else if (queryCall is not null) call = new(queryCall.Name, queryCall.Arguments, queryCall.Result);
                            else if (arrayCallback is not null) call = new(arrayCallback.Name, arrayCallback.Arguments, arrayCallback.Result, Instruction: arrayCallback.Instruction);
                            else if (delegateCall is not null) call = new(delegateCall.Name, delegateCall.Arguments, delegateCall.Result, Instruction: delegateCall.Instruction);
                            else
                            {
                                var binding = collectionProfile ? CollectionBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt, libraryOwner is null ? null : t => ProfileType(t)) : null;
                                var taskBinding = collectionProfile ? WorkerBindings.Bind(reference, targetMethod, false, libraryOwner is not null) ?? AsyncBindings.Bind(reference, targetMethod, false) ?? TaskBindings.Bind(reference, targetMethod, false, libraryOwner is not null) : null;
                                var textBinding = StringBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt);
                                if (taskBinding is not null) call = new("", taskBinding.Arguments, taskBinding.Result, Instruction: taskBinding.Instruction);
                                else if (textBinding is not null) call = new("", textBinding.Arguments, textBinding.Result, Instruction: textBinding.Instruction);
                                else if (binding is not null) call = new("", binding.Arguments, binding.Result, Instruction: binding.Instruction);
                                else
                                {
                                    if (instruction.OpCode.Code == Code.Callvirt) throw new InvalidDataException("Unsupported runtime callvirt: " + reference.FullName);
                                    call = Bind(reference, targetMethod);
                                }
                            }
                        }
                        else throw new InvalidDataException("Unsupported dependency call.");
                        var conditionalOut = -1;
                        var actualArguments = new string[call.Arguments.Length];
                        for (var n = call.Arguments.Length - 1; n >= 0; n--)
                        {
                            var argument = Argument(call.Arguments[n]);
                            actualArguments[n] = argument.Type;
                            if (argument.Type.EndsWith('&'))
                            {
                                if (argument.Argument >= 0)
                                {
                                    if (n != 0 || !reference.HasThis || !(ApplicationTypes.Type(reference.DeclaringType) is not null || PrimitiveBindings.IsReceiver(reference.DeclaringType.Name) || CalendarBindings.Types.Contains(reference.DeclaringType.FullName) || ErrorBindings.IsType(reference.DeclaringType.FullName.Replace('/', '.')) || GenericUnionBindings.IsType(GenericUnionBindings.Type(reference.DeclaringType) ?? "")))
                                        throw new InvalidDataException("Argument addresses are only admitted as primitive receivers.");
                                    continue;
                                }
                                if (argument.Local < 0) throw new InvalidDataException("Only local addresses admitted.");
                                if (n == call.OutArgument)
                                {
                                    if (call.ConditionalOutput) conditionalOut = argument.Local;
                                    else assigned[argument.Local] = true;
                                }
                                else if (!assigned[argument.Local]) throw new InvalidDataException($"Read through uninitialized carrier address in {method.Name} at {instruction.Offset:x4}, local {argument.Local}: {reference.FullName}.");
                            }
                        }
                        if (call.Result != "noresult") Push(new(call.Result == "Boolean" && conditionalOut < 0 ? "Int32" : PrimitiveBindings.Stack(call.Result), ConditionalOut: conditionalOut));
                        call = Coerce(call, actualArguments);
                        code.AppendLine(call.Instruction ?? $"call {call.Name}({string.Join(',', call.Arguments)})");
                        // Conditional-out results must reach their branch directly so the runtime
                        // verifier retains the relationship between success and assignment.
                        if (call.Result == "Boolean" && conditionalOut < 0) code.Append(BooleanBindings.Convert("Boolean", "Int32"));
                        break;
                    case Code.Ret:
                        if (result != "noresult") ConvertTop(result);
                        if (stack.Count != 0) throw new InvalidDataException($"Input ret stack not empty in {method.FullName}, expected {result}, remaining {string.Join(',', stack.Select(s => s.Type))}.");
                        if ((emitInstance && method.DeclaringType.IsValueType || libraryOwner == "System.Console") && result == "noresult") code.AppendLine("ldvoid");
                        code.AppendLine("ret"); terminates = true; break;
                    default: throw new InvalidDataException("Unsupported reachable instruction: " + instruction.OpCode);
                }
                bodies[index] = code.ToString();
                if (!terminates) successors.Add(index + 1);
                foreach (var successor in successors) Merge(successor, new(stack, assignmentEdges.GetValueOrDefault(successor, assigned)));
            }
            var methodStart = output.Length;
            // Unreachable guest instructions are omitted, not admitted as executable code.
            // Library metadata keeps author-supplied parameter names for introspection.
            var declaredParameters = args.Skip(method.HasThis ? 1 : 0).Select((t, i) =>
                libraryOwner is not null ? (GenericUnionLibrary.IsConditionalOutput(method, method.Parameters[i]) ? "out(true) " : "") + t + " " + OpaqueLibrary.ParameterName(method, i) : t);
            output.AppendLine(libraryOwner is not null && !emitInstance && !emitOwnedStatic ? $".function {(method.IsAssembly ? "internal " : "")}{Name(method)}({string.Join(',', args.Select((t, i) => t + " " + method.Parameters[i].Name))}) -> {(libraryOwner == "System.Console" && result == "noresult" ? "Void" : result)}" : emitOwnedStatic ? $".method {(method.IsPrivate ? "private " : method.IsAssembly ? "internal " : "")}static {method.Name}({string.Join(',', declaredParameters)}) -> {result}" : emitInstance ? $".method {(libraryOwner is not null && method.IsPrivate ? "private " : (DescriptorLibrary.IsBaseConstructor(method) || libraryOwner is not null && method.IsAssembly) ? "internal " : "")}instance {(LibraryImplementation.IsReadonlyReceiver(method) ? "readonly " : "")}{((method.DeclaringType.IsValueType && !LibraryImplementation.IsByValueReceiver(method) || OpaqueLibrary.IsByRefString(method)) ? "byref " : "")}{ApplicationTypes.Modifiers(method)}{ApplicationTypes.MethodName(method)}({string.Join(',', declaredParameters)}) -> {(method.DeclaringType.IsValueType && result == "noresult" ? "Void" : result)}" : $".function {Name(method)}({string.Join(',', args)}) -> {result}");
            if (libraryOwner is null) output.AppendLine(SourceMetadata.Method(method, explicitReceiver: method.HasThis && !emitInstance));
            for (var n = 0; n < locals.Length; n++) output.AppendLine($".local {locals[n]} local{n}");
            if (method.Body.InitLocals)
                for (var n = 0; n < locals.Length; n++)
                {
                    if (ApplicationTypes.LibraryUnionRequiresInitialization(locals[n]) || ErrorBindings.Cases.TryGetValue(locals[n], out var errorCases) && errorCases.Length > 0) continue;
                    if (TaskBindings.IsType(locals[n]) || ApplicationTypes.IsType(locals[n]) || ReflectionBindings.IsType(locals[n]) && locals[n] != "System.RuntimeTypeHandle" || ManagedArrayBindings.IsType(locals[n]) || CollectionBindings.IsReference(locals[n]) || CalendarBindings.Types.Contains(locals[n]) || GenericUnionBindings.IsType(locals[n]) && !GenericUnionBindings.RequiresInitialization(locals[n])) output.AppendLine($"ldloca local{n}\ninitobj {locals[n]}");
                    else if (!method.GenericParameters.Concat(method.DeclaringType.GenericParameters).Any(p => locals[n] == "T" + p.Position) && locals[n] != Carrier && locals[n] != Option && locals[n] != VoidOption && locals[n] != VoidResult && locals[n] != "String" && !NeedsInitialization(locals[n])) output.AppendLine(Default(locals[n]) + $"\nstloc local{n}");
                }
            foreach (var index in bodies.Keys.Order())
            {
                mappings.Add((method, methodId, instructions[index].Offset));
                output.AppendLine($"M{methodId:x8}_IL_{instructions[index].Offset:x4}:").Append(bodies[index]);
            }
            output.AppendLine(".end");
            if (emitInstance || emitOwnedStatic) { instanceBodies[method] = output.ToString(methodStart, output.Length - methodStart); output.Length = methodStart; }

            ApplicationTypes.Expand(ProfileType, pending);

            void Merge(int index, State next)
            {
                if (index < 0 || index >= instructions.Length) throw new InvalidDataException("Method falls through.");
                if (!states.TryGetValue(index, out var previous))
                { states[index] = new(next.Stack.ToList(), (bool[])next.Assigned.Clone()); work.Enqueue(index); return; }
                if (!previous.Stack.SequenceEqual(next.Stack)) throw new InvalidDataException($"Incompatible branch stack merge in {method.FullName} at {instructions[index]}: [{string.Join(",", previous.Stack)}] versus [{string.Join(",", next.Stack)}].");
                var changed = false;
                for (var n = 0; n < previous.Assigned.Length; n++)
                    if (previous.Assigned[n] && !next.Assigned[n]) { previous.Assigned[n] = false; changed = true; }
                if (changed) work.Enqueue(index);
            }
        }
        if (libraryOwner is not null)
        {
            if (!ApplicationTypes.OnlyLibraryTypes) throw new InvalidDataException("Library fragments cannot introduce application type identities.");
        }
        output.Append(ApplicationTypes.Declarations(ProfileType, instanceBodies));
        output.Append(Adapters()).Append(ResultBindings.Adapters()).Append(StringBindings.Adapters()).AppendLine(Int32Bindings.Adapters).AppendLine(DoubleBindings.Adapters).Append(PrimitiveBindings.Adapters).Append(CalendarBindings.Adapters).Append(ErrorBindings.Adapters()).Append(GenericUnionBindings.Adapters).AppendLine(ProcessBindings.Adapters(collectionProfile)).AppendLine(BooleanBindings.Adapters).AppendLine(ReflectionBindings.Adapters).AppendLine(EnumBindings.Adapters);
        foreach (var helper in coercions.Values) output.Append(helper.Body);
        output.Append(RuntimeServiceBindings.Adapters);
        foreach (var body in delegateAdapters.Values) output.Append(body);
        var generated = libraryOwner is null ? output.ToString() : LibraryImplementation.QualifyHelpers(output.ToString(), libraryOwner);
        var labelLines = generated.Split('\n').Select((line, index) => (line, index))
            .Where(p => System.Text.RegularExpressions.Regex.IsMatch(p.line, @"^M[0-9a-f]{8}_IL_[0-9a-f]{4}:$"))
            .ToDictionary(p => p.line, p => p.index + 1);
        File.WriteAllText(destination, generated);
        File.WriteAllText(destination + ".map.json", JsonSerializer.Serialize(new {
            IdentityEncoding = "assembly-signature-v1",
            AssemblyIdentity = app.Name.FullName, TypeIdentities = ApplicationTypes.IdentityMap(),
            MethodIdentities = seen
                .Select(m => new { AssemblyIdentity = m.Module.Assembly.Name.FullName, MethodToken = m.MetadataToken.ToUInt32(), MetadataName = m.FullName,
                    RuntimeName = (libraryOwner is not null && m.DeclaringType.IsValueType) || m.HasThis && (libraryOwner is not null || !(m.IsConstructor && m.DeclaringType.IsValueType))
                        ? ApplicationTypes.Type(m.DeclaringType) + "::" + ApplicationTypes.MethodName(m) : Name(m) }),
            Profile = libraryOwner is not null ? ((exports.Any(m => m.HasThis) || ApplicationTypes.IdentityMap().Length != 0) ? "instance-library-fragment-v1" : "namespace-library-fragment-v1") : collectionProfile ? "result-option-void-instance-libraries-v11" : "result-option-void-files-strings-arrays-v7",
            RequiredLibraryProfile = collectionProfile ? "raven-collections" : "bundled-system", ApplicationSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(application))),
            CoreSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(core))), DependencyImages = dependencies.Select(path => new { AssemblyIdentity = System.Reflection.AssemblyName.GetAssemblyName(path).FullName,
                Sha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))) }),
            ReachableMethods = seen.Select(m => new { AssemblyIdentity = m.Module.Assembly.Name.FullName, MethodToken = m.MetadataToken.ToUInt32() }),
            Mappings = mappings.Select(m => new { AssemblyIdentity = m.Method.Module.Assembly.Name.FullName, MethodToken = m.Method.MetadataToken.ToUInt32(), m.Offset,
                OutputLabel = $"M{m.MethodId:x8}_IL_{m.Offset:x4}:", OutputLine = labelLines[$"M{m.MethodId:x8}_IL_{m.Offset:x4}:"] }),
            Scope = "Bounded application class/value fields, constructors and instance methods; Int32/String vectors, optional closed collection references, file UTF-8 APIs, String helpers and generic Result/Option bindings; CFG stack/definite-assignment checked; observable default carriers rejected; explicit nongeneric library types and bodies admitted; no reference-core declaration bodies executed."
        }, new JsonSerializerOptions { WriteIndented = true }));
    }

    // Raven materializes pattern-test success as 1/0 before branching again.
    // Thread this exact diamond so conditional-out proof reaches the matched arm.
    // The false block and join must have no other incoming branch edges.
    static void NormalizePatternBranches(MethodDefinition method)
    {
        var instructions = method.Body.Instructions;
        foreach (var join in instructions.Where(i => i.OpCode.Code is Code.Brfalse or Code.Brfalse_S or Code.Brtrue or Code.Brtrue_S).ToArray())
        {
            var miss = join.Previous;
            var jump = miss?.Previous;
            var hit = jump?.Previous;
            if (join.Next is null || miss?.OpCode.Code != Code.Ldc_I4_0 || hit?.OpCode.Code != Code.Ldc_I4_1
                || jump is null || jump.OpCode.Code is not (Code.Br or Code.Br_S) || jump.Operand != join) continue;
            var incomingMiss = instructions.Where(i => ReferenceEquals(i.Operand, miss)).ToArray();
            if (incomingMiss.Length == 0 || incomingMiss.Any(i => i.OpCode.Code is not (Code.Brfalse or Code.Brfalse_S))
                || instructions.Count(i => ReferenceEquals(i.Operand, join)) != 1
                || instructions.Any(i => ReferenceEquals(i.Operand, jump))
                || instructions.Any(i => i.Operand is Instruction[] targets && targets.Any(t => t == miss || t == join || t == jump))) continue;
            var branchesOnSuccess = join.OpCode.Code is Code.Brtrue or Code.Brtrue_S;
            foreach (var incoming in incomingMiss)
                incoming.Operand = branchesOnSuccess ? join.Next : join.Operand;
            hit.OpCode = OpCodes.Nop;
            jump.Operand = branchesOnSuccess ? join.Operand : join.Next;
        }
    }

    static void CheckStatic(MethodReference method)
    {
        if (method.HasThis || method.ExplicitThis || method.HasGenericParameters || method is GenericInstanceMethod
            || method.DeclaringType.HasGenericParameters || method.DeclaringType is GenericInstanceType
            || method.CallingConvention != MethodCallingConvention.Default)
            throw new InvalidDataException("Unsupported application signature.");
    }
    static string Type(TypeReference type, bool result = false) => type.FullName switch {
        "System.Object" when type.MetadataType == MetadataType.Object || RuntimeSignatures.IsCore(type.Scope) => "System.Object",
        "System.Int32[]" when type is ArrayType { IsVector: true } array && array.ElementType.MetadataType == MetadataType.Int32 => IntArray,
        "System.Unit" when type.IsValueType && type.Resolve() is { } unit && !unit.Fields.Any(f => !f.IsStatic)
            && !unit.Methods.Any(m => m.IsConstructor && m.IsStatic) => "Void",
        "System.Void" when !result && type.IsValueType => "Void",
        "System.OverflowError" when type.IsValueType => Overflow,
        "System.Double" => "Double", "System.Boolean" => "Boolean", "System.Void" when result => "noresult", "System.Int32" => "Int32", "System.String" => "String",
        "System.Result`2<System.Void,System.OverflowError>" when type.IsValueType && type is GenericInstanceType g && g.GenericArguments[0].IsValueType && g.GenericArguments[0].Scope.Name == CoreDeclarations.Identity => VoidResult,
        "System.Result/Ok`1<System.Void>" when type.IsValueType && HasNamedVoid(type) => VoidOk,
        "System.Option`1<System.Void>" when type.IsValueType && HasNamedVoid(type) => VoidOption,
        "System.Option/Some`1<System.Void>" when type.IsValueType && HasNamedVoid(type) => VoidSome,
        "System.Option`1<System.Int32>" when type.IsValueType => Option,
        "System.Option/Some`1<System.Int32>" when type.IsValueType => Some,
        "System.Option/None" when type.IsValueType => None,
        "System.Result`2<System.Int32,System.OverflowError>" when type.IsValueType => Carrier,
        "System.Result/Ok`1<System.Int32>" when type.IsValueType => Ok,
        "System.Result/Error`1<System.OverflowError>" when type.IsValueType => Error,
        _ => throw new InvalidDataException("Unsupported Result profile type: " + type.FullName)
    };
    static bool Converts(string source, string target) => source != target && target == "System.Object" && ManagedArrayBindings.IsReference(source) || ApplicationTypes.IsLibraryUnion(target) && source == target + "&" || InterfaceBindings.Converts(source, target) || BooleanBindings.Converts(source, target) || EnumBindings.Converts(source, target);
    static string ConvertStack(string source, string target) => (source != target && target == "System.Object" && ManagedArrayBindings.IsReference(source) ? "castclass System.Object\n" : "") + (ApplicationTypes.IsLibraryUnion(target) && source == target + "&" ? "ldobj " + target + "\n" : "") + InterfaceBindings.Convert(source, target) + BooleanBindings.Convert(source, target) + EnumBindings.Convert(source, target);
    static bool NeedsInitialization(string type) => ApplicationTypes.LibraryUnionRequiresInitialization(type) || CalendarBindings.IsReference(type) || type is "System.Object" or ParameterSnapshotBindings.Vector || InterfaceBindings.IsInterface(type) || NativeMemoryBindings.IsPointer(type) || type is "System.RuntimeTypeHandle" or "Value" || DelegateBindings.IsType(type) || (GenericUnionBindings.IsType(type)
        ? GenericUnionBindings.RequiresInitialization(type) : ResultBindings.RequiresInitialization(type));
    static bool HasNamedVoid(TypeReference type) => type is GenericInstanceType generic
        && generic.GenericArguments.Count == 1 && generic.GenericArguments[0].IsValueType
        && generic.GenericArguments[0].FullName == "System.Void"
        && generic.GenericArguments[0].Scope.Name == CoreDeclarations.Identity;
    static Call Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!definition.IsPublic || reference.ExplicitThis || reference is GenericInstanceMethod || reference.HasGenericParameters
            || reference.CallingConvention != MethodCallingConvention.Default || reference.HasThis != definition.HasThis)
            throw new InvalidDataException("Unsupported runtime signature.");
        // Reuse the declaration catalog for its bounded static Int32 APIs. Check
        // both sides before mapping a resolved CLI reference to the runtime library.
        var file = FaultBindings.Bind(reference, definition) ?? BooleanBindings.Bind(reference, definition) ?? ProcessBindings.Bind(reference, definition) ?? GenericUnionBindings.Bind(reference, definition) ?? ErrorBindings.Bind(reference, definition) ?? CalendarBindings.Bind(reference, definition) ?? PrimitiveBindings.Bind(reference, definition) ?? DoubleBindings.Bind(reference, definition) ?? Int32Bindings.Bind(reference, definition) ?? UnicodeScalarBindings.Bind(reference, definition) ?? Utf8Bindings.Bind(reference, definition) ?? PathBindings.Bind(reference, definition) ?? FileBindings.Bind(reference, definition) ?? ResultBindings.Bind(reference, definition);
        if (file is not null)
        {
            if ((file.OutArgument >= 0 && file.Result == "Boolean" || reference.Name == "FromResidual")
                && definition.DeclaringType.FullName != "System.Tasks.TaskOutcome`1") ValidatePropagation(definition.DeclaringType);
            return new(file.Name, file.Arguments, file.Result, file.OutArgument, file.Instruction, file.OutArgument >= 0 && file.Result == "Boolean");
        }
        if (NamespaceFunctions.Owner(reference.DeclaringType) == "System.Math" && reference.Name == "Clamp")
        {
            var signature = RuntimeSignatures.Match(reference, definition, ResultBindings.Type);
            if (reference.HasThis || !signature.Args.SequenceEqual(new[] { "Int32", "Int32", "Int32" })
                || signature.Result != "System.Result<Int32,System.InvalidRangeError>")
                throw new InvalidDataException("Unsupported Clamp signature.");
            return new("System.Math::Clamp", signature.Args, signature.Result);
        }
        var surface = TargetSurface.Bind(definition);
        if (surface is { Returns: "Int32" } && !reference.HasThis && !definition.HasThis
            && reference.FullName == definition.FullName)
            return new(surface.ImportTarget, surface.Parameters, surface.Returns);
        // Exact definition signatures include generic parameter positions, not just arity/names.
        var key = definition.FullName;
        if (reference.DeclaringType.FullName == "System.Result`2<System.Int32,System.OverflowError>"
            && definition.DeclaringType.FullName == "System.Result`2")
        {
            if (reference.Name is "TryGetOutput" or "TryGetResidual" or "FromResidual")
                ValidatePropagation(definition.DeclaringType);
            if (key == "System.Boolean System.Result`2::TryGetOutput(T&)" && reference.HasThis
                && definition.Parameters[0].IsOut && reference.ReturnType.MetadataType == MetadataType.Boolean
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!0&")
                return new("RuntimePropagationOutput", [Carrier + "&", "Int32&"], "Int32", 1);
            if (key == "System.Boolean System.Result`2::TryGetResidual(E&)" && reference.HasThis
                && definition.Parameters[0].IsOut && reference.ReturnType.MetadataType == MetadataType.Boolean
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!1&")
                return new("RuntimePropagationResidual", [Carrier + "&", Overflow + "&"], "Int32", 1);
            if (key == "System.Result`2<T,E> System.Result`2::FromResidual(E)" && !reference.HasThis
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!1"
                && reference.ReturnType.FullName == "System.Result`2<!0,!1>")
                return new($"System.Result<Int32,{Overflow}>::FromResidual", [Overflow], Carrier);
        }
        if (reference.DeclaringType.FullName == "System.Result`2<System.Void,System.OverflowError>"
            && definition.DeclaringType.FullName == "System.Result`2")
        {
            if (reference.Name is "TryGetOutput" or "TryGetResidual" or "FromResidual")
                ValidatePropagation(definition.DeclaringType);
            if (key == "System.Boolean System.Result`2::TryGetOutput(T&)" && reference.HasThis
                && definition.Parameters[0].IsOut && reference.ReturnType.MetadataType == MetadataType.Boolean
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!0&")
                return new("RuntimeVoidResultOutput", [VoidResult + "&", "Void&"], "Int32", 1);
            if (key == "System.Boolean System.Result`2::TryGetResidual(E&)" && reference.HasThis
                && definition.Parameters[0].IsOut && reference.ReturnType.MetadataType == MetadataType.Boolean
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!1&")
                return new("RuntimeVoidResultResidual", [VoidResult + "&", Overflow + "&"], "Int32", 1);
            if (key == "System.Result`2<T,E> System.Result`2::FromResidual(E)" && !reference.HasThis
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!1"
                && reference.ReturnType.FullName == "System.Result`2<!0,!1>")
                return new($"System.Result<Void,{Overflow}>::FromResidual", [Overflow], VoidResult);
        }
        if (reference.DeclaringType.FullName == "System.Option`1<System.Int32>" && definition.DeclaringType.FullName == "System.Option`1")
        {
            if (reference.Name is "TryGetOutput" or "TryGetResidual" or "FromResidual") ValidatePropagation(definition.DeclaringType);
            if (key == "System.Boolean System.Option`1::TryGetOutput(T&)" && reference.ReturnType.MetadataType == MetadataType.Boolean && reference.HasThis && definition.Parameters[0].IsOut
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "!0&")
                return new("RuntimeOptionOutput", [Option + "&", "Int32&"], "Int32", 1);
            if (key == "System.Boolean System.Option`1::TryGetResidual(System.Void&)" && reference.ReturnType.MetadataType == MetadataType.Boolean && reference.HasThis && definition.Parameters[0].IsOut
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "System.Void&")
                return new("RuntimeOptionResidual", [Option + "&", "Void&"], "Int32", 1);
            if (key == "System.Option`1<T> System.Option`1::FromResidual(System.Void)" && !reference.HasThis
                && reference.Parameters.Count == 1 && reference.Parameters[0].ParameterType.FullName == "System.Void"
                && reference.ReturnType.FullName == "System.Option`1<!0>")
                return new("System.Option<Int32>::FromResidual", ["Void"], Option);
        }
        if (NamespaceFunctions.Owner(definition.DeclaringType) == "System.Math" && definition.Name == "Abs"
            && key == $"System.Result`2<System.Int32,System.OverflowError> {definition.DeclaringType.FullName}::Abs(System.Int32)" && reference.FullName == key && !reference.HasThis)
            return new("System.Math::Abs", ["Int32"], Carrier);
        if (key == "System.Void System.Console::WriteLine(System.Int32)" && reference.FullName == key && !reference.HasThis)
            return new("RuntimeWriteInt32", ["Int32"], "noresult");
        if (key == "System.Void System.Console::WriteLine(System.String)" && reference.FullName == key && !reference.HasThis)
            return new("RuntimeWriteString", ["String"], "noresult");
        if (reference.DeclaringType.FullName == "System.Result`2<System.Int32,System.OverflowError>" && reference.HasThis
            && reference.ReturnType.FullName == "System.Boolean" && reference.Parameters.Count == 1
            && definition.Parameters.Count == 1 && definition.Parameters[0].IsOut)
        {
            if (key == "System.Boolean System.Result`2::TryGetValue(System.Result/Ok`1<T>&)" && reference.Parameters[0].ParameterType.FullName == "System.Result/Ok`1<!0>&")
                return new("RuntimeTryOk", [Carrier + "&", Ok + "&"], "Int32", 1);
            if (key == "System.Boolean System.Result`2::TryGetValue(System.Result/Error`1<E>&)" && reference.Parameters[0].ParameterType.FullName == "System.Result/Error`1<!1>&")
                return new("RuntimeTryError", [Carrier + "&", Error + "&"], "Int32", 1);
        }
        if (reference.DeclaringType.FullName == "System.Result`2<System.Void,System.OverflowError>" && reference.HasThis
            && reference.ReturnType.FullName == "System.Boolean" && reference.Parameters.Count == 1
            && definition.Parameters.Count == 1 && definition.Parameters[0].IsOut)
        {
            if (key == "System.Boolean System.Result`2::TryGetValue(System.Result/Ok`1<T>&)" && reference.Parameters[0].ParameterType.FullName == "System.Result/Ok`1<!0>&")
                return new("RuntimeTryVoidOk", [VoidResult + "&", VoidOk + "&"], "Int32", 1);
            if (key == "System.Boolean System.Result`2::TryGetValue(System.Result/Error`1<E>&)" && reference.Parameters[0].ParameterType.FullName == "System.Result/Error`1<!1>&")
                return new("RuntimeTryVoidError", [VoidResult + "&", Error + "&"], "Int32", 1);
        }
        if (reference.DeclaringType.FullName == "System.Result/Ok`1<System.Int32>" && reference.HasThis
            && key == "T System.Result/Ok`1::get_Value()" && reference.ReturnType is GenericParameter { Position: 0 }
            && reference.Parameters.Count == 0)
            return new("RuntimeOkValue", [Ok + "&"], "Int32");
        if (reference.DeclaringType.FullName == "System.Option`1<System.Int32>" && reference.HasThis
            && reference.ReturnType.FullName == "System.Boolean" && reference.Parameters.Count == 1
            && definition.Parameters.Count == 1 && definition.Parameters[0].IsOut)
        {
            if (key == "System.Boolean System.Option`1::TryGetValue(System.Option/Some`1<T>&)" && reference.Parameters[0].ParameterType.FullName == "System.Option/Some`1<!0>&")
                return new("RuntimeTrySome", [Option + "&", Some + "&"], "Int32", 1);
            if (key == "System.Boolean System.Option`1::TryGetValue(System.Option/None&)" && reference.Parameters[0].ParameterType.FullName == "System.Option/None&")
                return new("RuntimeTryNone", [Option + "&", None + "&"], "Int32", 1);
        }
        if (reference.DeclaringType.FullName == "System.Option/Some`1<System.Int32>" && reference.HasThis
            && key == "T System.Option/Some`1::get_Value()" && reference.ReturnType is GenericParameter { Position: 0 }
            && reference.Parameters.Count == 0)
            return new("RuntimeSomeValue", [Some + "&"], "Int32");
        if (reference.DeclaringType.FullName == "System.Option`1<System.Void>" && HasNamedVoid(reference.DeclaringType)
            && reference.HasThis && reference.ReturnType.FullName == "System.Boolean" && reference.Parameters.Count == 1
            && definition.Parameters.Count == 1 && definition.Parameters[0].IsOut)
        {
            if (key == "System.Boolean System.Option`1::TryGetValue(System.Option/Some`1<T>&)" && reference.Parameters[0].ParameterType.FullName == "System.Option/Some`1<!0>&")
                return new("RuntimeTryVoidSome", [VoidOption + "&", VoidSome + "&"], "Int32", 1);
            if (key == "System.Boolean System.Option`1::TryGetValue(System.Option/None&)" && reference.Parameters[0].ParameterType.FullName == "System.Option/None&")
                return new("RuntimeTryVoidNone", [VoidOption + "&", None + "&"], "Int32", 1);
        }
        throw new InvalidDataException("Unsupported runtime binding: " + reference.FullName + " definition=" + key);
    }
    static void ValidatePropagation(TypeDefinition carrier)
    {
        var contracts = carrier.Interfaces.Where(i => i.InterfaceType.FullName ==
            (carrier.FullName == "System.Option`1" ? "System.Propagatable`3<System.Option`1<T>,T,System.Void>" : "System.Propagatable`3<System.Result`2<T,E>,T,E>")).ToArray();
        if (contracts.Length != 1)
            throw new InvalidDataException("Missing or incompatible propagation carrier contract.");
        var contract = contracts[0].InterfaceType.Resolve();
        if (contract is null || contract.Module != carrier.Module || !contract.IsInterface
            || contract.GenericParameters.Count != 3 || contract.Interfaces.Count != 0
            || contract.GenericParameters.Any(p => p.Attributes != GenericParameterAttributes.NonVariant || p.HasConstraints)
            || contract.Methods.Count != 2)
            throw new InvalidDataException("Incompatible propagation interface.");
        foreach (var (name, position) in new[] { ("TryGetOutput", 1), ("TryGetResidual", 2) })
        {
            var methods = contract.Methods.Where(m => m.Name == name).ToArray();
            if (methods.Length != 1) throw new InvalidDataException("Missing propagation extraction member.");
            var method = methods[0];
            if (!method.IsPublic || !method.IsAbstract || !method.IsVirtual || !method.IsNewSlot || method.HasBody
                || !method.HasThis || method.HasGenericParameters || method.ReturnType.MetadataType != MetadataType.Boolean
                || method.Parameters.Count != 1 || !method.Parameters[0].IsOut
                || method.Parameters[0].ParameterType is not ByReferenceType { ElementType: GenericParameter p }
                || p.Type != GenericParameterType.Type || p.Position != position)
                throw new InvalidDataException("Incompatible propagation extraction signature.");
        }
    }

    static Call Construct(MethodReference reference, MethodDefinition definition)
    {
        if (!definition.IsConstructor || !definition.IsPublic || !reference.HasThis || reference.ExplicitThis
            || reference.HasGenericParameters || reference is GenericInstanceMethod || reference.CallingConvention != MethodCallingConvention.Default
            || reference.ReturnType.MetadataType != MetadataType.Void || reference.HasThis != definition.HasThis)
            throw new InvalidDataException("Unsupported constructor signature.");
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && reference.DeclaringType.FullName == "System.SystemClock")
        {
            var shape = RuntimeSignatures.Match(reference, definition, CalendarBindings.Type);
            if (shape.Args.Length != 0 || shape.Result != "noresult")
                throw new InvalidDataException("Unsupported SystemClock constructor.");
            return new("RuntimeNewSystemClock", [], "System.SystemClock");
        }
        var file = GenericUnionBindings.Construct(reference, definition) ?? ErrorBindings.Construct(reference, definition) ?? ResultBindings.Construct(reference, definition);
        if (file is not null) return new(file.Name, file.Arguments, file.Result);
        var key = definition.FullName;
        var owner = Type(reference.DeclaringType);
        var parameters = reference.Parameters.Select(p => p.ParameterType.FullName).ToArray();
        if (owner == VoidOk && key == "System.Void System.Result/Ok`1::.ctor(T)" && parameters.SequenceEqual(new[] { "!0" }))
            return new("RuntimeNewVoidOk", ["Void"], VoidOk);
        if (owner == VoidResult && key == "System.Void System.Result`2::.ctor(System.Result/Ok`1<T>)" && parameters.SequenceEqual(new[] { "System.Result/Ok`1<!0>" }))
            return new("RuntimeVoidResultOk", [VoidOk], VoidResult);
        if (owner == Ok && key == "System.Void System.Result/Ok`1::.ctor(T)" && parameters.SequenceEqual(new[] { "!0" }))
            return new("RuntimeNewOk", ["Int32"], Ok);
        if (owner == Carrier && key == "System.Void System.Result`2::.ctor(System.Result/Ok`1<T>)" && parameters.SequenceEqual(new[] { "System.Result/Ok`1<!0>" }))
            return new("RuntimeResultOk", [Ok], Carrier);
        if (owner == Some && key == "System.Void System.Option/Some`1::.ctor(T)" && parameters.SequenceEqual(new[] { "!0" }))
            return new("RuntimeNewSome", ["Int32"], Some);
        if (owner == None && key == "System.Void System.Option/None::.ctor()" && parameters.Length == 0)
            return new("RuntimeNewNone", [], None);
        if (owner == Option && key == "System.Void System.Option`1::.ctor(System.Option/Some`1<T>)" && parameters.SequenceEqual(new[] { "System.Option/Some`1<!0>" }))
            return new("RuntimeOptionSome", [Some], Option);
        if (owner == Option && key == "System.Void System.Option`1::.ctor(System.Option/None)" && parameters.SequenceEqual(new[] { "System.Option/None" }))
            return new("RuntimeOptionNone", [None], Option);
        if (owner == VoidSome && key == "System.Void System.Option/Some`1::.ctor(T)" && parameters.SequenceEqual(new[] { "!0" }))
            return new("RuntimeNewVoidSome", ["Void"], VoidSome);
        if (owner == VoidOption && key == "System.Void System.Option`1::.ctor(System.Option/Some`1<T>)" && parameters.SequenceEqual(new[] { "System.Option/Some`1<!0>" }))
            return new("RuntimeVoidOptionSome", [VoidSome], VoidOption);
        if (owner == VoidOption && key == "System.Void System.Option`1::.ctor(System.Option/None)" && parameters.SequenceEqual(new[] { "System.Option/None" }))
            return new("RuntimeVoidOptionNone", [None], VoidOption);
        throw new InvalidDataException("Unsupported constructor: " + reference.FullName + " definition=" + key);
    }
    static string Default(string type) => (EnumBindings.IsType(type) ? "ldc.i4 0\ncall " + type + "::FromValue(Int32)" : null) ?? (ErrorBindings.IsEmpty(type) ? "newobj " + type : null) ?? PrimitiveBindings.Default(type) ?? (type switch {
        VoidOk => $"ldvoid\nnewobj {VoidOk}",
        "Void" => "ldvoid",
        Overflow => "newobj System.OverflowError",
        "Double" => "ldc.r8 0", "Boolean" => "ldc.bool false", "Int32" => "ldc.i4 0", VoidSome => $"ldvoid\nnewobj {VoidSome}", Some => $"ldc.i4 0\nnewobj {Some}", None => $"newobj {None}", Ok => $"ldc.i4 0\nnewobj {Ok}",
        Error => $"newobj System.OverflowError\nnewobj {Error}",
        _ => throw new InvalidDataException("Unsupported default.")
    });
    static string Adapters()
    {
        var text = new StringBuilder();
        foreach (var (name, type) in new[] { ("Output", "Void"), ("Residual", Overflow) })
        {
            text.AppendLine($".function RuntimeVoidResult{name}({VoidResult}& source,out {type}& destination) -> Int32");
            text.AppendLine($"ldarg destination\n{Default(type)}\nstobj {type}");
            text.AppendLine($"ldarg source\nldarg destination\ncall instance {VoidResult}::TryGet{name}({type}&)");
            text.AppendLine("brfalse Miss\nldc.i4 1\nret\nMiss:\nldc.i4 0\nret\n.end");
        }
        text.AppendLine($".function RuntimeNewVoidOk(Void value) -> {VoidOk}\nldarg value\nnewobj instance {VoidOk}::.ctor(Void)\nret\n.end");
        text.AppendLine($".function RuntimeVoidResultOk({VoidOk} value) -> {VoidResult}\nldarg value\nnewobj instance {VoidResult}::.ctor({VoidOk})\nret\n.end");
        foreach (var (name, type) in new[] { ("Output", "Int32"), ("Residual", "Void") })
        {
            text.AppendLine($".function RuntimeOption{name}({Option}& source,out {type}& destination) -> Int32");
            text.AppendLine($"ldarg destination\n{Default(type)}\nstobj {type}");
            text.AppendLine($"ldarg source\nldarg destination\ncall instance {Option}::TryGet{name}({type}&)");
            text.AppendLine("brfalse Miss\nldc.i4 1\nret\nMiss:\nldc.i4 0\nret\n.end");
        }
        foreach (var (name, type) in new[] { ("Output", "Int32"), ("Residual", Overflow) })
        {
            // CLI out is initialized on both paths; the runtime contract is true-only.
            text.AppendLine($".function RuntimePropagation{name}({Carrier}& source,out {type}& destination) -> Int32");
            text.AppendLine($"ldarg destination\n{Default(type)}\nstobj {type}");
            text.AppendLine($"ldarg source\nldarg destination\ncall instance {Carrier}::TryGet{name}({type}&)");
            text.AppendLine("brfalse Miss\nldc.i4 1\nret\nMiss:\nldc.i4 0\nret\n.end");
        }
        foreach (var (name, type, carrier) in new[] { ("VoidOk", VoidOk, VoidResult), ("VoidError", Error, VoidResult), ("Ok", Ok, Carrier), ("Error", Error, Carrier), ("Some", Some, Option), ("None", None, Option), ("VoidSome", VoidSome, VoidOption), ("VoidNone", None, VoidOption) })
        {
            text.AppendLine($".function RuntimeTry{name}({carrier}& source,out {type}& destination) -> Int32");
            text.AppendLine($"ldarg destination\n{Default(type)}\nstobj {type}");
            text.AppendLine($"ldarg source\nldobj {carrier}\nldarg destination\ncall instance {carrier}::TryGet({type}&)");
            text.AppendLine("brfalse Miss\nldc.i4 1\nret\nMiss:\nldc.i4 0\nret\n.end");
        }
        text.AppendLine($".function RuntimeOkValue({Ok}& source) -> Int32\nldarg source\nldobj {Ok}\ncall instance {Ok}::get_Value()\nret\n.end");
        text.AppendLine($".function RuntimeSomeValue({Some}& source) -> Int32\nldarg source\nldobj {Some}\ncall instance {Some}::get_Value()\nret\n.end");
        text.AppendLine(".function RuntimeEqual(Int32 left,Int32 right) -> Int32\nldarg left\nldarg right\nceq\nbrfalse False\nldc.i4 1\nret\nFalse:\nldc.i4 0\nret\n.end");
        text.AppendLine($".function RuntimeNewOk(Int32 value) -> {Ok}\nldarg value\nnewobj instance {Ok}::.ctor(Int32)\nret\n.end");
        text.AppendLine($".function RuntimeResultOk({Ok} value) -> {Carrier}\nldarg value\nnewobj instance {Carrier}::.ctor({Ok})\nret\n.end");
        text.AppendLine($".function RuntimeNewSome(Int32 value) -> {Some}\nldarg value\nnewobj instance {Some}::.ctor(Int32)\nret\n.end");
        text.AppendLine($".function RuntimeNewNone() -> {None}\nnewobj instance {None}::.ctor()\nret\n.end");
        foreach (var (name, type) in new[] { ("Some", Some), ("None", None) })
            text.AppendLine($".function RuntimeOption{name}({type} value) -> {Option}\nldarg value\nnewobj instance {Option}::.ctor({type})\nret\n.end");
        text.AppendLine($".function RuntimeNewVoidSome(Void value) -> {VoidSome}\nldarg value\nnewobj instance {VoidSome}::.ctor(Void)\nret\n.end");
        foreach (var (name, type) in new[] { ("Some", VoidSome), ("None", None) })
            text.AppendLine($".function RuntimeVoidOption{name}({type} value) -> {VoidOption}\nldarg value\nnewobj instance {VoidOption}::.ctor({type})\nret\n.end");
        foreach (var type in new[] { "Int32", "String" })
            text.AppendLine($".function RuntimeWrite{type}({type} value) -> noresult\nldarg value\ncall System.Console::WriteLine({type})\npop\nret\n.end");
        return text.ToString();
    }
}
