using System.Text;
using System.Text.Json;
using System.Security.Cryptography;
using Mono.Cecil;
using Mono.Cecil.Cil;

// Experimental straight-line static profile. No guest CLR execution or ambient resolver.
static class StaticImport
{
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
            throw new InvalidDataException("Module initializers are unsupported.");
        var entry = app.EntryPoint ?? throw new InvalidDataException("Missing managed entry point.");
        if (entry.Parameters.Count != 0) throw new InvalidDataException("Entry parameters unsupported.");
        var text = new StringBuilder($".module Imported{app.Name.Name}\n.entry {Name(entry)}\n");
        var pending = new Queue<MethodDefinition>(); pending.Enqueue(entry);
        var seen = new HashSet<uint>();
        var mappings = new List<object>();
        var bindings = new Dictionary<string, TargetSurface.Method>();
        while (pending.TryDequeue(out var method))
        {
            if (!seen.Add(method.MetadataToken.ToUInt32())) continue;
            if (seen.Count > 128) throw new InvalidDataException("Method count exceeds profile limit.");
            Signature(method);
            if (method.DeclaringType.Methods.Any(m => m.IsConstructor && m.IsStatic))
                throw new InvalidDataException("Type initializers are unsupported.");
            if (!method.HasBody || method.IsPInvokeImpl || method.IsInternalCall || method.Body.HasExceptionHandlers
                || method.Body.CodeSize > 65536 || method.Body.Variables.Count > 256)
                throw new InvalidDataException("Unsupported body: " + method.FullName);
            var parameters = method.Parameters.Select(p => Type(p.ParameterType)).ToArray();
            var result = Type(method.ReturnType, true);
            text.AppendLine($".function {Name(method)}({string.Join(',', parameters)}) -> {result}");
            var locals = method.Body.Variables.Select(v => Type(v.VariableType)).ToArray();
            var initialized = new bool[locals.Length];
            foreach (var (type, index) in locals.Select((type, index) => (type, index)))
            {
                if (type != "Int32") throw new InvalidDataException("Only Int32 locals supported in static profile.");
                text.AppendLine($".local {type} local{index}");
            }
            if (method.Body.InitLocals)
                for (var i = 0; i < locals.Length; i++) { text.AppendLine($"ldc.i4 0\nstloc local{i}"); initialized[i] = true; }
            var stack = new List<string>();
            var returned = false;
            foreach (var instruction in method.Body.Instructions)
            {
                if (returned) throw new InvalidDataException("Instructions after ret are outside this straight-line profile.");
                var line = text.ToString().Count(c => c == '\n') + 1;
                mappings.Add(new { MethodToken = method.MetadataToken.ToUInt32(), instruction.Offset, OutputLine = line });
                void Push(string type) { stack.Add(type); if (stack.Count > method.Body.MaxStackSize) throw new InvalidDataException("Declared maxstack exceeded."); }
                string Pop() { if (stack.Count == 0) throw new InvalidDataException("Input stack underflow."); var value = stack[^1]; stack.RemoveAt(stack.Count-1); return value; }
                void Store(string type) { if (Pop() != type) throw new InvalidDataException("Input stack type mismatch."); }
                void LoadLocal(int i) { if (!initialized[i]) throw new InvalidDataException("Uninitialized local."); Push(locals[i]); text.AppendLine($"ldloc local{i}"); }
                void StoreLocal(int i) { Store(locals[i]); initialized[i] = true; text.AppendLine($"stloc local{i}"); }
                void Arg(int i) { Push(parameters[i]); text.AppendLine($"ldarg {i}"); }
                switch (instruction.OpCode.Code)
                {
                    case Code.Nop: break;
                    case Code.Ldstr:
                        var value = (string)instruction.Operand;
                        if (value.Length > 65536) throw new InvalidDataException("String exceeds profile limit.");
                        Push("String"); text.AppendLine("ldstr " + JsonSerializer.Serialize(value)); break;
                    case Code.Ldc_I4_M1: case Code.Ldc_I4_0: case Code.Ldc_I4_1: case Code.Ldc_I4_2:
                    case Code.Ldc_I4_3: case Code.Ldc_I4_4: case Code.Ldc_I4_5: case Code.Ldc_I4_6:
                    case Code.Ldc_I4_7: case Code.Ldc_I4_8: case Code.Ldc_I4_S: case Code.Ldc_I4:
                        var number = instruction.Operand is null ? (int)instruction.OpCode.Code - (int)Code.Ldc_I4_0 : Convert.ToInt32(instruction.Operand);
                        Push("Int32"); text.AppendLine($"ldc.i4 {number}"); break;
                    case Code.Ldarg_0: case Code.Ldarg_1: case Code.Ldarg_2: case Code.Ldarg_3:
                        Arg((int)instruction.OpCode.Code - (int)Code.Ldarg_0); break;
                    case Code.Ldarg: case Code.Ldarg_S: Arg(((ParameterDefinition)instruction.Operand).Index); break;
                    case Code.Ldloc_0: case Code.Ldloc_1: case Code.Ldloc_2: case Code.Ldloc_3:
                        LoadLocal((int)instruction.OpCode.Code - (int)Code.Ldloc_0); break;
                    case Code.Ldloc: case Code.Ldloc_S: LoadLocal(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Stloc_0: case Code.Stloc_1: case Code.Stloc_2: case Code.Stloc_3:
                        StoreLocal((int)instruction.OpCode.Code - (int)Code.Stloc_0); break;
                    case Code.Stloc: case Code.Stloc_S: StoreLocal(((VariableDefinition)instruction.Operand).Index); break;
                    case Code.Pop: Pop(); text.AppendLine("pop"); break;
                    case Code.Dup: var top = Pop(); Push(top); Push(top); text.AppendLine("dup"); break;
                    case Code.Call:
                        var reference = (MethodReference)instruction.Operand;
                        Signature(reference);
                        var target = reference.Resolve() ?? throw new InvalidDataException("Unresolved call.");
                        var args = reference.Parameters.Select(p => Type(p.ParameterType)).ToArray();
                        for (var i = args.Length-1; i >= 0; i--) Store(args[i]);
                        var returns = Type(reference.ReturnType, true);
                        Signature(target);
                        if (!target.Parameters.Select(p => Type(p.ParameterType)).SequenceEqual(args) || Type(target.ReturnType, true) != returns)
                            throw new InvalidDataException("Resolved signature mismatch.");
                        if (!target.IsPublic && target.DeclaringType != method.DeclaringType)
                            throw new InvalidDataException("Cross-type nonpublic calls unsupported.");
                        if (returns != "noresult") Push(returns);
                        string name;
                        if (target.Module == app.MainModule) { pending.Enqueue(target); name = Name(target); }
                        else if (target.Module == library.MainModule && TargetSurface.Bind(target) is { } binding)
                        { bindings[binding.Signature] = binding; name = binding.ImportTarget; }
                        else throw new InvalidDataException("Unsupported runtime binding: " + reference.FullName);
                        text.AppendLine($"call {name}({string.Join(',', args)})"); break;
                    case Code.Ret:
                        if (result != "noresult") Store(result);
                        if (stack.Count != 0) throw new InvalidDataException("Input ret stack must be empty after its declared result.");
                        text.AppendLine("ret"); returned = true; break;
                    default: throw new InvalidDataException($"Unsupported reachable instruction {method.FullName} IL_{instruction.Offset:x4}: {instruction.OpCode}");
                }
            }
            if (!returned) throw new InvalidDataException("Method falls through.");
            text.AppendLine(".end");
        }
        foreach (var binding in bindings.Values.Where(b => b.Returns == "Void"))
        {
            text.AppendLine($".function {binding.ImportTarget}({string.Join(',', binding.Parameters)}) -> noresult");
            for (var i = 0; i < binding.Parameters.Length; i++) text.AppendLine($"ldarg {i}");
            text.AppendLine($"call {binding.Signature}\npop\nret\n.end");
        }
        File.WriteAllText(destination, text.ToString());
        File.WriteAllText(destination + ".map.json", JsonSerializer.Serialize(new {
            Profile = "straight-line-static-v1",
            ApplicationSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(application))),
            CoreSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(core))),
            ReachableMethods = seen.Order().ToArray(), ConsoleBinding = bindings.Values.Any(b => b.Owner == "Console"), RuntimeBindings = bindings.Keys.Order().ToArray(), Mappings = mappings,
            Scope = "Explicit metadata closure audited; reachable bodies checked; unreachable helper IL not admitted for execution."
        }, new JsonSerializerOptions { WriteIndented = true }));
    }
    static string Name(MethodDefinition method) => $"Method_{method.MetadataToken.ToUInt32():x8}";
    static string Type(TypeReference type, bool result = false) => type.MetadataType switch {
        MetadataType.Void when result => "noresult", MetadataType.Int32 => "Int32", MetadataType.String => "String",
        _ => throw new InvalidDataException("Unsupported signature type: " + type.FullName)
    };
    static void Signature(MethodReference method)
    {
        if (method.HasThis || method.ExplicitThis || method.HasGenericParameters || method is GenericInstanceMethod
            || method.DeclaringType.HasGenericParameters || method.DeclaringType is GenericInstanceType
            || method.CallingConvention != MethodCallingConvention.Default)
            throw new InvalidDataException("Only ordinary non-generic static signatures are admitted: " + method.FullName);
    }
}
