"""Check saved-source execution and rejection without stale-artifact fallback."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--collections', action='store_true')
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-project-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               '--raven', str(args.raven.resolve()), '--runtime', str(args.runtime.resolve())]
    cases = [('Parsing', 'library-parsing.rvn', 'Parsed\n42\nParsed\n-2147483648\nParsed\n2147483647\nParsed\n7\nOverflow\nOverflow\nInvalid format\nInvalid format\nInvalid format\nInvalid format\n'),
             ('Paths', 'library-paths.rvn', 'summary.txt\nreport.txt\nreport.txt\n\n\nfinal.txt\n世界.txt\nleaf.txt\n'),
             ('StringSlices', 'library-string-slices.rvn', 'Sliced\né\nSliced\n😀\nSliced\n\nInvalid boundary\nInvalid boundary\nOut of range\nOut of range\nOut of range\nOut of range\nOut of range\nSliced\n\n'),
             ('Strings', 'library-strings.rvn', 'Hello, värld!\n14\nyes\nno\nyes\nno\nyes\nyes\nyes\nno\n-1\n0\n1\n'),
             ('StringBoundaries', 'library-string-boundaries.rvn', 'yes\nyes\nyes\nyes\nno\n-1\n4\n'),
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
        cases += [('PropagationWorkflow', 'library-propagation-workflow.rvn', '42\nSaved\nCompleted\nOverflow\nValue found\n42\nAbsent\n'),
                  ('ValueCopy', 'library-value-copy.rvn', '42\n7\n'),
                  ('Arrays', 'library-arrays.rvn', '42\n2\n'),
                  ('Workflow', 'library-workflow.rvn', '42\nCompleted\nPrice overflow\nSkipped\nProduct not found\nSkipped\n'),
                  ('ForEach', 'library-foreach.rvn', '41\n42\n41\n41\n'),
                 ('Aliases', 'library-collection-aliases.rvn', '7\n42\n2\n')]
    for label, sample, expected in cases:
        (root / 'Main.rvn').write_text((bridge / 'samples' / sample).read_text())
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if run.returncode or not run.stdout.endswith(expected):
            raise AssertionError(run.stdout + run.stderr)
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
        ('CompileFailure', 'func Main() { MissingCall() }', 'RAV'),
        ('PathArgumentMismatch', 'func Main() { System.IO.Path.Combine(42, 7) }', 'RAV'),
        ('PathUnsupportedApi', 'func Main() { System.IO.Path.GetFullPath(".") }', 'RAV'),
        ('StringArgumentMismatch', 'func Main() { System.String.Concat(42, 7) }', 'RAV'),
        ('StringUnsupportedApi', 'func Main() { System.String.IsNullOrEmpty(\"\") }', 'RAV'),
        ('ImportFailure', 'func Add(value: int) -> int { return value + 1 }\nfunc Main() { Add(2) }', 'Unsupported')]:
        before = set(root.rglob('App.neoil'))
        (root / 'Main.rvn').write_text(source)
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if run.returncode == 0 or diagnostic not in run.stderr or set(root.rglob('App.neoil')) != before:
            raise AssertionError(run.stdout + run.stderr)
        results[label] = 'Rejected; no executable produced or stale output run'
print(json.dumps(results, indent=2))
