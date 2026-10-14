import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from native_api_snapshot import extract_snapshot, validate_source_links, SOURCE_LINK_CHECKS


class NativeApiSnapshot(unittest.TestCase):
    def fixture(self, root, extra=None):
        archive = root / 'reference.zip'
        with zipfile.ZipFile(archive, 'w') as output:
            output.writestr('site/docs/api/System/Object/index.html', 'System.Runtime.dll')
            if extra:
                output.writestr(extra, 'bad')
        manifest = root / 'reference.json'
        manifest.write_text(json.dumps(dict(format=1, archive=archive.name,
            sha256=hashlib.sha256(archive.read_bytes()).hexdigest())))
        return manifest

    def test_type_and_member_source_links_are_required(self):
        repository = dict(url='https://github.com/example/runtime', revision='main')
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for page, source in SOURCE_LINK_CHECKS.items():
                path = root / page
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f'<a href="https://github.com/example/runtime/blob/main/{source}">Source</a>')
            validate_source_links(root, repository)
            member = root / 'System/Object/method_GetHashCode.html'
            for content in ['No source', '<a href="https://github.com/example/runtime">Repository</a>',
                            '<a href="https://github.com/example/runtime/blob/main/wrong.rvn">Source</a>']:
                member.write_text(content)
                with self.assertRaisesRegex(ValueError, 'method_GetHashCode'):
                    validate_source_links(root, repository)

    def test_missing_source_links_preserve_previous_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destination = root / 'output'
            destination.mkdir()
            (destination / 'previous').write_text('keep')
            manifest = self.fixture(root)
            data = json.loads(manifest.read_text())
            data['sourceRepository'] = dict(url='https://github.com/example/runtime', revision='main')
            manifest.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'source link'):
                extract_snapshot(manifest, destination)
            self.assertEqual((destination / 'previous').read_text(), 'keep')

    def test_checked_archive_replaces_previous_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destination = root / 'output'
            destination.mkdir()
            (destination / 'stale').write_text('old')
            extract_snapshot(self.fixture(root), destination)
            self.assertFalse((destination / 'stale').exists())
            self.assertEqual((destination / 'site/docs/api/System/Object/index.html').read_text(), 'System.Runtime.dll')

    def test_corrupt_or_escaping_archive_preserves_previous_snapshot(self):
        for extra in (None, 'site/docs/api/../../escape'):
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                destination = root / 'output'
                destination.mkdir()
                (destination / 'previous').write_text('keep')
                manifest = self.fixture(root, extra)
                if extra is None:
                    (root / 'reference.zip').write_bytes(b'corrupt')
                with self.assertRaises(ValueError):
                    extract_snapshot(manifest, destination)
                self.assertEqual((destination / 'previous').read_text(), 'keep')


if __name__ == '__main__':
    unittest.main()
