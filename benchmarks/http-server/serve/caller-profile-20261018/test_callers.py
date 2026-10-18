import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]

class CallerTests(unittest.TestCase):
    def capture(self, capacity):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            source = out/'test.c'
            source.write_text('''#include "native-gc.h"
#include <stdio.h>
static const neoclr_probe_frame *active;
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return active; }
void record_caller(uint32_t, uint64_t);
void record_idle(void);
void record_boundary(unsigned);
void report_callers(FILE *);
int main(void) {
    neoclr_probe_frame a = {.function=1,.instruction=10};
    neoclr_probe_frame b = {.function=2,.instruction=20,.previous=&a};
    neoclr_probe_frame c = {.function=1,.instruction=30,.previous=&b};
    neoclr_probe_frame d = {.function=4,.previous=&c};
    neoclr_probe_frame e = {.function=5,.previous=&d};
    active=&e;
    record_caller(3,32); record_caller(3,32);
    active=NULL; record_caller(2,8);
    active=&e; record_caller(3,64);
    e.instruction=1; record_caller(3,32);
    record_caller(2,32);
    active=&a; record_caller(3,32);
    record_boundary(0); record_boundary(1); record_boundary(2); record_idle();
    report_callers(stdout);
}
''')
            command = ['clang','-std=c11','-Wall','-Wextra','-Werror','-DCALLER_TEST',
                '-DCALLER_CAPACITY='+str(capacity),'-I',str(ROOT/'runtime/native/gc'),str(source),str(HERE/'callers.c'),'-o',str(out/'test')]
            built=subprocess.run(command,capture_output=True,text=True)
            self.assertEqual(built.returncode,0,built.stderr)
            return json.loads(subprocess.check_output([str(out/'test')],text=True))

    def test_frames_sizes_kinds_instructions_and_unframed(self):
        data=self.capture(32)
        self.assertEqual(data['allocations'],7)
        self.assertEqual(data['allocatedBytes'],232)
        self.assertEqual(data['dropped'],0)
        self.assertEqual(data['boundaryReasons'],[1,1,1])
        self.assertEqual(data['idleCalls'],1)
        repeated=next(r for r in data['rows'] if r['count']==2)
        self.assertEqual([f['function'] for f in repeated['frames']],[5,4,1,2])
        self.assertEqual(repeated['truncated'],1)
        self.assertEqual(next(r for r in data['rows'] if not r['frames'])['size'],8)
        short=next(r for r in data['rows'] if len(r['frames'])==1)
        self.assertEqual(short['truncated'],0)
        self.assertEqual(short['frames'][0]['instruction'],10)
        self.assertEqual(sum(r['count'] for r in data['rows']),7)
        self.assertEqual(sum(r['count']*r['size'] for r in data['rows']),232)

    def test_overflow_is_accounted_and_existing_keys_still_count(self):
        data=self.capture(2)
        self.assertEqual(len(data['rows']),2)
        self.assertEqual(data['dropped'],4)
        self.assertEqual(sum(r['count'] for r in data['rows'])+data['dropped'],7)
        self.assertEqual(sum(r['count']*r['size'] for r in data['rows'])+data['droppedBytes'],232)

if __name__=='__main__':unittest.main()
