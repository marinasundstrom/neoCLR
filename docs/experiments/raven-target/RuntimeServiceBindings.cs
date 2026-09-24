using Mono.Cecil;

// Bootstrap-only access to existing typed host services. This is an authoring
// boundary, not a public API or a way to select arbitrary native entry points.
static class RuntimeServiceBindings
{
    const string Owner = "System.Runtime.CompilerServices.RuntimeServices";
    static readonly string[] UnaryMath = ["Abs", "Sqrt", "Floor", "Ceiling", "Truncate", "Round", "Exp", "Log", "Log10", "Sin", "Cos", "Tan"];
    static readonly string[] BinaryMath = ["Pow", "Min", "Max"];
    static readonly (string Name, string[] Args, string Result)[] Members =
        UnaryMath.Select(n => ("Math" + n, new[] { "Double" }, "Double"))
        .Concat(BinaryMath.Select(n => ("Math" + n, new[] { "Double", "Double" }, "Double"))).Concat(new (string Name, string[] Args, string Result)[] {
            ("StorageList", ["String", "Int32"], "Value"),
            ("StorageNames", ["Value"], "arrayref<String>"),
            ("StorageKind", ["String"], "Value"),
            ("FileOpenRead", ["String"], "Value"),
            ("FileCreateNew", ["String"], "Value"),
            ("FilePosition", ["Int32"], "Value"),
            ("FileSeek", ["Int32", "Int64"], "Value"),
            ("FileReadInto", ["Int32", "arrayref<Byte>", "Int32", "Int32"], "Value"),
            ("FileWriteChunk", ["Int32", "arrayref<Byte>", "Int32", "Int32"], "Value"),
            ("FileFlush", ["Int32"], "Value"),
            ("FileClose", ["Int32"], "Value"),
            ("StartWorker", ["System.Func<String,String>", "String"], "Int32"),
            ("QueueWorker", ["System.Func<String,String>", "String"], "Int32"),
            ("JoinWorker", ["Int32"], "String"),
            ("RequestWorkerCancellation", ["Int32"], "Boolean"),
            ("JoinWorkerResult", ["Int32"], "Value"),
            ("NotifyWorker", ["Int32", "System.Func<Void>"], "noresult"),
            ("LocalDateTime", ["Int64"], "System.LocalDateTime"),
            ("UnixTimeTicks", [], "Int64"),
            ("UnixTimeToLocal", ["Int64"], "arrayref<Int32>"),
            ("PathCombine", ["String", "String"], "String"),
            ("PathGetFileName", ["String"], "String"),
            ("WriteAllText", ["String", "String", "Int32"], "Int32"),
            ("StringEquals", ["String", "String"], "Boolean"),
            ("StringConcat", ["String", "String"], "String"),
            ("StringCompareOrdinal", ["String", "String"], "Int32"),
            ("StringContainsOrdinal", ["String", "String"], "Boolean"),
            ("StringStartsWithOrdinal", ["String", "String"], "Boolean"),
            ("StringEndsWithOrdinal", ["String", "String"], "Boolean"),
            ("Utf8Encode", ["String"], "arrayref<Byte>"),
            ("Utf8Decode", ["arrayref<Byte>"], "Value"),
            ("StringGraphemeCount", ["String"], "Int32"),
            ("CharFromString", ["String"], "Char"),
            ("CharText", ["Char"], "String"),
            ("StringGraphemes", ["String"], "arrayref<Char>"),
            ("StringScalars", ["String"], "arrayref<UInt32>"),
            ("StringByteCount", ["String"], "Int32"),
            ("StringSliceUtf8", ["String", "Int32", "Int32"], "Value"),
            ("ReadAllText", ["String", "Int32"], "Value"),
            ("ParseInt32", ["String"], "Value"),
            ("ConsoleWriteBytes", ["Boolean", "arrayref<Byte>", "Int32", "Int32"], "Value"),
            ("ConsoleFlush", ["Boolean"], "Value"),
            ("ConsoleReadByte", [], "Value"),
            ("WriteLine", ["String"], "noresult"),
            ("EnvironmentArguments", [], "arrayref<String>"),
            ("EnvironmentCurrentDirectory", [], "Value"),
            ("EnvironmentVariable", ["String"], "Value"),
            ("Int32ToString", ["Int32"], "String"),
            ("CharCategory", ["UInt32"], "Int32"),
            ("IntPtrToInt64", ["IntPtr"], "Int64"),
            ("UIntPtrToUInt64", ["UIntPtr"], "UInt64"),
            ("ObjectEquals", ["System.Object", "System.Object"], "Boolean"),
            ("ObjectReferenceEquals", ["System.Object", "System.Object"], "Boolean"),
            ("ObjectIdentityHash", ["System.Object"], "Int32"),
            ("ObjectTypeHandle", ["System.Object"], "System.RuntimeTypeHandle"),
            ("TypeName", ["System.RuntimeTypeHandle"], "String"),
            ("TypeEquals", ["System.RuntimeTypeHandle", "System.RuntimeTypeHandle"], "Boolean"),
            ("TypeArgumentCount", ["System.RuntimeTypeHandle"], "Int32"),
            ("TypeArgument", ["System.RuntimeTypeHandle", "Int32"], "System.RuntimeTypeHandle"),
            ("TypeShape", ["System.RuntimeTypeHandle", "Int32"], "Boolean"),
            ("TypeDisplayName", ["System.RuntimeTypeHandle", "Int32"], "String"),
            ("DefaultTaskQueue", [], "System.Tasks.TaskQueue"),
            ("RegisterDefaultTaskQueue", ["System.Tasks.TaskQueue"], "noresult"),
            ("CurrentTaskQueue", [], "System.Tasks.TaskQueue"),
            ("ExecutingAssembly", [], "System.Introspection.AssemblyInfo"),
            ("AssemblyName", ["String"], "String"),
            ("AssemblyMetadataToken", ["String"], "Int32"),
            ("AssemblyReferences", ["String"], "arrayref<System.Introspection.AssemblyInfo>"),
            ("AssemblyModules", ["String"], "arrayref<System.Introspection.ModuleInfo>"),
            ("AssemblyTypes", ["String"], "arrayref<System.Introspection.TypeInfo>"),
            ("ModuleAssembly", ["String", "String"], "System.Introspection.AssemblyInfo"),
            ("ModuleMetadataToken", ["String", "String"], "Int32"),
            ("ModuleTypes", ["String", "String"], "arrayref<System.Introspection.TypeInfo>"),
            ("TypeMetadataToken", ["System.RuntimeTypeHandle"], "Int32"),
            ("TypeDeclaringType", ["System.RuntimeTypeHandle"], "System.Option<System.Introspection.TypeInfo>"),
            ("TypeModule", ["System.RuntimeTypeHandle"], "System.Introspection.ModuleInfo"),
            ("TypeInfo", ["System.RuntimeTypeHandle"], "System.Introspection.TypeInfo"),
            ("TypeFields", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.FieldInfo>"),
            ("TypeMethods", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.MethodInfo>"),
            ("TypeProperties", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.PropertyInfo>"),
            ("TypeBaseType", ["System.RuntimeTypeHandle"], "System.Option<System.Introspection.TypeInfo>"),
            ("TypeElementType", ["System.RuntimeTypeHandle"], "System.Option<System.Introspection.TypeInfo>"),
            ("TypeInterfaces", ["System.RuntimeTypeHandle"], "arrayref<System.Introspection.TypeInfo>"),
            ("TypeGenericArguments", ["System.RuntimeTypeHandle"], "arrayref<System.Introspection.TypeInfo>"),
            ("TypeEnumNames", ["System.RuntimeTypeHandle"], "arrayref<String>"),
            ("TypeEnumUnderlying", ["System.RuntimeTypeHandle"], "System.Introspection.TypeInfo")
        }).ToArray();
    static string CSharp(string type) => type switch {
        "UInt32" => "uint", "Byte" => "byte", "Double" => "double", "String" => "string", "Int32" => "int", "Char" => "char",
        "Boolean" => "bool", "Int64" => "long", "Value" => "System.Value", "noresult" => "void",
        "IntPtr" => "System.IntPtr", "UIntPtr" => "System.UIntPtr", "UInt64" => "ulong",
        "System.Func<String,String>" => "System.Func<string,string>",
        "System.Func<Void>" => "System.Func<System.PropagationUnit>",
        _ when type.StartsWith("System.") => type,
        _ when type.StartsWith("arrayref<") => CSharp(type[9..^1]) + "[]",
        _ => throw new InvalidDataException("Unsupported runtime service declaration.")
    };
    static bool IsProperty(string name) => name is "CurrentTaskQueue" or "DefaultTaskQueue";
    public static string Declarations => "\n#nullable enable annotations\nnamespace Runtime.CompilerServices { public static class RuntimeServices { "
        + string.Join(" ", Members.Select(m => IsProperty(m.Name) ? $"public static {CSharp(m.Result)} {m.Name} => default;" : $"public static {CSharp(m.Result)} {m.Name}({string.Join(',', m.Args.Select((t, i) => CSharp(t) + ((m.Name == "ObjectReferenceEquals" || m.Name == "ObjectEquals" && i == 1) ? "?" : "") + " arg" + i))}) {(m.Result == "noresult" ? "{ }" : "=> default;")}")) + " public static bool IsValue<T>(System.Value value) => default; public static T UnpackValue<T>(System.Value value) => default; } }\n#nullable restore annotations\n";

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (reference.Name is "IsValue" or "UnpackValue")
            return BindValue(reference, definition);
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || !definition.IsPublic || !definition.IsStatic || definition.IsVirtual
            || definition.HasGenericParameters || reference is GenericInstanceMethod)
            throw new InvalidDataException("Unsupported runtime service call.");
        var (args, result) = RuntimeSignatures.Match(reference, definition,
            t => t.FullName == "System.Object" && (t.MetadataType == MetadataType.Object || RuntimeSignatures.IsCore(t.Scope) || ApplicationTypes.IsLibrary(t)) ? "System.Object" : t is ArrayType { IsVector: true, ElementType.MetadataType: MetadataType.Int32 } ? "arrayref<Int32>"
                : ManagedArrayBindings.Type(t) ?? ReflectionBindings.Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t));
        if (!Members.Any(m => (IsProperty(m.Name) ? "get_" + m.Name : m.Name) == reference.Name && m.Args.SequenceEqual(args) && m.Result == result))
            throw new InvalidDataException("Unsupported runtime service signature: " + reference.FullName);
        if (reference.Name == "StringEquals")
            return new("", args, result, Instruction: "ceq");
        if (reference.Name == "NotifyWorker")
            return new("", args, result, Instruction: "call neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)\npop");
        if (reference.Name == "RegisterDefaultTaskQueue")
            return new("", args, result, Instruction: "call neoCLR.Runtime.RegisterDefaultTaskQueue(System.Tasks.TaskQueue)\npop");
        if (reference.Name == "WriteLine")
            return new("", args, result, Instruction: "call neoCLR.Runtime.WriteLine(String)\npop");
        if (reference.Name is "IntPtrToInt64" or "UIntPtrToUInt64")
            return new("", args, result, Instruction: reference.Name == "IntPtrToInt64" ? "conv.i8" : "conv.u8");
        if (reference.Name == "LocalDateTime")
            return new("System.LocalDateTime::FromUnixTimeTicks", args, result);
        if (reference.Name == "TypeInfo")
            return new("System.Introspection.RuntimeTypeInfo::FromHandle", args, result);
        if (result.StartsWith("arrayref<"))
        {
            var element = result[9..^1];
            var name = "RuntimeService" + reference.Name;
            var parameters = string.Join(",", args.Select((t, i) => t + " arg" + i));
            var loads = string.Join("\n", args.Select((_, i) => "ldarg arg" + i));
            var signature = string.Join(",", args);
            Helpers[name] = $".function {name}({parameters}) -> {result}\n.local {element}[] source\n.local {result} destination\n.local Int32 index\n{loads}\ncall neoCLR.Runtime.{reference.Name}({signature})\nstloc source\nldloc source\nldlen\nconv.i4\nnewarr {element}\nstloc destination\nldc.i4 0\nstloc index\nbr Test\nCopy:\nldloc destination\nldloc index\nldloc source\nldloc index\nldelem {element}\nstelem {element}\nldloc index\nldc.i4 1\nadd\nstloc index\nTest:\nldloc index\nldloc source\nldlen\nconv.i4\nblt Copy\nldloc destination\nret\n.end\n";
            return new(name, args, result);
        }
        var serviceName = reference.Name.StartsWith("get_", StringComparison.Ordinal) ? reference.Name[4..] : reference.Name;
        if (IsProperty(serviceName) && !definition.IsGetter)
            throw new InvalidDataException("Runtime context lookup must be a property getter.");
        return new("neoCLR.Runtime." + serviceName, args, result);
    }
    static ResultBindings.Binding BindValue(MethodReference reference, MethodDefinition definition)
    {
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference is not GenericInstanceMethod method || method.GenericArguments.Count != 1
            || !definition.IsStatic || definition.IsVirtual || definition.GenericParameters.Count != 1
            || definition.GenericParameters[0].HasConstraints
            || definition.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant
            || definition.Parameters.Count != 1 || definition.Parameters[0].ParameterType.FullName != "System.Value"
            || !RuntimeSignatures.IsCore(definition.Parameters[0].ParameterType.Scope)
            || (reference.Name == "IsValue" ? definition.ReturnType.MetadataType != MetadataType.Boolean
                : definition.ReturnType is not GenericParameter parameter || parameter.Owner != definition || parameter.Position != 0))
            throw new InvalidDataException("Unsupported erased native value intrinsic.");
        var element = RuntimeSignatures.Map(method.GenericArguments[0], GenericUnionBindings.Type);
        if (element is not ("String" or "Char" or "UInt32" or "Byte" or "Int32" or "Int64" or "Void"))
            throw new InvalidDataException("Unsupported erased native payload type.");
        var shape = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        if (!shape.Args.SequenceEqual(new[] { "Value" })
            || shape.Result != (reference.Name == "IsValue" ? "Boolean" : element))
            throw new InvalidDataException("Invalid erased native value signature.");
        return new("", shape.Args, shape.Result,
            Instruction: (reference.Name == "IsValue" ? "value.is " : "value.unpack ") + element);
    }
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);
}
