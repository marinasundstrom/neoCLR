using Mono.Cecil;

// Keep runtime startup parameterless; inject a copied application argument vector.
static class EntryPointBindings
{
    public const string Startup = "RuntimeMainArguments";

    public static bool HasArguments(MethodDefinition entry, bool collectionProfile)
    {
        if (!entry.IsStatic || entry.HasGenericParameters || entry.DeclaringType.HasGenericParameters
            || entry.ReturnType.MetadataType != MetadataType.Void)
            throw new InvalidDataException("Entry must be static, nongeneric and return no result.");
        if (entry.Parameters.Count == 0) return false;
        if (!collectionProfile || entry.Parameters.Count != 1
            || entry.Parameters[0].IsOut || entry.Parameters[0].IsIn
            || entry.Parameters[0].ParameterType is not ArrayType { IsVector: true } array
            || array.ElementType.MetadataType != MetadataType.String)
            throw new InvalidDataException("Entry requires no parameters or one String[] parameter in the managed collection profile.");
        return true;
    }

    public static string Adapter(string entry) => $$"""
        .function {{Startup}}() -> noresult
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
            call {{entry}}(arrayref<String>)
            ret
        .end

        """;
}
