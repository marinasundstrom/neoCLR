using System.Runtime.InteropServices;

static void Check(bool condition, string message)
{
    if (!condition) throw new Exception(message);
}

var queue = new Queue<int>(4);
for (var i = 0; i < 4; i++) queue.Enqueue(i);
Check(queue.Dequeue() == 0 && queue.Dequeue() == 1, "FIFO removal");
for (var i = 4; i < 10; i++) queue.Enqueue(i);
Check(queue.SequenceEqual(Enumerable.Range(2, 8)), "wrap/growth preserves FIFO");
Check(queue.Peek() == 2 && queue.Count == 8, "peek does not remove");
var queueIterator = queue.GetEnumerator();
Check(queueIterator.MoveNext(), "queue iterator begins");
queue.Enqueue(10);
try { queueIterator.MoveNext(); throw new Exception("queue mutation accepted"); }
catch (InvalidOperationException) { }
queue.Clear();
Check(queue.Count == 0 && !queue.TryDequeue(out _) && !queue.TryPeek(out _), "empty queue");

var stack = new Stack<int>(1);
for (var i = 0; i < 10; i++) stack.Push(i);
Check(stack.SequenceEqual(Enumerable.Range(0, 10).Reverse()), "LIFO iteration");
Check(stack.Peek() == 9 && stack.Pop() == 9 && stack.Count == 9, "LIFO removal");
var stackIterator = stack.GetEnumerator();
Check(stackIterator.MoveNext(), "stack iterator begins");
stack.Pop();
try { stackIterator.MoveNext(); throw new Exception("stack mutation accepted"); }
catch (InvalidOperationException) { }
stack.Clear();
Check(stack.Count == 0 && !stack.TryPop(out _) && !stack.TryPeek(out _), "empty stack");

var nullableQueue = new Queue<string?>();
nullableQueue.Enqueue(null);
Check(nullableQueue.TryDequeue(out var item) && item is null, "null is a present element");
Check(!nullableQueue.TryDequeue(out _), "empty differs from present null");
Console.WriteLine($"PASS: {RuntimeInformation.FrameworkDescription}; FIFO wrap/growth, LIFO, peek, mutation, clear, empty/null");
