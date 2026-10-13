import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
spec = importlib.util.spec_from_file_location('http_profile', HERE / 'profile.py')
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)


class AllocationSitesTests(unittest.TestCase):
    def test_window_recursion_unframed_and_reconciliation(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            source = out / 'test.c'
            source.write_text('''#include "native-gc.h"
#include <stdio.h>
static const neoclr_probe_frame *active;
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return active; }
void neoclr_allocation_sites_allocated(uint32_t, uint64_t);
void neoclr_allocation_sites_request(uint64_t);
void neoclr_allocation_sites_report(FILE *);
int main(void) {
    neoclr_probe_frame a = {.function=1}, b = {.function=2, .previous=&a}, c = {.function=1, .previous=&b};
    active = &c;
    neoclr_allocation_sites_allocated(NEOCLR_GC_TEXT, 100);
    neoclr_allocation_sites_request(8);
    neoclr_allocation_sites_allocated(NEOCLR_GC_OBJECT, 16);
    active = &b;
    neoclr_allocation_sites_allocated(NEOCLR_GC_TEXT, 3);
    active = NULL;
    neoclr_allocation_sites_allocated(NEOCLR_GC_BYTES, 5);
    neoclr_allocation_sites_request(32);
    neoclr_allocation_sites_allocated(NEOCLR_GC_TEXT, 100);
    neoclr_allocation_sites_report(stdout);
    return 0;
}
''')
            includes = ROOT / 'docs/experiments/aot-console'
            if platform.system() == 'Windows':
                exe = out / 'test.exe'
                command = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/I' + str(includes), str(source), str(HERE / 'allocation-sites.c'), '/Fe:' + str(exe)]
            else:
                exe = out / 'test'
                cc = ['clang']
                if platform.system() == 'Darwin':
                    env = dict(os.environ, DEVELOPER_DIR='/Applications/Xcode.app/Contents/Developer')
                    cc = [subprocess.check_output(['xcrun', '--sdk', 'macosx', '--find', 'clang'], env=env, text=True).strip(), '-isysroot', subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], env=env, text=True).strip()]
                command = cc + ['-std=c11', '-Wall', '-Wextra', '-Werror', '-I', str(includes), str(source), str(HERE / 'allocation-sites.c'), '-o', str(exe)]
            built = subprocess.run(command, cwd=out, capture_output=True, text=True)
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            sites = json.loads(subprocess.check_output([str(exe)], text=True))['sites']
        rows = {s['function']: s for s in sites}
        self.assertEqual(rows[1]['exclusive']['blocks'], 1)
        self.assertEqual(rows[1]['inclusive']['blocks'], 2)  # recursion counted once
        self.assertEqual(rows[2]['inclusive']['bytes'], 19)
        self.assertEqual(rows[2]['exclusive']['textBytes'], 3)
        self.assertEqual(rows[-1]['exclusive']['bytes'], 5)
        build = {'commands': [{'phase': 'native-compilation', 'stdout': json.dumps({'functions': [
            {'compiledIndex': 1, 'name': 'NeoMetadata_ABCD.T_53797374656D_54657374.M_52756E'},
            {'compiledIndex': 2, 'name': 'Helper'}], 'int32Boxes': [{'helperCompiledIndex': 99, 'typeCompiledIndex': 7}]})}]}
        totals = dict(allocations=3, allocatedBytes=24, textBlocks=1, textBytes=3)
        result = profile.describe_sites(sites, build, totals)
        self.assertEqual(result['sites'][0]['displayName'], 'NeoMetadata_ABCD.T_System_Test.M_Run')
        with self.assertRaisesRegex(ValueError, 'reconcile'):
            profile.describe_sites(sites, build, dict(totals, allocations=4))
        unmapped = [dict(s, function=99) if s['function']==1 else dict(s) for s in sites]
        result = profile.describe_sites(unmapped, build, totals)
        self.assertTrue(result['sites'][0]['mapped'])
        self.assertIn('generated Int32 boxing helper', result['sites'][0]['name'])
        result = profile.describe_sites([dict(s, function=100) if s['function']==1 else dict(s) for s in sites], build, totals)
        self.assertFalse(result['sites'][0]['mapped'])


if __name__ == '__main__':
    unittest.main()
