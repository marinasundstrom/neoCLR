"""Check saved-source execution and rejection without stale-artifact fallback."""
import argparse
from runner_options import add_toolchain_arguments, runner_arguments
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--collections', action='store_true')
parser.add_argument('--only', nargs='+', help='Select positive sample labels; always run edit/rejection checks')
parser.add_argument('--start-at', help='Resume positive sample checks at this label; always run edit/rejection checks')
add_toolchain_arguments(parser)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-project-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    cases = [('Utf8', 'library-utf8.rvn', (bridge / 'samples/library-utf8.expected.txt').read_text()),
             ('NestedTypeInfo', 'library-nested-type-info.rvn', 'Outer/Inner\nOuter\nActual declaring type\nTop-level type\nType member\n'),
             ('IntrospectionTour', 'library-introspection-tour.rvn', (bridge / 'samples/library-introspection-tour.expected.txt').read_text()),
             ('AssemblyInfo', 'library-assembly-info.rvn', 'Demo\nSystem.Runtime\nRuntime types available\nDemo\n1\nWidget\nType token available\nDemo\nSame definition token\n'),
             ('TypeAcquisition', 'library-type-acquisition.rvn', 'Concrete class\nSystem.String\nSame type\nArray type\nSystem.Int32\nSystem.Int32\n'),
             ('Instants', 'library-instants.rvn', '0\n-1\n0\n-1\nSame duration\nSystem clock\n'),
             ('Basics', 'library-basics.rvn', '42\n1\n0\nLibrary calls from Raven\n'),
             ('CasePayloads', 'library-case-payloads.rvn', '7\n42\nAfter\nUpdated error\n'),
             ('GenericUnions', 'library-unions.rvn', 'Ok\n0\nError\nFailure\n0\nFailure\nFound\nNone\n0\n'),
             ('ErrorValues', 'library-errors.rvn', (bridge / 'samples/library-errors.expected.txt').read_text()),
             ('Calendar', 'library-calendar.rvn', '2024\n2\n29\n60\n738944\n0\nSame date\nDate accepted\n1\n1\n1\n1\n0\n0\nSame date\nDate accepted\nInvalid date\nInvalid date\n' + '12\n34\n56\n789\n7890123\n0\n0\nSame time\nTime accepted\n' * 2 + '0\n0\n0\n0\n0\n-1\n0\nSame time\nTime accepted\nInvalid time\nInvalid time\n'),
             ('Primitives', 'library-primitives.rvn', '-1\n1\n-1\n1\n1\n1\n0\n0\n0\n-1\n-1\nDigit\nNumber\nLetter\nUpper\nLower\nSeparator\nControl\nPunctuation\nSymbol\nSurrogate\nHigh\nLow\nASCII\nASCII digit\nLetter or digit\nWhitespace\n'),
             ('NumericOperators', 'library-numeric-operators.rvn', '0\n' * 13),
             ('NumericWidening', 'library-numeric-widening.rvn', '0\n0\n0\n0\n'),
             ('FloatingMath', 'library-floating-math.rvn', '0\n' * 19 + '-1\n'),
             ('Clamp', 'library-clamp.rvn', 'Clamped\n5\nClamped\n0\nClamped\n10\nClamped\n7\nClamped\n-2147483648\nClamped\n2147483647\nInvalid range\n'),
             ('Integers', 'library-integers.rvn', '42\n1\nEqual\n-2147483648\n-1\nDifferent\n2147483647\n1\nDifferent\n0\n0\nEqual\n'),
             ('Division', 'library-division.rvn', 'Divided\n3\nDivided\n-3\nDivided\n-3\nDivided\n0\nDivided\n-2147483648\nDivision by zero\nOverflow\n'),
             ('Parsing', 'library-parsing.rvn', 'Parsed\n42\nParsed\n-2147483648\nParsed\n2147483647\nParsed\n7\nOverflow\nOverflow\nInvalid format\nInvalid format\nInvalid format\nInvalid format\n'),
             ('Paths', 'library-paths.rvn', 'summary.txt\nreport.txt\nreport.txt\n\n\nfinal.txt\n世界.txt\nleaf.txt\n'),
             ('StringSlices', 'library-string-slices.rvn', 'Sliced\né\nSliced\n😀\nSliced\n\nInvalid boundary\nInvalid boundary\nOut of range\nOut of range\nOut of range\nOut of range\nOut of range\nSliced\n\n'),
             ('Strings', 'library-strings.rvn', 'Hello, värld!\n14\nyes\nno\nyes\nno\nyes\nyes\nyes\nno\n-1\n0\n1\n'),
             ('StringBoundaries', 'library-string-boundaries.rvn', 'yes\nyes\nyes\nyes\nno\n1\n4\n'),
             ('Patterns', 'library-patterns.rvn', '42\n-1\n7\n-1\n'),
             ('MatchForms', 'library-match.rvn', '42\n-1\nPresent\nAbsent\n42\nOverflow\n'),
             ('MatchVoid', 'library-match-void.rvn', '42\nSaved\nCompleted\nOverflow\n'),
             ('Math', 'library-math.rvn', '-2147483648\n2147483647\n7\n-7\n-1\n0\n1\n'),
             ('Result', 'library-result.rvn', '42\nOverflow\n'),
             ('Propagation', 'library-propagation.rvn', 'Continued\n42\nOverflow propagated\n'),
             ('OptionPropagation', 'library-option-propagation.rvn', 'Value found\n42\nAbsent\n'),
             ('VoidResultPropagation', 'library-result-void-propagation.rvn', '42\nSaved\nCompleted\nOverflow\n'),
             ('Option', 'library-option.rvn', '42\nProduct not found\n'),
             ('Void', 'library-void.rvn', 'Completed without a payload\nNot completed\n')]
    if args.collections:
        cases += [('Interfaces', 'library-interfaces.rvn', '1\n42\n'),
                  ('ListFilters', 'library-list-filters.rvn', '7\n7\n1\n3\nAbsent\nExists\nNot all positive\n3\n7\n42\n7\n7\nAbsent\nAbsent\nAbsent\n0\nAll empty elements satisfy the predicate\n5\n7\n99\n1\n2\n'),
                  ('QueryTerminals', 'library-query-terminals.rvn', 'Absent\nAbsent\n0\n42\nAbsent\nEmpty\n0\nMultiple\n42\nMultiple\n42\nAbsent\nSystem.String\nNo result\n42\nAbsent\nAbsent\n0\n0\nEmpty\nMultiple\n42\n'),
                  ('Maps', 'library-maps.rvn', 'Added\nDuplicate rejected\nPending\nUnknown order\nShipped\n1\n2\nFound\n42\n99\nStored\nStored\nShipped\n23\n2\n1\n'),
                  ('CollectionCapabilities', 'library-collection-capabilities.rvn', '2\n42\n2\n2\n7\n2\n9\n2\n3\n11\n'),
                  ('UnifiedArray', 'library-array-unified.rvn', '42\n2\n42\n8\n50\n42\n1\n42\n1\nSystem.Int32\n4\n9\n'),
                  ('ArrayShapes', 'library-array-shapes.rvn', '0\n0\nBoolean elements\nSystem.Int32\nSystem.String\n0\n255\n65535\n65535\n42\n'),
                  ('ReferencePayloads', 'library-reference-payloads.rvn', 'Copied TypeInfo reference\nSystem.Int32\nSystem.String\n3\n2\n0\n3\n3\n42\nStored error\n7\n42\nSystem.Int32\n'),
                  ('ValueInterfaces', 'library-value-interfaces.rvn', '0\n1\nEqual integer\nEqual string\nEqual type\n0\nEqual date\n0\n0\n'),
                  ('NativeBuffer', 'library-native-buffer.rvn', 'Native allocation released\n'),
                  ('Flags', 'library-flags.rvn', '28\n8\n20\n-29\n0\nSame flags\nPublic included\n5\nStoredDayNumber\n'),
                  ('ArrayQueries', 'library-array-queries.rvn', 'Parse\nDivide\nEquals\nToString\nCompareTo\n0\n17\n13\n3\n17\n52\n6\n0\n0\n43\n'),
                  ('ArrayForEach', 'library-array-foreach.rvn', 'Parse\nDivide\nEquals\nToString\nCompareTo\n42\n1\n2\n3\n'),
                  ('ManagedArrayMetadata', 'library-managed-array-metadata.rvn', '42\n2\n1\nSystem.Int32\n0\nEmpty\nLength\nCount\nItem\n4\n42\n8\n50\n2\n'),
                  ('IntrospectionInterfaces', 'library-introspection-interfaces.rvn', 'Interface\n' * 6 + 'System.Date\nField\nMethod\nProperty\nType\n'),
                  ('Reflection', 'library-reflection.rvn', (bridge / 'samples/library-reflection.expected.txt').read_text()),
                  ('ArrayCallbacks', 'library-array-callbacks.rvn', '7\n42\nFirst\nSecond\n'),
                  ('Delegates', 'library-delegates.rvn', '42\n' * 5 + 'Done\n1\nExists\n42\nNo index\nNone\n'),
                  ('Booleans', 'library-booleans.rvn', '1\n-1\n0\n1\n42\nNot false\nEqual\nDifferent\n0\n1\n0\n'),
                  ('GenericCollections', 'library-generic-collections.rvn', 'Changed\nSecond\n0\n0\n1\n2\n1\n'),
                  ('PropagationWorkflow', 'library-propagation-workflow.rvn', '42\nSaved\nCompleted\nOverflow\nValue found\n42\nAbsent\n'),
                  ('ValueCopy', 'library-value-copy.rvn', '42\n7\n'),
                  ('Arrays', 'library-arrays.rvn', '42\n2\n'),
                  ('Workflow', 'library-workflow.rvn', '42\nCompleted\nPrice overflow\nSkipped\nProduct not found\nSkipped\n'),
                  ('ForEach', 'library-foreach.rvn', '41\n42\n41\n41\n'),
                 ('Aliases', 'library-collection-aliases.rvn', '7\n42\n2\n')]
    if args.start_at:
        labels = [label for label, _, _ in cases]
        if args.start_at not in labels:
            parser.error('Unknown sample label: ' + args.start_at)
        cases = cases[labels.index(args.start_at):]
    if args.only:
        unknown = set(args.only) - {label for label, _, _ in cases}
        if unknown:
            parser.error('Unknown sample labels: ' + ', '.join(sorted(unknown)))
        cases = [case for case in cases if case[0] in args.only]
    for label, sample, expected in cases:
        (root / 'Main.rvn').write_text((bridge / 'samples' / sample).read_text())
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if run.returncode or not run.stdout.endswith(expected):
            raise AssertionError(label + ': ' + run.stdout + run.stderr)
        results[label] = expected
    source = ((bridge / 'samples/library-foreach.rvn').read_text().replace('values.Add(41)', 'values.Add(7)') if args.collections
              else (bridge / 'samples/library-result.rvn').read_text().replace('Show(-42)', 'Show(-7)'))
    saved_expected = '7\n42\n7\n7\n' if args.collections else '7\nOverflow\n'
    (root / 'Main.rvn').write_text(source)
    run = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if run.returncode or not run.stdout.endswith(saved_expected):
        raise AssertionError(run.stdout + run.stderr)
    results['SavedEdit'] = saved_expected
    for label, source, diagnostic in [
        *[(f'Missing{case}Match', (bridge / 'samples/library-introspection-interfaces.rvn').read_text()
           .replace(f'        {case} => "{label}"\n', ''), 'RAV2100')
          for case, label in [('FieldInfo', 'Field'), ('MethodInfo', 'Method'), ('PropertyInfo', 'Property'), ('TypeInfo', 'Type')]],
        ('CompileFailure', 'func Main() { MissingCall() }', 'RAV'),
        ('IntrospectionArrayAssignment', 'import System.Introspection.*\nfunc Main() { let methods: MethodInfo[] = typeof(int).GetMethods() }', 'RAV'),
        ('IntrospectionMutation', 'func Main() { let methods = typeof(int).GetMethods(); methods[0] = methods[0] }', 'RAV'),
        ('ExternalIntrospectionProvider', 'import System.Introspection.*\nclass UserInfo : TypeInfo { }\nfunc Main() {}', 'RAV033'),
        ('HiddenIntrospectionProvider', 'func Main() { let info = typeof(System.Introspection.RuntimeTypeInfo) }', 'RAV'),
        ('RemovedTypeClass', 'func Main() { let value: System.Type = typeof(int) }', 'RAV'),
        ('RemovedInfoProperty', 'func Main() { let value = typeof(int).Info }', 'RAV'),
        ('RemovedTypeOfHelper', 'func Main() { System.TypeOf<int>.Of(42) }', 'RAV'),
        ('InheritedIntegerMember', 'func Main() { let value = 42\n value.GetHashCode() }', 'Unsupported'),
        ('PathArgumentMismatch', 'func Main() { System.IO.Path.Combine(42, 7) }', 'RAV'),
        ('PathUnsupportedApi', 'func Main() { System.IO.Path.GetFullPath(".") }', 'RAV'),
        ('StringArgumentMismatch', 'func Main() { System.String.Concat(42, 7) }', 'RAV'),
        ('StringUnsupportedApi', 'func Main() { System.String.IsNullOrEmpty(\"\") }', 'RAV'),
        ('ArrayImplicitCovariance', 'import System.*\nimport System.Introspection.*\nfunc Main() { let methods: MethodInfo[] = []; let members: MemberInfo[] = methods }', ('RAV1504', 'identical element types')),
        ('ArrayExplicitCovariance', 'import System.*\nimport System.Introspection.*\nfunc Main() { let methods: MethodInfo[] = []; let members = (MemberInfo[])methods }', ('RAV1503', 'identical element types')),
        ('ImportFailure', 'func Negate(value: int) -> int { return -value }\nfunc Main() { Negate(2) }', 'Unsupported')]:
        before = set(root.rglob('App.neoil'))
        (root / 'Main.rvn').write_text(source)
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        # Imported MSBuild props can configure compiler invariance without an inline
        # property. Both compiler rejection and the legacy importer guard are valid.
        diagnostics = (diagnostic,) if isinstance(diagnostic, str) else diagnostic
        if run.returncode == 0 or not any(d in run.stderr for d in diagnostics) or set(root.rglob('App.neoil')) != before:
            raise AssertionError(run.stdout + run.stderr)
        results[label] = 'Rejected; no executable produced or stale output run'
    for label, type_name, left, right, diagnostic in (
            ('DivisionByZero', 'uint', '(uint)42', '(uint)0', 'division by zero'),
            ('DivisionOverflow', 'long', '-9223372036854775808L', '-1L', 'overflow')):
        (root / 'Main.rvn').write_text(f'''
import System.Console.*
func Divide(left: {type_name}, right: {type_name}) -> {type_name} {{
    return left / right
}}
func Main() {{
    Divide({left}, {right})
    WriteLine("Must not continue")
}}
''')
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        assert run.returncode != 0 and diagnostic in run.stderr, run.stdout + run.stderr
        assert 'Verified saved project:' in run.stdout and 'Must not continue' not in run.stdout
        results[label] = 'Verified, then faulted during execution'
print(json.dumps(results, indent=2))
