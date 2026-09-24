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
GC.Collect();
GC.WaitForPendingFinalizers();
if (!ReferenceEquals(array[0], holder.Text) || holder.Text != "hello 👩‍💻")
    throw new Exception("String roots did not survive collection");
Console.WriteLine(".NET String identity through locals, Object, records, arrays and GC: passed");
record Holder(string Text);
