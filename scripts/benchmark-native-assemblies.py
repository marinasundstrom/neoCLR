"""Release benchmark: in-memory phases and interleaved fresh-process JSON/native runs.

Uses already verified schema-3 consumer artifacts. Filesystem caches are warmed;
this is not a cold-disk or throughput benchmark. Raw samples and hashes are retained.
"""
import argparse
import hashlib
import json
import platform
from pathlib import Path
import statistics
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--phase-benchmark', type=Path, required=True)
    parser.add_argument('--json', type=Path, required=True, help='Prior Raven schema-3 baseline output directory')
    parser.add_argument('--assemblies', type=Path, required=True, help='Direct assembler output directory')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    runtime, phases, json_dir, binary_dir, output = [p.resolve() for p in
        (args.runtime, args.phase_benchmark, args.json, args.assemblies, args.output)]
    if output.exists():
        parser.error('Output already exists; preserve prior benchmark evidence')
    expected = '0\n' * 19 + '-1\n'
    report = {'runtime_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              'platform': platform.platform(), 'processor': platform.processor(), 'samples': 9,
              'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
              'cpu': subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip() if platform.system() == 'Darwin' else platform.processor(),
              'runtime_sha256': hashlib.sha256(runtime.read_bytes()).hexdigest(),
              'phase_benchmark_sha256': hashlib.sha256(phases.read_bytes()).hexdigest()}
    try:
        measured = subprocess.run([str(phases), str(json_dir), str(binary_dir)],
                                  stdout=subprocess.PIPE, text=True, timeout=240, check=True)
        report['phases'] = json.loads(measured.stdout)
        samples = {'json': [], 'assembly': []}

        def run(kind):
            directory, extension = (json_dir, 'json') if kind == 'json' else (binary_dir, 'neox')
            start = time.perf_counter()
            process = subprocess.run([str(runtime), 'run', str(directory / ('FloatingMath.' + extension)),
                                      '--system', str(directory / ('System.' + extension))],
                                     capture_output=True, text=True, timeout=90)
            elapsed = (time.perf_counter() - start) * 1000
            if process.returncode or process.stdout != expected or process.stderr:
                raise RuntimeError(f'{kind}: {process.returncode}: {process.stdout}{process.stderr}')
            return elapsed

        for kind in samples:
            run(kind)  # Warm file cache and launch path; every measured run remains a new process.
        for sample in range(9):
            order = ['json', 'assembly'] if sample % 2 == 0 else ['assembly', 'json']
            for kind in order:
                samples[kind].append(run(kind))
            print(f'Completed fresh-process pair {sample + 1}/9', flush=True)
        report['fresh_process'] = {kind: {'samples_ms': values, 'median_ms': statistics.median(values),
                                         'min_ms': min(values), 'max_ms': max(values)}
                                   for kind, values in samples.items()}
        differences = [left - right for left, right in zip(samples['json'], samples['assembly'])]
        report['paired_json_minus_assembly_ms'] = {'samples': differences, 'median': statistics.median(differences)}
        report['fresh_process_includes'] = ['process launch', 'warm-cache file reads', 'decode',
                                           'System admission', 'module-set validation and linking',
                                           'preparation', 'execution', 'captured console output', 'shutdown']
        report['limitations'] = ['one machine/run', 'no cold-disk measurement', 'no memory/peak-RSS measurement',
                                 'microbenchmarks include object destruction', 'phase totals are not CLI totals',
                                 'direct-typed JSON is an unshipped diagnostic, not the current loader']
    finally:
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open('x') as stream:
            json.dump(report, stream, indent=2)
            stream.write('\n')


if __name__ == '__main__':
    main()
