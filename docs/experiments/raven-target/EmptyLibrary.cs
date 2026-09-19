using Mono.Cecil;

// These existing values carry no payload. Unlike message Error, their defaults
// are meaningful values; their constructors and formatting remain library code.
static class EmptyLibrary
{
    public static bool IsError(TypeDefinition type) => ErrorBindings.Cases.TryGetValue(type.FullName, out var cases)
        && cases.Length == 0;
    public static bool IsByValueReceiver(MethodReference method) => ApplicationTypes.IsLibrary(method.DeclaringType)
        && (IsError(method.DeclaringType.Resolve()) || ErrorCarrierLibrary.IsCase(method.DeclaringType.Resolve())) && method.HasThis;
    public static bool OmitConstructor(MethodDefinition method) => method.DeclaringType.FullName == "System.EnvironmentError"
        && PrimitiveLibrary.IsDefaultConstructor(method);
    public static void Project(ModuleDefinition module)
    {
        foreach (var type in module.Types.Where(t => IsError(t) || t.FullName == "System.Void"))
        {
            if (!type.IsValueType || type.HasFields) throw new InvalidDataException("Expected empty value declaration.");
            type.PackingSize = -1;
            type.ClassSize = -1;
        }
    }
}
