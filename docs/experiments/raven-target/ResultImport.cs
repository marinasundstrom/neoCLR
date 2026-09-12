using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;

// Bounded Result profile, not a general CLI loader. Only supplied metadata is resolved.
static class ResultImport
{
    const string Carrier = "System.Result<Int32,System.OverflowError>";
    const string Ok = "System.Result.Ok<Int32>";
    const string Error = "System.Result.Error<System.OverflowError>";
    sealed record Slot(string Type, int Local = -1);
    sealed record State(List<Slot> Stack, bool[] Assigned);
    sealed record Call(string Name, string[] Arguments, string Result, int OutArgument = -1);

    public static void Write(string application, string core, string destination)
    {
        foreach (var path in new[] { application, core })
            if (new FileInfo(path).Length > 16 * 1024 * 1024) throw new InvalidDataException("Image exceeds profile limit.");
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
        var output = new StringBuilder($".module ImportedResult\n.entry {Name(entry)}\n");
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
            var args = method.Parameters.Select(p => Type(p.ParameterType)).ToArray();
            var result = Type(method.ReturnType, true);
            var locals = method.Body.Variables.Select(v => Type(v.VariableType)).ToArray();
            if (locals.Any(t => t is not ("Int32" or Carrier or Ok or Error)))
                throw new InvalidDataException("Unsupported local default in Result profile.");
            var instructions = method.Body.Instructions.ToArray();
            var indexes = instructions.Select((i, n) => (i, n)).ToDictionary(p => p.i, p => p.n);
            var states = new Dictionary<int, State>();
            var work = new Queue<int>();
            var bodies = new Dictionary<int, string>();
            Merge(0, new([], locals.Select(t => method.Body.InitLocals && t != Carrier).ToArray()));
            var visits = 0;
            while (work.TryDequeue(out var index))
            {
                if (++visits > 100000) throw new InvalidDataException("Control-flow analysis limit exceeded.");
                var input = states[index];
                var stack = input.Stack.ToList(); var assigned = (bool[])input.Assigned.Clone();
                var instruction = instructions[index]; var code = new StringBuilder(); var successors = new List<int>();
                void Push(Slot slot) { stack.Add(slot); if (stack.Count > method.Body.MaxStackSize) throw new InvalidDataException("Declared maxstack exceeded."); }
                Slot Pop() { if (stack.Count == 0) throw new InvalidDataException("Input stack underflow."); var top = stack[^1]; stack.RemoveAt(stack.Count - 1); return top; }
                Slot Expect(string type) { var top = Pop(); if (top.Type != type) throw new InvalidDataException("Input stack type mismatch."); return top; }
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
                    case Code.Ldloc_0: case Code.Ldloc_1: case Code.Ldloc_2: case Code.Ldloc_3: Load((int)instruction.OpCode.Code - (int)Code.Ldloc_0); break;
                    case Code.Ldloc: case Code.Ldloc_S: Load(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Stloc_0: case Code.Stloc_1: case Code.Stloc_2: case Code.Stloc_3: Store((int)instruction.OpCode.Code - (int)Code.Stloc_0); break;
                    case Code.Stloc: case Code.Stloc_S: Store(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Ldloca: case Code.Ldloca_S:
                        var local = Local(((VariableDefinition)instruction.Operand).Index);
                        Push(new(locals[local] + "&", local)); code.AppendLine($"ldloca local{local}"); break;
                    case Code.Pop: Pop(); code.AppendLine("pop"); break;
                    case Code.Dup: var top = Pop(); Push(top); Push(top); code.AppendLine("dup"); break;
                    case Code.Br: case Code.Br_S:
                        var branch = Target(); successors.Add(branch); code.AppendLine($"br IL_{instructions[branch].Offset:x4}"); terminates = true; break;
                    case Code.Brtrue: case Code.Brtrue_S: case Code.Brfalse: case Code.Brfalse_S:
                        Expect("Int32"); var conditional = Target(); successors.Add(conditional);
                        code.AppendLine($"{(instruction.OpCode.Code is Code.Brtrue or Code.Brtrue_S ? "brtrue" : "brfalse")} IL_{instructions[conditional].Offset:x4}"); break;
                    case Code.Call:
                        var reference = (MethodReference)instruction.Operand;
                        var targetMethod = reference.Resolve() ?? throw new InvalidDataException("Unresolved call.");
                        Call call;
                        if (targetMethod.Module == app.MainModule)
                        {
                            CheckStatic(reference); CheckStatic(targetMethod);
                            if (!targetMethod.IsPublic && targetMethod.DeclaringType != method.DeclaringType)
                                throw new InvalidDataException("Nonpublic cross-type call unsupported.");
                            if (reference.FullName != targetMethod.FullName) throw new InvalidDataException("Resolved signature mismatch.");
                            pending.Enqueue(targetMethod);
                            call = new(Name(targetMethod), reference.Parameters.Select(p => Type(p.ParameterType)).ToArray(), Type(reference.ReturnType, true));
                        }
                        else if (targetMethod.Module == library.MainModule) call = Bind(reference, targetMethod);
                        else throw new InvalidDataException("Unsupported dependency call.");
                        for (var n = call.Arguments.Length - 1; n >= 0; n--)
                        {
                            var argument = Expect(call.Arguments[n]);
                            if (argument.Type.EndsWith('&'))
                            {
                                if (argument.Local < 0) throw new InvalidDataException("Only local addresses admitted.");
                                if (n == call.OutArgument) assigned[argument.Local] = true;
                                else if (!assigned[argument.Local]) throw new InvalidDataException("Read through uninitialized carrier address.");
                            }
                        }
                        if (call.Result != "noresult") Push(new(call.Result));
                        code.AppendLine($"call {call.Name}({string.Join(',', call.Arguments)})"); break;
                    case Code.Ret:
                        if (result != "noresult") Expect(result);
                        if (stack.Count != 0) throw new InvalidDataException("Input ret stack not empty.");
                        code.AppendLine("ret"); terminates = true; break;
                    default: throw new InvalidDataException("Unsupported reachable instruction: " + instruction.OpCode);
                }
                bodies[index] = code.ToString();
                if (!terminates) successors.Add(index + 1);
                foreach (var successor in successors) Merge(successor, new(stack, assigned));
            }
            // Unreachable guest instructions are omitted, not admitted as executable code.
            output.AppendLine($".function {Name(method)}({string.Join(',', args)}) -> {result}");
            for (var n = 0; n < locals.Length; n++) output.AppendLine($".local {locals[n]} local{n}");
            if (method.Body.InitLocals)
                for (var n = 0; n < locals.Length; n++)
                    if (locals[n] != Carrier) output.AppendLine(Default(locals[n]) + $"\nstloc local{n}");
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
        output.Append(Adapters());
        File.WriteAllText(destination, output.ToString());
        File.WriteAllText(destination + ".map.json", JsonSerializer.Serialize(new {
            Profile = "result-int32-overflow-v1", ApplicationSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(application))),
            CoreSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(core))), ReachableMethods = seen.Order().ToArray(), Mappings = mappings,
            Scope = "Bounded generic Result bindings; CFG stack/definite-assignment checked; observable default carrier rejected; no guest declaration bodies executed."
        }, new JsonSerializerOptions { WriteIndented = true }));
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
        "System.Void" when result => "noresult", "System.Int32" => "Int32", "System.String" => "String",
        "System.Result`2<System.Int32,System.OverflowError>" when type.IsValueType => Carrier,
        "System.Result/Ok`1<System.Int32>" when type.IsValueType => Ok,
        "System.Result/Error`1<System.OverflowError>" when type.IsValueType => Error,
        _ => throw new InvalidDataException("Unsupported Result profile type: " + type.FullName)
    };
    static Call Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!definition.IsPublic || reference.ExplicitThis || reference is GenericInstanceMethod || reference.HasGenericParameters
            || reference.CallingConvention != MethodCallingConvention.Default || reference.HasThis != definition.HasThis)
            throw new InvalidDataException("Unsupported runtime signature.");
        // Exact definition signatures include generic parameter positions, not just arity/names.
        var key = definition.FullName;
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
        if (reference.DeclaringType.FullName == "System.Result/Ok`1<System.Int32>" && reference.HasThis
            && key == "T System.Result/Ok`1::get_Value()" && reference.ReturnType is GenericParameter { Position: 0 }
            && reference.Parameters.Count == 0)
            return new("RuntimeOkValue", [Ok + "&"], "Int32");
        throw new InvalidDataException("Unsupported runtime binding: " + reference.FullName + " definition=" + key);
    }
    static string Default(string type) => type switch {
        "Int32" => "ldc.i4 0", Ok => $"ldc.i4 0\nnewobj {Ok}",
        Error => $"newobj System.OverflowError\nnewobj {Error}",
        _ => throw new InvalidDataException("Unsupported default.")
    };
    static string Adapters()
    {
        var text = new StringBuilder();
        foreach (var (name, type) in new[] { ("Ok", Ok), ("Error", Error) })
        {
            text.AppendLine($".function RuntimeTry{name}({Carrier}& source,out {type}& destination) -> Int32");
            text.AppendLine($"ldarg destination\n{Default(type)}\nstobj {type}");
            text.AppendLine($"ldarg source\nldobj {Carrier}\nldarg destination\ncall instance {Carrier}::TryGet({type}&)");
            text.AppendLine("brfalse Miss\nldc.i4 1\nret\nMiss:\nldc.i4 0\nret\n.end");
        }
        text.AppendLine($".function RuntimeOkValue({Ok}& source) -> Int32\nldarg source\nldobj {Ok}\ncall instance {Ok}::get_Value()\nret\n.end");
        foreach (var type in new[] { "Int32", "String" })
            text.AppendLine($".function RuntimeWrite{type}({type} value) -> noresult\nldarg value\ncall System.Console::WriteLine({type})\npop\nret\n.end");
        return text.ToString();
    }
}
