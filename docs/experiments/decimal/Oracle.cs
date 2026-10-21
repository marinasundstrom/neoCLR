using System;
using System.Globalization;
class Oracle {
static void Main() {
string[] values = ["0", "-0.00", "1.00", "-1.25", "0.1", "3", "79228162514264337593543950335", "0.0000000000000000000000000001", "123456789012345.6789", "-999999999.999999999", "2.5", "0.0000000000000000000000000003"];
Console.WriteLine("module NeoClr.RuntimeTests\nimport System.*\nimport System.Result.*\nimport NeoClr.Testing.*\n\n[Test(\"Decimal arithmetic agrees with CLR boundary vectors\")]\nfunc FOracle() -> Result<unit, TestFailure> {");
int n=0;
for(int i=0;i<values.Length;i++) {
string a=values[i],b=values[(i+3)%values.Length];
decimal x=decimal.Parse(a,CultureInfo.InvariantCulture), y=decimal.Parse(b,CultureInfo.InvariantCulture);
Console.WriteLine($"    let Ok(a{i}) = Decimal.Parse(\"{a}\") else {{ return Assert.True(false, \"a{i}\") }}\n    let Ok(b{i}) = Decimal.Parse(\"{b}\") else {{ return Assert.True(false, \"b{i}\") }}");
foreach(string op in new[]{"+","-","*","/","%"}) try {
decimal z=op switch {"+"=>x+y,"-"=>x-y,"*"=>x*y,"/"=>x/y,_=>x%y};
Console.WriteLine($"    _ = Assert.Equal(\"{z.ToString(CultureInfo.InvariantCulture)}\", (a{i} {op} b{i}).ToString())?");n++;
} catch(ArithmeticException) {}
}
Console.WriteLine("    return Assert.True(true, \"CLR arithmetic vectors\")\n}");
Console.Error.WriteLine(n+" vectors");
}
}
