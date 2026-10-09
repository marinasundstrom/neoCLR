#!/usr/bin/env python3
"""Integrate a themed native audit into an existing local neoCLR website preview."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('audit', type=Path, help='Audit output generated with the repository-root argument')
    parser.add_argument('--site', type=Path, help='Integrate into a build staging directory without changing the selected preview')
    args = parser.parse_args()
    source = args.audit.resolve() / 'site/docs/api'
    if not (source / 'System/BooleanParseError/index.html').is_file():
        parser.error('Themed native audit output is missing')
    exp = (source / 'System/Math/method_Exp.html').read_text()
    assert 'Returns e raised to the supplied power.' in exp
    assert 'The value on which this operation acts.' in exp
    assert 'System.Runtime.dll' in exp and 'CLR container' not in exp
    assert (source / 'System/Object/method_GetHashCode.html').is_file()
    navigation = (source / 'api-navigation.html').read_text()
    assert 'method_Exp.html' in navigation
    assert 'structural-types.html' in navigation and 'arrays.html' in navigation
    assert 'Platform foundations' not in navigation, 'Guides leaked into the API tree'
    function_link = re.search(r'<a[^>]*href="[^"]*System/Math/method_Exp\.html"[^>]*>.*?</a>', navigation, re.S)
    assert function_link and 'symbol-icon--function' in function_link[0]
    module_page = (source / 'System/Math/index.html').read_text()
    assert '>Constants</h2>' in module_page and '>Functions</h2>' in module_page
    assert '>Members</h2>' not in module_page
    for constant in ('Pi', 'E', 'Tau'):
        page = (source / f'System/Math/field_{constant}.html').read_text()
        assert 'System.Runtime.dll' in page and 'CLR container' not in page
        assert f'field_{constant}.html' in navigation
        constant_link = re.search(r'<a[^>]*href="[^"]*System/Math/field_' + constant + r'\.html"[^>]*>.*?</a>', navigation, re.S)
        assert constant_link and 'symbol-icon--field' in constant_link[0]
    linq_module = (source / 'System/Linq/index.html').read_text()
    assert 'id="type-extensions"' in linq_module
    assert 'Operators for Iterable&lt;T&gt;' in linq_module
    extension_group = re.search(r'<section[^>]*aria-labelledby="type-extensions".*?</section>', linq_module, re.S)
    assert extension_group and 'Operators' in extension_group[0] and 'SingleError' not in extension_group[0]
    operators = (source / 'System/Linq/Operators/index.html').read_text()
    assert '<span>Type extension</span>' in operators
    assert 'extension Operators for Iterable&lt;T&gt;' in operators
    assert 'id="receiver"' in operators
    assert 'id="type-parameters"' in operators
    assert operators.index('id="receiver"') < operators.index('id="type-parameters"') < operators.index('id="remarks"')
    assert 'The element type.' in operators
    assert '>Iterable</a>&lt;<a href="#type-parameters">T</a>&gt;' in operators
    assert '<strong>Receiver type</strong>' not in operators
    assert 'Iterable%601/index.html' in operators
    assert 'static class Operators' not in operators
    extension_link = re.search(r'<a[^>]*href="[^"]*System/Linq/Operators/index\.html"[^>]*>.*?</a>', navigation, re.S)
    assert extension_link and 'symbol-icon--extension' in extension_link[0]
    overloads = (source / 'System/Math/method_Abs.html').read_text()
    assert 'Abs(value: int)' in overloads and 'Abs(value: double)' in overloads
    destination = args.site.resolve() if args.site else ROOT / 'target/website'
    if not (destination / 'docs/index.html').is_file():
        parser.error('Build the complete website first with scripts/build-website.py')
    with tempfile.TemporaryDirectory(prefix='native-website-', dir=ROOT / 'target') as temporary:
        staging = Path(temporary) / 'site'
        shutil.copytree(destination, staging)
        native = staging / 'docs/api'
        shutil.copytree(source, native, dirs_exist_ok=True)
        landing = staging / 'docs/index.html'
        landing.write_text(re.sub(r'<aside data-native-preview="true">.*?</aside>', '', landing.read_text(), flags=re.S))
        gaps = []
        # Existing API URLs are canonical. Retain unmatched legacy pages with an
        # explicit migration notice; never relabel an aggregate symbol's owner.
        for page in sorted(native.rglob('*.html')):
            relative = page.relative_to(native)
            if (source / relative).is_file():
                continue
            gaps.append(str(relative))
            content = page.read_text()
            content = re.sub(r'<aside data-native-preview="true">.*?</aside>', '', content, flags=re.S)
            notice = ('<aside data-native-preview="true"><p><strong>Migration gap:</strong> '
                      'This page still uses the previous reference snapshot. Native documentation coverage is pending.</p></aside>')
            content = re.sub(r'(<main\b[^>]*>)', lambda match: match[1] + notice, content, count=1)
            page.write_text(content)
        # Preserve URLs shared during the earlier preview with redirects.
        alias = staging / 'docs/native-api'
        if alias.exists():
            shutil.rmtree(alias)
        for page in source.rglob('*.html'):
            relative = page.relative_to(source)
            target = alias / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            href = os.path.relpath(native / relative, target.parent).replace(os.sep, '/')
            target.write_text('<!doctype html><html><head><meta charset="utf-8">'
                              f'<meta http-equiv="refresh" content="0;url={href}">'
                              '<title>neoCLR API reference moved</title></head><body>'
                              f'<a href="{href}">Open the neoCLR API reference</a></body></html>')
        config = Path(temporary) / 'site.json'
        config.write_text(json.dumps({'output': str(staging), 'search': True, 'copyCode': True}))
        subprocess.run(['python3', str(ROOT / 'scripts/ravendoc.py'), '--finalize-site', str(config)], check=True)
        spec = importlib.util.spec_from_file_location('website_checks', ROOT / 'scripts/build-website.py')
        checks = importlib.util.module_from_spec(spec)
        sys.path.insert(0, str(ROOT / 'scripts'))
        spec.loader.exec_module(checks)
        checks.OUTPUT = staging
        checks.make_reference_links_relative()
        pages = {}
        for page in staging.rglob('*.html'):
            check = checks.PageCheck()
            check.feed(page.read_text())
            pages[page.resolve()] = check
        for page, check in pages.items():
            check.check(page, pages)
        # Update only after the integrated copy passes the same local-link checks.
        shutil.copytree(staging, destination, dirs_exist_ok=True)
        (args.audit / 'website-validation.json').write_text(json.dumps({'checkedPages': len(pages), 'legacyCoverageGaps': gaps, 'result': 'PASS HTML, local links, Math XML, native ownership, Object member route and assembly-member navigation'}, indent=2) + '\n')
        if args.site is None:
            selection = ROOT / 'target/native-api-preview.json'
            pending = selection.with_suffix('.json.tmp')
            pending.write_text(json.dumps({'audit': str(args.audit.resolve())}) + '\n')
            pending.replace(selection)
        print(f'Integrated native preview; checked {len(pages)} neoCLR website pages; {len(gaps)} legacy pages retained')


if __name__ == '__main__':
    main()
