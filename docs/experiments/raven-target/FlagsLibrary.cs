using Mono.Cecil;

// A normal Raven/CLI enum declaration. Representation and common enum operations
// are target lowering, like the runtime's existing generic enum emitter.
static class FlagsLibrary
{
    public static bool IsMatched(TypeDefinition type) => type.FullName == EnumBindings.Flags && ApplicationTypes.IsLibrary(type);
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core)
    {
        EnumBindings.Validate(core);
        EnumBindings.Validate(source);
        var type = source.GetType(EnumBindings.Flags);
        ApplicationTypes.BindLibrary(type, EnumBindings.Flags);
        _ = ApplicationTypes.Type(type);
        return [];
    }
}
