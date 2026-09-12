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
    sealed record Slot(string Type, int Local = -1, int ConditionalOut = -1, int Argument = -1);
    sealed record State(List<Slot> Stack, bool[] Assigned);
    sealed record Call(string Name, string[] Arguments, string Result, int OutArgument = -1, string? Instruction = null, bool ConditionalOutput = false);

    public static void Write(string application, string core, string destination, bool collectionProfile = false)
    {
        foreach (var path in new[] { application, core })
            if (new FileInfo(path).Length > 16 * 1024 * 1024) throw new InvalidDataException("Image exceeds profile limit.");
        VoidStorageValidation.Check(application);
        var errors = ClosureAudit.Inspect(application, core);
        if (errors.Length != 0) throw new InvalidDataException(string.Join("\n", errors));
        using var resolver = new ClosureAudit.SuppliedAssemblies();
        resolver.Add(application); resolver.Add(core);
        var app = resolver.Images.Single(a => a.Name.FullName == System.Reflection.AssemblyName.GetAssemblyName(application).FullName);
        var library = resolver.Images.Single(a => a.Name.FullName == System.Reflection.AssemblyName.GetAssemblyName(core).FullName);
        if (app.MainModule.Types.Any(t => t.Name == "<Module>" && t.Methods.Any(m => m.IsConstructor && m.IsStatic)))
            throw new InvalidDataException("Module initializers unsupported.");
        var entry = app.EntryPoint ?? throw new InvalidDataException("Missing entry point.");
        if (entry.Parameters.Count != 0 || entry.ReturnType.MetadataType != MetadataType.Void)
            throw new InvalidDataException("Result profile requires a parameterless no-result entry.");
        if (collectionProfile) CollectionBindings.Validate(library.MainModule);
        string ProfileType(TypeReference type, bool result = false)
        {
            var collection = CollectionBindings.Type(type);
            if (collection is not null && collectionProfile) return collection;
            return ResultBindings.Type(type) ?? Type(type, result);
        }
        var output = new StringBuilder($".module ImportedUnion\n.entry {Name(entry)}\n");
        var mappings = new List<object>();
        var pending = new Queue<MethodDefinition>(); pending.Enqueue(entry);
        var seen = new HashSet<uint>();
        while (pending.TryDequeue(out var method))
        {
            if (!seen.Add(method.MetadataToken.ToUInt32())) continue;
            if (seen.Count > 128) throw new InvalidDataException("Method limit exceeded.");
            CheckStatic(method);
            if (!method.HasBody || method.IsPInvokeImpl || method.IsInternalCall || method.Body.HasExceptionHandlers
                || method.Body.CodeSize > 65536 || method.Body.MaxStackSize > 256 || method.Body.Variables.Count > 256 || method.Body.Instructions.Count == 0
                || method.DeclaringType.Methods.Any(m => m.IsConstructor && m.IsStatic))
                throw new InvalidDataException("Unsupported body: " + method.FullName);
            var args = method.Parameters.Select(p => ProfileType(p.ParameterType)).ToArray();
            var result = ProfileType(method.ReturnType, true);
            var locals = method.Body.Variables.Select(v => ProfileType(v.VariableType)).ToArray();
            if (locals.Any(t => !ResultBindings.IsType(t) && !CollectionBindings.IsReference(t) && t is not ("Boolean" or "Int32" or "String" or IntArray or Carrier or Ok or Error or Option or Some or None or VoidOption or VoidSome or Overflow or "Void" or VoidResult or VoidOk)))
                throw new InvalidDataException("Unsupported local default in Result profile.");
            NormalizePatternBranches(method);
            var instructions = method.Body.Instructions.ToArray();
            var indexes = instructions.Select((i, n) => (i, n)).ToDictionary(p => p.i, p => p.n);
            var states = new Dictionary<int, State>();
            var work = new Queue<int>();
            var bodies = new Dictionary<int, string>();
            Merge(0, new([], locals.Select(t => method.Body.InitLocals && t != Carrier && t != Option && t != VoidOption && t != VoidResult && t != "String" && !ResultBindings.IsType(t)).ToArray()));
            var visits = 0;
            while (work.TryDequeue(out var index))
            {
                if (++visits > 100000) throw new InvalidDataException("Control-flow analysis limit exceeded.");
                var input = states[index];
                var stack = input.Stack.ToList(); var assigned = (bool[])input.Assigned.Clone();
                var instruction = instructions[index]; var code = new StringBuilder(); var successors = new List<int>();
                var assignmentEdges = new Dictionary<int, bool[]>();
                if (stack.Any(slot => slot.ConditionalOut >= 0) && instruction.OpCode.Code is not (Code.Nop or Code.Brtrue or Code.Brtrue_S or Code.Brfalse or Code.Brfalse_S))
                    throw new InvalidDataException("Conditional extraction output must be tested before use.");
                void Push(Slot slot) { stack.Add(slot); if (stack.Count > method.Body.MaxStackSize) throw new InvalidDataException("Declared maxstack exceeded."); }
                Slot Pop() { if (stack.Count == 0) throw new InvalidDataException("Input stack underflow."); var top = stack[^1]; stack.RemoveAt(stack.Count - 1); return top; }
                Slot Expect(string type) { var top = Pop(); if (!CollectionBindings.Assignable(top.Type, type)) throw new InvalidDataException("Input stack type mismatch."); return top; }
                int Local(int n) { if (n < 0 || n >= locals.Length) throw new InvalidDataException("Invalid local index."); return n; }
                void Load(int n) { Local(n); if (!assigned[n]) throw new InvalidDataException("Read of uninitialized or unsupported default local."); Push(new(locals[n])); code.AppendLine($"ldloc local{n}"); }
                void Store(int n) { Local(n); Expect(locals[n]); assigned[n] = true; code.AppendLine($"stloc local{n}"); }
                void Arg(int n) { if (n < 0 || n >= args.Length) throw new InvalidDataException("Invalid parameter index."); Push(new(args[n])); code.AppendLine($"ldarg {n}"); }
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
                        if (initializedType is Carrier or Option or VoidOption or VoidResult || ResultBindings.IsType(initializedType))
                        {
                            if (assigned[address.Local]) throw new InvalidDataException("Resetting an initialized carrier is unsupported.");
                            // A CLI carrier default is not a valid union value. Keep it unreadable
                            // until an actual carrier assignment, rather than inventing a case.
                            code.AppendLine("pop");
                        }
                        else
                        {
                            code.AppendLine(Default(initializedType) + "\nstobj " + initializedType);
                            assigned[address.Local] = true;
                        }
                        break;
                    case Code.Ldnull: Push(new("FaultNull")); break;
                    case Code.Throw:
                        Expect("FaultNull");
                        code.AppendLine("fault \"Guest program reached a terminal failure\"");
                        terminates = true; break;
                    case Code.Newarr:
                        if (Type((TypeReference)instruction.Operand) != "Int32")
                            throw new InvalidDataException("Only Int32 vector allocation is admitted.");
                        Expect("Int32"); Push(new(IntArray)); code.AppendLine("newarr Int32"); break;
                    case Code.Ldlen:
                        Expect(IntArray); Push(new("UIntPtr")); code.AppendLine("ldlen"); break;
                    case Code.Conv_I4:
                        var converted = Pop();
                        if (converted.Type is not ("UIntPtr" or "Int32")) throw new InvalidDataException("Unsupported conv.i4 input.");
                        Push(new("Int32")); code.AppendLine("conv.i4"); break;
                    case Code.Ldelem_I4:
                        Expect("Int32"); Expect(IntArray); Push(new("Int32")); code.AppendLine("ldelem Int32"); break;
                    case Code.Stelem_I4:
                        Expect("Int32"); Expect("Int32"); Expect(IntArray); code.AppendLine("stelem Int32"); break;
                    case Code.Ldsfld:
                        var field = ((FieldReference)instruction.Operand).Resolve();
                        if (field is null || field.Module != app.MainModule || field.FullName != "System.Unit System.Unit::Value"
                            || !field.IsStatic || !field.IsInitOnly || !field.DeclaringType.IsValueType
                            || field.DeclaringType.Fields.Any(f => !f.IsStatic)
                            || field.DeclaringType.Methods.Any(m => m.IsConstructor && m.IsStatic))
                            throw new InvalidDataException("Only the empty Raven Unit literal field is admitted.");
                        Push(new("Void")); code.AppendLine("ldvoid"); break;
                    case Code.Ldstr:
                        var value = (string)instruction.Operand;
                        if (value.Length > 65536) throw new InvalidDataException("String limit exceeded.");
                        Push(new("String")); code.AppendLine("ldstr " + JsonSerializer.Serialize(value)); break;
                    case Code.Ldc_I4_M1: case Code.Ldc_I4_0: case Code.Ldc_I4_1: case Code.Ldc_I4_2:
                    case Code.Ldc_I4_3: case Code.Ldc_I4_4: case Code.Ldc_I4_5: case Code.Ldc_I4_6:
                    case Code.Ldc_I4_7: case Code.Ldc_I4_8: case Code.Ldc_I4_S: case Code.Ldc_I4:
                        var number = instruction.Operand is null ? (int)instruction.OpCode.Code - (int)Code.Ldc_I4_0 : Convert.ToInt32(instruction.Operand);
                        Push(new("Int32")); code.AppendLine($"ldc.i4 {number}"); break;
                    case Code.Ldarg_0: case Code.Ldarg_1: case Code.Ldarg_2: case Code.Ldarg_3: Arg((int)instruction.OpCode.Code - (int)Code.Ldarg_0); break;
                    case Code.Ldarg: case Code.Ldarg_S: Arg(((ParameterDefinition)instruction.Operand).Index); break;
                    case Code.Ldarga: case Code.Ldarga_S:
                        var parameter = ((ParameterDefinition)instruction.Operand).Index;
                        if (parameter < 0 || parameter >= args.Length || args[parameter] != "Int32")
                            throw new InvalidDataException("Only Int32 argument addresses admitted.");
                        Push(new("Int32&", Argument: parameter)); code.AppendLine($"ldarga {parameter}"); break;
                    case Code.Ldloc_0: case Code.Ldloc_1: case Code.Ldloc_2: case Code.Ldloc_3: Load((int)instruction.OpCode.Code - (int)Code.Ldloc_0); break;
                    case Code.Ldloc: case Code.Ldloc_S: Load(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Stloc_0: case Code.Stloc_1: case Code.Stloc_2: case Code.Stloc_3: Store((int)instruction.OpCode.Code - (int)Code.Stloc_0); break;
                    case Code.Stloc: case Code.Stloc_S: Store(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Ldloca: case Code.Ldloca_S:
                        var local = Local(((VariableDefinition)instruction.Operand).Index);
                        Push(new(locals[local] + "&", local)); code.AppendLine($"ldloca local{local}"); break;
                    case Code.Newobj:
                        var constructor = (MethodReference)instruction.Operand;
                        var constructorDefinition = constructor.Resolve() ?? throw new InvalidDataException("Unresolved constructor.");
                        if (constructorDefinition.Module != library.MainModule) throw new InvalidDataException("Only admitted library constructors supported.");
                        if (collectionProfile && CollectionBindings.Type(constructor.DeclaringType) == CollectionBindings.ArrayList)
                        {
                            var signature = RuntimeSignatures.Match(constructor, constructorDefinition, CollectionBindings.Type);
                            if (!constructorDefinition.IsConstructor || !constructor.HasThis || signature.Result != "noresult"
                                || signature.Args.Length > 1 || signature.Args.Any(p => p != "Int32"))
                                throw new InvalidDataException("Unsupported collection constructor.");
                            if (signature.Args.Length == 1) Expect("Int32");
                            Push(new(CollectionBindings.ArrayList));
                            code.AppendLine($"newobj instance {CollectionBindings.ArrayList}::.ctor({string.Join(',', signature.Args)})");
                            break;
                        }
                        var construction = Construct(constructor, constructorDefinition);
                        for (var n = construction.Arguments.Length - 1; n >= 0; n--) Expect(construction.Arguments[n]);
                        Push(new(construction.Result));
                        code.AppendLine($"call {construction.Name}({string.Join(',', construction.Arguments)})"); break;
                    case Code.Ceq:
                        Expect("Int32"); Expect("Int32"); Push(new("Int32"));
                        code.AppendLine("call RuntimeEqual(Int32,Int32)"); break;
                    case Code.Pop: if (Pop().Type != "FaultNull") code.AppendLine("pop"); break;
                    case Code.Dup: var top = Pop(); Push(top); Push(top); if (top.Type != "FaultNull") code.AppendLine("dup"); break;
                    case Code.Br: case Code.Br_S:
                        var branch = Target(); successors.Add(branch); code.AppendLine($"br IL_{instructions[branch].Offset:x4}"); terminates = true; break;
                    case Code.Brtrue: case Code.Brtrue_S: case Code.Brfalse: case Code.Brfalse_S:
                        var condition = Pop();
                        if (condition.Type is not ("Int32" or "Boolean")) throw new InvalidDataException("Invalid branch condition.");
                        var conditional = Target(); successors.Add(conditional);
                        if (condition.ConditionalOut >= 0 && conditional != index + 1)
                        {
                            var success = (bool[])assigned.Clone(); success[condition.ConditionalOut] = true;
                            assignmentEdges[instruction.OpCode.Code is Code.Brtrue or Code.Brtrue_S ? conditional : index + 1] = success;
                        }
                        code.AppendLine($"{(instruction.OpCode.Code is Code.Brtrue or Code.Brtrue_S ? "brtrue" : "brfalse")} IL_{instructions[conditional].Offset:x4}"); break;
                    case Code.Call:
                    case Code.Callvirt:
                        var reference = (MethodReference)instruction.Operand;
                        var targetMethod = ClosureAudit.ResolveMethod(reference) ?? throw new InvalidDataException("Unresolved call.");
                        Call call;
                        if (targetMethod.Module == app.MainModule)
                        {
                            if (instruction.OpCode.Code == Code.Callvirt) throw new InvalidDataException("Application instance calls unsupported.");
                            CheckStatic(reference); CheckStatic(targetMethod);
                            if (!targetMethod.IsPublic && targetMethod.DeclaringType != method.DeclaringType)
                                throw new InvalidDataException("Nonpublic cross-type call unsupported.");
                            if (reference.FullName != targetMethod.FullName) throw new InvalidDataException("Resolved signature mismatch.");
                            pending.Enqueue(targetMethod);
                            call = new(Name(targetMethod), reference.Parameters.Select(p => ProfileType(p.ParameterType)).ToArray(), ProfileType(reference.ReturnType, true));
                        }
                        else if (targetMethod.Module == library.MainModule)
                        {
                            var binding = collectionProfile ? CollectionBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt) : null;
                            var textBinding = StringBindings.Bind(reference, targetMethod, instruction.OpCode.Code == Code.Callvirt);
                            if (textBinding is not null) call = new("", textBinding.Arguments, textBinding.Result, Instruction: textBinding.Instruction);
                            else if (binding is not null) call = new("", binding.Arguments, binding.Result, Instruction: binding.Instruction);
                            else
                            {
                                if (instruction.OpCode.Code == Code.Callvirt) throw new InvalidDataException("Unsupported runtime callvirt.");
                                call = Bind(reference, targetMethod);
                            }
                        }
                        else throw new InvalidDataException("Unsupported dependency call.");
                        var conditionalOut = -1;
                        for (var n = call.Arguments.Length - 1; n >= 0; n--)
                        {
                            var argument = Expect(call.Arguments[n]);
                            if (argument.Type.EndsWith('&'))
                            {
                                if (argument.Argument >= 0)
                                {
                                    if (n != 0 || !reference.HasThis || reference.DeclaringType.FullName != "System.Int32"
                                        || reference.Name is not ("Equals" or "CompareTo" or "ToString"))
                                        throw new InvalidDataException("Argument addresses are only admitted as Int32 receivers.");
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
                        if (call.Result != "noresult") Push(new(call.Result, ConditionalOut: conditionalOut));
                        code.AppendLine(call.Instruction ?? $"call {call.Name}({string.Join(',', call.Arguments)})"); break;
                    case Code.Ret:
                        if (result != "noresult") Expect(result);
                        if (stack.Count != 0) throw new InvalidDataException("Input ret stack not empty.");
                        code.AppendLine("ret"); terminates = true; break;
                    default: throw new InvalidDataException("Unsupported reachable instruction: " + instruction.OpCode);
                }
                bodies[index] = code.ToString();
                if (!terminates) successors.Add(index + 1);
                foreach (var successor in successors) Merge(successor, new(stack, assignmentEdges.GetValueOrDefault(successor, assigned)));
            }
            // Unreachable guest instructions are omitted, not admitted as executable code.
            output.AppendLine($".function {Name(method)}({string.Join(',', args)}) -> {result}");
            for (var n = 0; n < locals.Length; n++) output.AppendLine($".local {locals[n]} local{n}");
            if (method.Body.InitLocals)
                for (var n = 0; n < locals.Length; n++)
                    if (locals[n] == IntArray || CollectionBindings.IsReference(locals[n])) output.AppendLine($"ldloca local{n}\ninitobj {locals[n]}");
                    else if (locals[n] != Carrier && locals[n] != Option && locals[n] != VoidOption && locals[n] != VoidResult && locals[n] != "String" && !ResultBindings.IsType(locals[n])) output.AppendLine(Default(locals[n]) + $"\nstloc local{n}");
            foreach (var index in bodies.Keys.Order())
            {
                mappings.Add(new { MethodToken = method.MetadataToken.ToUInt32(), instructions[index].Offset, OutputLine = output.ToString().Count(c => c == '\n') + 1 });
                output.AppendLine($"IL_{instructions[index].Offset:x4}:").Append(bodies[index]);
            }
            output.AppendLine(".end");

            void Merge(int index, State next)
            {
                if (index < 0 || index >= instructions.Length) throw new InvalidDataException("Method falls through.");
                if (!states.TryGetValue(index, out var previous))
                { states[index] = new(next.Stack.ToList(), (bool[])next.Assigned.Clone()); work.Enqueue(index); return; }
                if (!previous.Stack.SequenceEqual(next.Stack)) throw new InvalidDataException("Incompatible branch stack merge.");
                var changed = false;
                for (var n = 0; n < previous.Assigned.Length; n++)
                    if (previous.Assigned[n] && !next.Assigned[n]) { previous.Assigned[n] = false; changed = true; }
                if (changed) work.Enqueue(index);
            }
        }
        output.Append(Adapters()).Append(ResultBindings.Adapters()).Append(StringBindings.Adapters()).Append(Int32Bindings.Adapters);
        File.WriteAllText(destination, output.ToString());
        File.WriteAllText(destination + ".map.json", JsonSerializer.Serialize(new {
            Profile = collectionProfile ? "result-option-void-files-strings-collections-v8" : "result-option-void-files-strings-arrays-v7",
            RequiredLibraryProfile = collectionProfile ? "raven-collections" : "bundled-system", ApplicationSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(application))),
            CoreSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(core))), ReachableMethods = seen.Order().ToArray(), Mappings = mappings,
            Scope = "Bounded Int32 vectors, optional Int32 collection references, file UTF-8 APIs, String helpers and generic Result/Option bindings; CFG stack/definite-assignment checked; observable default carriers rejected; no guest declaration bodies executed."
        }, new JsonSerializerOptions { WriteIndented = true }));
    }

    // Raven materializes pattern-test success as 1/0 before branching again.
    // Thread this exact diamond so conditional-out proof reaches the matched arm.
    // The false block and join must have no other incoming branch edges.
    static void NormalizePatternBranches(MethodDefinition method)
    {
        var instructions = method.Body.Instructions;
        foreach (var join in instructions.Where(i => i.OpCode.Code is Code.Brfalse or Code.Brfalse_S).ToArray())
        {
            var miss = join.Previous;
            var jump = miss?.Previous;
            var hit = jump?.Previous;
            if (join.Next is null || miss?.OpCode.Code != Code.Ldc_I4_0 || hit?.OpCode.Code != Code.Ldc_I4_1
                || jump is null || jump.OpCode.Code is not (Code.Br or Code.Br_S) || jump.Operand != join) continue;
            var incomingMiss = instructions.Where(i => ReferenceEquals(i.Operand, miss)).ToArray();
            if (incomingMiss.Length != 1 || incomingMiss[0].OpCode.Code is not (Code.Brfalse or Code.Brfalse_S)
                || instructions.Count(i => ReferenceEquals(i.Operand, join)) != 1
                || instructions.Any(i => ReferenceEquals(i.Operand, jump))
                || instructions.Any(i => i.Operand is Instruction[] targets && targets.Any(t => t == miss || t == join || t == jump))) continue;
            incomingMiss[0].Operand = join.Operand;
            hit.OpCode = OpCodes.Nop;
            jump.Operand = join.Next;
        }
    }

    static string Name(MethodDefinition method) => $"Method_{method.MetadataToken.ToUInt32():x8}";
    static void CheckStatic(MethodReference method)
    {
        if (method.HasThis || method.ExplicitThis || method.HasGenericParameters || method is GenericInstanceMethod
            || method.DeclaringType.HasGenericParameters || method.DeclaringType is GenericInstanceType
            || method.CallingConvention != MethodCallingConvention.Default)
            throw new InvalidDataException("Unsupported application signature.");
    }
    static string Type(TypeReference type, bool result = false) => type.FullName switch {
        "System.Int32[]" when type is ArrayType { IsVector: true } array && array.ElementType.MetadataType == MetadataType.Int32 => IntArray,
        "System.Unit" when type.IsValueType && type.Resolve() is { } unit && !unit.Fields.Any(f => !f.IsStatic)
            && !unit.Methods.Any(m => m.IsConstructor && m.IsStatic) => "Void",
        "System.Void" when !result && type.IsValueType => "Void",
        "System.OverflowError" when type.IsValueType => Overflow,
        "System.Boolean" => "Boolean", "System.Void" when result => "noresult", "System.Int32" => "Int32", "System.String" => "String",
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
        var file = Int32Bindings.Bind(reference, definition) ?? PathBindings.Bind(reference, definition) ?? FileBindings.Bind(reference, definition) ?? ResultBindings.Bind(reference, definition);
        if (file is not null)
        {
            if (file.OutArgument >= 0 || reference.Name == "FromResidual") ValidatePropagation(definition.DeclaringType);
            return new(file.Name, file.Arguments, file.Result, file.OutArgument, file.Instruction, file.OutArgument >= 0);
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
        if (key == "System.Result`2<System.Int32,System.OverflowError> System.Math::Abs(System.Int32)" && reference.FullName == key && !reference.HasThis)
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
        var file = ResultBindings.Construct(reference, definition);
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
    static string Default(string type) => type switch {
        VoidOk => $"ldvoid\nnewobj {VoidOk}",
        "Void" => "ldvoid",
        Overflow => "newobj System.OverflowError",
        "Boolean" => "ldc.bool false", "Int32" => "ldc.i4 0", VoidSome => $"ldvoid\nnewobj {VoidSome}", Some => $"ldc.i4 0\nnewobj {Some}", None => $"newobj {None}", Ok => $"ldc.i4 0\nnewobj {Ok}",
        Error => $"newobj System.OverflowError\nnewobj {Error}",
        _ => throw new InvalidDataException("Unsupported default.")
    };
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
