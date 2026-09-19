"""Behavioral cases for the basic Iterable operators; used by verify_queries.py."""
from pathlib import Path

HEADER = '''import System.*
import System.Collections.*
import System.Linq.*
import System.Console.*
func ShowBoolean(value: bool) {
    if value {
        WriteLine("True")
    } else {
        WriteLine("False")
    }
}

'''
TRACKED = HEADER + '''
class Source : Iterable<int> {
    var Acquired: int
    var Advanced: int
    var Read: int
    var Disposed: int
    var Limit: int

    init(limit: int) {
        Limit = limit
    }

    func GetIterator() -> Iterator<int> {
        Acquired = Acquired + 1
        return Cursor(self)
    }
}

class Cursor : Iterator<int> {
    var Owner: Source
    var Position: int

    init(owner: Source) {
        Owner = owner
    }

    func MoveNext() -> bool {
        Owner.Advanced = Owner.Advanced + 1
        Position = Position + 1
        return Position <= Owner.Limit
    }

    val Current: int {
        get {
            Owner.Read = Owner.Read + 1
            return Position
        }
    }

    func Dispose() {
        Owner.Disposed = Owner.Disposed + 1
    }
}

func State(source: Source) {
    WriteLine(source.Acquired)
    WriteLine(source.Advanced)
    WriteLine(source.Read)
    WriteLine(source.Disposed)
}
'''

