var first = new Counter(1);
var second = new Counter(10);
var selected = first;
selected = second;
selected.Age = 11;
Console.WriteLine(first.Age);
Console.WriteLine(second.Age);
if (first.Age != 1 || second.Age != 11) throw new Exception("Reference assignment failed");

ReplaceLocally(selected);
if (selected.Age != 11) throw new Exception("Parameter reassignment escaped");
ReplaceOutput(out selected);
Console.WriteLine(selected.Age);
Console.WriteLine(second.Age);
if (selected.Age != 40 || second.Age != 11) throw new Exception("Output replacement failed");

var a = 1;
var b = 10;
ref int alias = ref a;
alias = b;
b = 11;
Console.WriteLine(a);
if (a != 10) throw new Exception("Byref target write failed");
alias = ref b;
alias = 12;
if (b != 12 || a != 10) throw new Exception("Byref reassignment failed");

static void ReplaceLocally(Counter value) => value = new Counter(99);
static void ReplaceOutput(out Counter value) => value = new Counter(40);
sealed class Counter(int age) { public int Age = age; }
