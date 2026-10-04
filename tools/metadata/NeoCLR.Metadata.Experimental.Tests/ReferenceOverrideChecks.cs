using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;

internal static class ReferenceOverrideChecks
{
    internal static void Run()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ReferenceOverrides", new(1, 0, 0, 0)), core);
        foreach (bool manual in new[] { false, true })
        {
            var owner = graph.AddClass("Example", manual ? "Manual" : "Built");
            owner.AddConstructor(Array.Empty<PrimitiveType>()).GetILGenerator().Return();
            foreach (var (slot, signature) in new[] {
                ("ToString", new MethodSignature(PrimitiveType.String, [])),
                ("GetHashCode", new MethodSignature(PrimitiveType.Int32, [])),
                ("Equals", new MethodSignature(PrimitiveType.Boolean, [graph.CoreObjectType])) })
            {
                MethodBuilder method;
                if (manual)
                {
                    var definition = new MethodDefinition(slot, 0x46, signature);
                    owner.Definition.Methods.Add(definition); method = MethodBuilder.ForDefinition(definition);
                }
                else method = owner.AddOverride(slot, signature);
                var il = method.GetILGenerator();
                if (slot == "ToString") il.Emit(OpCode.Ldstr, "descriptor");
                else if (slot == "GetHashCode") il.LoadConstant(42);
                else il.Emit(OpCode.Ldc_Bool, true);
                il.Return();
            }
        }
        var image = graph.Write();
        var context = new AssemblyLoadContext("reference-override-check", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            foreach (var type in assembly.GetTypes().Where(t => t.Namespace == "Example"))
            {
                var value = Activator.CreateInstance(type)!;
                if (value.ToString() != "descriptor" || value.GetHashCode() != 42 || !value.Equals(new object()))
                    throw new Exception($"Object dispatch failed: {type.Name}: {value.ToString()} / {value.GetHashCode()} / {value.Equals(new object())}");
                foreach (var slot in new[] { "ToString", "GetHashCode", "Equals" })
                    if (type.GetMethods().Single(m => m.Name == slot && !m.IsStatic).GetBaseDefinition().DeclaringType != typeof(object))
                        throw new Exception("Object slot identity lost");
            }
        }
        finally { context.Unload(); }
        var wrong = graph.CreateTypeReference(new("OtherCore", new(1, 0, 0, 0)), core, new string('a', 64), "System", "Object");
        try { graph.AddClass("Example", "WrongCore").AddOverride("Equals", new(PrimitiveType.Boolean, [wrong])); }
        catch (InvalidOperationException) { return; }
        throw new Exception("foreign Object identity accepted");
    }
}
