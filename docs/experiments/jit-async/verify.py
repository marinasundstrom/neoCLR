#!/usr/bin/env python3
"""Fresh Raven async entries through interpreter, AOT and the owned JIT task scope."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--console-qualification', type=Path, required=True)
    parser.add_argument('--case', action='append', help='Run only named cases in the selected profile')
    profile = parser.add_mutually_exclusive_group()
    profile.add_argument('--timers', action='store_true')
    profile.add_argument('--scheduler', action='store_true')
    profile.add_argument('--continuations', action='store_true')
    profile.add_argument('--managed-continuations', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out, bundle = args.output.resolve(), args.bundle.resolve()
    out.mkdir(parents=True, exist_ok=False)
    prior = json.loads(args.console_qualification.read_text())
    if not prior['passed']:
        raise ValueError('Requires successful console qualification')
    jit = ROOT / 'tools/aot-poc/target/release/neoclr-aot-poc'
    runtime = ROOT / 'target/release/neoclr'
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    lib = bundle / 'lib'
    refs = [lib / (name + '.dll') for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']]
    seed = lib / 'System.runtime.neox'
    compile_base = ['dotnet', compiler, 'neoclr', '--core-reference', lib / 'Core.dll',
                    '--runtime-seed', seed, '--bootstrap-intrinsics', '--bootstrap-ownership', lib / 'ownership.json',
                    '--object-library', 'System.Runtime', '--async-library', 'System.Runtime']
    for ref in refs:
        compile_base += ['--reference', ref]
    native_base = prior['jitCommand'][1:]
    native_base.remove('--jit-managed')
    old_input, old_host = native_base[1], native_base[3]
    host = out / 'host.dylib'
    host_command = [str(host) if p == old_host else p for p in prior['hostCommand']]
    task_source = str(ROOT / 'docs/experiments/aot-console/task-queue.c')
    if task_source not in host_command:
        host_command += [task_source]
    socket_source = str(ROOT / 'docs/experiments/aot-console/socket-listener.c')
    if socket_source not in host_command:
        host_command += [socket_source]
    report = dict(passed=False, sourceBaseRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[], cases=[],
                  inputs={str(p):sha(p) for p in [jit, runtime, compiler, seed, *refs,
                          lib / 'Core.dll', lib / 'ownership.json', HERE / 'verify.py', ROOT / 'docs/experiments/jit-managed/host.c']})

    def run(cmd):
        cmd = list(map(str, cmd))
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        record = dict(command=cmd, exitCode=result.returncode, stderr=result.stderr)
        if len(result.stdout) > 4000:
            record.update(stdoutPrefix=result.stdout[:2000], stdoutBytes=len(result.stdout.encode()),
                          stdoutSha256=hashlib.sha256(result.stdout.encode()).hexdigest())
        else:
            record['stdout'] = result.stdout
        report['commands'].append(record)
        return result

    bootstrap = ROOT / 'docs/experiments/extended-cli-metadata/bootstrap'
    cases = [
        ('int', bootstrap/'native-async-entry-int.rvn', ['argument'], 23, '', None),
        ('queued', HERE/'Queued.rvn', [], 0, 'first callback\nsecond callback\n42\n', None),
        ('cancelled', bootstrap/'native-async-entry-cancelled.rvn', [], 1, '', 'Task is cancelled; consume its Outcome'),
        ('pending', bootstrap/'native-async-entry-pending.rvn', [], 1, '', 'Task is still pending'),
        ('callback-fault', HERE/'CallbackFault.rvn', [], 1, '', 'JIT callback fault'),
    ]
    if args.timers:
        names = ['one shot', 'stop', 'disposal', 'repeat and stop inside callback',
                 'reschedule inside callback', 'async repeat waits for completion',
                 'async restart does not overlap', 'disposal allows async callback to finish',
                 'cancelled callback stops repeating', 'pending callback cancellation stops repeating']
        cases = [('timers', ROOT/'docs/experiments/timers/Main.rvn', [], 0,
                  ''.join('PASS '+name+'\n' for name in names), None),
                 ('timer-fault', ROOT/'docs/experiments/jit-timers/CallbackFault.rvn', [], 1, '', 'JIT timer callback fault')]
    if args.scheduler:
        cases = [('scheduler', ROOT/'docs/experiments/scheduler-projection/Main.rvn', [], 0,
                  'PASS deferred submission and state-machine resume\n', None),
                 ('scheduler-fault', ROOT/'docs/experiments/scheduler-projection/Fault.rvn', [], 1, '', 'submitted callback fault')]
    if args.continuations or args.managed_continuations:
        cases = [('continuations', ROOT/'docs/experiments/scheduler-continuations/Main.rvn', [], 0,
                  'PASS multiple pending awaits without producer pump\n'
                  'PASS completed await and deferred observer\n'
                  'PASS cancellation without producer pump\n'
                  'PASS registration lifecycle is separate from task state\n'
                  'PASS scoped observer disposal preserves task state and admitted work\n'
                  'PASS dormant registration does not keep invocation alive\n', None),
                 ('continuation-fault', ROOT/'docs/experiments/scheduler-continuations/Fault.rvn', [], 1,
                  '', 'fault after continuation submission')]
    if args.managed_continuations:
        cases += [('continuation-batch', ROOT/'docs/experiments/scheduler-continuations/Batch.rvn', [], 0,
                   'PASS 128 awaiters and observer admitted with one available slot\n'
                   'PASS cancellation batch publishes outcome before observation\n', None),
                  ('batch-fault', ROOT/'docs/experiments/scheduler-continuations/BatchFault.rvn', [], 1, '', 'batch callback fault')]
        cases += [('managed-ownership', ROOT/'docs/experiments/scheduler-continuations/ManagedOwnership.rvn', [], 0,
                   'PASS abandoned await cycles reclaimed\n'
                   'PASS live operation retains unobserved waiter across collection\n'
                   'PASS runtime submission retains producer and waiting execution\n'
                   'PASS pending timer retains producer and waiting execution\n', None)]
    if args.case:
        known = {case[0] for case in cases}
        if set(args.case) - known:
            raise ValueError('Unknown case for selected profile')
        cases = [case for case in cases if case[0] in args.case]
    jit_command = None
    try:
        result = run(host_command)
        if result.returncode:
            raise RuntimeError('Host compilation failed: '+result.stderr)
        report['hostSha256'] = sha(host)
        for name, source, arguments, code, stdout, fault in cases:
            case_out = out/name
            case_out.mkdir()
            assembly, obj, exe = (case_out/n for n in ['app.dll', 'app.o', 'app'])
            extra_sources = []
            if args.continuations or args.managed_continuations:
                directory = ROOT/'docs/experiments/scheduler-continuations'
                extra_sources = [directory/'Completion.rvn', directory/('ManagedContinuation.rvn' if args.managed_continuations else 'RuntimeContinuation.rvn')]
            for extra in extra_sources:
                report['inputs'][str(extra)] = sha(extra)
            result = run([*compile_base, '-o', assembly, source, *extra_sources])
            if result.returncode:
                raise RuntimeError(name+' Raven compilation failed: '+result.stdout+result.stderr)
            interpret = [runtime, 'run', assembly, '--system', seed, '--instructions', '100000000', '--object-root', refs[0]]
            for ref in refs:
                interpret += ['--module', ref]
            if arguments:
                interpret += ['--', *arguments]
            if name == 'managed-ownership':
                # Native GC diagnostic services are not bound by this backend.
                # Keep the explicit-collection gate interpreter-only and report it.
                result = run(interpret)
                if result.returncode != code or result.stdout != stdout or result.stderr:
                    raise RuntimeError(name+' interpreter outcome mismatch: '+result.stdout+result.stderr)
                report['cases'].append(dict(name=name, sourceSha256=sha(source),
                    assemblySha256=sha(assembly), limitation='Native GC diagnostic bindings unavailable; explicit collection qualified only in interpreter',
                    outcomes=[dict(mode='interpreter', exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)]))
                print('PASS '+name+' in interpreter (native GC diagnostic bindings unavailable)', flush=True)
                continue
            native = [str(assembly) if p == old_input else str(obj) if p == old_host else p for p in native_base]
            if '--bind-task-queue' not in native:
                native += ['--bind-task-queue']
            if args.scheduler or args.continuations or args.managed_continuations:
                native += ['--bind-scheduler']
            if args.timers or args.managed_continuations:
                native += ['--bind-timers']
            result = run([jit, *native])
            if result.returncode:
                raise RuntimeError(name+' AOT failed: '+result.stderr)
            with_args = name == 'int'
            entry = 'neoclr_entry_args_v1' if with_args else 'neoclr_entry_v4'
            wrapper = case_out/'main.c'
            wrapper.write_text('#include "native-gc.h"\nextern int32_t '+entry+'(int32_t, int32_t *, neoclr_aot_context *'+(', const char **' if with_args else '')+');\nextern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);\nint main(int argc, char **argv) { return neoclr_jit_run_v1((const void *)'+entry+', '+str(int(with_args))+', argc, (const char **)argv); }\n')
            link = [p for p in host_command if p != '-dynamiclib']
            link[link.index(str(host))] = str(exe)
            link += [str(obj), str(wrapper)]
            result = run(link)
            if result.returncode:
                raise RuntimeError(name+' AOT host failed: '+result.stderr)
            jit_command = [jit, '--jit-managed', *[str(host) if p == str(obj) else p for p in native]]
            for argument in arguments:
                jit_command += ['--jit-entry-argument', argument]
            outcomes = []
            for mode, cmd in [('interpreter', interpret), ('aot', [exe, *arguments]), ('jit', jit_command)]:
                result = run(cmd)
                stderr = result.stderr
                exit_code = result.returncode
                if mode == 'jit':
                    summaries = [line for line in stderr.splitlines() if line.startswith('JIT_RESULT ')]
                    if result.returncode or len(summaries) != 1:
                        raise RuntimeError(name+' JIT failed: '+stderr)
                    data = json.loads(summaries[0].removeprefix('JIT_RESULT '))
                    exit_code = data['guestExitCode']
                    stderr = ''.join(line for line in stderr.splitlines(keepends=True) if not line.startswith('JIT_RESULT '))
                if exit_code != code or result.stdout != stdout or (fault not in stderr if fault else bool(stderr)):
                    raise RuntimeError(name+' '+mode+' outcome mismatch: '+result.stdout+stderr)
                outcomes.append(dict(mode=mode, exitCode=exit_code, stdout=result.stdout, stderr=stderr))
            report['cases'].append(dict(name=name, sourceSha256=sha(source), assemblySha256=sha(assembly), objectSha256=sha(obj), executableSha256=sha(exe), outcomes=outcomes))
            print('PASS '+name+' in interpreter/AOT/JIT', flush=True)
        report['rejectedBindings'] = []
        for flag in (['--native-host-bootstrap'] if jit_command is not None else []):
            result = run([*jit_command, flag])
            if result.returncode != 1 or result.stdout or 'JIT_RESULT ' in result.stderr or 'without bootstrap bindings' not in result.stderr:
                raise RuntimeError('Unsupported binding was not rejected: '+flag)
            report['rejectedBindings'].append(flag)
        report['passed'] = True
    finally:
        (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':
    main()
