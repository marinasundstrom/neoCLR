using Mono.Cecil;

// A normal Raven/CLI enum declaration. Representation and common enum operations
// are target lowering, like the runtime's existing generic enum emitter.
static class FlagsLibrary
{
    public static bool IsMatched(TypeDefinition type) => EnumBindings.IsType(type.FullName) && ApplicationTypes.IsLibrary(type);
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner)
    {
        EnumBindings.Validate(core, owner);
        EnumBindings.Validate(source, owner);
        var type = source.GetType(owner);
        ApplicationTypes.BindLibrary(type, owner);
        _ = ApplicationTypes.Type(type);
        return [];
    }
}
