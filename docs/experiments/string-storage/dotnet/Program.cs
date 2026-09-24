using System;

string original = new string("hello 👩‍💻".ToCharArray());
string alias = original;
object view = original;
string roundTrip = (string)view;
string separate = new string(original.ToCharArray());
var holder = new Holder(original);
string[] array = [original];
if (!ReferenceEquals(original, alias) || !ReferenceEquals(original, view)
    || !ReferenceEquals(original, roundTrip) || !ReferenceEquals(original, view.ToString())
    || !ReferenceEquals(original, holder.Text) || !ReferenceEquals(original, array[0]))
    throw new Exception("String identity was lost");
if (ReferenceEquals(original, separate) || !original.Equals(separate))
    throw new Exception("Identity and content equality were conflated");
int identityHash = System.Runtime.CompilerServices.RuntimeHelpers.GetHashCode(original);
if (identityHash != System.Runtime.CompilerServices.RuntimeHelpers.GetHashCode(view))
    throw new Exception("String identity hashes differ for aliases");
GC.Collect();
GC.WaitForPendingFinalizers();
if (identityHash != System.Runtime.CompilerServices.RuntimeHelpers.GetHashCode(original)
    || !ReferenceEquals(array[0], holder.Text) || holder.Text != "hello 👩‍💻")
    throw new Exception("String roots did not survive collection");
Console.WriteLine(".NET String identity through locals, Object, records, arrays and GC: passed");
char[] characters = ['F', 'o', 'o'];
string copied = new string(characters);
characters[0] = 'B';
if (copied != "Foo" || new string(Array.Empty<char>()) != ""
    || new string(['e', '\u0301']).Length != 2)
    throw new Exception("String char-array snapshot or UTF-16 length failed");
Console.WriteLine(".NET String char-array snapshot and UTF-16 length: passed");
record Holder(string Text);
