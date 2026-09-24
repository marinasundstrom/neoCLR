using Mono.Cecil;
using Mono.Cecil.Cil;

using var assembly = AssemblyDefinition.ReadAssembly(args[0]);
var originalReferences = assembly.MainModule.AssemblyReferences.ToHashSet();
if (args.Length == 3)
{
    var carrier = assembly.MainModule.Types.Single(t => t.FullName == "HttpErrorProbe.HttpError");
    if (args[1].StartsWith("explicit-"))
    {
        var limit = assembly.MainModule.Types.Single(t => t.FullName == "HttpErrorProbe.HttpLimit");
        if (args[1] == "explicit-unmarked")
            limit.CustomAttributes.Remove(limit.CustomAttributes.Single(a =>
                a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute"));
        else if (args[1] == "explicit-payload")
            limit.NestedTypes[0].Fields.Add(new FieldDefinition("Payload", FieldAttributes.Private,
                limit.Fields.Single(f => f.Name == "<Tag>").FieldType));
        else if (args[1] == "explicit-tag-overlap")
            limit.Fields.Single(f => f.Name == "<HeadersPayload>").Offset = 0;
        else throw new Exception("Unknown explicit-layout mutation");
    }
    else if (args[1] == "nonconstructor-init")
    {
        var body = carrier.Methods.Single(m => m.Name == "ToString").Body;
        var il = body.GetILProcessor();
        var first = body.Instructions[0];
        il.InsertBefore(first, il.Create(OpCodes.Ldarg_0));
        il.InsertBefore(first, il.Create(OpCodes.Initobj, carrier));
    }
    else if (args[1] == "ordinary-out")
    {
        carrier.CustomAttributes.Remove(carrier.CustomAttributes.Single(a =>
            a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute"));
    }
    else if (args[1] == "unassigned-success")
    {
        foreach (var method in carrier.Methods.Where(m => m.Name == "TryGetValue"))
        {
            var returns = method.Body.Instructions.Where(i => i.OpCode == OpCodes.Ret).ToArray();
            var failure = returns.Last().Previous;
            if (failure.OpCode != OpCodes.Ldc_I4_0) throw new Exception("Unexpected extraction shape");
            failure.OpCode = OpCodes.Ldc_I4_1;
        }
    }
    else throw new Exception("Unknown mutation");
    // Cecil can add an unused host-core reference while decoding method bodies.
    foreach (var method in assembly.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
    { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
    foreach (var reference in assembly.MainModule.AssemblyReferences.Where(r => !originalReferences.Contains(r)).ToArray())
    {
        if (assembly.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
            throw new Exception("Mutation introduced an external scope");
        assembly.MainModule.AssemblyReferences.Remove(reference);
    }
    assembly.Write(args[2]);
    return;
}
foreach (var type in assembly.MainModule.Types.Where(t => t.Namespace == "HttpErrorProbe" || t.Name == "IUnion"))
{
    Console.WriteLine($"Type {type.FullName}: {type.Attributes}; base={type.BaseType}");
    foreach (var contract in type.Interfaces)
        Console.WriteLine($"  implements {contract.InterfaceType}");
    foreach (var field in type.Fields)
        Console.WriteLine($"  field {field.Name}: {field.FieldType}; {field.Attributes}; offset={field.Offset}");
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
            Console.WriteLine($"    field {field.Name}: {field.FieldType}; {field.Attributes}; offset={field.Offset}");
    }
}
