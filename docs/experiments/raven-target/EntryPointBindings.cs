using Mono.Cecil;
using System.Text;

// Keep runtime startup parameterless; adapt arguments, completion and process results.
static class EntryPointBindings
{
    public const string Startup = "RuntimeMainArguments";

    public static bool HasArguments(MethodDefinition entry, bool collectionProfile)
    {
        if (!entry.IsStatic || entry.HasGenericParameters || entry.DeclaringType.HasGenericParameters)
            throw new InvalidDataException("Entry must be static and nongeneric.");
        ValidateReturn(entry.ReturnType, collectionProfile);
        if (entry.Parameters.Count == 0) return false;
        if (!collectionProfile || entry.Parameters.Count != 1
            || entry.Parameters[0].IsOut || entry.Parameters[0].IsIn
            || entry.Parameters[0].ParameterType is not ArrayType { IsVector: true } array
            || array.ElementType.MetadataType != MetadataType.String)
            throw new InvalidDataException("Entry requires no parameters or one String[] parameter in the managed collection profile.");
        return true;
    }

    static bool IsUnit(TypeReference type) => type.MetadataType == MetadataType.Void
        || type.FullName == "System.Void" && type.IsValueType && RuntimeSignatures.IsCore(type.Scope);

    static TypeReference ValidateReturn(TypeReference type, bool collectionProfile)
    {
        if (IsUnit(type)) return type;
        if (!collectionProfile) throw new InvalidDataException("Value-returning entries require the managed collection profile.");
        if (type is GenericInstanceType task && task.ElementType.FullName == "System.Tasks.Task`1"
            && RuntimeSignatures.IsCore(task.Scope) && task.GenericArguments.Count == 1)
            type = task.GenericArguments[0];
        if (type.MetadataType == MetadataType.Int32 || IsUnit(type)) return type;
        if (type is GenericInstanceType result && result.ElementType.FullName == "System.Result`2"
            && RuntimeSignatures.IsCore(result.Scope) && result.GenericArguments.Count == 2
            && (result.GenericArguments[0].MetadataType == MetadataType.Int32 || IsUnit(result.GenericArguments[0])))
            return type;
        throw new InvalidDataException("Entry must return unit, Int32, Result<Int32|unit,E>, or a target Task wrapping one of these.");
    }

    public static string Adapter(MethodDefinition method, string entry, Func<TypeReference, string> map)
    {
        var hasArguments = HasArguments(method, true);
        var produced = ValidateReturn(method.ReturnType, true);
        var task = method.ReturnType is GenericInstanceType candidate && candidate.ElementType.FullName == "System.Tasks.Task`1";
        var text = new StringBuilder($".function {Startup}() -> Int32\n");
        if (produced is GenericInstanceType resultLocals)
            text.AppendLine($".local {map(resultLocals)} result\n.local {map(resultLocals.GenericArguments[0])} success\n.local {map(resultLocals.GenericArguments[1])} error\n.local System.Object errorObject");
        if (hasArguments) text.Append(Arguments);
        text.AppendLine($"call {entry}({(hasArguments ? "arrayref<String>" : "")})");
        if (task)
        {
            text.AppendLine("call neoCLR.Runtime.DrainEntryTasks()\npop");
            text.AppendLine($"call instance {map(method.ReturnType)}::GetResult()");
        }
        if (produced is GenericInstanceType result)
        {
            var owner = map(result);
            var ok = map(result.GenericArguments[0]);
            var error = map(result.GenericArguments[1]);
            text.AppendLine($"stloc result\nldloca result\nldloca success\ncall instance {owner}::TryGetOutput({ok}&)\nbrfalse Error");
            text.AppendLine(ok == "Int32" ? "ldloc success\nret" : "ldc.i4 0\nret");
            text.AppendLine($"Error:\nldloca result\nldloca error\ncall instance {owner}::TryGetResidual({error}&)\nbrfalse InvalidResult");
            text.AppendLine($"ldloc error\nbox {error}\ncastclass System.Object\nstloc errorObject");
            text.AppendLine("""
                call System.Console::get_Error()
                ldloc errorObject
                ref.isnull
                brtrue NullError
                ldloc errorObject
                callvirt instance System.Object::ToString()
                br WriteError
            NullError:
                ldstr ""
            WriteError:
                callvirt instance System.IO.TextWriter::WriteLine(String)
                pop
                ldc.i4 1
                ret
            InvalidResult:
                fault "Invalid entry Result"
            """);
        }
        else if (produced.MetadataType == MetadataType.Int32) text.AppendLine("ret");
        else text.AppendLine((task || method.ReturnType.MetadataType != MetadataType.Void ? "pop\n" : "") + "ldc.i4 0\nret");
        text.AppendLine(".end");
        if (task) text.AppendLine(".function neoCLR.Runtime.DrainEntryTasks() -> Void\n.methodimpl InternalCall\n.end");
        return text.ToString();
    }

    const string Arguments = """
            .local arrayref<String> source
            .local arrayref<String> destination
            .local Int32 count
            .local Int32 index
            call RuntimeArguments()
            stloc source
            ldloc source
            ldlen
            conv.i4
            stloc count
            ldloc count
            ldc.i4 0
            ble Allocate
            ldloc count
            ldc.i4 1
            sub
            stloc count
        Allocate:
            ldloc count
            newarr String
            stloc destination
            ldc.i4 0
            stloc index
            br Test
        Copy:
            ldloc destination
            ldloc index
            ldloc source
            ldloc index
            ldc.i4 1
            add
            ldelem String
            stelem String
            ldloc index
            ldc.i4 1
            add
            stloc index
        Test:
            ldloc index
            ldloc count
            blt Copy
            ldloc destination

        """;
}
