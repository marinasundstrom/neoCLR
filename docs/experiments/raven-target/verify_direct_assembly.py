"""Check direct neoil assembly against saved Raven sample metadata and execution baselines."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', required=True, type=Path)
    parser.add_argument('--metadata-tests', required=True, type=Path, help='Built C# metadata contract-test DLL')
    parser.add_argument('--system-source', required=True, type=Path, help='Matching expanded Raven collection System.neoil')
    parser.add_argument('--baseline', required=True, type=Path, help='Output directory from schema-3 Raven sample check')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    runtime, tests, source, baseline, output = [p.resolve() for p in
        (args.runtime, args.metadata_tests, args.system_source, args.baseline, args.output)]
    output.mkdir(parents=True, exist_ok=False)
    original = json.loads((baseline / 'translation-results.json').read_text())
    report = {'producer': 'neoil -> native Module -> schema-3 CBOR (no JSON intermediate)',
              'runtimeSha256': hashlib.sha256(runtime.read_bytes()).hexdigest(),
              'metadataReaderSha256': hashlib.sha256((tests.parent / 'NeoCLR.Metadata.Experimental.dll').read_bytes()).hexdigest(),
              'systemSourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(), 'cases': {}}

    def run(*command):
        process = subprocess.run([str(arg) for arg in command], capture_output=True, text=True, timeout=180)
        if process.returncode:
            raise RuntimeError(f'{process.returncode}: {process.stdout}{process.stderr}')
        return process.stdout

    def compare(image, expected):
        run('dotnet', tests, '--compare-native', image, expected)

    try:
        system = output / 'System.neox'
        run(runtime, 'assemble', source, system, '--format', 'neox')
        compare(system, baseline / 'System.json')
        report['system'] = {'bytes': system.stat().st_size,
                            'sha256': hashlib.sha256(system.read_bytes()).hexdigest(),
                            'allMetadataValuesMatch': True, 'verifiedByAssembler': True}
        for name, relative in [('OptionPositional', 'matches/OptionPositional.neoil'),
                               ('FloatingMath', 'FloatingMath/compiled/App.neoil'),
                               ('ValueCopy', 'ValueCopy/compiled/App.neoil')]:
            image = output / (name + '.neox')
            run(runtime, 'assemble', baseline / relative, image, '--format', 'neox', '--system', system)
            compare(image, baseline / (name + '.json'))
            stdout = run(runtime, 'run', image, '--system', system)
            if stdout != original['cases'][name]['stdout']:
                raise RuntimeError(name + ': output differs from the verified source baseline')
            report['cases'][name] = {'allMetadataValuesMatch': True, 'verifiedByAssembler': True,
                                     'stdout': stdout, 'bytes': image.stat().st_size,
                                     'sha256': hashlib.sha256(image.read_bytes()).hexdigest()}
            print(name + ': PASS direct native assembly and independent C# reader', flush=True)
    finally:
        (output / 'direct-assembly-results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