def cases():
    samples = Path(__file__).resolve().parent / 'samples'
    result = [('Basic query sample', (samples / 'library-query-basics.rvn').read_text(),
               (samples / 'library-query-basics.expected.txt').read_text())]
    result.append(('Empty queries and seeded heterogeneous fold', HEADER + '''
func Main() {
    let empty: int[] = []
    ShowBoolean(empty.Any())
    ShowBoolean(empty.Any(value => true))
    ShowBoolean(empty.All(value => false))
    WriteLine(empty.Count())
    WriteLine(empty.Count(value => true))
    WriteLine(empty.Fold("seed", (text, value) => "wrong"))
    let values: int[] = [1, 2, 3]
    WriteLine(values.Fold("", (text, value) => String.Concat(text, value.ToString())))
    WriteLine(values.Fold(10, (total, value) => total - value))
    WriteLine(values.Take(-1).Count())
    WriteLine(values.Take(0).Count())
    WriteLine(values.Take(10).Count())
    WriteLine(values.Skip(-1).Count())
    WriteLine(values.Skip(0).Count())
    WriteLine(values.Skip(10).Count())
    WriteLine(empty.Concat(values).Concat(empty).Count())
}
''', 'False\nFalse\nTrue\n0\n0\nseed\n123\n4\n0\n0\n3\n3\n3\n0\n3\n'))
    for name, expression, expected in [
        ('Any', 'source.Any()', 'True\n1\n1\n0\n1\n'),
        ('Any predicate', 'source.Any(value => value == 2)', 'True\n1\n2\n2\n1\n'),
        ('All predicate', 'source.All(value => value < 2)', 'False\n1\n2\n2\n1\n'),
        ('Count', 'source.Count()', '3\n1\n4\n0\n1\n'),
        ('Count predicate', 'source.Count(value => value != 2)', '2\n1\n4\n3\n1\n'),
        ('Fold', 'source.Fold(10, (total, value) => total - value)', '4\n1\n4\n3\n1\n'),
    ]:
        result.append((name + ' traversal and disposal', TRACKED + '''
func Main() {
    let source = Source(3)
    ''' + ('ShowBoolean' if name.startswith(('Any', 'All')) else 'WriteLine') + '(' + expression + ''')
    State(source)
}
''', expected))
    result.append(('Take bounds consumption and repeated enumeration', TRACKED + '''
func Main() {
    let source = Source(5)
    let query = source.Take(2)
    let iterator = query.GetIterator()
    State(source)
    iterator.MoveNext()
    WriteLine(iterator.Current)
    WriteLine(iterator.Current)
    iterator.MoveNext()
    WriteLine(iterator.Current)
    ShowBoolean(iterator.MoveNext())
    ShowBoolean(iterator.MoveNext())
    iterator.Dispose()
    State(source)
    WriteLine(query.Count())
    State(source)
    let unused = Source(5)
    WriteLine(unused.Take(0).Count())
    let abandoned = unused.Take(2).GetIterator()
    abandoned.Dispose()
    ShowBoolean(abandoned.MoveNext())
    State(unused)
}
''', '0\n0\n0\n0\n1\n1\n2\nFalse\nFalse\n1\n2\n2\n1\n2\n2\n4\n4\n2\n0\nFalse\n0\n0\n0\n0\n'))
    result.append(('Skip avoids reading skipped values and disposes early', TRACKED + '''
func Main() {
    let source = Source(5)
    let iterator = source.Skip(2).GetIterator()
    State(source)
    iterator.MoveNext()
    WriteLine(iterator.Current)
    WriteLine(iterator.Current)
    iterator.Dispose()
    iterator.Dispose()
    ShowBoolean(iterator.MoveNext())
    State(source)
    let empty = Source(0)
    WriteLine(empty.Skip(9).Count())
    State(empty)
}
''', '0\n0\n0\n0\n3\n3\nFalse\n1\n3\n1\n1\n0\n1\n1\n0\n1\n'))
    result.append(('Concat defers second acquisition and owns active cursor', TRACKED + '''
func Main() {
    let left = Source(2)
    let right = Source(1)
    let query = left.Concat(right)
    let iterator = query.GetIterator()
    State(left)
    State(right)
    iterator.MoveNext()
    WriteLine(iterator.Current)
    iterator.Dispose()
    iterator.Dispose()
    State(left)
    State(right)
    for value in query {
        WriteLine(value)
    }
    State(left)
    State(right)
}
''', '0\n0\n0\n0\n0\n0\n0\n0\n1\n1\n1\n1\n1\n0\n0\n0\n0\n1\n2\n1\n2\n4\n3\n2\n1\n2\n1\n1\n'))
    result.append(('FlatMap nested lifetime, empty children and cached Current', TRACKED + '''
func Main() {
    let outer = Source(2)
    let child = Source(2)
    var calls = 0
    let query = outer.FlatMap((value: int) -> Iterable<int> => {
        calls = calls + 1
        return child
    })
    let iterator = query.GetIterator()
    WriteLine(calls)
    State(outer)
    iterator.MoveNext()
    WriteLine(iterator.Current)
    WriteLine(iterator.Current)
    WriteLine(calls)
    iterator.Dispose()
    iterator.Dispose()
    ShowBoolean(iterator.MoveNext())
    State(outer)
    State(child)
    WriteLine(query.Count())
    WriteLine(calls)
    State(outer)
    State(child)
    let empty = Source(0)
    let allEmpty = outer.FlatMap((value: int) -> Iterable<int> => empty)
    WriteLine(allEmpty.Count())
    State(empty)
}
''', '0\n0\n0\n0\n0\n1\n1\n1\nFalse\n1\n1\n1\n1\n1\n1\n1\n1\n4\n3\n2\n4\n3\n2\n3\n7\n5\n3\n0\n2\n2\n0\n2\n'))
    result.append(('FlatMap reference payload and empty outer', HEADER + '''
public func Names(value: int) -> Iterable<string> {
    if value == 2 {
        let empty: string[] = []
        return empty
    }
    let names: string[] = ["a", "b"]
    return names
}
func Main() {
    let values: int[] = [1, 2, 3]
    let query = values.FlatMap(value => Names(value))
    WriteLine(query.Fold("", (text, value) => String.Concat(text, value)))
    let empty: int[] = []
    WriteLine(empty.FlatMap(value => Names(value)).Count())
    let iterator = query.GetIterator()
    iterator.MoveNext()
    WriteLine(iterator.Current)
    iterator.Dispose()
}
''', 'abab\n0\na\n'))
    return result
