using Mono.Cecil;

static class LibrarySignatureChecks
{
    // Reproduce the external-token shape observed in Raven's emitted metadata,
    // independently of Cecil's mutable TypeSystem cache for the defining core.
    sealed class EncodedVoid(ModuleDefinition module) : TypeReference("System", "Void", module, module, true)
    {
        public override MetadataType MetadataType => MetadataType.Void;
    }

    public static void Verify(string core)
    {
        using var image = AssemblyDefinition.ReadAssembly(core);
        var module = image.MainModule;
        using var nominal = AssemblyDefinition.ReadAssembly(core);
        var value = new TypeReference("System", "Void", nominal.MainModule, nominal.MainModule, true);
        var primitive = new EncodedVoid(module);
        GenericInstanceType Option(TypeReference argument)
        {
            var result = new GenericInstanceType(module.GetType("System.Option`1"));
            result.GenericArguments.Add(argument);
            return result;
        }
        void Require(bool condition, string message)
        {
            if (!condition) throw new InvalidDataException(message);
        }
        Require(!LibraryImplementation.SameType(primitive, value), "No-result and value returns must stay distinct.");
        Require(LibraryImplementation.SameType(Option(primitive), Option(value)), "Generic Void encodings must match.");
        using var other = AssemblyDefinition.ReadAssembly(core);
        other.Name.Version = new Version(99, 0);
        Require(!LibraryImplementation.SameType(Option(primitive), Option(new TypeReference("System", "Void", other.MainModule, other.MainModule, true))), "Different core identities must not match.");
        other.Name.Name = "Other.Core";
        Require(!LibraryImplementation.SameType(Option(primitive), Option(new TypeReference("System", "Void", other.MainModule, other.MainModule, true))), "Foreign Void types must not match.");
        value.Resolve().Fields.Add(new FieldDefinition("Payload", FieldAttributes.Public, module.TypeSystem.Int32));
        Require(!LibraryImplementation.SameType(Option(primitive), Option(value)), "A nonempty Void must not match the storage projection.");
        Console.WriteLine("Generic Void encodings match; no-result returns, foreign/versioned cores and nonempty Void are rejected.");
    }
}
