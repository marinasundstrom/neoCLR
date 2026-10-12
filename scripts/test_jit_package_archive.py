import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('jit_package', Path(__file__).with_name('prepare-windows-jit-package.py'))
package = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class ArchiveTests(unittest.TestCase):
    def test_archive_contains_only_verified_catalog_and_manifest(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            bundle = root / 'bundle with spaces'
            bundle.mkdir()
            (bundle / 'payload').write_bytes(b'current')
            (bundle / 'stale').write_bytes(b'previous build')
            (bundle / 'manifest.json').write_text(json.dumps({'files': {'payload': hashlib.sha256(b'current').hexdigest()}}))
            archive = root / 'bundle.tar.gz'
            result = package.archive_bundle(bundle, archive)
            with tarfile.open(archive) as stream:
                self.assertEqual(stream.getnames(), ['neoclr-native-poc/manifest.json', 'neoclr-native-poc/payload'])
                self.assertEqual(stream.extractfile('neoclr-native-poc/payload').read(), b'current')
            self.assertEqual(result['archiveSha256'], package.sha(archive))
            (bundle / 'payload').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'catalog mismatch'):
                package.archive_bundle(bundle, root / 'bad.tar.gz')
            self.assertFalse((root / 'bad.tar.gz').exists())

    def test_catalog_cannot_archive_a_file_outside_bundle(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            bundle = root / 'bundle'
            bundle.mkdir()
            (root / 'outside').write_bytes(b'outside')
            (bundle / 'manifest.json').write_text(json.dumps({'files': {'../outside': package.sha(root / 'outside')}}))
            with self.assertRaisesRegex(ValueError, 'catalog mismatch'):
                package.archive_bundle(bundle, root / 'bad.tar.gz')


if __name__ == '__main__':
    unittest.main()
