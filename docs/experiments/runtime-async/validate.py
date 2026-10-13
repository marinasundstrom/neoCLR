#!/usr/bin/env python3
"""Compile and execute the runtime-async POC with an explicit matched bundle/runtime."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--compiler', type=Path, help='Override bundled rvnc.dll for compiler integration validation')
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
mode = parser.add_mutually_exclusive_group()
mode.add_argument('--language-await', action='store_true', help='Validate ordinary await via the compatibility adapter')
mode.add_argument('--runtime-await', action='store_true', help='Validate compiler-native runtime async lowering')
mode.add_argument('--managed-task', action='store_true', help='Validate retained bodies producing ordinary managed Tasks')
parser.add_argument('--runtime-sample', choices=['language', 'gc', 'fault', 'operation', 'contract', 'http', 'use'], default='language')
args = parser.parse_args()
if args.runtime_sample != 'language' and not args.runtime_await:
    parser.error('--runtime-sample requires --runtime-await')
bundle, runtime, output = args.bundle.resolve(), args.runtime.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
source = Path(__file__).with_name({'language': 'RuntimeAwait.rvn', 'gc': 'RuntimeAwaitGc.rvn', 'fault': 'RuntimeAwaitFault.rvn', 'operation': 'RuntimeOperation.rvn', 'contract': 'RuntimeContract.rvn', 'http': 'RuntimeHttp.rvn', 'use': 'RuntimeUse.rvn'}[args.runtime_sample] if args.runtime_await else 'ManagedTask.rvn' if args.managed_task else
                                'AwaitLanguage.rvn' if args.language_await else 'Await.rvn').resolve()
expected = ({'language': 'PASS Raven runtime async language support', 'gc': 'PASS Raven runtime async GC reclamation', 'fault': 'runtime async sample terminal fault', 'operation': 'PASS independent runtime operation await', 'contract': 'PASS imported runtime operation contract', 'http': 'PASS runtime async HTTP client and server', 'use': 'PASS runtime async scoped cleanup'}[args.runtime_sample] if args.runtime_await else
            'PASS managed Tasks through retained runtime bodies' if args.managed_task else
            'PASS ordinary Raven await through runtime suspension adapter' if args.language_await
            else 'PASS runtime-owned operation suspension and resumption')
sources = [source]
if args.runtime_await:
    sources.append(Path(__file__).resolve().parents[3] / 'runtime/raven/compiler-services/RuntimeAwait.rvn')
compiler = args.compiler.resolve() if args.compiler else bundle / 'sdk/tools/rvnc/rvnc.dll'
compiler_inputs = [compiler, *(compiler.parent / name for name in [
    'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'])]
lib = bundle / 'lib'
references = [lib / (name + '.dll') for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']]
assembly = output / 'app.dll'
# This directory is dedicated to generated POC artifacts.
assembly.unlink(missing_ok=True)
compile_command = ['dotnet', str(compiler), 'neoclr', '--core-reference', str(lib / 'Core.dll'),
                   '--runtime-seed', str(lib / 'System.runtime.neox'), '--bootstrap-intrinsics',
                   '--bootstrap-ownership', str(lib / 'ownership.json'), '--object-library', 'System.Runtime',
                   '--async-library', 'System.Runtime']
contract_command = None
contract_assembly = output / 'Operation.Contracts.dll'
if args.runtime_await and args.runtime_sample == 'contract':
    contract_assembly.unlink(missing_ok=True)
    contract_source = source.with_name('OperationContract.rvn')
    contract_command = compile_command.copy()
    for reference in references:
        contract_command += ['--reference', str(reference)]
    contract_command += ['--library', '-o', str(contract_assembly), str(contract_source)]
    references.append(contract_assembly)
if args.runtime_await:
    compile_command += ["--runtime-async-poc", {"operation": "RuntimeOperationProvider.Bridge", "contract": "RuntimeContractProvider.Bridge"}.get(args.runtime_sample, "RuntimeAwaitProvider.Bridge")]
for reference in references:
    compile_command += ['--reference', str(reference)]
compile_command += ['-o', str(assembly), *map(str, sources)]
run_command = [str(runtime), 'run', str(assembly), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
               '--instructions', '100000000', '--object-root', str(references[0])]
for reference in references:
    run_command += ['--module', str(reference)]
report = {'passed': False, 'commands': [], 'sha256': {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
          for path in [*sources, runtime, *compiler_inputs, lib / 'ownership.json', lib / 'Core.dll', lib / 'System.runtime.neox', *references] if path.exists()}}
commands = ([contract_command] if contract_command else []) + [compile_command, run_command]
if contract_command:
    report['sha256'][str(contract_source)] = hashlib.sha256(contract_source.read_bytes()).hexdigest()
try:
    for command in commands:
        result = subprocess.run(command, capture_output=True, text=True, timeout=120)
        report['commands'].append({'command': command, 'exitCode': result.returncode,
                                   'stdout': result.stdout, 'stderr': result.stderr})
        if result.returncode:
            break
    report['passed'] = (len(report['commands']) == len(commands) and result.returncode == 0
                        and result.stdout.strip() == expected
                        and not result.stderr)
    if args.runtime_await and args.runtime_sample == 'fault':
        report['passed'] = (len(report['commands']) == len(commands) and result.returncode != 0
                            and 'UserFault: ' + expected in result.stderr
                            and 'UNREACHABLE' not in result.stdout)
    if contract_command and contract_assembly.exists():
        report['sha256'][str(contract_assembly)] = hashlib.sha256(contract_assembly.read_bytes()).hexdigest()
    if report['passed'] and args.runtime_await:
        listing = output / 'app.neoil'
        listing.unlink(missing_ok=True)
        inspection = subprocess.run([str(runtime), 'disassemble', str(assembly), str(listing)],
                                    capture_output=True, text=True, timeout=120)
        report['metadataInspection'] = {'exitCode': inspection.returncode, 'stderr': inspection.stderr}
        if inspection.returncode == 0:
            report['metadataInspection']['stateMachineMentions'] = listing.read_text().count('AsyncStateMachine')
        report['passed'] = (inspection.returncode == 0 and
                            report['metadataInspection']['stateMachineMentions'] == 0)
    if report['passed'] and args.runtime_await and args.runtime_sample == 'use':
        command = compile_command.copy()
        index = command.index('--runtime-async-poc')
        del command[index:index + 2]
        rejected_output = output / 'state-machine-use.dll'
        rejected_output.unlink(missing_ok=True)
        command[command.index('-o') + 1] = str(rejected_output)
        rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
        passed = rejected.returncode != 0 and 'RAVT006' in rejected.stderr and not rejected_output.exists()
        report['rejectedContracts'] = [{'case': 'state-machine-use', 'passed': passed,
            'command': command, 'exitCode': rejected.returncode, 'stderr': rejected.stderr}]
        report['passed'] = report['passed'] and passed
    if report['passed'] and contract_command:
        invalid_source = output / 'invalid-contract.rvn'
        invalid_source.write_text(source.read_text().replace(
            'func WaitCompletion() -> bool {', 'func UnrelatedWaitCompletion() -> bool {', 1))
        invalid_assembly = output / 'invalid-contract.dll'
        invalid_assembly.unlink(missing_ok=True)
        command = compile_command.copy()
        command[command.index('-o') + 1] = str(invalid_assembly)
        command[command.index(str(source))] = str(invalid_source)
        rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
        passed = rejected.returncode != 0 and 'WaitCompletion' in rejected.stderr and not invalid_assembly.exists()
        report['rejectedContracts'] = [{'case': 'missing-interface-member', 'passed': passed,
            'command': command, 'exitCode': rejected.returncode, 'stderr': rejected.stderr}]
        report['passed'] = report['passed'] and passed
        command = run_command.copy()
        index = command.index(str(contract_assembly))
        del command[index - 1:index + 1]
        rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
        passed = rejected.returncode != 0 and not rejected.stdout and 'missing referenced module' in rejected.stderr
        report['rejectedContracts'].append({'case': 'missing-contract-module', 'passed': passed,
            'command': command, 'exitCode': rejected.returncode, 'stderr': rejected.stderr})
        report['passed'] = report['passed'] and passed
    if report['passed'] and args.runtime_await and args.runtime_sample == 'operation':
        report['rejectedContracts'] = []
        for case in ['missing-mapping', 'ambiguous-mapping']:
            command = compile_command.copy()
            rejected_output = output / (case + '.dll')
            rejected_output.unlink(missing_ok=True)
            command[command.index('-o') + 1] = str(rejected_output)
            if case == 'missing-mapping':
                command[command.index('--runtime-async-poc') + 1] = 'RuntimeAwaitProvider.Bridge'
            else:
                duplicate = output / 'ambiguous.rvn'
                duplicate.write_text(source.read_text().replace(
                    'static func Wait(operation: RuntimeOperationSample.ManualOperation) -> bool {',
                    'static func Wait<T>(operation: RuntimeOperationSample.ManualOperation) -> bool => true\n'
                    '        static func Wait(operation: RuntimeOperationSample.ManualOperation) -> bool {'))
                command[command.index(str(source))] = str(duplicate)
            rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
            passed = (rejected.returncode != 0 and 'one exact Boolean Wait mapping' in rejected.stderr
                      and not rejected_output.exists())
            report['rejectedContracts'].append({'case': case, 'passed': passed, 'command': command,
                                               'exitCode': rejected.returncode, 'stderr': rejected.stderr})
            report['passed'] = report['passed'] and passed
    if report['passed'] and args.runtime_await and args.runtime_sample == 'language':
        report['rejectedContracts'] = []
        for case in ['missing-helper', 'missing-async-library']:
            rejected_output = output / (case + '.dll')
            rejected_output.unlink(missing_ok=True)
            command = compile_command.copy()
            command[command.index('-o') + 1] = str(rejected_output)
            if case == 'missing-helper':
                command[command.index('--runtime-async-poc') + 1] = 'Missing.RuntimeHelper'
                diagnostic = 'helper type is unavailable'
            else:
                index = command.index('--async-library')
                del command[index:index + 2]
                diagnostic = '--runtime-async-poc requires --async-library'
            rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
            passed = rejected.returncode != 0 and diagnostic in rejected.stderr and not rejected_output.exists()
            report['rejectedContracts'].append({'case': case, 'passed': passed, 'command': command,
                                                'exitCode': rejected.returncode, 'stderr': rejected.stderr})
            report['passed'] = report['passed'] and passed
finally:
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
raise SystemExit(0 if report['passed'] else 1)
