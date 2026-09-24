using Mono.Cecil;

using var assembly = AssemblyDefinition.ReadAssembly(args[0]);
foreach (var type in assembly.MainModule.Types.Where(t => t.Namespace == "HttpErrorProbe" || t.Name == "IUnion"))
{
    Console.WriteLine($"Type {type.FullName}: {type.Attributes}; base={type.BaseType}");
    foreach (var contract in type.Interfaces)
        Console.WriteLine($"  implements {contract.InterfaceType}");
    foreach (var field in type.Fields)
        Console.WriteLine($"  field {field.Name}: {field.FieldType}; {field.Attributes}");
    foreach (var method in type.Methods)
    {
        Console.WriteLine($"  {method.FullName}");
        if (method.HasBody && (method.IsConstructor || method.Name == "TryGetValue"))
            foreach (var instruction in method.Body.Instructions)
                Console.WriteLine("    " + instruction);
    }
    foreach (var nested in type.NestedTypes)
    {
        Console.WriteLine($"  nested {nested.FullName}: {nested.Attributes}");
        foreach (var field in nested.Fields)
            Console.WriteLine($"    field {field.Name}: {field.FieldType}; {field.Attributes}");
    }
}
