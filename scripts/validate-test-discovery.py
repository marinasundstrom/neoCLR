#!/usr/bin/env python3
"""Qualify metadata-only TestAttribute signature rejection and stable registration."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('discovery', ROOT / 'scripts/discover-runtime-tests.py')
DISCOVERY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DISCOVERY)


def validate(bundle, output):
    output.mkdir(parents=True, exist_ok=False)
    report = {'passed': False, 'cases': []}
    cases = [
        ('parameters', '[Test]\nfunc Invalid(value: int) -> Result<unit, TestFailure> => Ok(())', 'parameterless, nongeneric module functions'),
        ('result', '[Test]\nfunc Invalid() -> Result<int, TestFailure> => Ok(1)', 'Result<unit, TestFailure>'),
        ('instance', 'class Fixture {\n[Test]\nfunc Invalid() -> Result<unit, TestFailure> => Ok(())\n}', 'parameterless, nongeneric module functions'),
        ('generic', '[Test]\nfunc Invalid<T>() -> Result<unit, TestFailure> => Ok(())', 'parameterless, nongeneric module functions'),
        ('empty', 'func NotATest() -> Result<unit, TestFailure> => Ok(())', 'No tests discovered'),
    ]
    try:
        for name, body, expected in cases:
            case = output / name
            case.mkdir()
            source = case / 'Main.rvn'
            source.write_text('module DiscoveryValidation\nimport System.*\nimport System.Result.*\nimport NeoClr.Testing.*\n' + body + '\nfunc Main() -> int => 0\n')
            project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
            ET.SubElement(project, 'Import', Project='$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props')
            properties = ET.SubElement(project, 'PropertyGroup')
            for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='DiscoveryValidation', EnableDefaultCompileItems='false').items():
                ET.SubElement(properties, key).text = value
            items = ET.SubElement(project, 'ItemGroup')
            for file in ['Assertions.rvn', 'TestSuite.rvn', 'TestAttribute.rvn']:
                ET.SubElement(items, 'Compile', Include=str(ROOT / 'runtime/raven/tests/framework' / file))
            ET.SubElement(items, 'Compile', Include=str(source.resolve()))
            ET.SubElement(items, 'Compile', Include='$(NeoClrTestRegistry)')
            project_path = case / 'Tests.rvnproj'
            ET.ElementTree(project).write(project_path, encoding='unicode')
            try:
                DISCOVERY.discover(project_path, bundle, case / 'discovery')
            except RuntimeError as error:
                if expected not in str(error):
                    raise
                if (case / 'discovery/TestRegistry.rvn').exists() or (case / 'discovery/tests.json').exists():
                    raise RuntimeError('Rejected discovery left a usable registry/manifest')
                report['cases'].append({'name': name, 'rejected': str(error), 'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest()})
            else:
                raise RuntimeError('Invalid test accepted: ' + name)
        report['passed'] = True
    finally:
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Test discovery signature checks: PASS')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    validate(args.bundle.resolve(), args.output.resolve())
