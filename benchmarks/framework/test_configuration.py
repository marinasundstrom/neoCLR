import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

from configuration import resolve
from runner import run, sha


class ConfigurationTests(unittest.TestCase):
    def test_precedence_and_zero_warmups(self):
        config, origins = resolve({'rounds': 5, 'warmups': 4},
                                  {'rounds': 6, 'operations': 2}, {'warmups': 0})
        self.assertEqual((config['rounds'],config['warmups'],config['operations']), (6,0,2))
        self.assertEqual(origins['warmups'], 'override')
        self.assertEqual(origins['rounds'], 'config')
        self.assertEqual(origins['modes'], 'default')

    def test_rejects_unsupported_or_misleading_configuration(self):
        for value in [{'measurement':'allocations'}, {'concurrency':4}, {'modes':[]},
                      {'modes':['jit','jit']}, {'modes':['unknown']}, {'warmups':True},
                      {'rounds':2}, {'operations':0}, {'timeout':float('nan')},
                      {'timeout':-1}, {'measurement':'first-invocation'},
                      {'measurement':'first-invocation','warmups':0,'operations':2}]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                resolve(external=value)
        config,_ = resolve(external={'measurement':'first-invocation','warmups':0,'operations':1})
        self.assertEqual(config['measurement'], 'first-invocation')

    def test_configured_mode_selection_metric_and_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            worker=root/'worker.py'
            worker.write_text("import sys,time\nprint('NEOBENCH/1 READY',flush=True)\nprint('NEOBENCH/1 BEGIN',flush=True)\ntime.sleep(.02)\nprint('NEOBENCH/1 END')\nprint(sys.argv[-1])\n")
            manifest=root/'manifest.json'
            manifest.write_text(json.dumps(dict(schemaVersion=1,
                commands={'aot':[sys.executable,str(worker)]}, cases=['one'],
                inputs={str(worker):sha(worker)}, benchmarkConfig={'rounds':4})))
            config=root/'config.json'
            config.write_text(json.dumps(dict(schemaVersion=1, modes=['aot'],
                measurement='first-invocation',warmups=0,operations=1)))
            with contextlib.redirect_stdout(io.StringIO()):
                report=run(manifest,root/'results.json',rounds=3,config_path=config)
            self.assertTrue(report['passed'])
            self.assertEqual(len(report['samples']),3)
            self.assertEqual(report['primaryMetric'],dict(name='batchSeconds',unit='seconds'))
            self.assertEqual(report['effectiveConfiguration']['rounds'],3)
            self.assertEqual(report['configurationOrigins']['rounds'],'override')
            self.assertEqual(report['inputs'][str(config.resolve())],sha(config))
            self.assertEqual({x['operations'] for x in report['samples']},{1})
            with self.assertRaisesRegex(ValueError,'Missing or invalid command'):
                run(manifest,root/'missing.json',config_path=config,modes=['jit'])
            self.assertFalse((root/'missing.json').exists())


if __name__=='__main__':
    unittest.main()
