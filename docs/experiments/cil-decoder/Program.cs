using System.Reflection;
using System.Reflection.Emit;
using System.Text.Json;

// Emit real method bodies with .NET's writer, independently of neoCLR's decoder.
var assembly = AssemblyBuilder.DefineDynamicAssembly(new AssemblyName("CilDecoderProbe"), AssemblyBuilderAccess.Run);
var module = assembly.DefineDynamicModule("Probe");
var type = module.DefineType("Calls", TypeAttributes.Public);
var answer = type.DefineMethod("Answer", MethodAttributes.Public | MethodAttributes.Static, typeof(int), Type.EmptyTypes);
var il = answer.GetILGenerator();
il.Emit(OpCodes.Nop);
il.Emit(OpCodes.Ldc_I4_S, (sbyte)42);
il.Emit(OpCodes.Ret);
var main = type.DefineMethod("Main", MethodAttributes.Public | MethodAttributes.Static, typeof(int), Type.EmptyTypes);
il = main.GetILGenerator();
il.Emit(OpCodes.Nop);
il.Emit(OpCodes.Call, answer);
il.Emit(OpCodes.Ret);
var created = type.CreateType();
var result = created.GetMethod("Main")!.Invoke(null, null);
if (!Equals(result, 42)) throw new Exception("CLR execution failed");
Console.WriteLine(JsonSerializer.Serialize(new {
    Result = result,
    Methods = new[] { "Answer", "Main" }.Select(name => new {
        Name = name,
        Token = created.GetMethod(name)!.MetadataToken,
        Bytes = created.GetMethod(name)!.GetMethodBody()!.GetILAsByteArray()!.Select(b => (int)b).ToArray()
    })
}, new JsonSerializerOptions { WriteIndented = true }));
